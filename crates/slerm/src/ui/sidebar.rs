use gpui::{App, Entity, FontWeight, IntoElement, RenderOnce, Window, div, prelude::*, px};

use crate::{project::model::Project, theme, workspace::model::WorkspaceState};

// ----------------------------------------------------------------
// Sidebar
// ----------------------------------------------------------------

#[derive(IntoElement)]
pub struct Sidebar {
    workspace: Entity<WorkspaceState>,
}

impl Sidebar {
    pub fn new(workspace: Entity<WorkspaceState>) -> Self {
        Self { workspace }
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme::active();
        let workspace = self.workspace.read(cx);

        let Some(project) = workspace.active_project() else {
            return div()
                .w(px(280.0))
                .h_full()
                .border_r_1()
                .border_color(theme.border)
                .bg(theme.float_bg)
                .p_4()
                .child("No project selected");
        };

        div()
            .w(px(280.0))
            .h_full()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(theme.border)
            .child(
                div()
                    .p_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(project.name.clone()),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_xs()
                            .text_color(theme.minus1)
                            .truncate()
                            .child(project.path.display().to_string()),
                    ),
            )
            .child(Section::new(project, "Terminals"))
            .child(Section::new(project, "Agents"))
            .child(Section::new(project, "Tasks"))
    }
}

// ----------------------------------------------------------------
// Section
// ----------------------------------------------------------------

#[derive(IntoElement)]
struct Section {
    label: &'static str,
    terminals: Vec<TerminalRow>,
}

impl Section {
    fn new(project: &Project, label: &'static str) -> Self {
        let terminals = project
            .terminals_in_sidebar_order()
            .into_iter()
            .enumerate()
            .filter(|(_, terminal)| terminal.extension.section_label() == label)
            .map(|(index, terminal)| {
                TerminalRow::new(
                    terminal.title.clone(),
                    project.active_terminal == Some(terminal.id),
                    (index < 9).then_some(index + 1),
                )
            })
            .collect();

        Self { label, terminals }
    }

    fn icon(&self) -> &'static str {
        match self.label {
            "Terminals" => "",
            "Agents" => "󰚩",
            "Tasks" => "✓",
            _ => "",
        }
    }
}

impl RenderOnce for Section {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = theme::active();

        div()
            .py_4()
            .child(
                div()
                    .mb_1()
                    .px_4()
                    .flex()
                    .items_center()
                    .gap_1()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.minus1)
                    .child(div().w(px(14.0)).child(self.icon()))
                    .child(tracked_uppercase(self.label)),
            )
            .children(self.terminals)
    }
}

fn tracked_uppercase(label: &str) -> String {
    label
        .to_uppercase()
        .chars()
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

// ----------------------------------------------------------------
// TerminalRow
// ----------------------------------------------------------------

#[derive(IntoElement)]
struct TerminalRow {
    title: String,
    is_active: bool,
    keybinding_index: Option<usize>,
}

impl TerminalRow {
    fn new(title: String, is_active: bool, keybinding_index: Option<usize>) -> Self {
        Self {
            title,
            is_active,
            keybinding_index,
        }
    }
}

impl RenderOnce for TerminalRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = theme::active();

        div()
            .py_1()
            .px_4()
            .border_l_2()
            .border_color(if self.is_active {
                theme.plus2
            } else {
                theme.bg
            })
            .text_sm()
            .text_color(if self.is_active {
                theme.plus2
            } else {
                theme.base
            })
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .child(div().text_color(theme.minus1).child("◦"))
            .child(div().flex_1().truncate().child(self.title))
            .when_some(self.keybinding_index, |row, index| {
                row.child(
                    div()
                        .text_xs()
                        .text_color(theme.minus2)
                        .child(format!("⌘{index}")),
                )
            })
    }
}
