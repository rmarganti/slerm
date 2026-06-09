use std::{collections::BTreeMap, env, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{project::model::ProjectId, terminal::extension::TerminalExtensionSpec};

/// Stable persisted identifier for a terminal within the workspace.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TerminalId(pub u64);

/// Structured process launch intent for a terminal.
///
/// This is persisted configuration, not a live process handle. A plain terminal
/// launches the user's shell, while agents and tasks launch their configured CLI
/// or command with explicit arguments and environment overrides.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProcessSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

impl ProcessSpec {
    pub fn new(
        program: impl Into<PathBuf>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            env: BTreeMap::new(),
        }
    }

    pub fn shell() -> Self {
        let shell = env::var_os("SHELL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/bin/sh"));
        Self::new(shell, [] as [&str; 0])
    }

    pub fn shell_command(command: impl Into<String>) -> Self {
        Self::new(Self::shell().program, ["-lc".to_string(), command.into()])
    }

    pub fn display_command_line(&self) -> String {
        std::iter::once(self.program.display().to_string())
            .chain(self.args.iter().cloned())
            .map(|part| quote_command_part(&part))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn quote_command_part(part: &str) -> String {
    if part.is_empty() || part.chars().any(char::is_whitespace) {
        format!("'{part}'")
    } else {
        part.to_string()
    }
}

/// Persisted configuration for one terminal-like unit in Slerm.
///
/// Every sidebar entry is a terminal at its core. Semantic behavior such as
/// "agent" or "task" lives in `extension`; live sessions and statuses live in
/// runtime state instead of this saved spec.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TerminalSpec {
    pub id: TerminalId,
    pub project_id: ProjectId,
    pub extension: TerminalExtensionSpec,
    pub title: String,
    pub cwd: PathBuf,
    pub command: ProcessSpec,
}

impl TerminalSpec {
    pub fn new(
        id: u64,
        project_id: ProjectId,
        extension: TerminalExtensionSpec,
        title: impl Into<String>,
        cwd: impl Into<PathBuf>,
        command: ProcessSpec,
    ) -> Self {
        Self {
            id: TerminalId(id),
            project_id,
            extension,
            title: title.into(),
            cwd: cwd.into(),
            command,
        }
    }
}
