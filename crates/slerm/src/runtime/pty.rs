use std::{collections::BTreeMap, ffi::CString, io, os::fd::RawFd, path::Path, time::SystemTime};

use anyhow::Context;

use crate::{
    runtime::{PtyBackend, SessionId, SpawnProcessRequest, TerminalSession, TerminalSize},
    terminal::{ProcessSpec, TerminalId, surface::TerminalDimensions},
};

/// Pixel-aware PTY size used by the live terminal surface integration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PtySize {
    pub columns: u16,
    pub rows: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl PtySize {
    pub fn from_dimensions(dimensions: TerminalDimensions) -> Self {
        Self {
            columns: dimensions.columns,
            rows: dimensions.rows,
            pixel_width: dimensions
                .columns
                .saturating_mul(dimensions.cell_width_px.min(u16::MAX as u32) as u16),
            pixel_height: dimensions
                .rows
                .saturating_mul(dimensions.cell_height_px.min(u16::MAX as u32) as u16),
        }
    }

    fn from_terminal_size(size: TerminalSize) -> Self {
        Self {
            columns: size.columns,
            rows: size.rows,
            pixel_width: 0,
            pixel_height: 0,
        }
    }

    fn winsize(self) -> libc::winsize {
        libc::winsize {
            ws_row: self.rows,
            ws_col: self.columns,
            ws_xpixel: self.pixel_width,
            ws_ypixel: self.pixel_height,
        }
    }
}

/// Handle to a spawned child process attached to a PTY master fd.
#[derive(Debug)]
pub struct Pty {
    terminal_id: TerminalId,
    session_id: SessionId,
    master_fd: RawFd,
    child_pid: libc::pid_t,
}

impl Pty {
    pub fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    pub fn master_fd(&self) -> RawFd {
        self.master_fd
    }

    pub fn read_available(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let read = unsafe { libc::read(self.master_fd, buf.as_mut_ptr().cast(), buf.len()) };
            if read >= 0 {
                return Ok(read as usize);
            }

            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(err);
        }
    }

    pub fn write_nonblocking(&self, bytes: &[u8]) -> io::Result<usize> {
        loop {
            let written =
                unsafe { libc::write(self.master_fd, bytes.as_ptr().cast(), bytes.len()) };
            if written >= 0 {
                return Ok(written as usize);
            }

            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(err);
        }
    }

    pub fn resize(&self, size: PtySize) -> io::Result<()> {
        let winsize = size.winsize();
        let rc = unsafe { libc::ioctl(self.master_fd, libc::TIOCSWINSZ, &winsize) };
        if rc == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn terminate(&self) -> io::Result<()> {
        let rc = unsafe { libc::kill(self.child_pid, libc::SIGHUP) };
        if rc == -1 {
            let err = io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ESRCH) {
                return Err(err);
            }
        }
        reap_child_nonblocking(self.child_pid)
    }

    fn set_nonblocking(&self) -> io::Result<()> {
        let flags = unsafe { libc::fcntl(self.master_fd, libc::F_GETFL) };
        if flags == -1 {
            return Err(io::Error::last_os_error());
        }
        let rc = unsafe { libc::fcntl(self.master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
        if rc == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.master_fd);
        }
    }
}

/// Real PTY/process backend. It is intentionally separate from libghostty
/// surface state; callers feed bytes read from `Pty` into a terminal surface.
#[derive(Debug, Default)]
pub struct UnixPtyBackend {
    next_session_id: u64,
    sessions: BTreeMap<SessionId, Pty>,
}

impl UnixPtyBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pty(&self, session_id: SessionId) -> Option<&Pty> {
        self.sessions.get(&session_id)
    }

    pub fn pty_mut(&mut self, session_id: SessionId) -> Option<&mut Pty> {
        self.sessions.get_mut(&session_id)
    }

    pub fn remove(&mut self, session_id: SessionId) -> Option<Pty> {
        self.sessions.remove(&session_id)
    }

    fn next_session_id(&mut self) -> SessionId {
        self.next_session_id += 1;
        SessionId(self.next_session_id)
    }
}

