use crate::terminal::TerminalId;

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
    TerminalFailedToStart,
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

pub(crate) fn add_attention(
    severity: &mut AttentionSeverity,
    reasons: &mut Vec<AttentionReason>,
    new_severity: AttentionSeverity,
    reason: AttentionReason,
) {
    *severity = (*severity).max(new_severity);
    reasons.push(reason);
}
