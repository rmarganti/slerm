use std::{collections::BTreeMap, path::PathBuf, time::SystemTime};

use crate::{
    project::model::Project,
    terminal::{
        extension::TerminalExtensionSpec,
        instance::{TerminalId, TerminalSpec},
    },
    workspace::model::WorkspaceState,
};

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

/// Serializable-ish snapshot of how a spawned process finished.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExitStatusSnapshot {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub message: Option<String>,
    pub finished_at: SystemTime,
}

/// Derived UI status for a terminal.
///
/// This intentionally composes lifecycle, activity, outcome, and attention so a
/// terminal can express combinations like "running and awaiting review".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalStatus {
    pub run: TerminalRunStatus,
    pub activity: TerminalActivityStatus,
    pub outcome: TerminalOutcomeStatus,
    pub attention: AttentionState,
}

impl TerminalStatus {
    pub fn derive(runtime: &TerminalRuntimeState) -> Self {
        let mut reasons = Vec::new();
        let mut severity = AttentionSeverity::None;
        let mut activity = TerminalActivityStatus::None;
        let mut outcome = TerminalOutcomeStatus::None;

        match &runtime.extension {
            TerminalExtensionRuntime::Plain => {}
            TerminalExtensionRuntime::Agent(agent) => match agent.status {
                AgentStatus::Unknown | AgentStatus::Idle => {
                    activity = TerminalActivityStatus::Idle;
                }
                AgentStatus::Working => {
                    activity = TerminalActivityStatus::Working;
                    add_attention(
                        &mut severity,
                        &mut reasons,
                        AttentionSeverity::Activity,
                        AttentionReason::AgentWorking,
                    );
                }
                AgentStatus::AwaitingReview => {
                    activity = TerminalActivityStatus::AwaitingReview;
                    add_attention(
                        &mut severity,
                        &mut reasons,
                        AttentionSeverity::NeedsUser,
                        AttentionReason::AgentAwaitingReview,
                    );
                }
                AgentStatus::Errored => {
                    outcome = TerminalOutcomeStatus::Failed;
                    add_attention(
                        &mut severity,
                        &mut reasons,
                        AttentionSeverity::Error,
                        AttentionReason::AgentErrored,
                    );
                }
            },
            TerminalExtensionRuntime::Task(task) => match task.status {
                TaskStatus::PendingManualStart | TaskStatus::Stopped => {}
                TaskStatus::Running | TaskStatus::Restarting => {
                    activity = TerminalActivityStatus::Working;
                    add_attention(
                        &mut severity,
                        &mut reasons,
                        AttentionSeverity::Activity,
                        AttentionReason::TaskRunning,
                    );
                }
                TaskStatus::Succeeded => {
                    outcome = TerminalOutcomeStatus::Succeeded;
                    add_attention(
                        &mut severity,
                        &mut reasons,
                        AttentionSeverity::Info,
                        AttentionReason::TaskSucceeded,
                    );
                }
                TaskStatus::Failed => {
                    outcome = TerminalOutcomeStatus::Failed;
                    add_attention(
                        &mut severity,
                        &mut reasons,
                        AttentionSeverity::Error,
                        AttentionReason::TaskFailed,
                    );
                }
            },
        }

        match runtime.session.status {
            TerminalRunStatus::NotStarted
            | TerminalRunStatus::Starting
            | TerminalRunStatus::Running => {}
            TerminalRunStatus::Exited => {
                if outcome == TerminalOutcomeStatus::None {
                    outcome = TerminalOutcomeStatus::Stopped;
                }
                add_attention(
                    &mut severity,
                    &mut reasons,
                    AttentionSeverity::Info,
                    AttentionReason::TerminalExited,
                );
            }
            TerminalRunStatus::FailedToStart => {
                if outcome == TerminalOutcomeStatus::None {
                    outcome = TerminalOutcomeStatus::Failed;
                }
                add_attention(
                    &mut severity,
                    &mut reasons,
                    AttentionSeverity::Error,
                    AttentionReason::TerminalExited,
                );
            }
        }

        Self {
            run: runtime.session.status.clone(),
            activity,
            outcome,
            attention: AttentionState { severity, reasons },
        }
    }
}

/// User-facing activity signal derived from agent/task runtime state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalActivityStatus {
    None,
    Idle,
    Working,
    AwaitingReview,
}

/// User-facing outcome signal derived from process or task completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalOutcomeStatus {
    None,
    Succeeded,
    Failed,
    Stopped,
}

