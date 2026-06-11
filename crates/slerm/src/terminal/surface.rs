#![allow(
    dead_code,
    reason = "phase 1 terminal surface skeleton is wired in later phases"
)]

use std::{borrow::Cow, cell::RefCell, fmt::Write as _, rc::Rc};

use libghostty_vt::{
    RenderState, Terminal, TerminalOptions, key, mouse,
    render::{CellIterator, CursorVisualStyle, Dirty, RowIterator, Snapshot},
    style::RgbColor,
    terminal::{
        ConformanceLevel, DeviceAttributeFeature, DeviceAttributes, DeviceType,
        PrimaryDeviceAttributes, ScrollViewport, SecondaryDeviceAttributes, SizeReportSize,
        TertiaryDeviceAttributes,
    },
};

use crate::theme::{self, TerminalTheme};

/// Terminal grid and pixel dimensions used by libghostty and the PTY winsize.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalDimensions {
    pub columns: u16,
    pub rows: u16,
    pub cell_width_px: u32,
    pub cell_height_px: u32,
}

impl TerminalDimensions {
    pub const DEFAULT: Self = Self {
        columns: 80,
        rows: 24,
        cell_width_px: 8,
        cell_height_px: 16,
    };

    pub fn new(columns: u16, rows: u16, cell_width_px: u32, cell_height_px: u32) -> Self {
        Self {
            columns: columns.max(1),
            rows: rows.max(1),
            cell_width_px: cell_width_px.max(1),
            cell_height_px: cell_height_px.max(1),
        }
    }