impl PtyBackend for UnixPtyBackend {
    fn spawn(&mut self, request: SpawnProcessRequest) -> anyhow::Result<TerminalSession> {
        let session_id = self.next_session_id();
        let size = PtySize::from_terminal_size(request.initial_size);
        let pty = spawn_pty(
            request.terminal_id,
            session_id,
            &request.process,
            &request.cwd,
            size,
        )?;
        self.sessions.insert(session_id, pty);
        Ok(TerminalSession {
            id: session_id,
            terminal_id: request.terminal_id,
            started_at: SystemTime::now(),
        })
    }

    fn kill(&mut self, session_id: SessionId) -> anyhow::Result<()> {
        if let Some(pty) = self.sessions.remove(&session_id) {
            pty.terminate()?;
        }
        Ok(())
    }

    fn resize(&mut self, session_id: SessionId, size: TerminalSize) -> anyhow::Result<()> {
        if let Some(pty) = self.sessions.get(&session_id) {
            pty.resize(PtySize::from_terminal_size(size))?;
        }
        Ok(())
    }

    fn write(&mut self, session_id: SessionId, bytes: &[u8]) -> anyhow::Result<()> {
        let Some(pty) = self.sessions.get(&session_id) else {
            return Ok(());
        };

        let mut offset = 0;
        while offset < bytes.len() {
            match pty.write_nonblocking(&bytes[offset..]) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        format!(
                            "PTY write made no progress after {offset} of {} bytes",
                            bytes.len()
                        ),
                    )
                    .into());
                }
                Ok(written) => offset += written,
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        format!(
                            "PTY write would block after {offset} of {} bytes",
                            bytes.len()
                        ),
                    )
                    .into());
                }
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }
}

pub fn spawn_pty(
    terminal_id: TerminalId,
    session_id: SessionId,
    process: &ProcessSpec,
    cwd: &Path,
    size: PtySize,
) -> anyhow::Result<Pty> {
    let mut master_fd: RawFd = -1;
    let mut winsize = size.winsize();
    let pid = unsafe {
        libc::forkpty(
            &mut master_fd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut winsize,
        )
    };

    if pid < 0 {
        return Err(io::Error::last_os_error()).context("forkpty failed");
    }

    if pid == 0 {
        child_exec(process, cwd);
    }

    let pty = Pty {
        terminal_id,
        session_id,
        master_fd,
        child_pid: pid,
    };
    pty.set_nonblocking()?;
    Ok(pty)
}

fn child_exec(process: &ProcessSpec, cwd: &Path) -> ! {
    if let Err(err) = std::env::set_current_dir(cwd) {
        eprintln!("slerm: failed to set cwd to {}: {err}", cwd.display());
        unsafe { libc::_exit(127) }
    }

    for (key, value) in &process.env {
        unsafe { std::env::set_var(key, value) };
    }

    let program = match CString::new(process.program.as_os_str().as_encoded_bytes()) {
        Ok(program) => program,
        Err(_) => unsafe { libc::_exit(127) },
    };
    let mut args = Vec::with_capacity(process.args.len() + 2);
    args.push(program.clone());
    for arg in &process.args {
        match CString::new(arg.as_bytes()) {
            Ok(arg) => args.push(arg),
            Err(_) => unsafe { libc::_exit(127) },
        }
    }

    let mut argv = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
    argv.push(std::ptr::null());

    unsafe {
        libc::execvp(program.as_ptr(), argv.as_ptr());
        libc::_exit(127);
    }
}

fn reap_child_nonblocking(pid: libc::pid_t) -> io::Result<()> {
    let mut status = 0;
    loop {
        let rc = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if rc == pid || rc == 0 {
            return Ok(());
        }
        if rc >= 0 {
            return Ok(());
        }

        let err = io::Error::last_os_error();
        if err.kind() == io::ErrorKind::Interrupted {
            continue;
        }
        if err.raw_os_error() == Some(libc::ECHILD) {
            return Ok(());
        }
        return Err(err);
    }
}