/// Aggregated attention signal used for terminal and project badges.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttentionState {
    pub severity: AttentionSeverity,
    pub reasons: Vec<AttentionReason>,
}

/// Ordering of attention levels; higher severities dominate aggregates.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AttentionSeverity {
    None,
    Info,
    Activity,
    NeedsUser,
    Error,
}

/// Concrete reason a terminal is surfacing attention.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttentionReason {
    AgentWorking,
    AgentAwaitingReview,
    AgentErrored,
    TaskRunning,
    TaskSucceeded,
    TaskFailed,
    TerminalExited,
}

/// Attention summary for a project, derived from its terminals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectAttention {
    pub severity: AttentionSeverity,
    pub reasons: Vec<ProjectAttentionReason>,
}

/// A terminal-scoped reason contributing to project-level attention.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectAttentionReason {
    pub terminal_id: TerminalId,
    pub reason: AttentionReason,
}

impl ProjectAttention {
    pub fn none() -> Self {
        Self {
            severity: AttentionSeverity::None,
            reasons: Vec::new(),
        }
    }
}

fn add_attention(
    severity: &mut AttentionSeverity,
    reasons: &mut Vec<AttentionReason>,
    new_severity: AttentionSeverity,
    reason: AttentionReason,
) {
    *severity = (*severity).max(new_severity);
    reasons.push(reason);
}

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
    pub process: crate::terminal::instance::ProcessSpec,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub initial_size: TerminalSize,
}