    pub fn size_report(self) -> SizeReportSize {
        SizeReportSize {
            rows: self.rows,
            columns: self.columns,
            cell_width: self.cell_width_px,
            cell_height: self.cell_height_px,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalKeyAction {
    Press,
    Release,
    Repeat,
}

impl From<TerminalKeyAction> for key::Action {
    fn from(value: TerminalKeyAction) -> Self {
        match value {
            TerminalKeyAction::Press => key::Action::Press,
            TerminalKeyAction::Release => key::Action::Release,
            TerminalKeyAction::Repeat => key::Action::Repeat,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalKeyInput {
    pub action: TerminalKeyAction,
    pub key: key::Key,
    pub mods: key::Mods,
    pub consumed_mods: key::Mods,
    pub unshifted_codepoint: Option<char>,
    pub utf8: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalMouseAction {
    Press,
    Release,
    Motion,
}

impl From<TerminalMouseAction> for mouse::Action {
    fn from(value: TerminalMouseAction) -> Self {
        match value {
            TerminalMouseAction::Press => mouse::Action::Press,
            TerminalMouseAction::Release => mouse::Action::Release,
            TerminalMouseAction::Motion => mouse::Action::Motion,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerminalMouseInput {
    pub action: TerminalMouseAction,
    pub button: Option<mouse::Button>,
    pub mods: key::Mods,
    pub x_px: f32,
    pub y_px: f32,
    pub screen_width_px: u32,
    pub screen_height_px: u32,
    pub any_button_pressed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerminalScrollInput {
    pub x_px: f32,
    pub y_px: f32,
    pub delta_rows: isize,
    pub mods: key::Mods,
    pub screen_width_px: u32,
    pub screen_height_px: u32,
    pub any_button_pressed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalScrollOutcome {
    Encoded,
    Scrolled,
    Ignored,
}

/// Reusable libghostty input state. Events and encoders are intentionally kept
/// alive across frames/input events to avoid allocation churn.
#[derive(Debug)]
pub struct GhosttyInputState {
    pub key_encoder: key::Encoder<'static>,
    pub key_event: key::Event<'static>,
    pub mouse_encoder: mouse::Encoder<'static>,
    pub mouse_event: mouse::Event<'static>,
    pub response: Vec<u8>,
}

impl GhosttyInputState {
    pub fn new() -> libghostty_vt::error::Result<Self> {
        Ok(Self {
            key_encoder: key::Encoder::new()?,
            key_event: key::Event::new()?,
            mouse_encoder: mouse::Encoder::new()?,
            mouse_event: mouse::Event::new()?,
            response: Vec::with_capacity(128),
        })
    }

    pub fn sync_from_terminal(&mut self, terminal: &Terminal<'static, 'static>) {
        self.key_encoder.set_options_from_terminal(terminal);
        self.mouse_encoder.set_options_from_terminal(terminal);
    }
}

/// Slerm's live terminal surface backed by libghostty.
#[derive(Debug)]
pub struct TerminalRenderSnapshot {
    pub columns: u16,
    pub rows: u16,
    pub colors: TerminalRenderColors,
    pub row_runs: Vec<TerminalRenderRow>,
    pub cursor: Option<TerminalRenderCursor>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalRenderColors {
    pub foreground: RgbColor,
    pub background: RgbColor,
    pub cursor: Option<RgbColor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalRenderRow {
    pub y: u16,
    pub runs: Vec<TerminalRenderRun>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalRenderRun {
    pub x: u16,
    pub cells: u16,
    pub text: String,
    pub foreground: RgbColor,
    pub background: Option<RgbColor>,
    pub bold: bool,
    pub inverse: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TerminalRunStyle {
    foreground: RgbColor,
    background: Option<RgbColor>,
    bold: bool,
    inverse: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalRenderCursor {
    pub x: u16,
    pub y: u16,
    pub style: CursorVisualStyle,
}

#[derive(Debug)]
pub struct GhosttyTerminalSurface {
    terminal: Box<Terminal<'static, 'static>>,
    render_state: RenderState<'static>,
    row_it: RowIterator<'static>,
    cell_it: CellIterator<'static>,
    input: GhosttyInputState,
    dimensions: Rc<RefCell<TerminalDimensions>>,
    pending_pty_writes: Rc<RefCell<Vec<Vec<u8>>>>,
    dirty: bool,
}

impl GhosttyTerminalSurface {
    pub fn new(dimensions: TerminalDimensions) -> libghostty_vt::error::Result<Self> {
        let dimensions = Rc::new(RefCell::new(dimensions));
        let pending_pty_writes = Rc::new(RefCell::new(Vec::new()));

        let mut terminal = Box::new(Terminal::new(TerminalOptions {
            cols: dimensions.borrow().columns,
            rows: dimensions.borrow().rows,
            max_scrollback: 1000,
        })?);
        apply_theme(&mut terminal, theme::active().terminal);
        // TerminalOptions carries cell dimensions only; resize also informs
        // libghostty of per-cell pixel dimensions used for size reports and
        // future image protocols.
        terminal.resize(
            dimensions.borrow().columns,
            dimensions.borrow().rows,
            dimensions.borrow().cell_width_px,
            dimensions.borrow().cell_height_px,
        )?;

        // libghostty stores callback userdata as a pointer to the Terminal's
        // internal vtable. Keep the Terminal boxed before registering effects
        // so moving GhosttyTerminalSurface does not invalidate that pointer.
        register_effects(
            &mut terminal,
            dimensions.clone(),
            pending_pty_writes.clone(),
        )?;

        Ok(Self {
            terminal,
            render_state: RenderState::new()?,
            row_it: RowIterator::new()?,
            cell_it: CellIterator::new()?,
            input: GhosttyInputState::new()?,
            dimensions,
            pending_pty_writes,
            dirty: true,
        })
    }

    pub fn terminal(&self) -> &Terminal<'static, 'static> {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<'static, 'static> {
        &mut self.terminal
    }

    pub fn input_mut(&mut self) -> &mut GhosttyInputState {
        &mut self.input
    }

    pub fn row_iterator_mut(&mut self) -> &mut RowIterator<'static> {
        &mut self.row_it
    }

    pub fn cell_iterator_mut(&mut self) -> &mut CellIterator<'static> {
        &mut self.cell_it
    }

    pub fn dimensions(&self) -> TerminalDimensions {
        *self.dimensions.borrow()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn vt_write(&mut self, bytes: &[u8]) {
        if !bytes.is_empty() {
            let bytes = filter_unsupported_vt_modes(bytes);
            if !bytes.is_empty() {
                self.terminal.vt_write(&bytes);
            }
            self.dirty = true;
        }
    }

    pub fn resize(&mut self, dimensions: TerminalDimensions) -> libghostty_vt::error::Result<()> {
        *self.dimensions.borrow_mut() = dimensions;
        self.terminal.resize(
            dimensions.columns,
            dimensions.rows,
            dimensions.cell_width_px,
            dimensions.cell_height_px,
        )?;
        self.dirty = true;
        Ok(())
    }

    pub fn update_render_state(&mut self) -> libghostty_vt::error::Result<Snapshot<'static, '_>> {
        self.render_state.update(&self.terminal)
    }

    pub fn encode_key_input(
        &mut self,
        key_input: TerminalKeyInput,
    ) -> libghostty_vt::error::Result<&[u8]> {
        self.input.response.clear();
        self.input.sync_from_terminal(&self.terminal);
        self.input
            .key_event
            .set_action(key_input.action.into())
            .set_key(key_input.key)
            .set_mods(key_input.mods)
            .set_consumed_mods(key_input.consumed_mods)
            .set_utf8(key_input.utf8);
        self.input
            .key_event
            .set_unshifted_codepoint(key_input.unshifted_codepoint.unwrap_or('\0'));
        self.input
            .key_encoder
            .encode_to_vec(&self.input.key_event, &mut self.input.response)?;
        Ok(&self.input.response)
    }

    pub fn encode_mouse_input(
        &mut self,
        mouse_input: TerminalMouseInput,
    ) -> libghostty_vt::error::Result<&[u8]> {
        self.input.response.clear();
        self.input.sync_from_terminal(&self.terminal);
        let dimensions = self.dimensions();
        self.input
            .mouse_encoder
            .set_size(mouse::EncoderSize {
                screen_width: mouse_input.screen_width_px.max(1),
                screen_height: mouse_input.screen_height_px.max(1),
                cell_width: dimensions.cell_width_px,
                cell_height: dimensions.cell_height_px,
                padding_top: 0,
                padding_bottom: 0,
                padding_right: 0,
                padding_left: 0,
            })
            .set_any_button_pressed(mouse_input.any_button_pressed)
            .set_track_last_cell(true);
        self.input
            .mouse_event
            .set_action(mouse_input.action.into())
            .set_button(mouse_input.button)
            .set_mods(mouse_input.mods)
            .set_position(mouse::Position {
                x: mouse_input.x_px.max(0.0),
                y: mouse_input.y_px.max(0.0),
            });
        self.input
            .mouse_encoder
            .encode_to_vec(&self.input.mouse_event, &mut self.input.response)?;
        Ok(&self.input.response)
    }

    pub fn handle_scroll_input(
        &mut self,
        scroll_input: TerminalScrollInput,
    ) -> libghostty_vt::error::Result<TerminalScrollOutcome> {
        self.input.response.clear();
        if scroll_input.delta_rows == 0 {
            return Ok(TerminalScrollOutcome::Ignored);
        }
        if self.terminal.is_mouse_tracking()? {
            let button = if scroll_input.delta_rows < 0 {
                mouse::Button::Four
            } else {
                mouse::Button::Five
            };
            let press = TerminalMouseInput {
                action: TerminalMouseAction::Press,
                button: Some(button),
                mods: scroll_input.mods,
                x_px: scroll_input.x_px,
                y_px: scroll_input.y_px,
                screen_width_px: scroll_input.screen_width_px,
                screen_height_px: scroll_input.screen_height_px,
                any_button_pressed: scroll_input.any_button_pressed,
            };
            let release = TerminalMouseInput {
                action: TerminalMouseAction::Release,
                any_button_pressed: false,
                ..press
            };
            self.encode_mouse_input(press)?;
            self.input.sync_from_terminal(&self.terminal);
            let dimensions = self.dimensions();
            self.input
                .mouse_encoder
                .set_size(mouse::EncoderSize {
                    screen_width: scroll_input.screen_width_px.max(1),
                    screen_height: scroll_input.screen_height_px.max(1),
                    cell_width: dimensions.cell_width_px,
                    cell_height: dimensions.cell_height_px,
                    padding_top: 0,
                    padding_bottom: 0,
                    padding_right: 0,
                    padding_left: 0,
                })
                .set_any_button_pressed(false)
                .set_track_last_cell(true);
            self.input
                .mouse_event
                .set_action(release.action.into())
                .set_button(release.button)
                .set_mods(release.mods)
                .set_position(mouse::Position {
                    x: release.x_px.max(0.0),
                    y: release.y_px.max(0.0),
                });
            self.input
                .mouse_encoder
                .encode_to_vec(&self.input.mouse_event, &mut self.input.response)?;
            Ok(if self.input.response.is_empty() {
                TerminalScrollOutcome::Ignored
            } else {
                TerminalScrollOutcome::Encoded
            })
        } else {
            self.terminal
                .scroll_viewport(ScrollViewport::Delta(scroll_input.delta_rows));
            self.dirty = true;
            Ok(TerminalScrollOutcome::Scrolled)
        }
    }

    pub fn encoded_input_response(&self) -> &[u8] {
        &self.input.response
    }

    pub fn render_snapshot(&mut self) -> libghostty_vt::error::Result<TerminalRenderSnapshot> {
        let snapshot = self.render_state.update(&self.terminal)?;
        let raw_colors = snapshot.colors()?;
        let colors = TerminalRenderColors {
            foreground: raw_colors.foreground,
            background: raw_colors.background,
            cursor: raw_colors.cursor,
        };
        let columns = snapshot.cols()?;
        let rows = snapshot.rows()?;
        let cursor = if snapshot.cursor_visible()? {
            snapshot
                .cursor_viewport()?
                .map(|cursor| TerminalRenderCursor {
                    x: cursor.x,
                    y: cursor.y,
                    style: snapshot
                        .cursor_visual_style()
                        .unwrap_or(CursorVisualStyle::Block),
                })
        } else {
            None
        };

        let mut row_runs = Vec::new();
        let mut grapheme_buf = Vec::new();
        let mut row_index = 0;
        let mut rows_iter = self.row_it.update(&snapshot)?;
        while let Some(row) = rows_iter.next() {
            let mut runs = Vec::new();
            let mut pending_run = None;
            let mut col_index = 0;
            let mut cell_iter = self.cell_it.update(row)?;
            while let Some(cell) = cell_iter.next() {
                let graphemes_len = cell.graphemes_len()?;
                grapheme_buf.resize(graphemes_len, '\0');
                if graphemes_len > 0 {
                    cell.graphemes_buf(&mut grapheme_buf)?;
                }
                let style = cell.style()?;
                let foreground = cell.fg_color()?.unwrap_or(colors.foreground);
                let background = cell.bg_color()?;
                let has_text = graphemes_len > 0;
                if has_text || background.is_some() {
                    append_render_run(
                        &mut pending_run,
                        &mut runs,
                        col_index,
                        &grapheme_buf,
                        TerminalRunStyle {
                            foreground,
                            background,
                            bold: style.bold,
                            inverse: style.inverse,
                        },
                    );
                } else if let Some(run) = pending_run.take() {
                    runs.push(run);
                }
                col_index += 1;
            }
            if let Some(run) = pending_run.take() {
                runs.push(run);
            }
            if !runs.is_empty() {
                row_runs.push(TerminalRenderRow { y: row_index, runs });
            }
            row.set_dirty(false)?;
            row_index += 1;
        }
        snapshot.set_dirty(Dirty::Clean)?;
        self.dirty = false;

        Ok(TerminalRenderSnapshot {
            columns,
            rows,
            colors,
            row_runs,
            cursor,
        })
    }

    /// Drain terminal responses generated by libghostty effects. The caller
    /// should write these bytes back to the PTY.
    pub fn take_pending_pty_writes(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut *self.pending_pty_writes.borrow_mut())
    }
}

fn append_render_run(
    pending_run: &mut Option<TerminalRenderRun>,
    runs: &mut Vec<TerminalRenderRun>,
    x: u16,
    graphemes: &[char],
    style: TerminalRunStyle,
) {
    let text = graphemes.iter().collect::<String>();
    let can_extend = pending_run.as_ref().is_some_and(|run| {
        run.x + run.cells == x
            && run.foreground == style.foreground
            && run.background == style.background
            && run.bold == style.bold
            && run.inverse == style.inverse
    });

    if !can_extend && let Some(run) = pending_run.take() {
        runs.push(run);
    }

    if let Some(run) = pending_run.as_mut() {
        if text.is_empty() {
            run.text.push(' ');
        } else {
            run.text.push_str(&text);
        }
        run.cells += 1;
    } else {
        *pending_run = Some(TerminalRenderRun {
            x,
            cells: 1,
            text,
            foreground: style.foreground,
            background: style.background,
            bold: style.bold,
            inverse: style.inverse,
        });
    }
}

fn filter_unsupported_vt_modes(bytes: &[u8]) -> Cow<'_, [u8]> {
    let mut output = None::<Vec<u8>>;
    let mut copied_until = 0;
    let mut index = 0;

    while index < bytes.len() {
        if bytes.get(index..index + 3) == Some(b"\x1b[?") {
            let params_start = index + 3;
            let mut final_index = params_start;
            while matches!(bytes.get(final_index), Some(b'0'..=b'9' | b';')) {
                final_index += 1;
            }

            if matches!(bytes.get(final_index), Some(b'h' | b'l'))
                && let Some(filtered_params) =
                    private_mode_params_without_34(&bytes[params_start..final_index])
            {
                let output = output.get_or_insert_with(Vec::new);
                output.extend_from_slice(&bytes[copied_until..index]);
                if !filtered_params.is_empty() {
                    output.extend_from_slice(b"\x1b[?");
                    output.extend_from_slice(&filtered_params);
                    output.push(bytes[final_index]);
                }
                index = final_index + 1;
                copied_until = index;
                continue;
            }
        }

        index += 1;
    }

    match output {
        Some(mut output) => {
            output.extend_from_slice(&bytes[copied_until..]);
            Cow::Owned(output)
        }
        None => Cow::Borrowed(bytes),
    }
}

fn private_mode_params_without_34(params: &[u8]) -> Option<Vec<u8>> {
    let mut filtered = Vec::with_capacity(params.len());
    let mut changed = false;
    let mut start = 0;

    for index in 0..=params.len() {
        if index != params.len() && params[index] != b';' {
            continue;
        }

        let param = &params[start..index];
        if param.is_empty() || !param.iter().all(u8::is_ascii_digit) {
            return None;
        }

        if parse_u32(param) == Some(34) {
            changed = true;
        } else {
            if !filtered.is_empty() {
                filtered.push(b';');
            }
            filtered.extend_from_slice(param);
        }
        start = index + 1;
    }

    changed.then_some(filtered)
}

fn parse_u32(bytes: &[u8]) -> Option<u32> {
    let mut value = 0_u32;
    for byte in bytes {
        value = value.checked_mul(10)?;
        value = value.checked_add(u32::from(byte - b'0'))?;
    }
    Some(value)
}

fn apply_theme(terminal: &mut Terminal<'static, 'static>, theme: TerminalTheme) {
    let mut sequence = String::new();

    for (index, color) in theme.palette.iter().enumerate() {
        let _ = write!(sequence, "\x1b]4;{index};#{color:06x}\x1b\\");
    }
    let _ = write!(sequence, "\x1b]10;#{:06x}\x1b\\", theme.foreground);
    let _ = write!(sequence, "\x1b]11;#{:06x}\x1b\\", theme.background);
    let _ = write!(sequence, "\x1b]12;#{:06x}\x1b\\", theme.cursor);

    terminal.vt_write(sequence.as_bytes());
}

fn register_effects(
    terminal: &mut Terminal<'static, 'static>,
    dimensions: Rc<RefCell<TerminalDimensions>>,
    pending_pty_writes: Rc<RefCell<Vec<Vec<u8>>>>,
) -> libghostty_vt::error::Result<()> {
    terminal
        .on_pty_write(move |_terminal, data| {
            pending_pty_writes.borrow_mut().push(data.to_vec());
        })?
        .on_size(move |_terminal| Some(dimensions.borrow().size_report()))?
        .on_device_attributes(|_terminal| Some(device_attributes()))?
        .on_xtversion(|_terminal| Some("slerm"))?
        .on_color_scheme(|_terminal| None)?
        .on_bell(|_terminal| {})?
        .on_enquiry(|_terminal| Some(""))?
        .on_title_changed(|_terminal| {})?;
    Ok(())
}

fn device_attributes() -> DeviceAttributes {
    DeviceAttributes {
        primary: PrimaryDeviceAttributes::new(
            ConformanceLevel::VT220,
            [
                DeviceAttributeFeature::ANSI_COLOR,
                DeviceAttributeFeature::SELECTIVE_ERASE,
            ],
        ),
        secondary: SecondaryDeviceAttributes {
            device_type: DeviceType::VT220,
            firmware_version: 1,
            rom_cartridge: 0,
        },
        tertiary: TertiaryDeviceAttributes::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface() -> GhosttyTerminalSurface {
        GhosttyTerminalSurface::new(TerminalDimensions::new(20, 5, 8, 16))
            .expect("surface initializes")
    }

    fn rendered_text(snapshot: &TerminalRenderSnapshot) -> String {
        snapshot
            .row_runs
            .iter()
            .flat_map(|row| &row.runs)
            .map(|run| run.text.as_str())
            .collect()
    }

    fn all_runs(snapshot: &TerminalRenderSnapshot) -> Vec<&TerminalRenderRun> {
        snapshot.row_runs.iter().flat_map(|row| &row.runs).collect()
    }

    #[test]
    fn vt_write_plain_text_appears_in_render_snapshot() {
        let mut surface = surface();

        surface.vt_write(b"hello");
        let snapshot = surface.render_snapshot().expect("snapshot renders");

        assert!(rendered_text(&snapshot).contains("hello"));
        assert_eq!(snapshot.columns, 20);
        assert_eq!(snapshot.rows, 5);
    }

    #[test]
    fn vt_write_marks_dirty_and_render_snapshot_clears_it() {
        let mut surface = surface();
        surface.render_snapshot().expect("initial render succeeds");
        assert!(!surface.is_dirty());

        surface.vt_write(b"x");
        assert!(surface.is_dirty());

        surface.render_snapshot().expect("render succeeds");
        assert!(!surface.is_dirty());
    }

    #[test]
    fn resize_updates_dimensions_and_marks_dirty() {
        let mut surface = surface();
        surface.render_snapshot().expect("initial render succeeds");

        let dimensions = TerminalDimensions::new(10, 4, 9, 18);
        surface.resize(dimensions).expect("resize succeeds");

        assert_eq!(surface.dimensions(), dimensions);
        assert!(surface.is_dirty());
        let snapshot = surface.render_snapshot().expect("snapshot renders");
        assert_eq!(snapshot.columns, 10);
        assert_eq!(snapshot.rows, 4);
    }

    #[test]
    fn ansi_styles_are_exposed_on_cells() {
        let mut surface = surface();

        surface.vt_write(b"\x1b[1;7mb\x1b[0m");
        let snapshot = surface.render_snapshot().expect("snapshot renders");
        let styled = all_runs(&snapshot)
            .into_iter()
            .find(|run| run.text == "b")
            .expect("styled run is present");

        assert!(styled.bold);
        assert!(styled.inverse);
    }

    #[test]
    fn plain_contiguous_text_becomes_one_run() {
        let mut surface = surface();

        surface.vt_write(b"hello");
        let snapshot = surface.render_snapshot().expect("snapshot renders");
        let runs = all_runs(&snapshot);

        assert!(runs.iter().any(|run| run.x == 0 && run.text == "hello"));
    }

    #[test]
    fn style_boundary_splits_runs() {
        let mut surface = surface();

        surface.vt_write(b"a\x1b[31mb\x1b[0m");
        let snapshot = surface.render_snapshot().expect("snapshot renders");
        let row = snapshot
            .row_runs
            .iter()
            .find(|row| row.y == 0)
            .expect("first row has runs");

        assert!(row.runs.iter().any(|run| run.text == "a"));
        assert!(row.runs.iter().any(|run| run.text == "b"));
        assert!(row.runs.len() >= 2);
    }

    #[test]
    fn empty_background_cells_are_preserved_as_runs() {
        let foreground = RgbColor { r: 1, g: 2, b: 3 };
        let background = Some(RgbColor { r: 4, g: 5, b: 6 });
        let mut pending = None;
        let mut runs = Vec::new();

        let style = TerminalRunStyle {
            foreground,
            background,
            bold: false,
            inverse: false,
        };
        append_render_run(&mut pending, &mut runs, 2, &[], style);
        append_render_run(&mut pending, &mut runs, 3, &[], style);
        runs.push(pending.take().expect("pending run exists"));

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].x, 2);
        assert_eq!(runs[0].cells, 2);
        assert_eq!(runs[0].text, " ");
        assert_eq!(runs[0].background, background);
    }

    #[test]
    fn unsupported_decrlm_mode_is_filtered_from_vt_input() {
        assert_eq!(filter_unsupported_vt_modes(b"a\x1b[?34hb").as_ref(), b"ab");
        assert_eq!(
            filter_unsupported_vt_modes(b"\x1b[?25;34;1004l").as_ref(),
            b"\x1b[?25;1004l"
        );
        assert_eq!(
            filter_unsupported_vt_modes(b"\x1b[?34;25h").as_ref(),
            b"\x1b[?25h"
        );
    }

    #[test]
    fn device_attribute_query_buffers_pty_response() {
        let mut surface = surface();

        surface.vt_write(b"\x1b[c");
        let responses = surface.take_pending_pty_writes();

        assert!(
            responses.iter().any(|response| !response.is_empty()),
            "expected libghostty to generate a device-attribute response"
        );
    }

    #[test]
    fn key_encoder_produces_shell_input_bytes() {
        let mut surface = surface();

        let bytes = surface
            .encode_key_input(TerminalKeyInput {
                action: TerminalKeyAction::Press,
                key: key::Key::C,
                mods: key::Mods::CTRL,
                consumed_mods: key::Mods::empty(),
                unshifted_codepoint: Some('c'),
                utf8: None,
            })
            .expect("key encodes");

        assert_eq!(bytes, b"\x03");
    }

    #[test]
    fn arrow_key_encoder_produces_escape_sequence() {
        let mut surface = surface();

        let bytes = surface
            .encode_key_input(TerminalKeyInput {
                action: TerminalKeyAction::Press,
                key: key::Key::ArrowUp,
                mods: key::Mods::empty(),
                consumed_mods: key::Mods::empty(),
                unshifted_codepoint: None,
                utf8: None,
            })
            .expect("key encodes");

        assert!(!bytes.is_empty());
    }

    #[test]
    fn wheel_without_mouse_tracking_scrolls_viewport_and_marks_dirty() {
        let mut surface = surface();
        surface.render_snapshot().expect("initial render succeeds");
        assert!(!surface.is_dirty());

        let outcome = surface
            .handle_scroll_input(TerminalScrollInput {
                x_px: 0.0,
                y_px: 0.0,
                delta_rows: -1,
                mods: key::Mods::empty(),
                screen_width_px: 160,
                screen_height_px: 80,
                any_button_pressed: false,
            })
            .expect("scroll handled");

        assert_eq!(outcome, TerminalScrollOutcome::Scrolled);
        assert!(surface.is_dirty());
    }
}
