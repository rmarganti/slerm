use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    project::model::{CycleDirection, Project, ProjectId},
    terminal::{
        extension::{AgentKind, AgentSpec, TaskSpec, TerminalExtensionSpec},
        spec::{ProcessSpec, TerminalId, TerminalSpec},
    },
};

/// Persisted workspace model for projects and active project selection.
///
/// This state describes what Slerm should know across launches. Live terminal
/// sessions, process handles, task status, and agent status belong in runtime
/// services instead.
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
                    TerminalExtensionSpec::Plain,
                    "shell",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    ProcessSpec::shell(),
                ),
                TerminalSpec::new(
                    2,
                    slerm_id,
                    TerminalExtensionSpec::Agent(AgentSpec::new(AgentKind::Pi)),
                    "pi coding agent",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    ProcessSpec::new("pi", [] as [&str; 0]),
                ),
                TerminalSpec::new(
                    3,
                    slerm_id,
                    TerminalExtensionSpec::Task(TaskSpec::default()),
                    "cargo run",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    ProcessSpec::new("cargo", ["run", "-p", "slerm"]),
                ),
                TerminalSpec::new(
                    4,
                    slerm_id,
                    TerminalExtensionSpec::Task(TaskSpec::default()),
                    "cargo test",
                    "/Users/rmarganti/code/rmarganti/slerm",
                    ProcessSpec::new("cargo", ["test"]),
                ),
            ]);

        let zed = Project::new(2, "zed", "/Users/rmarganti/code/github/zed").with_terminals(vec![
            TerminalSpec::new(
                5,
                zed_id,
                TerminalExtensionSpec::Agent(AgentSpec::new(AgentKind::Codex)),
                "codex",
                "/Users/rmarganti/code/github/zed",
                ProcessSpec::new("codex", [] as [&str; 0]),
            ),
        ]);

        let notes = Project::new(3, "notes", "/Users/rmarganti/notes").with_terminals(vec![
            TerminalSpec::new(
                6,
                notes_id,
                TerminalExtensionSpec::Plain,
                "shell",
                "/Users/rmarganti/notes",
                ProcessSpec::shell(),
            ),
            TerminalSpec::new(
                7,
                notes_id,
                TerminalExtensionSpec::Task(TaskSpec::default()),
                "sync vault",
                "/Users/rmarganti/notes",
                ProcessSpec::shell_command("git pull --rebase && git push"),
            ),
            TerminalSpec::new(
                8,
                notes_id,
                TerminalExtensionSpec::Task(TaskSpec::default()),
                "publish",
                "/Users/rmarganti/notes",
                ProcessSpec::new("make", ["publish"]),
            ),
            TerminalSpec::new(
                9,
                notes_id,
                TerminalExtensionSpec::Agent(AgentSpec::new(AgentKind::OpenCode)),
                "opencode",
                "/Users/rmarganti/notes",
                ProcessSpec::new("opencode", [] as [&str; 0]),
            ),
            TerminalSpec::new(
                10,
                notes_id,
                TerminalExtensionSpec::Agent(AgentSpec::new(AgentKind::Custom(
                    "Claude".to_string(),
                ))),
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

    pub fn next_project_id(&self) -> ProjectId {
        ProjectId(
            self.projects
                .iter()
                .map(|project| project.id.0)
                .max()
                .unwrap_or(0)
                + 1,
        )
    }

    pub fn next_terminal_id(&self) -> TerminalId {
        TerminalId(
            self.projects
                .iter()
                .flat_map(|project| project.terminals.iter())
                .map(|terminal| terminal.id.0)
                .max()
                .unwrap_or(0)
                + 1,
        )
    }

    pub fn add_project(&mut self, path: impl Into<PathBuf>) -> Project {
        let path = path.into();
        let project_id = self.next_project_id();
        let terminal_id = self.next_terminal_id();
        let name = infer_project_name(&path);

        let terminal = TerminalSpec::new(
            terminal_id.0,
            project_id,
            TerminalExtensionSpec::Plain,
            "shell",
            path.clone(),
            ProcessSpec::shell(),
        );
        let project = Project::new(project_id.0, name, path).with_terminals(vec![terminal]);

        self.projects.push(project.clone());
        self.active_project = Some(project_id);

        project
    }

    pub fn add_terminal_to_active_project(&mut self) -> Option<TerminalSpec> {
        let active_project = self.active_project?;
        let next_id = self.next_terminal_id();
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)?;

        let terminal = TerminalSpec::new(
            next_id.0,
            project.id,
            TerminalExtensionSpec::Plain,
            "shell",
            project.path.clone(),
            ProcessSpec::shell(),
        );
        project.add_terminal(terminal.clone());
        Some(terminal)
    }

    pub fn add_agent_to_active_project(&mut self, kind: AgentKind) -> Option<TerminalSpec> {
        let active_project = self.active_project?;
        let next_id = self.next_terminal_id();
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)?;

        let terminal = TerminalSpec::new(
            next_id.0,
            project.id,
            TerminalExtensionSpec::Agent(AgentSpec::new(kind.clone())),
            kind.display_name(),
            project.path.clone(),
            ProcessSpec::new(kind.command_name(), [] as [&str; 0]),
        );
        project.add_terminal(terminal.clone());
        Some(terminal)
    }

    pub fn select_active_project_by_id(&mut self, project_id: ProjectId) -> bool {
        if self.projects.iter().any(|project| project.id == project_id) {
            self.active_project = Some(project_id);
            true
        } else {
            false
        }
    }

    pub fn select_active_project_by_index(&mut self, index: usize) -> bool {
        if let Some(project) = self.projects.get(index) {
            self.active_project = Some(project.id);
            true
        } else {
            false
        }
    }

    pub fn remove_active_project(&mut self) -> Vec<TerminalId> {
        let Some(active_project) = self.active_project else {
            return Vec::new();
        };

        let Some(removed_index) = self
            .projects
            .iter()
            .position(|project| project.id == active_project)
        else {
            self.active_project = None;
            return Vec::new();
        };

        let removed_project = self.projects.remove(removed_index);
        let removed_terminal_ids = removed_project
            .terminals
            .iter()
            .map(|terminal| terminal.id)
            .collect();

        self.active_project = self
            .projects
            .get(removed_index)
            .or_else(|| {
                removed_index
                    .checked_sub(1)
                    .and_then(|index| self.projects.get(index))
            })
            .map(|project| project.id);

        removed_terminal_ids
    }

    pub fn rename_active_project(&mut self, new_name: impl AsRef<str>) -> bool {
        let Some(active_project) = self.active_project else {
            return false;
        };
        let trimmed_name = new_name.as_ref().trim();
        if trimmed_name.is_empty() {
            return false;
        }

        let Some(project) = self
            .projects
            .iter_mut()
            .find(|project| project.id == active_project)
        else {
            return false;
        };

        project.name = trimmed_name.to_string();
        true
    }

    pub fn move_active_project(&mut self, direction: CycleDirection) -> bool {
        let Some(active_project) = self.active_project else {
            return false;
        };
        let Some(active_index) = self
            .projects
            .iter()
            .position(|project| project.id == active_project)
        else {
            return false;
        };

        let Some(target_index) = (match direction {
            CycleDirection::Prev => active_index.checked_sub(1),
            CycleDirection::Next => {
                let next_index = active_index + 1;
                (next_index < self.projects.len()).then_some(next_index)
            }
        }) else {
            return false;
        };

        self.projects.swap(active_index, target_index);
        self.active_project = Some(active_project);
        true
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

    pub fn close_active_terminal(&mut self) -> Option<TerminalId> {
        let active_project = self.active_project?;

        self.projects
            .iter_mut()
            .find(|project| project.id == active_project)?
            .close_active_terminal()
    }
}

fn infer_project_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "Untitled Project".to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn workspace_with_three_projects() -> WorkspaceState {
        WorkspaceState {
            projects: vec![
                Project::new(4, "first", "/tmp/first").with_terminals(vec![TerminalSpec::new(
                    10,
                    ProjectId(4),
                    TerminalExtensionSpec::Plain,
                    "shell",
                    "/tmp/first",
                    ProcessSpec::shell(),
                )]),
                Project::new(9, "second", "/tmp/second").with_terminals(vec![TerminalSpec::new(
                    12,
                    ProjectId(9),
                    TerminalExtensionSpec::Plain,
                    "shell",
                    "/tmp/second",
                    ProcessSpec::shell(),
                )]),
                Project::new(6, "third", "/tmp/third").with_terminals(vec![TerminalSpec::new(
                    11,
                    ProjectId(6),
                    TerminalExtensionSpec::Plain,
                    "shell",
                    "/tmp/third",
                    ProcessSpec::shell(),
                )]),
            ],
            active_project: Some(ProjectId(9)),
        }
    }

    #[test]
    fn next_ids_are_max_existing_plus_one() {
        let workspace = workspace_with_three_projects();

        assert_eq!(workspace.next_project_id(), ProjectId(10));
        assert_eq!(workspace.next_terminal_id(), TerminalId(13));
    }

    #[test]
    fn next_ids_start_at_one_for_empty_workspace() {
        let workspace = WorkspaceState {
            projects: Vec::new(),
            active_project: None,
        };

        assert_eq!(workspace.next_project_id(), ProjectId(1));
        assert_eq!(workspace.next_terminal_id(), TerminalId(1));
    }

    #[test]
    fn add_project_infers_name_adds_shell_terminal_and_selects_it() {
        let mut workspace = workspace_with_three_projects();

        let project = workspace.add_project("/tmp/new-project");

        assert_eq!(project.id, ProjectId(10));
        assert_eq!(project.name, "new-project");
        assert_eq!(project.path, PathBuf::from("/tmp/new-project"));
        assert_eq!(project.terminals.len(), 1);
        assert_eq!(project.terminals[0].id, TerminalId(13));
        assert_eq!(project.terminals[0].project_id, project.id);
        assert_eq!(project.terminals[0].extension, TerminalExtensionSpec::Plain);
        assert_eq!(project.terminals[0].title, "shell");
        assert_eq!(project.terminals[0].cwd, project.path);
        assert_eq!(project.active_terminal, Some(TerminalId(13)));
        assert_eq!(workspace.active_project, Some(project.id));
        assert_eq!(
            workspace.projects.last().map(|project| project.id),
            Some(project.id)
        );
    }

    #[test]
    fn add_project_handles_paths_without_a_basename() {
        let mut workspace = WorkspaceState {
            projects: Vec::new(),
            active_project: None,
        };

        let project = workspace.add_project("/");

        assert_eq!(project.name, "Untitled Project");
        assert_eq!(workspace.active_project, Some(ProjectId(1)));
    }

    #[test]
    fn add_agent_to_active_project_uses_agent_launch_spec_and_selects_it() {
        for (kind, title, command) in [
            (AgentKind::Pi, "Pi Coding Agent", "pi"),
            (AgentKind::OpenCode, "OpenCode", "opencode"),
            (AgentKind::Gemini, "Gemini", "gemini"),
            (AgentKind::Codex, "Codex", "codex"),
        ] {
            let mut workspace = workspace_with_three_projects();

            let terminal = workspace
                .add_agent_to_active_project(kind.clone())
                .expect("active project should accept an agent");

            assert_eq!(terminal.id, TerminalId(13));
            assert_eq!(terminal.project_id, ProjectId(9));
            assert_eq!(
                terminal.extension,
                TerminalExtensionSpec::Agent(AgentSpec::new(kind))
            );
            assert_eq!(terminal.title, title);
            assert_eq!(terminal.cwd, PathBuf::from("/tmp/second"));
            assert_eq!(terminal.command, ProcessSpec::new(command, [] as [&str; 0]));
            assert_eq!(
                workspace
                    .active_project()
                    .and_then(|project| project.active_terminal),
                Some(TerminalId(13))
            );
        }
    }

    #[test]
    fn add_agent_to_active_project_returns_none_without_active_project() {
        let mut workspace = workspace_with_three_projects();
        workspace.active_project = None;

        assert!(
            workspace
                .add_agent_to_active_project(AgentKind::Pi)
                .is_none()
        );
    }

    #[test]
    fn project_selection_by_id_and_index_reports_success() {
        let mut workspace = workspace_with_three_projects();

        assert!(workspace.select_active_project_by_id(ProjectId(6)));
        assert_eq!(workspace.active_project, Some(ProjectId(6)));
        assert!(!workspace.select_active_project_by_id(ProjectId(99)));
        assert_eq!(workspace.active_project, Some(ProjectId(6)));

        assert!(workspace.select_active_project_by_index(0));
        assert_eq!(workspace.active_project, Some(ProjectId(4)));
        assert!(!workspace.select_active_project_by_index(99));
        assert_eq!(workspace.active_project, Some(ProjectId(4)));
    }

    #[test]
    fn remove_active_project_returns_terminal_ids_and_selects_next_project() {
        let mut workspace = workspace_with_three_projects();

        let removed_terminal_ids = workspace.remove_active_project();

        assert_eq!(removed_terminal_ids, vec![TerminalId(12)]);
        assert_eq!(
            workspace
                .projects
                .iter()
                .map(|project| project.id)
                .collect::<Vec<_>>(),
            vec![ProjectId(4), ProjectId(6)]
        );
        assert_eq!(workspace.active_project, Some(ProjectId(6)));
    }

    #[test]
    fn remove_active_project_selects_previous_when_removing_last() {
        let mut workspace = workspace_with_three_projects();
        workspace.active_project = Some(ProjectId(6));

        let removed_terminal_ids = workspace.remove_active_project();

        assert_eq!(removed_terminal_ids, vec![TerminalId(11)]);
        assert_eq!(workspace.active_project, Some(ProjectId(9)));
    }

    #[test]
    fn remove_active_project_handles_empty_and_stale_selection() {
        let mut empty = WorkspaceState {
            projects: Vec::new(),
            active_project: None,
        };
        assert!(empty.remove_active_project().is_empty());
        assert_eq!(empty.active_project, None);

        let mut workspace = workspace_with_three_projects();
        workspace.active_project = Some(ProjectId(99));
        assert!(workspace.remove_active_project().is_empty());
        assert_eq!(workspace.active_project, None);
        assert_eq!(workspace.projects.len(), 3);
    }

    #[test]
    fn rename_active_project_trims_names_and_rejects_empty_names() {
        let mut workspace = workspace_with_three_projects();

        assert!(workspace.rename_active_project("  renamed  "));
        assert_eq!(
            workspace
                .active_project()
                .map(|project| project.name.as_str()),
            Some("renamed")
        );
        assert!(!workspace.rename_active_project("   "));
        assert_eq!(
            workspace
                .active_project()
                .map(|project| project.name.as_str()),
            Some("renamed")
        );

        workspace.active_project = None;
        assert!(!workspace.rename_active_project("ignored"));
    }

    #[test]
    fn move_active_project_reorders_without_changing_selection() {
        let mut workspace = workspace_with_three_projects();

        assert!(workspace.move_active_project(CycleDirection::Prev));
        assert_eq!(
            workspace
                .projects
                .iter()
                .map(|project| project.id)
                .collect::<Vec<_>>(),
            vec![ProjectId(9), ProjectId(4), ProjectId(6)]
        );
        assert_eq!(workspace.active_project, Some(ProjectId(9)));

        assert!(workspace.move_active_project(CycleDirection::Next));
        assert_eq!(
            workspace
                .projects
                .iter()
                .map(|project| project.id)
                .collect::<Vec<_>>(),
            vec![ProjectId(4), ProjectId(9), ProjectId(6)]
        );
        assert_eq!(workspace.active_project, Some(ProjectId(9)));
    }

    #[test]
    fn move_active_project_is_noop_at_boundaries_or_without_active_project() {
        let mut workspace = workspace_with_three_projects();
        workspace.active_project = Some(ProjectId(4));

        assert!(!workspace.move_active_project(CycleDirection::Prev));
        assert_eq!(
            workspace
                .projects
                .iter()
                .map(|project| project.id)
                .collect::<Vec<_>>(),
            vec![ProjectId(4), ProjectId(9), ProjectId(6)]
        );

        workspace.active_project = Some(ProjectId(6));
        assert!(!workspace.move_active_project(CycleDirection::Next));
        assert_eq!(
            workspace
                .projects
                .iter()
                .map(|project| project.id)
                .collect::<Vec<_>>(),
            vec![ProjectId(4), ProjectId(9), ProjectId(6)]
        );

        workspace.active_project = None;
        assert!(!workspace.move_active_project(CycleDirection::Next));
    }
}
