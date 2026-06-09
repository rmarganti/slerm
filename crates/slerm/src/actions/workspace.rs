use gpui::{Action, actions};

actions!(
    slerm,
    [
        ActiveTerminalCycleNext,
        ActiveTerminalCyclePrev,
        ActiveTerminalClose,
        ActiveProjectCycleNext,
        ActiveProjectCyclePrev,
        OpenAddTerminalPicker,
    ]
);

#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = slerm, no_json)]
pub struct ActiveTerminalSelectByIndex {
    pub index: usize,
}
