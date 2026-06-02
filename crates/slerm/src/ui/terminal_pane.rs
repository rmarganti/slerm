use gpui::{App, Entity, FontWeight, IntoElement, RenderOnce, Window, div, prelude::*, px};

use crate::{theme, workspace::model::WorkspaceState};

#[derive(IntoElement)]
pub struct TerminalPane {
    workspace: Entity<WorkspaceState>,
}

impl TerminalPane {
    pub fn new(workspace: Entity<WorkspaceState>) -> Self {
        Self { workspace }
    }
}

impl RenderOnce for TerminalPane {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme::active();
        let workspace = self.workspace.read(cx);
        let active_item = workspace
            .active_project()
            .and_then(|project| project.active_item());

        let title = active_item
            .map(|item| item.title.clone())
            .unwrap_or_else(|| "No terminal selected".to_string());
        let cwd = active_item
            .map(|item| item.cwd.display().to_string())
            .unwrap_or_default();
        let command = active_item
            .and_then(|item| item.command.clone())
            .unwrap_or_else(|| "$SHELL".to_string());
        let project_id = active_item
            .map(|item| format!("project #{}", item.project_id.0))
            .unwrap_or_default();

        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .bg(theme.bg)
            .child(
                div()
                    .h(px(42.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(theme.border)
                    .px_4()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
                    .child(div().text_xs().text_color(theme.minus1).child(cwd)),
            )
            .child(
                div()
                    .flex_1()
                    .m_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.bg)
                    .p_4()
                    .font_family("monospace")
                    .child(div().text_color(theme.minus1).child("# terminal preview"))
                    .child(div().mt_4().child(format!("$ {command}")))
                    .child(
                        div()
                            .mt_2()
                            .text_color(theme.minus1)
                            .child("libghostty will render the live terminal here."),
                    )
                    .child(div().mt_2().text_color(theme.minus1).child(project_id)),
            )
    }
}
