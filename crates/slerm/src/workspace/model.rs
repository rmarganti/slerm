use crate::{
    project::model::{Project, ProjectId},
    terminal::{
        instance::TerminalInstance,
        kind::{AgentKind, TaskStatus, TerminalKind},
    },
};

#[derive(Clone, Debug)]
pub struct WorkspaceState {
    pub projects: Vec<Project>,
    pub active_project: Option<ProjectId>,
}

impl WorkspaceState {
    pub fn mock() -> Self {
        let slerm_id = ProjectId(1);
        let zed_id = ProjectId(2);
        let notes_id = ProjectId(3);

        let slerm =
            Project::new(1, "slerm", "/Users/rmarganti/code/rmarganti/slerm").with_items(vec![
                TerminalInstance::new(
                    1,
                    slerm_id,
                    TerminalKind::Terminal,
                    "shell",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    None::<String>,
                ),
                TerminalInstance::new(
                    2,
                    slerm_id,
                    TerminalKind::Agent(AgentKind::Pi),
                    "pi coding agent",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    Some("pi"),
                ),
                TerminalInstance::new(
                    3,
                    slerm_id,
                    TerminalKind::Task {
                        status: TaskStatus::Running,
                    },
                    "cargo run",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    Some("cargo run -p slerm"),
                ),
                TerminalInstance::new(
                    4,
                    slerm_id,
                    TerminalKind::Task {
                        status: TaskStatus::Idle,
                    },
                    "cargo test",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    Some("cargo test"),
                ),
            ]);

        let zed = Project::new(2, "zed", "/Users/rmarganti/code/github/zed").with_items(vec![
            TerminalInstance::new(
                5,
                zed_id,
                TerminalKind::Agent(AgentKind::Codex),
                "codex",
                "/Users/rmarganti/code/github/zed",
                Some("codex"),
            ),
        ]);

        let notes = Project::new(3, "notes", "/Users/rmarganti/notes").with_items(vec![
            TerminalInstance::new(
                6,
                notes_id,
                TerminalKind::Terminal,
                "shell",
                "/Users/rmarganti/notes",
                None::<String>,
            ),
            TerminalInstance::new(
                7,
                notes_id,
                TerminalKind::Task {
                    status: TaskStatus::Succeeded,
                },
                "sync vault",
                "/Users/rmarganti/notes",
                Some("git pull --rebase && git push"),
            ),
            TerminalInstance::new(
                8,
                notes_id,
                TerminalKind::Task {
                    status: TaskStatus::Failed,
                },
                "publish",
                "/Users/rmarganti/notes",
                Some("make publish"),
            ),
            TerminalInstance::new(
                9,
                notes_id,
                TerminalKind::Agent(AgentKind::OpenCode),
                "opencode",
                "/Users/rmarganti/notes",
                Some("opencode"),
            ),
            TerminalInstance::new(
                10,
                notes_id,
                TerminalKind::Agent(AgentKind::Custom("Claude".to_string())),
                "claude",
                "/Users/rmarganti/notes",
                Some("claude"),
            ),
        ]);

        Self {
            projects: vec![slerm, zed, notes],
            active_project: Some(slerm_id),
        }
    }

    pub fn active_project(&self) -> Option<&Project> {
        let active_project = self.active_project?;
        self.projects
            .iter()
            .find(|project| project.id == active_project)
    }
}
