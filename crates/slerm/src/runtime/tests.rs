use std::{
    cell::RefCell,
    collections::VecDeque,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use super::*;
use crate::{
    project::model::{Project, ProjectId},
    terminal::{
        ProcessSpec, TerminalId, TerminalSpec,
        extension::{AgentKind, AgentSpec, TerminalExtensionSpec},
        surface::{
            TerminalDimensions, TerminalKeyAction, TerminalKeyInput, TerminalRenderSnapshot,
        },
    },
    workspace::model::WorkspaceState,
};

fn snapshot_text(snapshot: &TerminalRenderSnapshot) -> String {
    snapshot
        .row_runs
        .iter()
        .flat_map(|row| &row.runs)
        .map(|run| run.text.as_str())
        .collect()
}

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

#[derive(Clone, Debug, Default)]
struct MockLivePtyInner {
    reads: VecDeque<Vec<u8>>,
    writes: Vec<Vec<u8>>,
    resizes: Vec<PtySize>,
    resize_errors: VecDeque<io::ErrorKind>,
    terminated: bool,
}

#[derive(Clone, Debug, Default)]
struct MockLivePty {
    inner: Rc<RefCell<MockLivePtyInner>>,
}

impl MockLivePty {
    fn push_read(&self, bytes: impl Into<Vec<u8>>) {
        self.inner.borrow_mut().reads.push_back(bytes.into());
    }

    fn written(&self) -> Vec<Vec<u8>> {
        self.inner.borrow().writes.clone()
    }

    fn resizes(&self) -> Vec<PtySize> {
        self.inner.borrow().resizes.clone()
    }

    fn fail_next_resize(&self, kind: io::ErrorKind) {
        self.inner.borrow_mut().resize_errors.push_back(kind);
    }

    fn terminated(&self) -> bool {
        self.inner.borrow().terminated
    }
}

impl LivePty for MockLivePty {
    fn read_available(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut inner = self.inner.borrow_mut();
        let Some(mut bytes) = inner.reads.pop_front() else {
            return Err(io::ErrorKind::WouldBlock.into());
        };
        let len = bytes.len().min(buf.len());
        buf[..len].copy_from_slice(&bytes[..len]);
        if len < bytes.len() {
            bytes.drain(..len);
            inner.reads.push_front(bytes);
        }
        Ok(len)
    }

    fn write_nonblocking(&self, bytes: &[u8]) -> io::Result<usize> {
        self.inner.borrow_mut().writes.push(bytes.to_vec());
        Ok(bytes.len())
    }

    fn resize(&self, size: PtySize) -> io::Result<()> {
        let mut inner = self.inner.borrow_mut();
        if let Some(kind) = inner.resize_errors.pop_front() {
            return Err(kind.into());
        }
        inner.resizes.push(size);
        Ok(())
    }

    fn terminate(&self) -> io::Result<()> {
        self.inner.borrow_mut().terminated = true;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct MockLiveSpawner {
    spawned: Vec<(TerminalId, SessionId, PtySize)>,
    handles: Vec<MockLivePty>,
}

impl LiveTerminalSpawner for MockLiveSpawner {
    type Pty = MockLivePty;

    fn spawn_live_terminal(
        &mut self,
        terminal_id: TerminalId,
        session_id: SessionId,
        _process: &ProcessSpec,
        _cwd: &Path,
        size: PtySize,
    ) -> anyhow::Result<Self::Pty> {
        let pty = MockLivePty::default();
        self.spawned.push((terminal_id, session_id, size));
        self.handles.push(pty.clone());
        Ok(pty)
    }
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
fn terminal_failed_to_start_derives_failed_outcome_and_reason() {
    let mut runtime = terminal(TerminalExtensionSpec::Plain);
    runtime.session.status = TerminalRunStatus::FailedToStart;

    let status = TerminalStatus::derive(&runtime);

    assert_eq!(status.outcome, TerminalOutcomeStatus::Failed);
    assert_eq!(status.attention.severity, AttentionSeverity::Error);
    assert_eq!(
        status.attention.reasons,
        vec![AttentionReason::TerminalFailedToStart]
    );
}

#[test]
fn backend_spawn_request_uses_process_spec_and_terminal_metadata() {
    let mut process = ProcessSpec::new("cargo", ["run", "-p", "slerm"]);
    process.env.insert("RUST_LOG".into(), "debug".into());
    let spec = TerminalSpec::new(
        7,
        ProjectId(3),
        TerminalExtensionSpec::Plain,
        "server",
        "/workspace/slerm",
        process,
    );

    let request = SpawnProcessRequest::from_spec(&spec, TerminalSize::new(120, 40));

    assert_eq!(request.terminal_id, TerminalId(7));
    assert_eq!(request.cwd, PathBuf::from("/workspace/slerm"));
    assert_eq!(request.process.display_command_line(), "cargo run -p slerm");
    assert_eq!(
        request.process.env.get("RUST_LOG").map(String::as_str),
        Some("debug")
    );
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

#[test]
fn live_terminal_spawner_seam_starts_terminal_once() {
    let spec = TerminalSpec::new(
        11,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "terminal",
        "/tmp",
        ProcessSpec::shell(),
    );
    let dimensions = TerminalDimensions::new(80, 24, 8, 16);
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();

    service
        .ensure_live_terminal_with(&spec, dimensions, &mut spawner)
        .expect("first live spawn succeeds");
    service
        .ensure_live_terminal_with(&spec, dimensions, &mut spawner)
        .expect("second ensure reuses live runtime");

    assert_eq!(spawner.spawned.len(), 1);
    assert_eq!(spawner.spawned[0].0, spec.id);
    assert_eq!(spawner.spawned[0].2, PtySize::from_dimensions(dimensions));
    let state = service.terminal(spec.id).expect("runtime state exists");
    assert_eq!(state.session.status, TerminalRunStatus::Running);
}

#[test]
fn live_terminal_drain_feeds_surface_without_real_pty() {
    let spec = TerminalSpec::new(
        12,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "terminal",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&spec, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("live spawn succeeds");
    spawner.handles[0].push_read(b"hidden-output".to_vec());

    assert!(service.drain_live_terminal(spec.id));
    let snapshot = service
        .live_terminal_mut(spec.id)
        .expect("live terminal exists")
        .surface
        .render_snapshot()
        .expect("snapshot renders");
    let text = snapshot_text(&snapshot);

    assert!(text.contains("hidden-output"));
}

#[test]
fn live_terminal_drain_writes_libghostty_responses_back_to_pty() {
    let spec = TerminalSpec::new(
        13,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "terminal",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&spec, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("live spawn succeeds");
    spawner.handles[0].push_read(b"\x1b[c".to_vec());

    service.drain_live_terminal(spec.id);

    assert!(
        spawner.handles[0]
            .written()
            .iter()
            .any(|response| !response.is_empty()),
        "device-attribute response should be written to PTY"
    );
}

#[test]
fn drain_live_terminals_feeds_hidden_and_active_surfaces() {
    let active = TerminalSpec::new(
        16,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "active",
        "/tmp",
        ProcessSpec::shell(),
    );
    let hidden = TerminalSpec::new(
        17,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "hidden",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&active, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("active live spawn succeeds");
    service
        .ensure_live_terminal_with(&hidden, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("hidden live spawn succeeds");
    spawner.handles[0].push_read(b"active-output".to_vec());
    spawner.handles[1].push_read(b"hidden-output".to_vec());

    assert!(service.drain_live_terminals());

    let active_snapshot = service
        .live_terminal_mut(active.id)
        .expect("active live terminal exists")
        .surface
        .render_snapshot()
        .expect("active snapshot renders");
    let active_text = snapshot_text(&active_snapshot);
    let hidden_snapshot = service
        .live_terminal_mut(hidden.id)
        .expect("hidden live terminal exists")
        .surface
        .render_snapshot()
        .expect("hidden snapshot renders");
    let hidden_text = snapshot_text(&hidden_snapshot);

    assert!(active_text.contains("active-output"));
    assert!(hidden_text.contains("hidden-output"));
}

#[test]
fn drain_live_terminals_records_last_tick_perf_for_ui_instrumentation() {
    let active = TerminalSpec::new(
        18,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "active",
        "/tmp",
        ProcessSpec::shell(),
    );
    let hidden = TerminalSpec::new(
        19,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "hidden",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&active, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("active live spawn succeeds");
    service
        .ensure_live_terminal_with(&hidden, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("hidden live spawn succeeds");
    spawner.handles[0].push_read(b"active-output".to_vec());
    spawner.handles[1].push_read(b"hidden-output".to_vec());

    assert!(service.drain_live_terminals());

    let perf = service.last_drain_perf();
    assert_eq!(perf.terminals, 2);
    assert_eq!(perf.changed_terminals, 2);
    assert_eq!(
        perf.bytes_read,
        b"active-output".len() + b"hidden-output".len()
    );
}

#[test]
fn drain_live_terminal_reads_past_small_chunk_bursts() {
    let spec = TerminalSpec::new(
        18,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "terminal",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&spec, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("live spawn succeeds");
    for _ in 0..128 {
        spawner.handles[0].push_read(b"x".to_vec());
    }

    assert!(service.drain_live_terminal(spec.id));

    let snapshot = service
        .live_terminal_mut(spec.id)
        .expect("live terminal exists")
        .surface
        .render_snapshot()
        .expect("snapshot renders");
    let rendered_text = snapshot_text(&snapshot);
    assert_eq!(
        rendered_text.chars().filter(|char| *char == 'x').count(),
        128
    );
}

#[test]
fn switching_live_terminals_reuses_existing_runtime_state() {
    let first = TerminalSpec::new(
        18,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "first",
        "/tmp",
        ProcessSpec::shell(),
    );
    let second = TerminalSpec::new(
        19,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "second",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&first, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("first live spawn succeeds");
    spawner.handles[0].push_read(b"first-state".to_vec());
    service.drain_live_terminals();
    service
        .ensure_live_terminal_with(&second, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("second live spawn succeeds");
    service
        .ensure_live_terminal_with(&first, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("switching back to first reuses live terminal");

    assert_eq!(spawner.spawned.len(), 2);
    assert_eq!(service.live_terminal_count(), 2);
    let first_snapshot = service
        .live_terminal_mut(first.id)
        .expect("first live terminal still exists")
        .surface
        .render_snapshot()
        .expect("first snapshot renders");
    let first_text = snapshot_text(&first_snapshot);
    assert!(first_text.contains("first-state"));
}

#[test]
fn resizing_live_terminals_updates_hidden_and_active_ptys() {
    let first = TerminalSpec::new(
        20,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "first",
        "/tmp",
        ProcessSpec::shell(),
    );
    let second = TerminalSpec::new(
        21,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "second",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&first, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("first live spawn succeeds");
    service
        .ensure_live_terminal_with(&second, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("second live spawn succeeds");

    let dimensions = TerminalDimensions::new(100, 30, 9, 18);
    service
        .resize_live_terminals(dimensions)
        .expect("resize succeeds");

    assert_eq!(
        spawner.handles[0].resizes(),
        vec![PtySize::from_dimensions(dimensions)]
    );
    assert_eq!(
        spawner.handles[1].resizes(),
        vec![PtySize::from_dimensions(dimensions)]
    );
}

#[test]
fn resize_live_terminals_continues_after_error_and_retries_failed_pty() {
    let first = TerminalSpec::new(
        22,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "first",
        "/tmp",
        ProcessSpec::shell(),
    );
    let second = TerminalSpec::new(
        23,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "second",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&first, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("first live spawn succeeds");
    service
        .ensure_live_terminal_with(&second, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("second live spawn succeeds");
    spawner.handles[0].fail_next_resize(io::ErrorKind::Other);

    let dimensions = TerminalDimensions::new(100, 30, 9, 18);
    let error = service
        .resize_live_terminals(dimensions)
        .expect_err("first resize reports the failing terminal");

    assert!(
        error
            .to_string()
            .contains("failed to resize PTY for terminal TerminalId(22)")
    );
    assert_eq!(spawner.handles[0].resizes(), Vec::<PtySize>::new());
    assert_eq!(
        spawner.handles[1].resizes(),
        vec![PtySize::from_dimensions(dimensions)]
    );
    assert_eq!(
        service
            .live_terminal_mut(first.id)
            .expect("first live terminal exists")
            .surface
            .dimensions(),
        TerminalDimensions::DEFAULT
    );

    service
        .resize_live_terminals(dimensions)
        .expect("failed PTY resize is retried");

    assert_eq!(
        spawner.handles[0].resizes(),
        vec![PtySize::from_dimensions(dimensions)]
    );
}

#[test]
fn live_terminal_resize_updates_surface_and_pty() {
    let spec = TerminalSpec::new(
        14,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "terminal",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&spec, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("live spawn succeeds");

    let dimensions = TerminalDimensions::new(100, 30, 9, 18);
    service
        .resize_live_terminal(spec.id, dimensions)
        .expect("resize succeeds");

    assert_eq!(
        service
            .live_terminal_mut(spec.id)
            .expect("live terminal exists")
            .surface
            .dimensions(),
        dimensions
    );
    assert_eq!(
        spawner.handles[0].resizes(),
        vec![PtySize::from_dimensions(dimensions)]
    );
}

#[test]
fn write_key_input_encodes_and_writes_to_live_pty() {
    let spec = TerminalSpec::new(
        16,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "terminal",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&spec, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("live spawn succeeds");

    let wrote = service.write_key_input(
        spec.id,
        TerminalKeyInput {
            action: TerminalKeyAction::Press,
            key: libghostty_vt::key::Key::C,
            mods: libghostty_vt::key::Mods::CTRL,
            consumed_mods: libghostty_vt::key::Mods::empty(),
            unshifted_codepoint: Some('c'),
            utf8: None,
        },
    );

    assert!(wrote);
    assert_eq!(spawner.handles[0].written(), vec![b"\x03".to_vec()]);
}

#[test]
fn remove_terminal_terminates_live_pty() {
    let spec = TerminalSpec::new(
        15,
        ProjectId(1),
        TerminalExtensionSpec::Plain,
        "terminal",
        "/tmp",
        ProcessSpec::shell(),
    );
    let mut service = TerminalRuntimeService::<MockLivePty>::new_with_live_pty();
    let mut spawner = MockLiveSpawner::default();
    service
        .ensure_live_terminal_with(&spec, TerminalDimensions::DEFAULT, &mut spawner)
        .expect("live spawn succeeds");

    let removed = service.remove_terminal(spec.id);

    assert!(removed.is_some());
    assert!(spawner.handles[0].terminated());
    assert!(service.terminal(spec.id).is_none());
    assert!(service.live_terminal_mut(spec.id).is_none());
}
