use serde::{Deserialize, Serialize};

use crate::{
    project::model::{CycleDirection, Project, ProjectId},
    terminal::{
        instance::TerminalInstance,
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

    pub fn add_terminal_to_active_project(
        &mut self,
    ) -> Option<crate::terminal::instance::TerminalInstanceId> {
        let active_project = self.active_project?;
        let next_id = self.next_terminal_instance_id();
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)?;

        let terminal = TerminalInstance::new(
            next_id.0,
            project.id,
            TerminalKind::Terminal,
            "shell",
            project.path.clone(),
            None::<String>,
        );
        project.add_item(terminal);
        Some(next_id)
    }

    fn next_terminal_instance_id(&self) -> crate::terminal::instance::TerminalInstanceId {
        crate::terminal::instance::TerminalInstanceId(
            self.projects
                .iter()
                .flat_map(|project| project.items.iter())
                .map(|item| item.id.0)
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

    pub fn cycle_active_item(&mut self, direction: CycleDirection) {
        let Some(active_project) = self.active_project else {
            return;
        };

        if let Some(project) = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)
        {
            project.cycle_active_item(direction);
        }
    }

    pub fn select_active_item_by_sidebar_index(&mut self, index: usize) {
        let Some(active_project) = self.active_project else {
            return;
        };

        if let Some(project) = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)
        {
            project.select_active_item_by_sidebar_index(index);
        }
    }
}
