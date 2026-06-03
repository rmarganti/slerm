use gpui::{App, KeyBinding};

use crate::{
    actions::{
        ActiveItemClose, ActiveItemCycleNext, ActiveItemCyclePrev, ActiveItemSelectByIndex,
        ActiveProjectCycleNext, ActiveProjectCyclePrev, OpenAddItemPicker, Quit,
    },
    ui::{
        fuzzy_finder::{
            FuzzyFinderCancel, FuzzyFinderConfirm, FuzzyFinderSelectNext, FuzzyFinderSelectPrev,
        },
        text_input::{
            TextInputBackspace, TextInputDelete, TextInputMoveLeft, TextInputMoveRight,
            TextInputMoveToEnd, TextInputMoveToStart, TextInputPaste,
        },
    },
};

const WORKSPACE_CONTEXT: &str = "workspace";
const TEXT_INPUT_CONTEXT: &str = "TextInput";
const FUZZY_FINDER_CONTEXT: &str = "FuzzyFinder";

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("ctrl-q", Quit, None),
        KeyBinding::new("cmd-t", OpenAddItemPicker, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-w", ActiveItemClose, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-down", ActiveItemCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-j", ActiveItemCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-up", ActiveItemCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-k", ActiveItemCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new(
            "cmd-1",
            ActiveItemSelectByIndex { index: 0 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-2",
            ActiveItemSelectByIndex { index: 1 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-3",
            ActiveItemSelectByIndex { index: 2 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-4",
            ActiveItemSelectByIndex { index: 3 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-5",
            ActiveItemSelectByIndex { index: 4 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-6",
            ActiveItemSelectByIndex { index: 5 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-7",
            ActiveItemSelectByIndex { index: 6 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-8",
            ActiveItemSelectByIndex { index: 7 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-9",
            ActiveItemSelectByIndex { index: 8 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new("cmd-right", ActiveProjectCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-l", ActiveProjectCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-left", ActiveProjectCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-h", ActiveProjectCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("left", TextInputMoveLeft, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-b", TextInputMoveLeft, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("right", TextInputMoveRight, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-f", TextInputMoveRight, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("cmd-left", TextInputMoveToStart, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-a", TextInputMoveToStart, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("cmd-right", TextInputMoveToEnd, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-e", TextInputMoveToEnd, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("backspace", TextInputBackspace, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-h", TextInputBackspace, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("delete", TextInputDelete, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-d", TextInputDelete, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("cmd-v", TextInputPaste, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("up", FuzzyFinderSelectPrev, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("ctrl-p", FuzzyFinderSelectPrev, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("down", FuzzyFinderSelectNext, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("ctrl-n", FuzzyFinderSelectNext, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("enter", FuzzyFinderConfirm, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("escape", FuzzyFinderCancel, Some(FUZZY_FINDER_CONTEXT)),
    ]);
}
