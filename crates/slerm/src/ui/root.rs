use gpui::{
    Context, Entity, FocusHandle, Focusable, IntoElement, PathPromptOptions, Render, Window, div,
    prelude::*,
};

use crate::{
    actions::{
        ActiveProjectCycleNext, ActiveProjectCyclePrev, ActiveProjectMoveLeft,
        ActiveProjectMoveRight, ActiveProjectRemove, ActiveProjectSelectByIndex,
        ActiveTerminalClose, ActiveTerminalCycleNext, ActiveTerminalCyclePrev,
        ActiveTerminalSelectByIndex, OpenAddProjectPicker, OpenAddTerminalPicker,
        OpenProjectPicker, OpenRenameProjectModal,
    },
    project::model::CycleDirection,
    runtime::TerminalRuntimeService,
    storage, theme,
    ui::{
        add_terminal_picker::AddTerminalPicker,
        modal_layer::{ActiveModal, ModalLayer},
        project_bar::ProjectBar,
        project_picker::ProjectPicker,
        rename_project_modal::RenameProjectModal,
        sidebar::Sidebar,
        terminal_pane::TerminalPane,
    },
    workspace::model::WorkspaceState,
};

/// Root GPUI model for a Slerm window.
///
/// It keeps persisted workspace state separate from live terminal runtime state
/// while coordinating keyboard actions, modal UI, and child views.
pub struct SlermApp {
    workspace: Entity<WorkspaceState>,
    runtime: Entity<TerminalRuntimeService>,
    focus_handle: FocusHandle,
    active_modal: Option<ActiveModal>,
}

impl SlermApp {
    pub fn new(workspace: WorkspaceState, cx: &mut Context<Self>) -> Self {
        let runtime = TerminalRuntimeService::from_workspace(&workspace);

        Self {
            workspace: cx.new(|_| workspace),
            runtime: cx.new(|_| runtime),
            focus_handle: cx.focus_handle(),
            active_modal: None,
        }
    }
}

impl SlermApp {
    fn active_terminal_close(
        &mut self,
        _: &ActiveTerminalClose,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_active_terminal(cx);
    }

    fn active_terminal_cycle_next(
        &mut self,
        _: &ActiveTerminalCycleNext,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cycle_active_terminal(CycleDirection::Next, cx);
    }

    fn active_terminal_cycle_prev(
        &mut self,
        _: &ActiveTerminalCyclePrev,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cycle_active_terminal(CycleDirection::Prev, cx);
    }

    fn active_project_cycle_next(
        &mut self,
        _: &ActiveProjectCycleNext,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cycle_active_project(CycleDirection::Next, cx);
    }

    fn active_project_move_left(
        &mut self,
        _: &ActiveProjectMoveLeft,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_active_project(CycleDirection::Prev, cx);
    }

    fn active_project_move_right(
        &mut self,
        _: &ActiveProjectMoveRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_active_project(CycleDirection::Next, cx);
    }

    fn active_project_remove(
        &mut self,
        _: &ActiveProjectRemove,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.remove_active_project(cx);
        self.focus_handle.focus(window);
    }

