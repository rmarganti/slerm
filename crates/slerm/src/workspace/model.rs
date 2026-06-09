use serde::{Deserialize, Serialize};

use crate::{
    project::model::{CycleDirection, Project, ProjectId},
    terminal::{
        instance::{ProcessSpec, TerminalSpec},
        kind::{AgentKind, TaskStatus, TerminalKind},
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkspaceState {
    pub projects: Vec<Project>,
    pub active_project: Option<ProjectId>,
}

impl WorkspaceState {
    pub fn mock() -> Self {
        let slerm_id = ProjectId(1);
        let zed_id = ProjectId(2);
        let notes_id = ProjectId(3);

        let slerm = Project::new(1, "slerm", "/Users/rmarganti/code/rmarganti/slerm")
            .with_terminals(vec![
                TerminalSpec::new(
                    1,
                    slerm_id,
                    TerminalKind::Terminal,
                    "shell",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    ProcessSpec::shell(),
                ),
                TerminalSpec::new(
                    2,
                    slerm_id,
                    TerminalKind::Agent(AgentKind::Pi),
                    "pi coding agent",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    ProcessSpec::new("pi", [] as [&str; 0]),
                ),
                TerminalSpec::new(
                    3,
                    slerm_id,
                    TerminalKind::Task {
                        status: TaskStatus::Running,
                    },
                    "cargo run",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    ProcessSpec::new("cargo", ["run", "-p", "slerm"]),
                ),
                TerminalSpec::new(
                    4,
                    slerm_id,
                    TerminalKind::Task {
                        status: TaskStatus::Idle,
                    },
                    "cargo test",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    ProcessSpec::new("cargo", ["test"]),
                ),
            ]);

        let zed = Project::new(2, "zed", "/Users/rmarganti/code/github/zed").with_terminals(vec![
            TerminalSpec::new(
                5,
                zed_id,
                TerminalKind::Agent(AgentKind::Codex),
                "codex",
                "/Users/rmarganti/code/github/zed",
                ProcessSpec::new("codex", [] as [&str; 0]),
            ),
        ]);

        let notes = Project::new(3, "notes", "/Users/rmarganti/notes").with_terminals(vec![
            TerminalSpec::new(
                6,
                notes_id,
                TerminalKind::Terminal,
                "shell",
                "/Users/rmarganti/notes",
                ProcessSpec::shell(),
            ),
            TerminalSpec::new(
                7,
                notes_id,
                TerminalKind::Task {
                    status: TaskStatus::Succeeded,
                },
                "sync vault",
                "/Users/rmarganti/notes",
                ProcessSpec::shell_command("git pull --rebase && git push"),
            ),
            TerminalSpec::new(
                8,
                notes_id,
                TerminalKind::Task {
                    status: TaskStatus::Failed,
                },
                "publish",
                "/Users/rmarganti/notes",
                ProcessSpec::new("make", ["publish"]),
            ),
            TerminalSpec::new(
                9,
                notes_id,
                TerminalKind::Agent(AgentKind::OpenCode),
                "opencode",
                "/Users/rmarganti/notes",
                ProcessSpec::new("opencode", [] as [&str; 0]),
            ),
            TerminalSpec::new(
                10,
                notes_id,
                TerminalKind::Agent(AgentKind::Custom("Claude".to_string())),
                "claude",
                "/Users/rmarganti/notes",
                ProcessSpec::new("claude", [] as [&str; 0]),
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

    pub fn add_terminal_to_active_project(
        &mut self,
    ) -> Option<crate::terminal::instance::TerminalId> {
        let active_project = self.active_project?;
        let next_id = self.next_terminal_id();
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)?;

        let terminal = TerminalSpec::new(
            next_id.0,
            project.id,
            TerminalKind::Terminal,
            "shell",
            project.path.clone(),
            ProcessSpec::shell(),
        );
        project.add_terminal(terminal);
        Some(next_id)
    }

    fn next_terminal_id(&self) -> crate::terminal::instance::TerminalId {
        crate::terminal::instance::TerminalId(
            self.projects
                .iter()
                .flat_map(|project| project.terminals.iter())
                .map(|terminal| terminal.id.0)
                .max()
                .unwrap_or(0)
                + 1,
        )
    }

    pub fn cycle_active_project(&mut self, direction: CycleDirection) {
        if self.projects.is_empty() {
            self.active_project = None;
            return;
        }

        let next_index = self
            .active_project
            .and_then(|active_project| {
                self.projects
                    .iter()
                    .position(|project| project.id == active_project)
            })
            .map(|active_index| match direction {
                CycleDirection::Next => (active_index + 1) % self.projects.len(),
                CycleDirection::Prev => active_index
                    .checked_sub(1)
                    .unwrap_or_else(|| self.projects.len() - 1),
            })
            .unwrap_or_else(|| match direction {
                CycleDirection::Next => 0,
                CycleDirection::Prev => self.projects.len() - 1,
            });

        self.active_project = Some(self.projects[next_index].id);
    }

    pub fn cycle_active_terminal(&mut self, direction: CycleDirection) {
        let Some(active_project) = self.active_project else {
            return;
        };

        if let Some(project) = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)
        {
            project.cycle_active_terminal(direction);
        }
    }

    pub fn select_active_terminal_by_sidebar_index(&mut self, index: usize) {
        let Some(active_project) = self.active_project else {
            return;
        };

        if let Some(project) = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)
        {
            project.select_active_terminal_by_sidebar_index(index);
        }
    }

    pub fn close_active_terminal(&mut self) {
        let Some(active_project) = self.active_project else {
            return;
        };

        if let Some(project) = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)
        {
            project.close_active_terminal();
        }
    }
}
