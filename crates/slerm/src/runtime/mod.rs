pub mod attention;
pub mod backend;
pub mod extension;
pub mod mock_backend;
pub mod pty;
pub mod service;
pub mod session;
pub mod status;

pub(crate) use attention::add_attention;
pub use attention::{
    AttentionReason, AttentionSeverity, AttentionState, ProjectAttention, ProjectAttentionReason,
};
pub use backend::{PtyBackend, SpawnProcessRequest, TerminalSize};
pub use extension::{
    AgentRuntime, AgentStatus, TaskRuntime, TaskStatus, TerminalExtensionRuntime,
    TerminalRuntimeState,
};
pub use mock_backend::MockPtyBackend;
pub use pty::{Pty, PtySize, UnixPtyBackend};
pub use service::TerminalRuntimeService;
pub use session::{
    ExitStatusSnapshot, SessionId, TerminalRunStatus, TerminalSession, TerminalSessionState,
};
pub use status::{TerminalActivityStatus, TerminalOutcomeStatus, TerminalStatus};

#[cfg(test)]
mod tests;
