use std::time::SystemTime;

use crate::terminal::TerminalId;

/// Runtime-only identifier for a spawned terminal session.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SessionId(pub u64);

/// Live process/session metadata for a terminal after it has been spawned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalSession {
    pub id: SessionId,
    pub terminal_id: TerminalId,
    pub started_at: SystemTime,
}

/// Runtime lifecycle state for a terminal's process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalSessionState {
    pub session: Option<TerminalSession>,
    pub status: TerminalRunStatus,
    pub exit_status: Option<ExitStatusSnapshot>,
    pub updated_at: SystemTime,
}

impl TerminalSessionState {
    pub fn not_started() -> Self {
        Self {
            session: None,
            status: TerminalRunStatus::NotStarted,
            exit_status: None,
            updated_at: SystemTime::now(),
        }
    }
}

/// Coarse lifecycle state of the process backing a terminal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalRunStatus {
    NotStarted,
    Starting,
    Running,
    Exited,
    FailedToStart,
}

/// Serializable-ish snapshot of how a spawned process finished.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExitStatusSnapshot {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub message: Option<String>,
    pub finished_at: SystemTime,
}
