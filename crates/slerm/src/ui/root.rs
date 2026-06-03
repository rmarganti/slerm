use gpui::{Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window, div, prelude::*};

use crate::{
    actions::{
        ActiveItemClose, ActiveItemCycleNext, ActiveItemCyclePrev, ActiveItemSelectByIndex,
        ActiveProjectCycleNext, ActiveProjectCyclePrev, OpenAddItemPicker,
    },
    project::model::CycleDirection,
    storage, theme,
    ui::{
        add_item_picker::AddItemPicker,
        modal_layer::{ActiveModal, ModalLayer},
        project_bar::ProjectBar,
        sidebar::Sidebar,
        terminal_pane::TerminalPane,
    },
    workspace::model::WorkspaceState,
};

pub struct SlermApp {
    workspace: Entity<WorkspaceState>,
    focus_handle: FocusHandle,
    active_modal: Option<ActiveModal>,
}

impl SlermApp {
    pub fn new(workspace: WorkspaceState, cx: &mut Context<Self>) -> Self {
        Self {
            workspace: cx.new(|_| workspace),
            focus_handle: cx.focus_handle(),
            active_modal: None,
        }
    }
}

impl SlermApp {
    fn active_item_close(&mut self, _: &ActiveItemClose, _: &mut Window, cx: &mut Context<Self>) {
        self.close_active_item(cx);
    }

    fn active_item_cycle_next(
        &mut self,
        _: &ActiveItemCycleNext,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cycle_active_item(CycleDirection::Next, cx);
    }

    fn active_item_cycle_prev(
        &mut self,
        _: &ActiveItemCyclePrev,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cycle_active_item(CycleDirection::Prev, cx);
    }

    fn active_project_cycle_next(
        &mut self,
        _: &ActiveProjectCycleNext,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cycle_active_project(CycleDirection::Next, cx);
    }

    fn open_add_item_picker(
        &mut self,
        _: &OpenAddItemPicker,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let app = cx.entity();
        let workspace = self.workspace.clone();
        let picker = cx.new(|cx| {
            AddItemPicker::new(
                workspace,
                move |window, cx| {
                    app.update(cx, |app, cx| {
                        app.active_modal = None;
                        app.focus_handle.focus(window);
                        cx.notify();
                    });
                },
                cx,
            )
        });
        self.active_modal = Some(ActiveModal::AddItemPicker(picker));
        cx.notify();
        if let Some(ActiveModal::AddItemPicker(picker)) = &self.active_modal {
            picker.read(cx).focus_handle(cx).focus(window);
        }
    }

    fn dismiss_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active_modal = None;
        self.focus_handle.focus(window);
        cx.notify();
    }

    fn active_item_select_by_index(
        &mut self,
        action: &ActiveItemSelectByIndex,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_active_item_by_sidebar_index(action.index, cx);
    }

    fn active_project_cycle_prev(
        &mut self,
        _: &ActiveProjectCyclePrev,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cycle_active_project(CycleDirection::Prev, cx);
    }

    fn close_active_item(&mut self, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.close_active_item();
        });
    }

    fn cycle_active_item(&mut self, direction: CycleDirection, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.cycle_active_item(direction);
        });
    }

    fn cycle_active_project(&mut self, direction: CycleDirection, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.cycle_active_project(direction);
        });
    }

    fn select_active_item_by_sidebar_index(&mut self, index: usize, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.select_active_item_by_sidebar_index(index);
        });
    }

    fn update_workspace(
        &mut self,
        cx: &mut Context<Self>,
        update: impl FnOnce(&mut WorkspaceState),
    ) {
        self.workspace.update(cx, |workspace, cx| {
            update(workspace);
            cx.notify();
        });

        if let Err(error) = storage::save_workspace(self.workspace.read(cx)) {
            eprintln!("failed to save workspace: {error}");
        }

        cx.notify();
    }
}

impl Focusable for SlermApp {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SlermApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::active();

        if window.focused(cx).is_none() {
            self.focus_handle.focus(window);
        }

        div()
            .key_context("workspace")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::active_item_close))
            .on_action(cx.listener(Self::active_item_cycle_next))
            .on_action(cx.listener(Self::active_item_cycle_prev))
            .on_action(cx.listener(Self::active_item_select_by_index))
            .on_action(cx.listener(Self::active_project_cycle_next))
            .on_action(cx.listener(Self::active_project_cycle_prev))
            .on_action(cx.listener(Self::open_add_item_picker))
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
            .child(ModalLayer::new(self.active_modal.clone(), {
                let app = cx.entity();
                move |window, cx| {
                    app.update(cx, |app, cx| app.dismiss_modal(window, cx));
                }
            }))
    }
}
