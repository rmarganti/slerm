use std::{fmt::Debug, io, path::Path};

use crate::{
    runtime::{Pty, PtySize, SessionId, pty::spawn_pty},
    terminal::{ProcessSpec, TerminalId},
};

/// Non-blocking live PTY operations used by terminal surfaces.
///
/// This trait is deliberately small and monomorphized through
/// `TerminalRuntimeService<P>` so tests can use deterministic in-memory PTYs
/// without adding dynamic dispatch overhead to the real UI/read path.
pub trait LivePty: Debug {
    fn read_available(&self, buf: &mut [u8]) -> io::Result<usize>;

    fn write_nonblocking(&self, bytes: &[u8]) -> io::Result<usize>;

    fn resize(&self, size: PtySize) -> io::Result<()>;

    fn terminate(&self) -> io::Result<()>;
}

impl LivePty for Pty {
    fn read_available(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_available(buf)
    }

    fn write_nonblocking(&self, bytes: &[u8]) -> io::Result<usize> {
        self.write_nonblocking(bytes)
    }

    fn resize(&self, size: PtySize) -> io::Result<()> {
        self.resize(size)
    }

    fn terminate(&self) -> io::Result<()> {
        self.terminate()
    }
}

/// Factory seam for creating live PTYs.
///
/// Keeping spawning behind this trait lets Phase 3 exercise multi-terminal
/// lifecycle/drain/switching behavior with a fake PTY while production keeps a
/// concrete Unix implementation.
pub trait LiveTerminalSpawner {
    type Pty: LivePty;

    fn spawn_live_terminal(
        &mut self,
        terminal_id: TerminalId,
        session_id: SessionId,
        process: &ProcessSpec,
        cwd: &Path,
        size: PtySize,
    ) -> anyhow::Result<Self::Pty>;
}

#[derive(Debug, Default)]
pub struct UnixLiveTerminalSpawner;

impl LiveTerminalSpawner for UnixLiveTerminalSpawner {
    type Pty = Pty;

    fn spawn_live_terminal(
        &mut self,
        terminal_id: TerminalId,
        session_id: SessionId,
        process: &ProcessSpec,
        cwd: &Path,
        size: PtySize,
    ) -> anyhow::Result<Self::Pty> {
        spawn_pty(terminal_id, session_id, process, cwd, size)
    }
}
