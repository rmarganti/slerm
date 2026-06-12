use std::time::Instant;

use gpui::{
    App, Bounds, Element, ElementId, Entity, GlobalElementId, Hsla, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PaintQuad, Pixels, RenderOnce, ScrollDelta, ScrollWheelEvent, ShapedLine, Style, TextRun,
    Window, div, fill, point, prelude::*, px, relative, size,
};
use libghostty_vt::{key, mouse, render::CursorVisualStyle, style::RgbColor};

use crate::{
    perf::TerminalFramePerf,
    runtime::TerminalRuntimeService,
    terminal::{
        font::TerminalFontSelection,
        surface::{
            TerminalDimensions, TerminalKeyAction, TerminalKeyInput, TerminalMouseAction,
            TerminalMouseInput, TerminalScrollInput,
        },
    },
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
    runs: Vec<PaintedRun>,
    cursor: Option<PaintQuad>,
}

struct PaintedRun {
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
        let prepaint_started_at = Instant::now();
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
        let cell_width = probe.width.max(px(1.0));
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

        let mut runs = Vec::new();
        let mut cursor = None;
        let mut background: Hsla = theme.bg.into();
        let mut frame_perf = TerminalFramePerf {
            shape_line_calls: 1,
            ..TerminalFramePerf::default()
        };
        let terminal_id = spec.id;
        self.runtime.update(cx, |runtime, _cx| {
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
            frame_perf.drain = runtime.last_drain_perf();
            if active_live_ready && let Some(live) = runtime.live_terminal_mut(terminal_id) {
                let snapshot_started_at = Instant::now();
                match live.surface.render_snapshot() {
                    Ok(snapshot) => {
                        frame_perf.snapshot_duration = snapshot_started_at.elapsed();
                        frame_perf.rows_considered = usize::from(snapshot.rows);
                        frame_perf.cells_considered =
                            usize::from(snapshot.rows) * usize::from(snapshot.columns);
                        frame_perf.render_items =
                            snapshot.row_runs.iter().map(|row| row.runs.len()).sum();
                        background = rgb_to_hsla(snapshot.colors.background);
                        for row in snapshot.row_runs {
                            let y = bounds.top() + cell_height * f32::from(row.y);
                            for run in row.runs {
                                let mut foreground = run.foreground;
                                let mut background_color = run.background;
                                if run.inverse {
                                    let original_foreground = foreground;
                                    foreground =
                                        background_color.unwrap_or(snapshot.colors.background);
                                    background_color = Some(original_foreground);
                                }
                                let x = bounds.left() + cell_width * f32::from(run.x);
                                let width = cell_width * f32::from(run.cells);
                                if let Some(color) = background_color {
                                    runs.push(PaintedRun {
                                        background: Some(fill(
                                            Bounds::new(point(x, y), size(width, cell_height)),
                                            rgb_to_hsla(color),
                                        )),
                                        line: None,
                                        origin: point(x, y),
                                    });
                                }

                                append_text_cells_to_prepaint(
                                    &mut runs,
                                    run.x,
                                    &run.text,
                                    font.clone(),
                                    rgb_to_hsla(foreground),
                                    bounds.left(),
                                    y,
                                    cell_width,
                                    font_size,
                                    window,
                                    &mut frame_perf,
                                );
                            }
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
        });
        frame_perf.prepaint_duration = prepaint_started_at.elapsed();
        frame_perf.log_if_enabled();

        PrepaintState {
            background_quad: fill(bounds, background),
            cell_height,
            runs,
            cursor,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_quad(prepaint.background_quad.clone());
        for run in &mut prepaint.runs {
            if let Some(background) = run.background.take() {
                window.paint_quad(background);
            }
            if let Some(line) = run.line.take() {
                line.paint(run.origin, prepaint.cell_height, window, cx)
                    .ok();
            }
        }
        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }

        register_terminal_input_handlers(
            self.workspace.clone(),
            self.runtime.clone(),
            bounds,
            prepaint.cell_height,
            window,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_text_cells_to_prepaint(
    painted_runs: &mut Vec<PaintedRun>,
    x: u16,
    text: &str,
    font: gpui::Font,
    color: Hsla,
    bounds_left: Pixels,
    y: Pixels,
    cell_width: Pixels,
    font_size: Pixels,
    window: &mut Window,
    frame_perf: &mut TerminalFramePerf,
) {
    if text
        .chars()
        .all(|character| character == ' ' || character.is_control())
    {
        return;
    }

    let origin = point(bounds_left + cell_width * f32::from(x), y);
    frame_perf.shape_line_calls += 1;
    painted_runs.push(PaintedRun {
        background: None,
        line: Some(window.text_system().shape_line(
            text.to_string().into(),
            font_size,
            &[TextRun {
                len: text.len(),
                font,
                color,
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        )),
        origin,
    });
}

fn register_terminal_input_handlers(
    workspace: Entity<WorkspaceState>,
    runtime: Entity<TerminalRuntimeService>,
    bounds: Bounds<Pixels>,
    cell_height: Pixels,
    window: &mut Window,
) {
    let key_workspace = workspace.clone();
    let key_runtime = runtime.clone();
    window.on_key_event(move |event: &KeyDownEvent, phase, window, cx| {
        if !phase.bubble() {
            return;
        }
        let Some(terminal_id) = active_terminal_id(&key_workspace, cx) else {
            return;
        };
        let action = if event.is_held {
            TerminalKeyAction::Repeat
        } else {
            TerminalKeyAction::Press
        };
        let Some(input) = key_input_from_keystroke(&event.keystroke, action) else {
            return;
        };
        if key_runtime.update(cx, |runtime, _| runtime.write_key_input(terminal_id, input)) {
            cx.stop_propagation();
            window.refresh();
        }
    });

    let key_up_workspace = workspace.clone();
    let key_up_runtime = runtime.clone();
    window.on_key_event(move |event: &KeyUpEvent, phase, window, cx| {
        if !phase.bubble() {
            return;
        }
        let Some(terminal_id) = active_terminal_id(&key_up_workspace, cx) else {
            return;
        };
        let Some(input) = key_input_from_keystroke(&event.keystroke, TerminalKeyAction::Release)
        else {
            return;
        };
        if key_up_runtime.update(cx, |runtime, _| runtime.write_key_input(terminal_id, input)) {
            cx.stop_propagation();
            window.refresh();
        }
    });

    let mouse_down_workspace = workspace.clone();
    let mouse_down_runtime = runtime.clone();
    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
        if !phase.bubble() || !point_in_bounds(event.position, bounds) {
            return;
        }
        let Some(terminal_id) = active_terminal_id(&mouse_down_workspace, cx) else {
            return;
        };
        let Some(button) = mouse_button(event.button) else {
            return;
        };
        let input = mouse_input(
            TerminalMouseAction::Press,
            Some(button),
            event.position,
            event.modifiers,
            bounds,
            true,
        );
        if mouse_down_runtime.update(cx, |runtime, _| {
            runtime.write_mouse_input(terminal_id, input)
        }) {
            cx.stop_propagation();
            window.refresh();
        }
    });

    let mouse_up_workspace = workspace.clone();
    let mouse_up_runtime = runtime.clone();
    window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
        if !phase.bubble() || !point_in_bounds(event.position, bounds) {
            return;
        }
        let Some(terminal_id) = active_terminal_id(&mouse_up_workspace, cx) else {
            return;
        };
        let Some(button) = mouse_button(event.button) else {
            return;
        };
        let input = mouse_input(
            TerminalMouseAction::Release,
            Some(button),
            event.position,
            event.modifiers,
            bounds,
            false,
        );
        if mouse_up_runtime.update(cx, |runtime, _| {
            runtime.write_mouse_input(terminal_id, input)
        }) {
            cx.stop_propagation();
            window.refresh();
        }
    });

    let mouse_move_workspace = workspace.clone();
    let mouse_move_runtime = runtime.clone();
    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
        let any_button_pressed = event.pressed_button.is_some();
        if !phase.bubble() || (!point_in_bounds(event.position, bounds) && !any_button_pressed) {
            return;
        }
        let Some(terminal_id) = active_terminal_id(&mouse_move_workspace, cx) else {
            return;
        };
        let input = mouse_input(
            TerminalMouseAction::Motion,
            event.pressed_button.and_then(mouse_button),
            event.position,
            event.modifiers,
            bounds,
            any_button_pressed,
        );
        if mouse_move_runtime.update(cx, |runtime, _| {
            runtime.write_mouse_input(terminal_id, input)
        }) {
            cx.stop_propagation();
            window.refresh();
        }
    });

    let scroll_workspace = workspace;
    let scroll_runtime = runtime;
    window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
        if !phase.bubble() || !point_in_bounds(event.position, bounds) {
            return;
        }
        let Some(terminal_id) = active_terminal_id(&scroll_workspace, cx) else {
            return;
        };
        let input = scroll_input(event, bounds, cell_height);
        if scroll_runtime.update(cx, |runtime, _| {
            runtime.handle_scroll_input(terminal_id, input)
        }) {
            cx.stop_propagation();
            window.refresh();
        }
    });
}

