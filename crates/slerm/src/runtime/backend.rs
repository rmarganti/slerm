use std::path::PathBuf;

use crate::{
    runtime::{SessionId, TerminalSession},
    terminal::{ProcessSpec, TerminalId, TerminalSpec},
};

/// Minimal PTY/process backend seam.
///
/// Real PTY and libghostty integration should attach behind this trait later.
/// This trait intentionally models process/session control only; terminal
/// rendering, scrollback, grid state, damage tracking, and agent-output parsing
/// should be introduced with the libghostty integration instead of here.
pub trait PtyBackend {
    fn spawn(&mut self, request: SpawnProcessRequest) -> anyhow::Result<TerminalSession>;

    fn kill(&mut self, session_id: SessionId) -> anyhow::Result<()>;

    fn resize(&mut self, session_id: SessionId, size: TerminalSize) -> anyhow::Result<()>;

    fn write(&mut self, session_id: SessionId, bytes: &[u8]) -> anyhow::Result<()>;
}

/// Backend spawn request built from a persisted terminal spec and desired size.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnProcessRequest {
    pub terminal_id: TerminalId,
    pub process: ProcessSpec,
    pub cwd: PathBuf,
    pub initial_size: TerminalSize,
}

impl SpawnProcessRequest {
    pub fn from_spec(spec: &TerminalSpec, initial_size: TerminalSize) -> Self {
        Self {
            terminal_id: spec.id,
            process: spec.command.clone(),
            cwd: spec.cwd.clone(),
            initial_size,
        }
    }
}

/// Terminal grid dimensions in character cells.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalSize {
    pub columns: u16,
    pub rows: u16,
}

impl TerminalSize {
    pub const DEFAULT: Self = Self {
        columns: 80,
        rows: 24,
    };

    pub fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }
}
