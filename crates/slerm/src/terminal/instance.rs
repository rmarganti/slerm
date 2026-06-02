use std::path::PathBuf;

use crate::{project::model::ProjectId, terminal::kind::TerminalKind};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TerminalInstanceId(pub u64);

#[derive(Clone, Debug)]
pub struct TerminalInstance {
    pub id: TerminalInstanceId,
    pub project_id: ProjectId,
    pub kind: TerminalKind,
    pub title: String,
    pub cwd: PathBuf,
    pub command: Option<String>,
}

impl TerminalInstance {
    pub fn new(
        id: u64,
        project_id: ProjectId,
        kind: TerminalKind,
        title: impl Into<String>,
        cwd: impl Into<PathBuf>,
        command: Option<impl Into<String>>,
    ) -> Self {
        Self {
            id: TerminalInstanceId(id),
            project_id,
            kind,
            title: title.into(),
            cwd: cwd.into(),
            command: command.map(Into::into),
        }
    }
}
