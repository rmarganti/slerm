use std::path::PathBuf;

use super::*;
use crate::{
    project::model::{Project, ProjectId},
    terminal::{
        ProcessSpec, TerminalId, TerminalSpec,
        extension::{AgentKind, AgentSpec, TerminalExtensionSpec},
    },
    workspace::model::WorkspaceState,
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
