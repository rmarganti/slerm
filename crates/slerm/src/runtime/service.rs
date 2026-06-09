use std::{collections::BTreeMap, time::SystemTime};

use crate::{
    project::model::Project,
    runtime::{
        ProjectAttention, ProjectAttentionReason, PtyBackend, SpawnProcessRequest,
        TerminalRunStatus, TerminalRuntimeState, TerminalSession, TerminalSize, TerminalStatus,
    },
    terminal::{TerminalId, TerminalSpec},
    workspace::model::WorkspaceState,
};

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
