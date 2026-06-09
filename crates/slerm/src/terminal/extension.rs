use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AgentKind {
    Codex,
    OpenCode,
    Pi,
    Custom(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentDetectionSpec {
    pub prompt_patterns: Vec<String>,
}

impl Default for AgentDetectionSpec {
    fn default() -> Self {
        Self {
            prompt_patterns: Vec::new(),
        }
    }
}

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TaskPersistence {
    KeepUntilClosed,
    CloseOnSuccess,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TaskNotifyPolicy {
    Never,
    OnFailure,
    OnCompletion,
}

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
