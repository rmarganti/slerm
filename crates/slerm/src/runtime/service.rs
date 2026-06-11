use std::{
    collections::BTreeMap,
    io,
    time::{Duration, Instant, SystemTime},
};

use anyhow::bail;

use crate::{
    perf::TerminalDrainPerf,
    project::model::Project,
    runtime::{
        LivePty, LiveTerminalSpawner, ProjectAttention, ProjectAttentionReason, Pty, PtyBackend,
        PtySize, SessionId, SpawnProcessRequest, TerminalRunStatus, TerminalRuntimeState,
        TerminalSession, TerminalSize, TerminalStatus, UnixLiveTerminalSpawner,
    },
    terminal::{
        TerminalId, TerminalSpec,
        surface::{
            GhosttyTerminalSurface, TerminalKeyInput, TerminalMouseInput, TerminalScrollInput,
            TerminalScrollOutcome,
        },
    },
    workspace::model::WorkspaceState,
};

const LIVE_TERMINAL_DRAIN_TIME_BUDGET: Duration = Duration::from_millis(4);

/// Owns live runtime state for all known terminals.
///
/// The service is initialized from persisted specs, then updated by spawning,
/// killing, status detection, and future task/agent runtime events.
#[derive(Debug)]
pub struct LiveTerminalRuntime<P: LivePty = Pty> {
    pub pty: P,
    pub surface: GhosttyTerminalSurface,
}

#[derive(Debug)]
pub struct TerminalRuntimeService<P: LivePty = Pty> {
    states: BTreeMap<TerminalId, TerminalRuntimeState>,
    live: BTreeMap<TerminalId, LiveTerminalRuntime<P>>,
    next_live_session_id: u64,
    last_drain_perf: TerminalDrainPerf,
}

impl<P: LivePty> Default for TerminalRuntimeService<P> {
    fn default() -> Self {
        Self {
            states: BTreeMap::new(),
            live: BTreeMap::new(),
            next_live_session_id: 0,
            last_drain_perf: TerminalDrainPerf::default(),
        }
    }
}

impl<P: LivePty> TerminalRuntimeService<P> {
    pub fn new_with_live_pty() -> Self {
        Self::default()
    }

    pub fn from_workspace_with_live_pty(workspace: &WorkspaceState) -> Self {
        let mut service = Self::new_with_live_pty();
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
        if let Some(live) = self.live.remove(&terminal_id)
            && let Err(error) = live.pty.terminate()
        {
            eprintln!("failed to terminate terminal {terminal_id:?}: {error}");
        }
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

    pub fn live_terminal_mut(
        &mut self,
        terminal_id: TerminalId,
    ) -> Option<&mut LiveTerminalRuntime<P>> {
        self.live.get_mut(&terminal_id)
    }

    pub fn live_terminal_ids(&self) -> impl Iterator<Item = TerminalId> + '_ {
        self.live.keys().copied()
    }

    pub fn live_terminal_count(&self) -> usize {
        self.live.len()
    }

    pub fn ensure_live_terminal_with<S>(
        &mut self,
        spec: &TerminalSpec,
        dimensions: crate::terminal::surface::TerminalDimensions,
        spawner: &mut S,
    ) -> anyhow::Result<&mut LiveTerminalRuntime<P>>
    where
        S: LiveTerminalSpawner<Pty = P>,
    {
        self.ensure_terminal(spec);
        if !self.live.contains_key(&spec.id) {
            self.next_live_session_id += 1;
            let session = TerminalSession {
                id: SessionId(self.next_live_session_id),
                terminal_id: spec.id,
                started_at: SystemTime::now(),
            };
            let pty = spawner.spawn_live_terminal(
                spec.id,
                session.id,
                &spec.command,
                &spec.cwd,
                PtySize::from_dimensions(dimensions),
            )?;
            let surface = GhosttyTerminalSurface::new(dimensions)?;
            self.live
                .insert(spec.id, LiveTerminalRuntime { pty, surface });
            if let Some(state) = self.terminal_mut(spec.id) {
                state.session.session = Some(session);
                state.session.status = TerminalRunStatus::Running;
                state.session.exit_status = None;
                state.session.updated_at = SystemTime::now();
            }
        }
        Ok(self.live.get_mut(&spec.id).expect("live terminal inserted"))
    }

    pub fn drain_live_terminal(&mut self, terminal_id: TerminalId) -> bool {
        let Some(live) = self.live.get_mut(&terminal_id) else {
            self.last_drain_perf = TerminalDrainPerf::default();
            return false;
        };
        let (changed, perf) = drain_live_terminal(terminal_id, live);
        self.last_drain_perf = perf;
        changed
    }

    pub fn drain_live_terminals(&mut self) -> bool {
        self.drain_live_terminals_with_perf().0
    }

    pub fn drain_live_terminals_with_perf(&mut self) -> (bool, TerminalDrainPerf) {
        let mut changed = false;
        let mut perf = TerminalDrainPerf::default();
        for (terminal_id, live) in &mut self.live {
            let (terminal_changed, terminal_perf) = drain_live_terminal(*terminal_id, live);
            changed |= terminal_changed;
            perf.record_terminal(terminal_perf);
        }
        self.last_drain_perf = perf;
        (changed, perf)
    }

    pub fn last_drain_perf(&self) -> TerminalDrainPerf {
        self.last_drain_perf
    }

    pub fn resize_live_terminal(
        &mut self,
        terminal_id: TerminalId,
        dimensions: crate::terminal::surface::TerminalDimensions,
    ) -> anyhow::Result<()> {
        if let Some(live) = self.live.get_mut(&terminal_id)
            && live.surface.dimensions() != dimensions
        {
            live.pty
                .resize(PtySize::from_dimensions(dimensions))
                .map_err(|error| {
                    io::Error::new(
                        error.kind(),
                        format!("failed to resize PTY for terminal {terminal_id:?}: {error}"),
                    )
                })?;
            live.surface.resize(dimensions)?;
        }
        Ok(())
    }

