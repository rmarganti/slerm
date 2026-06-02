use gpui::{Context, Entity, IntoElement, Render, Window, div, prelude::*};

use crate::{
    theme,
    ui::{project_bar::ProjectBar, sidebar::Sidebar, terminal_pane::TerminalPane},
    workspace::model::WorkspaceState,
};

pub struct SlermApp {
    workspace: Entity<WorkspaceState>,
}

impl SlermApp {
    pub fn mock(cx: &mut Context<Self>) -> Self {
        Self {
            workspace: cx.new(|_| WorkspaceState::mock()),
        }
    }
}

impl Render for SlermApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::active();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.bg)
            .text_color(theme.fg)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .child(Sidebar::new(self.workspace.clone()))
                    .child(TerminalPane::new(self.workspace.clone())),
            )
            .child(ProjectBar::new(self.workspace.clone()))
    }
}
