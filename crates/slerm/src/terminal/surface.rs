#![allow(
    dead_code,
    reason = "phase 1 terminal surface skeleton is wired in later phases"
)]

use std::{cell::RefCell, rc::Rc};

use libghostty_vt::{
    RenderState, Terminal, TerminalOptions, key, mouse,
    render::{CellIterator, CursorVisualStyle, Dirty, RowIterator, Snapshot},
    style::RgbColor,
    terminal::{
        ConformanceLevel, DeviceAttributeFeature, DeviceAttributes, DeviceType,
        PrimaryDeviceAttributes, SecondaryDeviceAttributes, SizeReportSize,
        TertiaryDeviceAttributes,
    },
};

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
    pub cells: Vec<TerminalRenderCell>,
    pub cursor: Option<TerminalRenderCursor>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalRenderColors {
    pub foreground: RgbColor,
    pub background: RgbColor,
    pub cursor: Option<RgbColor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalRenderCell {
    pub x: u16,
    pub y: u16,
    pub text: String,
    pub foreground: RgbColor,
    pub background: Option<RgbColor>,
    pub bold: bool,
    pub inverse: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalRenderCursor {
    pub x: u16,
    pub y: u16,
    pub style: CursorVisualStyle,
}

#[derive(Debug)]
pub struct GhosttyTerminalSurface {
    terminal: Terminal<'static, 'static>,
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

        let mut terminal = Terminal::new(TerminalOptions {
            cols: dimensions.borrow().columns,
            rows: dimensions.borrow().rows,
            max_scrollback: 1000,
        })?;
        // TerminalOptions carries cell dimensions only; resize also informs
        // libghostty of per-cell pixel dimensions used for size reports and
        // future image protocols.
        terminal.resize(
            dimensions.borrow().columns,
            dimensions.borrow().rows,
            dimensions.borrow().cell_width_px,
            dimensions.borrow().cell_height_px,
        )?;

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
            self.terminal.vt_write(bytes);
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

        let mut cells = Vec::new();
        let mut row_index = 0;
        let mut rows_iter = self.row_it.update(&snapshot)?;
        while let Some(row) = rows_iter.next() {
            let mut col_index = 0;
            let mut cell_iter = self.cell_it.update(row)?;
            while let Some(cell) = cell_iter.next() {
                let graphemes = cell.graphemes()?;
                let text = graphemes.into_iter().collect::<String>();
                let style = cell.style()?;
                let foreground = cell.fg_color()?.unwrap_or(colors.foreground);
                let background = cell.bg_color()?;
                if !text.is_empty() || background.is_some() {
                    cells.push(TerminalRenderCell {
                        x: col_index,
                        y: row_index,
                        text,
                        foreground,
                        background,
                        bold: style.bold,
                        inverse: style.inverse,
                    });
                }
                col_index += 1;
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
            cells,
            cursor,
        })
    }

    /// Drain terminal responses generated by libghostty effects. The caller
    /// should write these bytes back to the PTY.
    pub fn take_pending_pty_writes(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut *self.pending_pty_writes.borrow_mut())
    }
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