fn active_terminal_id(
    workspace: &Entity<WorkspaceState>,
    cx: &App,
) -> Option<crate::terminal::TerminalId> {
    workspace
        .read(cx)
        .active_project()
        .and_then(|project| project.active_terminal())
        .map(|terminal| terminal.id)
}

pub(crate) fn key_input_from_keystroke(
    keystroke: &Keystroke,
    action: TerminalKeyAction,
) -> Option<TerminalKeyInput> {
    if keystroke.modifiers.platform {
        return None;
    }
    let key = map_key(&keystroke.key)?;
    let mods = key_mods(keystroke.modifiers);
    let utf8 = if action == TerminalKeyAction::Press || action == TerminalKeyAction::Repeat {
        keystroke.key_char.clone()
    } else {
        None
    };
    let consumed_mods = if utf8.is_some() && keystroke.modifiers.shift {
        key::Mods::SHIFT
    } else {
        key::Mods::empty()
    };
    Some(TerminalKeyInput {
        action,
        key,
        mods,
        consumed_mods,
        unshifted_codepoint: unshifted_codepoint(&keystroke.key),
        utf8,
    })
}

fn map_key(key_name: &str) -> Option<key::Key> {
    Some(match key_name {
        "a" | "A" => key::Key::A,
        "b" | "B" => key::Key::B,
        "c" | "C" => key::Key::C,
        "d" | "D" => key::Key::D,
        "e" | "E" => key::Key::E,
        "f" | "F" => key::Key::F,
        "g" | "G" => key::Key::G,
        "h" | "H" => key::Key::H,
        "i" | "I" => key::Key::I,
        "j" | "J" => key::Key::J,
        "k" | "K" => key::Key::K,
        "l" | "L" => key::Key::L,
        "m" | "M" => key::Key::M,
        "n" | "N" => key::Key::N,
        "o" | "O" => key::Key::O,
        "p" | "P" => key::Key::P,
        "q" | "Q" => key::Key::Q,
        "r" | "R" => key::Key::R,
        "s" | "S" => key::Key::S,
        "t" | "T" => key::Key::T,
        "u" | "U" => key::Key::U,
        "v" | "V" => key::Key::V,
        "w" | "W" => key::Key::W,
        "x" | "X" => key::Key::X,
        "y" | "Y" => key::Key::Y,
        "z" | "Z" => key::Key::Z,
        "0" => key::Key::Digit0,
        "1" => key::Key::Digit1,
        "2" => key::Key::Digit2,
        "3" => key::Key::Digit3,
        "4" => key::Key::Digit4,
        "5" => key::Key::Digit5,
        "6" => key::Key::Digit6,
        "7" => key::Key::Digit7,
        "8" => key::Key::Digit8,
        "9" => key::Key::Digit9,
        "-" | "_" | "minus" => key::Key::Minus,
        "=" | "+" | "equal" => key::Key::Equal,
        "[" | "{" | "leftbracket" => key::Key::BracketLeft,
        "]" | "}" | "rightbracket" => key::Key::BracketRight,
        "\\" | "|" | "backslash" => key::Key::Backslash,
        ";" | ":" | "semicolon" => key::Key::Semicolon,
        "'" | "\"" | "quote" => key::Key::Quote,
        "," | "<" | "comma" => key::Key::Comma,
        "." | ">" | "period" => key::Key::Period,
        "/" | "?" | "slash" => key::Key::Slash,
        "`" | "~" | "backquote" => key::Key::Backquote,
        "space" => key::Key::Space,
        "enter" => key::Key::Enter,
        "tab" => key::Key::Tab,
        "backspace" => key::Key::Backspace,
        "delete" => key::Key::Delete,
        "escape" => key::Key::Escape,
        "left" => key::Key::ArrowLeft,
        "right" => key::Key::ArrowRight,
        "up" => key::Key::ArrowUp,
        "down" => key::Key::ArrowDown,
        "home" => key::Key::Home,
        "end" => key::Key::End,
        "pageup" => key::Key::PageUp,
        "pagedown" => key::Key::PageDown,
        "insert" => key::Key::Insert,
        "f1" => key::Key::F1,
        "f2" => key::Key::F2,
        "f3" => key::Key::F3,
        "f4" => key::Key::F4,
        "f5" => key::Key::F5,
        "f6" => key::Key::F6,
        "f7" => key::Key::F7,
        "f8" => key::Key::F8,
        "f9" => key::Key::F9,
        "f10" => key::Key::F10,
        "f11" => key::Key::F11,
        "f12" => key::Key::F12,
        _ => return None,
    })
}

