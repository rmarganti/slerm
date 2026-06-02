#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentKind {
    Codex,
    OpenCode,
    Pi,
    Custom(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskStatus {
    Idle,
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalKind {
    Terminal,
    Agent(AgentKind),
    Task { status: TaskStatus },
}

impl TerminalKind {
    pub fn section_label(&self) -> &'static str {
        match self {
            Self::Terminal => "Terminals",
            Self::Agent(_) => "Agents",
            Self::Task { .. } => "Tasks",
        }
    }
}
