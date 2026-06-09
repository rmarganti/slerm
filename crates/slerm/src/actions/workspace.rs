use gpui::{Action, actions};

actions!(
    slerm,
    [
        ActiveTerminalCycleNext,
        ActiveTerminalCyclePrev,
        ActiveTerminalClose,
        ActiveProjectCycleNext,
        ActiveProjectCyclePrev,
        ActiveProjectMoveLeft,
        ActiveProjectMoveRight,
        ActiveProjectRemove,
        OpenAddProjectPicker,
        OpenProjectPicker,
        OpenAddTerminalPicker,
    ]
);

#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = slerm, no_json)]
pub struct ActiveTerminalSelectByIndex {
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = slerm, no_json)]
pub struct ActiveProjectSelectByIndex {
    pub index: usize,
}
