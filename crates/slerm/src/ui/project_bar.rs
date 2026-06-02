use gpui::{App, Entity, IntoElement, RenderOnce, Window, div, prelude::*};

use crate::{theme, workspace::model::WorkspaceState};

#[derive(IntoElement)]
pub struct ProjectBar {
    workspace: Entity<WorkspaceState>,
}

impl ProjectBar {
    pub fn new(workspace: Entity<WorkspaceState>) -> Self {
        Self { workspace }
    }
}

impl RenderOnce for ProjectBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme::active();
        let workspace = self.workspace.read(cx);
        let projects = workspace
            .projects
            .iter()
            .enumerate()
            .map(|(index, project)| {
                let is_active = workspace.active_project == Some(project.id);
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_color(if is_active { theme.plus1 } else { theme.minus1 })
                    .child(div().child(format!("{}", index + 1)))
                    .child(div().child(project.name.clone()))
            });

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_1()
            .border_t_1()
            .border_color(theme.border)
            .text_xs()
            .text_color(theme.minus1)
            .child(div().flex().items_center().gap_4().children(projects))
            .child(div().child("ctrl-p project · ctrl-n terminal · ctrl-q quit"))
    }
}
