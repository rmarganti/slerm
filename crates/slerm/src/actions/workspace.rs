use gpui::{Action, actions};

actions!(
    slerm,
    [
        ActiveItemCycleNext,
        ActiveItemCyclePrev,
        ActiveProjectCycleNext,
        ActiveProjectCyclePrev,
        OpenAddItemPicker,
    ]
);

#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = slerm, no_json)]
pub struct ActiveItemSelectByIndex {
    pub index: usize,
}