    fn open_add_terminal_picker(
        &mut self,
        _: &OpenAddTerminalPicker,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let app = cx.entity();
        let workspace = self.workspace.clone();
        let runtime = self.runtime.clone();
        let picker = cx.new(|cx| {
            AddTerminalPicker::new(
                workspace,
                runtime,
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
        self.active_modal = Some(ActiveModal::AddTerminalPicker(picker));
        cx.notify();
        if let Some(ActiveModal::AddTerminalPicker(picker)) = &self.active_modal {
            picker.read(cx).focus_handle(cx).focus(window);
        }
    }

    fn open_add_project_picker(
        &mut self,
        _: &OpenAddProjectPicker,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Add Project".into()),
        });

        cx.spawn_in(window, async move |app, cx| {
            let selected_path = match paths.await {
                Ok(Ok(Some(mut paths))) => paths.pop(),
                Ok(Ok(None)) => None,
                Ok(Err(error)) => {
                    eprintln!("failed to open project folder picker: {error}");
                    None
                }
                Err(_) => None,
            };

            app.update_in(cx, |app, window, cx| {
                if let Some(path) = selected_path {
                    app.add_project(path, cx);
                }
                app.focus_handle.focus(window);
            })
            .ok();
        })
        .detach();
    }

    fn open_project_picker(
        &mut self,
        _: &OpenProjectPicker,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let app = cx.entity();
        let workspace = self.workspace.clone();
        let picker = cx.new(|cx| {
            ProjectPicker::new(
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
        self.active_modal = Some(ActiveModal::ProjectPicker(picker));
        cx.notify();
        if let Some(ActiveModal::ProjectPicker(picker)) = &self.active_modal {
            picker.read(cx).focus_handle(cx).focus(window);
        }
    }

    fn open_rename_project_modal(
        &mut self,
        _: &OpenRenameProjectModal,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(current_name) = self
            .workspace
            .read(cx)
            .active_project()
            .map(|project| project.name.clone())
        else {
            self.focus_handle.focus(window);
            return;
        };

        let app = cx.entity();
        let workspace = self.workspace.clone();
        let modal = cx.new(|cx| {
            RenameProjectModal::new(
                workspace,
                current_name,
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
        self.active_modal = Some(ActiveModal::RenameProjectModal(modal));
        cx.notify();
        if let Some(ActiveModal::RenameProjectModal(modal)) = &self.active_modal {
            modal.read(cx).focus_handle(cx).focus(window);
        }
    }

    fn dismiss_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active_modal = None;
        self.focus_handle.focus(window);
        cx.notify();
    }

    fn active_terminal_select_by_index(
        &mut self,
        action: &ActiveTerminalSelectByIndex,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_active_terminal_by_sidebar_index(action.index, cx);
    }

    fn active_project_select_by_index(
        &mut self,
        action: &ActiveProjectSelectByIndex,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_active_project_by_index(action.index, cx);
    }

    fn active_project_cycle_prev(
        &mut self,
        _: &ActiveProjectCyclePrev,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cycle_active_project(CycleDirection::Prev, cx);
    }

    fn close_active_terminal(&mut self, cx: &mut Context<Self>) {
        let closed_terminal =
            self.update_workspace(cx, |workspace| workspace.close_active_terminal());

        if let Some(terminal_id) = closed_terminal {
            self.runtime.update(cx, |runtime, cx| {
                runtime.remove_terminal(terminal_id);
                cx.notify();
            });
        }
    }

    fn remove_active_project(&mut self, cx: &mut Context<Self>) {
        let removed_terminal_ids =
            self.update_workspace(cx, |workspace| workspace.remove_active_project());

        if !removed_terminal_ids.is_empty() {
            self.runtime.update(cx, |runtime, cx| {
                for terminal_id in removed_terminal_ids {
                    runtime.remove_terminal(terminal_id);
                }
                cx.notify();
            });
        }
    }

    fn add_project(&mut self, path: impl Into<std::path::PathBuf>, cx: &mut Context<Self>) {
        let project = self.update_workspace(cx, |workspace| workspace.add_project(path));

        self.runtime.update(cx, |runtime, cx| {
            for terminal in &project.terminals {
                runtime.ensure_terminal(terminal);
            }
            cx.notify();
        });
    }

    fn cycle_active_terminal(&mut self, direction: CycleDirection, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.cycle_active_terminal(direction);
        });
    }

    fn cycle_active_project(&mut self, direction: CycleDirection, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.cycle_active_project(direction);
        });
    }

    fn move_active_project(&mut self, direction: CycleDirection, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.move_active_project(direction);
        });
    }

    fn select_active_terminal_by_sidebar_index(&mut self, index: usize, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.select_active_terminal_by_sidebar_index(index);
        });
    }

    fn select_active_project_by_index(&mut self, index: usize, cx: &mut Context<Self>) {
        self.update_workspace(cx, |workspace| {
            workspace.select_active_project_by_index(index);
        });
    }

    fn update_workspace<T>(
        &mut self,
        cx: &mut Context<Self>,
        update: impl FnOnce(&mut WorkspaceState) -> T,
    ) -> T {
        let output = self.workspace.update(cx, |workspace, cx| {
            let output = update(workspace);
            cx.notify();
            output
        });

        if let Err(error) = storage::save_workspace(self.workspace.read(cx)) {
            eprintln!("failed to save workspace: {error}");
        }

        cx.notify();
        output
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
            .on_action(cx.listener(Self::active_terminal_close))
            .on_action(cx.listener(Self::active_terminal_cycle_next))
            .on_action(cx.listener(Self::active_terminal_cycle_prev))
            .on_action(cx.listener(Self::active_terminal_select_by_index))
            .on_action(cx.listener(Self::active_project_cycle_next))
            .on_action(cx.listener(Self::active_project_cycle_prev))
            .on_action(cx.listener(Self::active_project_move_left))
            .on_action(cx.listener(Self::active_project_move_right))
            .on_action(cx.listener(Self::active_project_remove))
            .on_action(cx.listener(Self::active_project_select_by_index))
            .on_action(cx.listener(Self::open_add_terminal_picker))
            .on_action(cx.listener(Self::open_add_project_picker))
            .on_action(cx.listener(Self::open_project_picker))
            .on_action(cx.listener(Self::open_rename_project_modal))
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
                    .child(Sidebar::new(self.workspace.clone(), self.runtime.clone()))
                    .child(TerminalPane::new(self.workspace.clone())),
            )
            .child(ProjectBar::new(
                self.workspace.clone(),
                self.runtime.clone(),
            ))
            .child(ModalLayer::new(self.active_modal.clone(), {
                let app = cx.entity();
                move |window, cx| {
                    app.update(cx, |app, cx| app.dismiss_modal(window, cx));
                }
            }))
    }
}