fn unshifted_codepoint(key_name: &str) -> Option<char> {
    Some(match key_name {
        "space" => ' ',
        "_" => '-',
        "+" => '=',
        "{" => '[',
        "}" => ']',
        "|" => '\\',
        ":" => ';',
        "\"" => '\'',
        "<" => ',',
        ">" => '.',
        "?" => '/',
        "~" => '`',
        key if key.chars().count() == 1 => key.chars().next()?,
        _ => return None,
    })
}

fn key_mods(modifiers: gpui::Modifiers) -> key::Mods {
    let mut mods = key::Mods::empty();
    if modifiers.shift {
        mods |= key::Mods::SHIFT;
    }
    if modifiers.alt {
        mods |= key::Mods::ALT;
    }
    if modifiers.control {
        mods |= key::Mods::CTRL;
    }
    if modifiers.platform {
        mods |= key::Mods::SUPER;
    }
    mods
}

fn mouse_button(button: MouseButton) -> Option<mouse::Button> {
    match button {
        MouseButton::Left => Some(mouse::Button::Left),
        MouseButton::Right => Some(mouse::Button::Right),
        MouseButton::Middle => Some(mouse::Button::Middle),
        MouseButton::Navigate(_) => None,
    }
}

fn mouse_input(
    action: TerminalMouseAction,
    button: Option<mouse::Button>,
    position: gpui::Point<Pixels>,
    modifiers: gpui::Modifiers,
    bounds: Bounds<Pixels>,
    any_button_pressed: bool,
) -> TerminalMouseInput {
    let (x_px, y_px) = local_position(position, bounds);
    let (screen_width_px, screen_height_px) = bounds_size_px(bounds);
    TerminalMouseInput {
        action,
        button,
        mods: key_mods(modifiers),
        x_px,
        y_px,
        screen_width_px,
        screen_height_px,
        any_button_pressed,
    }
}

