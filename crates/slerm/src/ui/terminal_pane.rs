use gpui::{
    App, Bounds, Element, ElementId, Entity, GlobalElementId, Hsla, IntoElement, LayoutId,
    PaintQuad, Pixels, RenderOnce, ShapedLine, Style, TextRun, Window, div, fill, point,
    prelude::*, px, relative, size,
};
use libghostty_vt::{render::CursorVisualStyle, style::RgbColor};

use crate::{
    runtime::TerminalRuntimeService,
    terminal::{font::TerminalFontSelection, surface::TerminalDimensions},
    theme,
    workspace::model::WorkspaceState,
};

#[derive(IntoElement)]
pub struct TerminalPane {
    workspace: Entity<WorkspaceState>,
    runtime: Entity<TerminalRuntimeService>,
}

impl TerminalPane {
    pub fn new(workspace: Entity<WorkspaceState>, runtime: Entity<TerminalRuntimeService>) -> Self {
        Self { workspace, runtime }
    }
}

impl RenderOnce for TerminalPane {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = theme::active();
        div()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .bg(theme.bg)
            .font_family(TerminalFontSelection::discover().family)
            .text_size(px(14.0))
            .child(TerminalElement {
                workspace: self.workspace,
                runtime: self.runtime,
            })
    }
}

struct TerminalElement {
    workspace: Entity<WorkspaceState>,
    runtime: Entity<TerminalRuntimeService>,
}

struct PrepaintState {
    background_quad: PaintQuad,
    cell_height: Pixels,
    cells: Vec<PaintedCell>,
    cursor: Option<PaintQuad>,
}

struct PaintedCell {
    background: Option<PaintQuad>,
    line: Option<ShapedLine>,
    origin: gpui::Point<Pixels>,
}

impl IntoElement for TerminalElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TerminalElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let theme = theme::active();
        let text_style = window.text_style();
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let font = text_style.font();
        let probe = window.text_system().shape_line(
            "M".into(),
            font_size,
            &[TextRun {
                len: 1,
                font: font.clone(),
                color: text_style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        );
        let cell_width = probe.width.ceil().max(px(1.0));
        let cell_height = text_style
            .line_height_in_pixels(window.rem_size())
            .max(px(1.0));
        let dimensions = dimensions_for_bounds(bounds, cell_width, cell_height);

        let Some(spec) = self
            .workspace
            .read(cx)
            .active_project()
            .and_then(|project| project.active_terminal())
            .cloned()
        else {
            return empty_prepaint(bounds, theme.bg.into(), cell_width, cell_height);
        };

        let mut cells = Vec::new();
        let mut cursor = None;
        let mut background: Hsla = theme.bg.into();
        let terminal_id = spec.id;
        self.runtime.update(cx, |runtime, cx| {
            let active_live_ready = match runtime.ensure_live_terminal(&spec, dimensions) {
                Ok(_) => true,
                Err(error) => {
                    eprintln!("failed to start terminal {terminal_id:?}: {error}");
                    false
                }
            };
            if let Err(error) = runtime.resize_live_terminals(dimensions) {
                eprintln!("failed to resize live terminals: {error}");
            }
            runtime.drain_live_terminals();
            if active_live_ready && let Some(live) = runtime.live_terminal_mut(terminal_id) {
                match live.surface.render_snapshot() {
                    Ok(snapshot) => {
                        background = rgb_to_hsla(snapshot.colors.background);
                        for cell in snapshot.cells {
                            let mut foreground = cell.foreground;
                            let mut background_color = cell.background;
                            if cell.inverse {
                                let original_foreground = foreground;
                                foreground = background_color.unwrap_or(snapshot.colors.background);
                                background_color = Some(original_foreground);
                            }
                            let x = bounds.left() + cell_width * f32::from(cell.x);
                            let y = bounds.top() + cell_height * f32::from(cell.y);
                            let background = background_color.map(|color| {
                                fill(
                                    Bounds::new(point(x, y), size(cell_width, cell_height)),
                                    rgb_to_hsla(color),
                                )
                            });
                            let line = if cell.text.is_empty() {
                                None
                            } else {
                                Some(window.text_system().shape_line(
                                    cell.text.clone().into(),
                                    font_size,
                                    &[TextRun {
                                        len: cell.text.len(),
                                        font: font.clone(),
                                        color: rgb_to_hsla(foreground),
                                        background_color: None,
                                        underline: None,
                                        strikethrough: None,
                                    }],
                                    None,
                                ))
                            };
                            cells.push(PaintedCell {
                                background,
                                line,
                                origin: point(x, y),
                            });
                        }
                        if let Some(cursor_position) = snapshot.cursor {
                            let x = bounds.left() + cell_width * f32::from(cursor_position.x);
                            let y = bounds.top() + cell_height * f32::from(cursor_position.y);
                            let cursor_color =
                                snapshot.colors.cursor.unwrap_or(snapshot.colors.foreground);
                            let cursor_bounds = match cursor_position.style {
                                CursorVisualStyle::Bar => {
                                    Bounds::new(point(x, y), size(px(2.0), cell_height))
                                }
                                CursorVisualStyle::Underline => Bounds::new(
                                    point(x, y + cell_height - px(2.0)),
                                    size(cell_width, px(2.0)),
                                ),
                                CursorVisualStyle::Block | CursorVisualStyle::BlockHollow => {
                                    Bounds::new(point(x, y), size(cell_width, cell_height))
                                }
                                _ => Bounds::new(point(x, y), size(cell_width, cell_height)),
                            };
                            cursor = Some(fill(cursor_bounds, rgb_to_hsla(cursor_color)));
                        }
                    }
                    Err(error) => {
                        eprintln!("failed to render terminal {terminal_id:?}: {error}")
                    }
                }
            }
            cx.notify();
        });
        window.refresh();

        PrepaintState {
            background_quad: fill(bounds, background),
            cell_height,
            cells,
            cursor,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_quad(prepaint.background_quad.clone());
        for cell in &mut prepaint.cells {
            if let Some(background) = cell.background.take() {
                window.paint_quad(background);
            }
            if let Some(line) = cell.line.take() {
                line.paint(cell.origin, prepaint.cell_height, window, cx)
                    .ok();
            }
        }
        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }
    }
}

fn empty_prepaint(
    bounds: Bounds<Pixels>,
    background: Hsla,
    _cell_width: Pixels,
    cell_height: Pixels,
) -> PrepaintState {
    PrepaintState {
        background_quad: fill(bounds, background),
        cell_height,
        cells: Vec::new(),
        cursor: None,
    }
}

fn dimensions_for_bounds(
    bounds: Bounds<Pixels>,
    cell_width: Pixels,
    cell_height: Pixels,
) -> TerminalDimensions {
    let width: f32 = bounds.size.width.into();
    let height: f32 = bounds.size.height.into();
    let cell_width_f: f32 = cell_width.into();
    let cell_height_f: f32 = cell_height.into();
    TerminalDimensions::new(
        (width / cell_width_f)
            .floor()
            .max(1.0)
            .min(f32::from(u16::MAX)) as u16,
        (height / cell_height_f)
            .floor()
            .max(1.0)
            .min(f32::from(u16::MAX)) as u16,
        cell_width_f.ceil().max(1.0).min(u32::MAX as f32) as u32,
        cell_height_f.ceil().max(1.0).min(u32::MAX as f32) as u32,
    )
}

fn rgb_to_hsla(color: RgbColor) -> Hsla {
    gpui::rgba(
        (u32::from(color.r) << 24) | (u32::from(color.g) << 16) | (u32::from(color.b) << 8) | 0xff,
    )
    .into()
}
