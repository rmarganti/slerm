use gpui::{
    Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window, actions, div, prelude::*,
    px,
};

use crate::{storage, theme, ui::text_input::TextInput, workspace::model::WorkspaceState};

type DoneHandler = dyn Fn(&mut Window, &mut gpui::App) + 'static;

actions!(
    slerm_rename_project_modal,
    [RenameProjectConfirm, RenameProjectCancel]
);

/// Minimal modal for renaming the active project.
pub struct RenameProjectModal {
    workspace: Entity<WorkspaceState>,
    input: Entity<TextInput>,
    focus_handle: FocusHandle,
    on_done: Box<DoneHandler>,
}

impl RenameProjectModal {
    pub fn new(
        workspace: Entity<WorkspaceState>,
        current_name: impl Into<String>,
        on_done: impl Fn(&mut Window, &mut gpui::App) + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        let current_name = current_name.into();
        let input = cx.new(|cx| {
            let mut input = TextInput::new("Project name", cx);
            input.set_text(current_name, cx);
            input
        });

        Self {
            workspace,
            input,
            focus_handle: cx.focus_handle(),
            on_done: Box::new(on_done),
        }
    }

    fn confirm(&mut self, _: &RenameProjectConfirm, window: &mut Window, cx: &mut Context<Self>) {
        let new_name = self.input.read(cx).text().to_string();
        let renamed = self.workspace.update(cx, |workspace, cx| {
            let renamed = workspace.rename_active_project(&new_name);
            if renamed {
                cx.notify();
            }
            renamed
        });

        if renamed && let Err(error) = storage::save_workspace(self.workspace.read(cx)) {
            eprintln!("failed to save workspace: {error}");
        }

        (self.on_done)(window, cx);
    }

    fn cancel(&mut self, _: &RenameProjectCancel, window: &mut Window, cx: &mut Context<Self>) {
        (self.on_done)(window, cx);
    }

    fn focus_input(&self, window: &mut Window, cx: &mut gpui::App) {
        self.input.read(cx).focus_handle(cx).focus(window);
    }
}

impl Focusable for RenameProjectModal {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RenameProjectModal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::active();
        self.focus_input(window, cx);

        div()
            .key_context("RenameProjectModal")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .w(px(420.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(theme.border)
            .bg(theme.float_bg)
            .shadow_lg()
            .p_3()
            .child(
                div()
                    .mb_2()
                    .text_sm()
                    .text_color(theme.minus1)
                    .child("Rename project"),
            )
            .child(self.input.clone())
            .child(
                div()
                    .mt_2()
                    .text_xs()
                    .text_color(theme.minus2)
                    .child("Enter to rename · Esc to cancel"),
            )
    }
}
