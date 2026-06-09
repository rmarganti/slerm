use std::time::SystemTime;

use crate::{
    runtime::session::{ExitStatusSnapshot, TerminalSessionState},
    terminal::{TerminalId, TerminalSpec, extension::TerminalExtensionSpec},
};

/// Runtime companion to `TerminalSpec` for one terminal.
///
/// This joins process/session state with extension-specific live state while the
/// persisted terminal spec remains launch/config intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalRuntimeState {
    pub terminal_id: TerminalId,
    pub session: TerminalSessionState,
    pub extension: TerminalExtensionRuntime,
}

impl TerminalRuntimeState {
    pub fn from_spec(spec: &TerminalSpec) -> Self {
        Self {
            terminal_id: spec.id,
            session: TerminalSessionState::not_started(),
            extension: TerminalExtensionRuntime::from_extension_spec(&spec.extension),
        }
    }
}

/// Live semantic state for a terminal extension.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalExtensionRuntime {
    Plain,
    Agent(AgentRuntime),
    Task(TaskRuntime),
}

impl TerminalExtensionRuntime {
    pub fn from_extension_spec(spec: &TerminalExtensionSpec) -> Self {
        match spec {
            TerminalExtensionSpec::Plain => Self::Plain,
            TerminalExtensionSpec::Agent(_) => Self::Agent(AgentRuntime::default()),
            TerminalExtensionSpec::Task(_) => Self::Task(TaskRuntime::default()),
        }
    }
}

/// Runtime interpretation of an agent terminal's current state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRuntime {
    pub status: AgentStatus,
    pub last_status_change: SystemTime,
    pub message: Option<String>,
}

impl Default for AgentRuntime {
    fn default() -> Self {
        Self {
            status: AgentStatus::Unknown,
            last_status_change: SystemTime::now(),
            message: None,
        }
    }
}

/// Agent-specific status derived from process state and output detection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentStatus {
    Unknown,
    Idle,
    Working,
    AwaitingReview,
    Errored,
}

/// Runtime lifecycle state for a task terminal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRuntime {
    pub status: TaskStatus,
    pub last_status_change: SystemTime,
    pub exit_status: Option<ExitStatusSnapshot>,
}

impl Default for TaskRuntime {
    fn default() -> Self {
        Self {
            status: TaskStatus::PendingManualStart,
            last_status_change: SystemTime::now(),
            exit_status: None,
        }
    }
}

/// Task-specific status for command execution and restart behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskStatus {
    PendingManualStart,
    Running,
    Succeeded,
    Failed,
    Restarting,
    Stopped,
}
