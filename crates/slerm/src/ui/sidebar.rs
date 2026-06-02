use gpui::{App, Entity, FontWeight, IntoElement, RenderOnce, Window, div, prelude::*, px};

use crate::{theme, workspace::model::WorkspaceState};

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
            .child(Section::new(&workspace, "Terminals"))
            .child(Section::new(&workspace, "Agents"))
            .child(Section::new(&workspace, "Tasks"))
    }
}

// ----------------------------------------------------------------
// Section
// ----------------------------------------------------------------

#[derive(IntoElement)]
struct Section {
    label: &'static str,
    items: Vec<ItemRow>,
}

impl Section {
    fn new(workspace: &WorkspaceState, label: &'static str) -> Self {
        let items = workspace
            .active_project()
            .into_iter()
            .flat_map(|project| {
                project
                    .items
                    .iter()
                    .filter(move |item| item.kind.section_label() == label)
                    .map(move |item| {
                        ItemRow::new(item.title.clone(), project.active_item == Some(item.id))
                    })
            })
            .collect();

        Self { label, items }
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
            .p_4()
            .child(
                div()
                    .mb_1()
                    .flex()
                    .items_center()
                    .gap_1()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.minus1)
                    .child(div().w(px(14.0)).child(self.icon()))
                    .child(tracked_uppercase(self.label)),
            )
            .children(self.items)
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
// ItemRow
// ----------------------------------------------------------------

#[derive(IntoElement)]
struct ItemRow {
    title: String,
    is_active: bool,
}

impl ItemRow {
    fn new(title: String, is_active: bool) -> Self {
        Self { title, is_active }
    }
}

impl RenderOnce for ItemRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = theme::active();

        div()
            .py_1()
            .text_sm()
            .text_color(if self.is_active {
                theme.plus2
            } else {
                theme.base
            })
            .truncate()
            .child(self.title)
    }
}
