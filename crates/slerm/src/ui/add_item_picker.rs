use gpui::{AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window};

use crate::{
    storage,
    terminal::kind::AgentKind,
    ui::fuzzy_finder::{FuzzyFinder, FuzzyFinderItem},
    workspace::model::WorkspaceState,
};

#[derive(Clone, Debug)]
pub enum AddItemKind {
    Terminal,
    #[allow(dead_code)]
    Agent(AgentKind),
    #[allow(dead_code)]
    Command,
}

pub struct AddItemPicker {
    finder: Entity<FuzzyFinder<AddItemKind>>,
}

impl AddItemPicker {
    pub fn new(
        workspace: Entity<WorkspaceState>,
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
                    AddItemKind::Terminal,
                )],
                move |kind, window, cx| match kind {
                    AddItemKind::Terminal => {
                        workspace.update(cx, |workspace, cx| {
                            workspace.add_terminal_to_active_project();
                            cx.notify();
                        });
                        if let Err(error) = storage::save_workspace(workspace.read(cx)) {
                            eprintln!("failed to save workspace: {error}");
                        }
                        done_on_confirm(window, cx);
                    }
                    AddItemKind::Agent(_) | AddItemKind::Command => {}
                },
                move |window, cx| on_done(window, cx),
                cx,
            )
        });
        Self { finder }
    }
}

impl Focusable for AddItemPicker {
    fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        self.finder.read(cx).focus_handle(cx)
    }
}

impl Render for AddItemPicker {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.finder.clone()
    }
}
