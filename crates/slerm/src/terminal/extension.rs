use serde::{Deserialize, Serialize};

/// Persisted agent-specific configuration for a terminal.
///
/// The process to launch is still stored on `TerminalSpec`; this only describes
/// how Slerm should interpret that terminal as an agent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentSpec {
    pub kind: AgentKind,
    pub detection: AgentDetectionSpec,
}

impl AgentSpec {
    pub fn new(kind: AgentKind) -> Self {
        Self {
            kind,
            detection: AgentDetectionSpec::default(),
        }
    }
}

/// Built-in agent families Slerm can recognize, plus custom agent labels.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AgentKind {
    Codex,
    Gemini,
    OpenCode,
    Pi,
    Custom(String),
}

impl AgentKind {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Codex => "Codex",
            Self::Gemini => "Gemini",
            Self::OpenCode => "OpenCode",
            Self::Pi => "Pi Coding Agent",
            Self::Custom(name) => name,
        }
    }

    pub fn command_name(&self) -> &str {
        match self {
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
            Self::Pi => "pi",
            Self::Custom(name) => name,
        }
    }
}

/// Persisted hints for deriving an agent's live status from terminal output.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentDetectionSpec {
    pub prompt_patterns: Vec<String>,
}

/// Persisted task behavior for a terminal that represents a repeatable command.
///
/// Runtime task state such as running, succeeded, or failed is intentionally not
/// saved here so stale process state is not restored on app launch.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaskSpec {
    pub restart: RestartPolicy,
    pub persistence: TaskPersistence,
    pub notify: TaskNotifyPolicy,
}

impl Default for TaskSpec {
    fn default() -> Self {
        Self {
            restart: RestartPolicy::Never,
            persistence: TaskPersistence::KeepUntilClosed,
            notify: TaskNotifyPolicy::OnFailure,
        }
    }
}

/// How Slerm should restart a task after its process exits.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
}

/// Whether a completed task terminal remains in the project model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TaskPersistence {
    KeepUntilClosed,
    CloseOnSuccess,
}

/// Which task outcomes should surface user attention.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TaskNotifyPolicy {
    Never,
    OnFailure,
    OnCompletion,
}

/// Persisted semantic extension for a terminal.
///
/// `Plain` terminals have no extra semantics. `Agent` and `Task` terminals use
/// the same terminal/process foundation but opt into agent or task behavior.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TerminalExtensionSpec {
    Plain,
    Agent(AgentSpec),
    Task(TaskSpec),
}

impl TerminalExtensionSpec {
    pub fn section_label(&self) -> &'static str {
        match self {
            Self::Plain => "Terminals",
            Self::Agent(_) => "Agents",
            Self::Task(_) => "Tasks",
        }
    }

    pub fn sidebar_order(&self) -> usize {
        match self {
            Self::Plain => 0,
            Self::Agent(_) => 1,
            Self::Task(_) => 2,
        }
    }
}
