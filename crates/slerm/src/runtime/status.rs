use crate::runtime::{
    AgentStatus, AttentionReason, AttentionSeverity, AttentionState, TaskStatus,
    TerminalExtensionRuntime, TerminalRunStatus, TerminalRuntimeState, add_attention,
};

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
                    AttentionReason::TerminalFailedToStart,
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
