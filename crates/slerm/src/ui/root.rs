use gpui::{Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window, div, prelude::*};

use crate::{
    actions::{
        ActiveItemCycleNext, ActiveItemCyclePrev, ActiveItemSelectByIndex, ActiveProjectCycleNext,
        ActiveProjectCyclePrev,
    },
    project::model::CycleDirection,
    theme,
    ui::{project_bar::ProjectBar, sidebar::Sidebar, terminal_pane::TerminalPane},
    workspace::model::WorkspaceState,
};

pub struct SlermApp {
    workspace: Entity<WorkspaceState>,
    focus_handle: FocusHandle,
}

impl SlermApp {
    pub fn mock(cx: &mut Context<Self>) -> Self {
        Self {
            workspace: cx.new(|_| WorkspaceState::mock()),
            focus_handle: cx.focus_handle(),
        }
    }
}

impl SlermApp {
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

    fn cycle_active_item(&mut self, direction: CycleDirection, cx: &mut Context<Self>) {
        self.workspace.update(cx, |workspace, cx| {
            workspace.cycle_active_item(direction);
            cx.notify();
        });
        cx.notify();
    }

    fn cycle_active_project(&mut self, direction: CycleDirection, cx: &mut Context<Self>) {
        self.workspace.update(cx, |workspace, cx| {
            workspace.cycle_active_project(direction);
            cx.notify();
        });
        cx.notify();
    }

    fn select_active_item_by_sidebar_index(&mut self, index: usize, cx: &mut Context<Self>) {
        self.workspace.update(cx, |workspace, cx| {
            workspace.select_active_item_by_sidebar_index(index);
            cx.notify();
        });
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
            .on_action(cx.listener(Self::active_item_cycle_next))
            .on_action(cx.listener(Self::active_item_cycle_prev))
            .on_action(cx.listener(Self::active_item_select_by_index))
            .on_action(cx.listener(Self::active_project_cycle_next))
            .on_action(cx.listener(Self::active_project_cycle_prev))
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
