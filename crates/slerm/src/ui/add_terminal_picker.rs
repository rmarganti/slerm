use gpui::{AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window};

use crate::{
    runtime::TerminalRuntimeService,
    storage,
    terminal::extension::AgentKind,
    ui::fuzzy_finder::{FuzzyFinder, FuzzyFinderItem},
    workspace::model::WorkspaceState,
};

#[derive(Clone, Debug)]
pub enum AddTerminalChoice {
    Terminal,
    #[allow(dead_code)]
    Agent(AgentKind),
    #[allow(dead_code)]
    Command,
}

pub struct AddTerminalPicker {
    finder: Entity<FuzzyFinder<AddTerminalChoice>>,
}

impl AddTerminalPicker {
    pub fn new(
        workspace: Entity<WorkspaceState>,
        runtime: Entity<TerminalRuntimeService>,
        on_done: impl Fn(&mut Window, &mut gpui::App) + Clone + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        let done_on_confirm = on_done.clone();
        let finder = cx.new(|cx| {
            FuzzyFinder::new(
                "Add to project...",
                vec![FuzzyFinderItem::new(
                    "Terminal",
                    Some("Open a placeholder shell terminal"),
                    AddTerminalChoice::Terminal,
                )],
                move |kind, window, cx| match kind {
                    AddTerminalChoice::Terminal => {
                        let added_terminal = workspace.update(cx, |workspace, cx| {
                            let added_terminal = workspace
                                .add_terminal_to_active_project()
                                .and_then(|terminal_id| {
                                    workspace
                                        .projects
                                        .iter()
                                        .flat_map(|project| project.terminals.iter())
                                        .find(|terminal| terminal.id == terminal_id)
                                        .cloned()
                                });
                            cx.notify();
                            added_terminal
                        });

                        if let Some(terminal) = added_terminal.as_ref() {
                            runtime.update(cx, |runtime, cx| {
                                runtime.ensure_terminal(terminal);
                                cx.notify();
                            });
                        }

                        if let Err(error) = storage::save_workspace(workspace.read(cx)) {
                            eprintln!("failed to save workspace: {error}");
                        }
                        done_on_confirm(window, cx);
                    }
                    AddTerminalChoice::Agent(_) | AddTerminalChoice::Command => {}
                },
                move |window, cx| on_done(window, cx),
                cx,
            )
        });
        Self { finder }
    }
}

impl Focusable for AddTerminalPicker {
    fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        self.finder.read(cx).focus_handle(cx)
    }
}

impl Render for AddTerminalPicker {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.finder.clone()
    }
}