fn scroll_input(
    event: &ScrollWheelEvent,
    bounds: Bounds<Pixels>,
    cell_height: Pixels,
) -> TerminalScrollInput {
    let (x_px, y_px) = local_position(event.position, bounds);
    let (screen_width_px, screen_height_px) = bounds_size_px(bounds);
    let delta_rows = match event.delta {
        ScrollDelta::Pixels(delta) => {
            let dy: f32 = delta.y.into();
            let line_height: f32 = cell_height.into();
            (dy / line_height.max(1.0)).round() as isize
        }
        ScrollDelta::Lines(delta) => delta.y.round() as isize,
    };
    TerminalScrollInput {
        x_px,
        y_px,
        delta_rows,
        mods: key_mods(event.modifiers),
        screen_width_px,
        screen_height_px,
        any_button_pressed: false,
    }
}

fn local_position(position: gpui::Point<Pixels>, bounds: Bounds<Pixels>) -> (f32, f32) {
    let x: f32 = (position.x - bounds.left()).into();
    let y: f32 = (position.y - bounds.top()).into();
    (x.max(0.0), y.max(0.0))
}

fn bounds_size_px(bounds: Bounds<Pixels>) -> (u32, u32) {
    let width: f32 = bounds.size.width.into();
    let height: f32 = bounds.size.height.into();
    (
        width.ceil().max(1.0).min(u32::MAX as f32) as u32,
        height.ceil().max(1.0).min(u32::MAX as f32) as u32,
    )
}

fn point_in_bounds(position: gpui::Point<Pixels>, bounds: Bounds<Pixels>) -> bool {
    position.x >= bounds.left()
        && position.x <= bounds.right()
        && position.y >= bounds.top()
        && position.y <= bounds.bottom()
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
        runs: Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_modified_key_is_not_terminal_input() {
        let keystroke = Keystroke {
            modifiers: gpui::Modifiers {
                platform: true,
                ..Default::default()
            },
            key: "q".into(),
            key_char: None,
        };

        assert!(key_input_from_keystroke(&keystroke, TerminalKeyAction::Press).is_none());
    }

    #[test]
    fn control_key_is_terminal_input() {
        let keystroke = Keystroke {
            modifiers: gpui::Modifiers {
                control: true,
                ..Default::default()
            },
            key: "c".into(),
            key_char: None,
        };

        let input = key_input_from_keystroke(&keystroke, TerminalKeyAction::Press)
            .expect("ctrl-c maps to terminal input");
        assert_eq!(input.key, key::Key::C);
        assert!(input.mods.contains(key::Mods::CTRL));
    }

    #[test]
    fn shifted_punctuation_maps_to_physical_key_with_unshifted_codepoint() {
        let keystroke = Keystroke {
            modifiers: gpui::Modifiers {
                shift: true,
                ..Default::default()
            },
            key: ":".into(),
            key_char: Some(":".into()),
        };

        let input = key_input_from_keystroke(&keystroke, TerminalKeyAction::Press)
            .expect("colon maps to terminal input");
        assert_eq!(input.key, key::Key::Semicolon);
        assert_eq!(input.unshifted_codepoint, Some(';'));
        assert_eq!(input.utf8.as_deref(), Some(":"));
        assert!(input.consumed_mods.contains(key::Mods::SHIFT));
    }
}
