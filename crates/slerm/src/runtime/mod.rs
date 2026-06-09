use std::{collections::BTreeMap, time::SystemTime};

use crate::{
    terminal::{
        extension::TerminalExtensionSpec,
        instance::{TerminalId, TerminalSpec},
    },
    workspace::model::WorkspaceState,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SessionId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalSession {
    pub id: SessionId,
    pub terminal_id: TerminalId,
    pub started_at: SystemTime,
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalRunStatus {
    NotStarted,
    Starting,
    Running,
    Exited,
    FailedToStart,
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentStatus {
    Unknown,
    Idle,
    Working,
    AwaitingReview,
    Errored,
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskStatus {
    PendingManualStart,
    Running,
    Succeeded,
    Failed,
    Restarting,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExitStatusSnapshot {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub message: Option<String>,
    pub finished_at: SystemTime,
}

#[derive(Clone, Debug, Default)]
pub struct TerminalRuntimeService {
    states: BTreeMap<TerminalId, TerminalRuntimeState>,
}

impl TerminalRuntimeService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_workspace(workspace: &WorkspaceState) -> Self {
        let mut service = Self::new();
        service.initialize_from_specs(
            workspace
                .projects
                .iter()
                .flat_map(|project| project.terminals.iter()),
        );
        service
    }

    pub fn initialize_from_specs<'a>(&mut self, specs: impl IntoIterator<Item = &'a TerminalSpec>) {
        for spec in specs {
            self.ensure_terminal(spec);
        }
    }

    pub fn ensure_terminal(&mut self, spec: &TerminalSpec) -> &mut TerminalRuntimeState {
        self.states
            .entry(spec.id)
            .or_insert_with(|| TerminalRuntimeState::from_spec(spec))
    }

    pub fn remove_terminal(&mut self, terminal_id: TerminalId) -> Option<TerminalRuntimeState> {
        self.states.remove(&terminal_id)
    }

    pub fn terminal(&self, terminal_id: TerminalId) -> Option<&TerminalRuntimeState> {
        self.states.get(&terminal_id)
    }

    pub fn terminal_mut(&mut self, terminal_id: TerminalId) -> Option<&mut TerminalRuntimeState> {
        self.states.get_mut(&terminal_id)
    }

    pub fn states(&self) -> &BTreeMap<TerminalId, TerminalRuntimeState> {
        &self.states
    }

    pub fn states_mut(&mut self) -> &mut BTreeMap<TerminalId, TerminalRuntimeState> {
        &mut self.states
    }
}
