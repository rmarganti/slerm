use std::path::PathBuf;

use crate::terminal::instance::{TerminalInstance, TerminalInstanceId};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProjectId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CycleDirection {
    Next,
    Prev,
}

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

    pub fn cycle_active_item(&mut self, direction: CycleDirection) {
        let item_ids = self.item_ids_in_sidebar_order();

        if item_ids.is_empty() {
            self.active_item = None;
            return;
        }

        let next_index = self
            .active_item
            .and_then(|active_item| item_ids.iter().position(|item_id| *item_id == active_item))
            .map(|active_index| match direction {
                CycleDirection::Next => (active_index + 1) % item_ids.len(),
                CycleDirection::Prev => active_index
                    .checked_sub(1)
                    .unwrap_or_else(|| item_ids.len() - 1),
            })
            .unwrap_or_else(|| match direction {
                CycleDirection::Next => 0,
                CycleDirection::Prev => item_ids.len() - 1,
            });

        self.active_item = Some(item_ids[next_index]);
    }

    fn item_ids_in_sidebar_order(&self) -> Vec<TerminalInstanceId> {
        let mut items = self.items.iter().collect::<Vec<_>>();
        items.sort_by_key(|item| item.kind.sidebar_order());
        items.into_iter().map(|item| item.id).collect()
    }
}