impl SpawnProcessRequest {
    pub fn from_spec(spec: &TerminalSpec, initial_size: TerminalSize) -> Self {
        Self {
            terminal_id: spec.id,
            process: spec.command.clone(),
            cwd: spec.cwd.clone(),
            env: spec.command.env.clone(),
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

/// In-memory backend for exercising runtime service behavior before real PTY integration.
#[derive(Clone, Debug)]
pub struct MockPtyBackend {
    next_session_id: u64,
    pub spawned: Vec<SpawnProcessRequest>,
    pub killed: Vec<SessionId>,
    pub resized: Vec<(SessionId, TerminalSize)>,
    pub written: Vec<(SessionId, Vec<u8>)>,
}

impl Default for MockPtyBackend {
    fn default() -> Self {
        Self {
            next_session_id: 1,
            spawned: Vec::new(),
            killed: Vec::new(),
            resized: Vec::new(),
            written: Vec::new(),
        }
    }
}

impl MockPtyBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PtyBackend for MockPtyBackend {
    fn spawn(&mut self, request: SpawnProcessRequest) -> anyhow::Result<TerminalSession> {
        let id = SessionId(self.next_session_id);
        self.next_session_id += 1;
        let terminal_id = request.terminal_id;
        self.spawned.push(request);

        Ok(TerminalSession {
            id,
            terminal_id,
            started_at: SystemTime::now(),
        })
    }

    fn kill(&mut self, session_id: SessionId) -> anyhow::Result<()> {
        self.killed.push(session_id);
        Ok(())
    }

    fn resize(&mut self, session_id: SessionId, size: TerminalSize) -> anyhow::Result<()> {
        self.resized.push((session_id, size));
        Ok(())
    }

    fn write(&mut self, session_id: SessionId, bytes: &[u8]) -> anyhow::Result<()> {
        self.written.push((session_id, bytes.to_vec()));
        Ok(())
    }
}

/// Owns live runtime state for all known terminals.
///
/// The service is initialized from persisted specs, then updated by spawning,
/// killing, status detection, and future task/agent runtime events.
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

    pub fn spawn_terminal(
        &mut self,
        spec: &TerminalSpec,
        size: TerminalSize,
        backend: &mut impl PtyBackend,
    ) -> anyhow::Result<TerminalSession> {
        let terminal_id = spec.id;
        {
            let state = self.ensure_terminal(spec);
            state.session.status = TerminalRunStatus::Starting;
            state.session.updated_at = SystemTime::now();
        }

        match backend.spawn(SpawnProcessRequest::from_spec(spec, size)) {
            Ok(session) => {
                if let Some(state) = self.terminal_mut(terminal_id) {
                    state.session.session = Some(session.clone());
                    state.session.status = TerminalRunStatus::Running;
                    state.session.exit_status = None;
                    state.session.updated_at = SystemTime::now();
                }
                Ok(session)
            }
            Err(error) => {
                if let Some(state) = self.terminal_mut(terminal_id) {
                    state.session.session = None;
                    state.session.status = TerminalRunStatus::FailedToStart;
                    state.session.updated_at = SystemTime::now();
                }
                Err(error)
            }
        }
    }

    pub fn kill_terminal(
        &mut self,
        terminal_id: TerminalId,
        backend: &mut impl PtyBackend,
    ) -> anyhow::Result<()> {
        let Some(session_id) = self
            .terminal(terminal_id)
            .and_then(|state| state.session.session.as_ref())
            .map(|session| session.id)
        else {
            return Ok(());
        };

        backend.kill(session_id)?;

        if let Some(state) = self.terminal_mut(terminal_id) {
            state.session.session = None;
            state.session.status = TerminalRunStatus::Exited;
            state.session.updated_at = SystemTime::now();
        }

        Ok(())
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

    pub fn terminal_status(&self, terminal_id: TerminalId) -> Option<TerminalStatus> {
        self.terminal(terminal_id).map(TerminalStatus::derive)
    }

    pub fn project_attention(&self, project: &Project) -> ProjectAttention {
        let mut attention = ProjectAttention::none();

        for terminal in &project.terminals {
            let Some(status) = self.terminal_status(terminal.id) else {
                continue;
            };

            attention.severity = attention.severity.max(status.attention.severity);
            attention
                .reasons
                .extend(status.attention.reasons.into_iter().map(|reason| {
                    ProjectAttentionReason {
                        terminal_id: terminal.id,
                        reason,
                    }
                }));
        }

        attention
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        project::model::ProjectId,
        terminal::{
            extension::{AgentKind, AgentSpec, TerminalExtensionSpec},
            instance::{ProcessSpec, TerminalSpec},
        },
    };

    fn terminal(extension: TerminalExtensionSpec) -> TerminalRuntimeState {
        TerminalRuntimeState::from_spec(&TerminalSpec::new(
            1,
            ProjectId(1),
            extension,
            "terminal",
            "/tmp",
            ProcessSpec::shell(),
        ))
    }

    #[test]
    fn agent_working_derives_activity_attention() {
        let mut runtime = terminal(TerminalExtensionSpec::Agent(AgentSpec::new(AgentKind::Pi)));
        if let TerminalExtensionRuntime::Agent(agent) = &mut runtime.extension {
            agent.status = AgentStatus::Working;
        }

        let status = TerminalStatus::derive(&runtime);

        assert_eq!(status.activity, TerminalActivityStatus::Working);
        assert_eq!(status.attention.severity, AttentionSeverity::Activity);
        assert_eq!(
            status.attention.reasons,
            vec![AttentionReason::AgentWorking]
        );
    }

    #[test]
    fn agent_awaiting_review_derives_needs_user_attention() {
        let mut runtime = terminal(TerminalExtensionSpec::Agent(AgentSpec::new(
            AgentKind::Codex,
        )));
        if let TerminalExtensionRuntime::Agent(agent) = &mut runtime.extension {
            agent.status = AgentStatus::AwaitingReview;
        }

        let status = TerminalStatus::derive(&runtime);

        assert_eq!(status.activity, TerminalActivityStatus::AwaitingReview);
        assert_eq!(status.attention.severity, AttentionSeverity::NeedsUser);
        assert_eq!(
            status.attention.reasons,
            vec![AttentionReason::AgentAwaitingReview]
        );
    }

    #[test]
    fn task_running_derives_activity() {
        let mut runtime = terminal(TerminalExtensionSpec::Task(Default::default()));
        if let TerminalExtensionRuntime::Task(task) = &mut runtime.extension {
            task.status = TaskStatus::Running;
        }

        let status = TerminalStatus::derive(&runtime);

        assert_eq!(status.activity, TerminalActivityStatus::Working);
        assert_eq!(status.attention.severity, AttentionSeverity::Activity);
        assert_eq!(status.attention.reasons, vec![AttentionReason::TaskRunning]);
    }

    #[test]
    fn task_succeeded_and_failed_derive_outcomes() {
        let mut succeeded = terminal(TerminalExtensionSpec::Task(Default::default()));
        if let TerminalExtensionRuntime::Task(task) = &mut succeeded.extension {
            task.status = TaskStatus::Succeeded;
        }

        let succeeded_status = TerminalStatus::derive(&succeeded);
        assert_eq!(succeeded_status.outcome, TerminalOutcomeStatus::Succeeded);
        assert_eq!(succeeded_status.attention.severity, AttentionSeverity::Info);
        assert_eq!(
            succeeded_status.attention.reasons,
            vec![AttentionReason::TaskSucceeded]
        );

        let mut failed = terminal(TerminalExtensionSpec::Task(Default::default()));
        if let TerminalExtensionRuntime::Task(task) = &mut failed.extension {
            task.status = TaskStatus::Failed;
        }

        let failed_status = TerminalStatus::derive(&failed);
        assert_eq!(failed_status.outcome, TerminalOutcomeStatus::Failed);
        assert_eq!(failed_status.attention.severity, AttentionSeverity::Error);
        assert_eq!(
            failed_status.attention.reasons,
            vec![AttentionReason::TaskFailed]
        );
    }

    #[test]
    fn terminal_exited_derives_stopped_outcome_and_reason() {
        let mut runtime = terminal(TerminalExtensionSpec::Plain);
        runtime.session.status = TerminalRunStatus::Exited;

        let status = TerminalStatus::derive(&runtime);

        assert_eq!(status.outcome, TerminalOutcomeStatus::Stopped);
        assert_eq!(status.attention.severity, AttentionSeverity::Info);
        assert_eq!(
            status.attention.reasons,
            vec![AttentionReason::TerminalExited]
        );
    }

    #[test]
    fn backend_spawn_request_uses_process_spec_and_terminal_metadata() {
        let spec = TerminalSpec::new(
            7,
            ProjectId(3),
            TerminalExtensionSpec::Plain,
            "server",
            "/workspace/slerm",
            ProcessSpec::new("cargo", ["run", "-p", "slerm"]),
        );

        let request = SpawnProcessRequest::from_spec(&spec, TerminalSize::new(120, 40));

        assert_eq!(request.terminal_id, TerminalId(7));
        assert_eq!(request.cwd, PathBuf::from("/workspace/slerm"));
        assert_eq!(request.process.display_command_line(), "cargo run -p slerm");
        assert_eq!(request.initial_size, TerminalSize::new(120, 40));
    }

    #[test]
    fn runtime_service_spawns_and_kills_through_backend_seam() {
        let spec = TerminalSpec::new(
            9,
            ProjectId(1),
            TerminalExtensionSpec::Plain,
            "terminal",
            "/tmp",
            ProcessSpec::shell(),
        );
        let mut service = TerminalRuntimeService::new();
        let mut backend = MockPtyBackend::new();

        let session = service
            .spawn_terminal(&spec, TerminalSize::DEFAULT, &mut backend)
            .expect("mock spawn succeeds");

        assert_eq!(session.id, SessionId(1));
        assert_eq!(backend.spawned.len(), 1);
        let state = service.terminal(spec.id).expect("runtime state exists");
        assert_eq!(state.session.status, TerminalRunStatus::Running);
        assert_eq!(
            state.session.session.as_ref().map(|session| session.id),
            Some(SessionId(1))
        );

        service
            .kill_terminal(spec.id, &mut backend)
            .expect("mock kill succeeds");

        assert_eq!(backend.killed, vec![SessionId(1)]);
        let state = service.terminal(spec.id).expect("runtime state remains");
        assert_eq!(state.session.status, TerminalRunStatus::Exited);
        assert!(state.session.session.is_none());
    }

    #[test]
    fn project_attention_aggregates_highest_severity() {
        let terminal_one = TerminalSpec::new(
            1,
            ProjectId(1),
            TerminalExtensionSpec::Agent(AgentSpec::new(AgentKind::Pi)),
            "agent",
            "/tmp",
            ProcessSpec::shell(),
        );
        let terminal_two = TerminalSpec::new(
            2,
            ProjectId(1),
            TerminalExtensionSpec::Task(Default::default()),
            "task",
            "/tmp",
            ProcessSpec::shell(),
        );
        let project =
            Project::new(1, "project", "/tmp").with_terminals(vec![terminal_one, terminal_two]);
        let mut service = TerminalRuntimeService::from_workspace(&WorkspaceState {
            projects: vec![project.clone()],
            active_project: Some(project.id),
        });

        if let Some(TerminalExtensionRuntime::Agent(agent)) = service
            .terminal_mut(TerminalId(1))
            .map(|state| &mut state.extension)
        {
            agent.status = AgentStatus::Working;
        }
        if let Some(TerminalExtensionRuntime::Task(task)) = service
            .terminal_mut(TerminalId(2))
            .map(|state| &mut state.extension)
        {
            task.status = TaskStatus::Failed;
        }

        let attention = service.project_attention(&project);

        assert_eq!(attention.severity, AttentionSeverity::Error);
        assert_eq!(attention.reasons.len(), 2);
        assert!(
            attention
                .reasons
                .iter()
                .any(|reason| reason.reason == AttentionReason::TaskFailed)
        );
    }
}
