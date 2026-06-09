use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::terminal::instance::{TerminalId, TerminalSpec};

/// Stable persisted identifier for a top-level project.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProjectId(pub u64);

/// Direction for keyboard-driven cycling through projects or terminals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CycleDirection {
    Next,
    Prev,
}

/// Persisted project workspace containing ordered terminal specs and selection.
///
/// Projects are Slerm's top-level navigation units. Only one project is active
/// in the main workspace, while inactive projects can still surface attention.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub path: PathBuf,
    pub terminals: Vec<TerminalSpec>,
    pub active_terminal: Option<TerminalId>,
}

impl Project {
    pub fn new(id: u64, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: ProjectId(id),
            name: name.into(),
            path: path.into(),
            terminals: Vec::new(),
            active_terminal: None,
        }
    }

    pub fn with_terminals(mut self, terminals: Vec<TerminalSpec>) -> Self {
        self.active_terminal = terminals.first().map(|terminal| terminal.id);
        self.terminals = terminals;
        self
    }

    pub fn active_terminal(&self) -> Option<&TerminalSpec> {
        let active_terminal = self.active_terminal?;
        self.terminals
            .iter()
            .find(|terminal| terminal.id == active_terminal)
    }

    pub fn cycle_active_terminal(&mut self, direction: CycleDirection) {
        let terminal_ids = self.terminal_ids_in_sidebar_order();

        if terminal_ids.is_empty() {
            self.active_terminal = None;
            return;
        }

        let next_index = self
            .active_terminal
            .and_then(|active_terminal| {
                terminal_ids
                    .iter()
                    .position(|terminal_id| *terminal_id == active_terminal)
            })
            .map(|active_index| match direction {
                CycleDirection::Next => (active_index + 1) % terminal_ids.len(),
                CycleDirection::Prev => active_index
                    .checked_sub(1)
                    .unwrap_or_else(|| terminal_ids.len() - 1),
            })
            .unwrap_or_else(|| match direction {
                CycleDirection::Next => 0,
                CycleDirection::Prev => terminal_ids.len() - 1,
            });

        self.active_terminal = Some(terminal_ids[next_index]);
    }

    pub fn select_active_terminal_by_sidebar_index(&mut self, index: usize) {
        if let Some(terminal_id) = self.terminal_ids_in_sidebar_order().get(index).copied() {
            self.active_terminal = Some(terminal_id);
        }
    }

    pub fn close_active_terminal(&mut self) -> Option<TerminalId> {
        let active_terminal = self.active_terminal?;

        let terminal_ids = self.terminal_ids_in_sidebar_order();
        let closed_index = terminal_ids
            .iter()
            .position(|terminal_id| *terminal_id == active_terminal)
            .unwrap_or(0);

        self.terminals
            .retain(|terminal| terminal.id != active_terminal);

        let remaining_terminal_ids = self.terminal_ids_in_sidebar_order();
        self.active_terminal = if remaining_terminal_ids.is_empty() {
            None
        } else {
            Some(remaining_terminal_ids[closed_index.min(remaining_terminal_ids.len() - 1)])
        };

        Some(active_terminal)
    }

    pub fn add_terminal(&mut self, terminal: TerminalSpec) {
        self.active_terminal = Some(terminal.id);
        self.terminals.push(terminal);
    }

    pub fn terminals_in_sidebar_order(&self) -> Vec<&TerminalSpec> {
        let mut terminals = self.terminals.iter().collect::<Vec<_>>();
        terminals.sort_by_key(|terminal| terminal.extension.sidebar_order());
        terminals
    }

    fn terminal_ids_in_sidebar_order(&self) -> Vec<TerminalId> {
        self.terminals_in_sidebar_order()
            .into_iter()
            .map(|terminal| terminal.id)
            .collect()
    }
}
