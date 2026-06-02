use std::path::PathBuf;

use crate::terminal::instance::{TerminalInstance, TerminalInstanceId};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProjectId(pub u64);

#[derive(Clone, Debug)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub path: PathBuf,
    pub items: Vec<TerminalInstance>,
    pub active_item: Option<TerminalInstanceId>,
}

impl Project {
    pub fn new(id: u64, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: ProjectId(id),
            name: name.into(),
            path: path.into(),
            items: Vec::new(),
            active_item: None,
        }
    }

    pub fn with_items(mut self, items: Vec<TerminalInstance>) -> Self {
        self.active_item = items.first().map(|item| item.id);
        self.items = items;
        self
    }

    pub fn active_item(&self) -> Option<&TerminalInstance> {
        let active_item = self.active_item?;
        self.items.iter().find(|item| item.id == active_item)
    }
}
