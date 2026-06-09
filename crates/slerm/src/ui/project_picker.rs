use gpui::{AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window};

use crate::{
    project::model::ProjectId,
    storage,
    ui::fuzzy_finder::{FuzzyFinder, FuzzyFinderItem},
    workspace::model::WorkspaceState,
};

pub struct ProjectPicker {
    finder: Entity<FuzzyFinder<ProjectId>>,
}

impl ProjectPicker {
    pub fn new(
        workspace: Entity<WorkspaceState>,
        on_done: impl Fn(&mut Window, &mut gpui::App) + Clone + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        let items = workspace
            .read(cx)
            .projects
            .iter()
            .map(|project| {
                FuzzyFinderItem::new(
                    project.name.clone(),
                    Some(project.path.display().to_string()),
                    project.id,
                )
            })
            .collect::<Vec<_>>();

        let done_on_confirm = on_done.clone();
        let finder = cx.new(|cx| {
            FuzzyFinder::new(
                "Open project...",
                items,
                move |project_id, window, cx| {
                    let selected = workspace.update(cx, |workspace, cx| {
                        let selected = workspace.select_active_project_by_id(project_id);
                        if selected {
                            cx.notify();
                        }
                        selected
                    });

                    if selected && let Err(error) = storage::save_workspace(workspace.read(cx)) {
                        eprintln!("failed to save workspace: {error}");
                    }

                    done_on_confirm(window, cx);
                },
                move |window, cx| on_done(window, cx),
                cx,
            )
        });

        Self { finder }
    }
}

impl Focusable for ProjectPicker {
    fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        self.finder.read(cx).focus_handle(cx)
    }
}

impl Render for ProjectPicker {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.finder.clone()
    }
}
