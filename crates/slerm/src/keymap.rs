use gpui::{App, KeyBinding};

use crate::{
    actions::{
        ActiveProjectCycleNext, ActiveProjectCyclePrev, ActiveProjectSelectByIndex,
        ActiveTerminalClose, ActiveTerminalCycleNext, ActiveTerminalCyclePrev,
        ActiveTerminalSelectByIndex, OpenAddTerminalPicker, OpenProjectPicker, Quit,
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
        KeyBinding::new("cmd-t", OpenAddTerminalPicker, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-alt-o", OpenProjectPicker, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-w", ActiveTerminalClose, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-down", ActiveTerminalCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-j", ActiveTerminalCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-up", ActiveTerminalCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-k", ActiveTerminalCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new(
            "cmd-1",
            ActiveTerminalSelectByIndex { index: 0 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-2",
            ActiveTerminalSelectByIndex { index: 1 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-3",
            ActiveTerminalSelectByIndex { index: 2 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-4",
            ActiveTerminalSelectByIndex { index: 3 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-5",
            ActiveTerminalSelectByIndex { index: 4 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-6",
            ActiveTerminalSelectByIndex { index: 5 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-7",
            ActiveTerminalSelectByIndex { index: 6 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-8",
            ActiveTerminalSelectByIndex { index: 7 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-9",
            ActiveTerminalSelectByIndex { index: 8 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new("cmd-right", ActiveProjectCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-l", ActiveProjectCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-left", ActiveProjectCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-h", ActiveProjectCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new(
            "cmd-alt-1",
            ActiveProjectSelectByIndex { index: 0 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-alt-2",
            ActiveProjectSelectByIndex { index: 1 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-alt-3",
            ActiveProjectSelectByIndex { index: 2 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-alt-4",
            ActiveProjectSelectByIndex { index: 3 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-alt-5",
            ActiveProjectSelectByIndex { index: 4 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-alt-6",
            ActiveProjectSelectByIndex { index: 5 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-alt-7",
            ActiveProjectSelectByIndex { index: 6 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-alt-8",
            ActiveProjectSelectByIndex { index: 7 },
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-alt-9",
            ActiveProjectSelectByIndex { index: 8 },
            Some(WORKSPACE_CONTEXT),
        ),
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
