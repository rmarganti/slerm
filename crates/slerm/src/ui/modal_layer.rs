use gpui::{App, Entity, IntoElement, MouseButton, RenderOnce, Window, div, prelude::*, px, rgba};

use crate::ui::{
    add_terminal_picker::AddTerminalPicker, project_picker::ProjectPicker,
    rename_project_modal::RenameProjectModal,
};

/// Modal routes that can temporarily take focus over the workspace.
#[derive(Clone)]
pub enum ActiveModal {
    AddTerminalPicker(Entity<AddTerminalPicker>),
    ProjectPicker(Entity<ProjectPicker>),
    RenameProjectModal(Entity<RenameProjectModal>),
}

type DismissHandler = dyn Fn(&mut Window, &mut App) + 'static;

/// Full-window overlay that renders the active modal and handles dismissal.
#[derive(IntoElement)]
pub struct ModalLayer {
    active_modal: Option<ActiveModal>,
    on_dismiss: Box<DismissHandler>,
}

impl ModalLayer {
    pub fn new(
        active_modal: Option<ActiveModal>,
        on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            active_modal,
            on_dismiss: Box::new(on_dismiss),
        }
    }
}

impl RenderOnce for ModalLayer {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let Some(active_modal) = self.active_modal else {
            return div().absolute().size_full().hidden();
        };
        let on_dismiss = self.on_dismiss;

        let modal = match active_modal {
            ActiveModal::AddTerminalPicker(picker) => picker.into_any_element(),
            ActiveModal::ProjectPicker(picker) => picker.into_any_element(),
            ActiveModal::RenameProjectModal(modal) => modal.into_any_element(),
        };

        div()
            .absolute()
            .size_full()
            .flex()
            .justify_center()
            .items_start()
            .pt(px(96.0))
            .bg(rgba(0x00000022))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                cx.stop_propagation();
                on_dismiss(window, cx);
            })
            .child(
                div()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(modal),
            )
    }
}
