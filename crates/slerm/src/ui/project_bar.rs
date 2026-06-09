use gpui::{App, Entity, IntoElement, RenderOnce, Window, div, prelude::*};

use crate::{
    runtime::{AttentionSeverity, TerminalRuntimeService},
    theme,
    workspace::model::WorkspaceState,
};

#[derive(IntoElement)]
pub struct ProjectBar {
    workspace: Entity<WorkspaceState>,
    runtime: Entity<TerminalRuntimeService>,
}

impl ProjectBar {
    pub fn new(workspace: Entity<WorkspaceState>, runtime: Entity<TerminalRuntimeService>) -> Self {
        Self { workspace, runtime }
    }
}

impl RenderOnce for ProjectBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme::active();
        let workspace = self.workspace.read(cx);
        let runtime = self.runtime.read(cx);
        let projects = workspace
            .projects
            .iter()
            .enumerate()
            .map(|(index, project)| {
                let is_active = workspace.active_project == Some(project.id);
                let attention = runtime.project_attention(project).severity;
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_color(if is_active { theme.plus1 } else { theme.minus1 })
                    .child(div().child(format!("{}", index + 1)))
                    .child(div().child(project.name.clone()))
                    .when(attention != AttentionSeverity::None, |project| {
                        project.child(
                            div()
                                .text_color(attention_color(attention))
                                .child(attention_icon(attention)),
                        )
                    })
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
            .child(div().child("cmd-←/→ project · cmd-↑/↓ terminal · cmd-w close · cmd-q quit"))
    }
}

fn attention_icon(severity: AttentionSeverity) -> &'static str {
    match severity {
        AttentionSeverity::None => "",
        AttentionSeverity::Info => "•",
        AttentionSeverity::Activity => "●",
        AttentionSeverity::NeedsUser => "◆",
        AttentionSeverity::Error => "!",
    }
}

fn attention_color(severity: AttentionSeverity) -> gpui::Rgba {
    let theme = theme::active();
    match severity {
        AttentionSeverity::None => theme.minus1,
        AttentionSeverity::Info => theme.info,
        AttentionSeverity::Activity => theme.plus2,
        AttentionSeverity::NeedsUser => theme.warning,
        AttentionSeverity::Error => theme.error,
    }
}