    pub fn write_key_input(&mut self, terminal_id: TerminalId, input: TerminalKeyInput) -> bool {
        let Some(live) = self.live.get_mut(&terminal_id) else {
            return false;
        };
        let bytes = match live.surface.encode_key_input(input) {
            Ok(bytes) => bytes.to_vec(),
            Err(error) => {
                eprintln!("failed to encode key input for terminal {terminal_id:?}: {error}");
                return false;
            }
        };
        write_encoded_input(terminal_id, live, &bytes)
    }

    pub fn write_mouse_input(
        &mut self,
        terminal_id: TerminalId,
        input: TerminalMouseInput,
    ) -> bool {
        let Some(live) = self.live.get_mut(&terminal_id) else {
            return false;
        };
        let bytes = match live.surface.encode_mouse_input(input) {
            Ok(bytes) => bytes.to_vec(),
            Err(error) => {
                eprintln!("failed to encode mouse input for terminal {terminal_id:?}: {error}");
                return false;
            }
        };
        write_encoded_input(terminal_id, live, &bytes)
    }

    pub fn handle_scroll_input(
        &mut self,
        terminal_id: TerminalId,
        input: TerminalScrollInput,
    ) -> bool {
        let Some(live) = self.live.get_mut(&terminal_id) else {
            return false;
        };
        let outcome = match live.surface.handle_scroll_input(input) {
            Ok(outcome) => outcome,
            Err(error) => {
                eprintln!("failed to handle scroll input for terminal {terminal_id:?}: {error}");
                return false;
            }
        };
        match outcome {
            TerminalScrollOutcome::Encoded => {
                let bytes = live.surface.encoded_input_response().to_vec();
                write_encoded_input(terminal_id, live, &bytes)
            }
            TerminalScrollOutcome::Scrolled => true,
            TerminalScrollOutcome::Ignored => false,
        }
    }

    pub fn resize_live_terminals(
        &mut self,
        dimensions: crate::terminal::surface::TerminalDimensions,
    ) -> anyhow::Result<()> {
        let mut errors = Vec::new();
        for (terminal_id, live) in &mut self.live {
            if live.surface.dimensions() != dimensions {
                if let Err(error) = live.pty.resize(PtySize::from_dimensions(dimensions)) {
                    errors.push(format!(
                        "failed to resize PTY for terminal {terminal_id:?}: {error}"
                    ));
                    continue;
                }
                if let Err(error) = live.surface.resize(dimensions) {
                    errors.push(format!(
                        "failed to resize surface for terminal {terminal_id:?}: {error}"
                    ));
                }
            }
        }
        if !errors.is_empty() {
            bail!(errors.join("; "));
        }
        Ok(())
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

impl TerminalRuntimeService<Pty> {
    pub fn new() -> Self {
        Self::new_with_live_pty()
    }

    pub fn from_workspace(workspace: &WorkspaceState) -> Self {
        Self::from_workspace_with_live_pty(workspace)
    }

    pub fn ensure_live_terminal(
        &mut self,
        spec: &TerminalSpec,
        dimensions: crate::terminal::surface::TerminalDimensions,
    ) -> anyhow::Result<&mut LiveTerminalRuntime<Pty>> {
        self.ensure_live_terminal_with(spec, dimensions, &mut UnixLiveTerminalSpawner)
    }
}

fn write_encoded_input<P: LivePty>(
    terminal_id: TerminalId,
    live: &mut LiveTerminalRuntime<P>,
    bytes: &[u8],
) -> bool {
    if bytes.is_empty() {
        return false;
    }
    if let Err(error) = write_all_nonblocking(&live.pty, bytes)
        && error.kind() != io::ErrorKind::WouldBlock
    {
        eprintln!("failed to write terminal input for {terminal_id:?}: {error}");
        return false;
    }
    true
}

fn drain_live_terminal<P: LivePty>(
    terminal_id: TerminalId,
    live: &mut LiveTerminalRuntime<P>,
) -> (bool, TerminalDrainPerf) {
    let mut changed = false;
    let mut bytes_read = 0;
    let mut buf = [0_u8; 16 * 1024];
    let started_at = Instant::now();
    loop {
        match live.pty.read_available(&mut buf) {
            Ok(0) => break,
            Ok(read) => {
                live.surface.vt_write(&buf[..read]);
                changed = true;
                bytes_read += read;
                if started_at.elapsed() >= LIVE_TERMINAL_DRAIN_TIME_BUDGET {
                    break;
                }
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
            Err(err) if err.raw_os_error() == Some(libc::EIO) => break,
            Err(error) => {
                eprintln!("failed to read PTY for terminal {terminal_id:?}: {error}");
                break;
            }
        }
    }
    for response in live.surface.take_pending_pty_writes() {
        if let Err(error) = write_all_nonblocking(&live.pty, &response)
            && error.kind() != io::ErrorKind::WouldBlock
        {
            eprintln!("failed to write terminal response for {terminal_id:?}: {error}");
        }
    }
    (
        changed,
        TerminalDrainPerf {
            terminals: 1,
            changed_terminals: usize::from(changed),
            bytes_read,
            duration: started_at.elapsed(),
        },
    )
}

fn write_all_nonblocking(pty: &impl LivePty, bytes: &[u8]) -> io::Result<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        match pty.write_nonblocking(&bytes[offset..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "live PTY write made no progress",
                ));
            }
            Ok(written) => offset += written,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        }
    }
    Ok(())
}
