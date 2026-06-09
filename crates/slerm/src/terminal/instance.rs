use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{project::model::ProjectId, terminal::kind::TerminalKind};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TerminalId(pub u64);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TerminalSpec {
    pub id: TerminalId,
    pub project_id: ProjectId,
    pub kind: TerminalKind,
    pub title: String,
    pub cwd: PathBuf,
    pub command: Option<String>,
}

impl TerminalSpec {
    pub fn new(
        id: u64,
        project_id: ProjectId,
        kind: TerminalKind,
        title: impl Into<String>,
        cwd: impl Into<PathBuf>,
        command: Option<impl Into<String>>,
    ) -> Self {
        Self {
            id: TerminalId(id),
            project_id,
            kind,
            title: title.into(),
            cwd: cwd.into(),
            command: command.map(Into::into),
        }
    }
}
