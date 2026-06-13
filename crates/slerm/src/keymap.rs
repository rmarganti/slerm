use gpui::{App, KeyBinding};

use crate::{
    actions::{
        ActiveProjectCycleNext, ActiveProjectCyclePrev, ActiveProjectMoveLeft,
        ActiveProjectMoveRight, ActiveProjectRemove, ActiveProjectSelectByIndex,
        ActiveTerminalClose, ActiveTerminalCycleNext, ActiveTerminalCyclePrev,
        ActiveTerminalSelectByIndex, OpenAddProjectPicker, OpenAddTerminalPicker,
        OpenProjectPicker, OpenRenameProjectModal, Quit,
    },
    ui::{
        menu,
        rename_project_modal::{RenameProjectCancel, RenameProjectConfirm},
        text_input::{
            TextInputBackspace, TextInputCopy, TextInputCut, TextInputDelete,
            TextInputDeleteNextWord, TextInputDeletePreviousWord, TextInputDeleteToEnd,
            TextInputDeleteToStart, TextInputMoveLeft, TextInputMoveLeftSelecting,
            TextInputMoveRight, TextInputMoveRightSelecting, TextInputMoveToEnd,
            TextInputMoveToEndSelecting, TextInputMoveToStart, TextInputMoveToStartSelecting,
            TextInputMoveWordLeft, TextInputMoveWordLeftSelecting, TextInputMoveWordRight,
            TextInputMoveWordRightSelecting, TextInputPaste, TextInputSelectAll,
        },
    },
};

const WORKSPACE_CONTEXT: &str = "workspace";
const TEXT_INPUT_CONTEXT: &str = "TextInput";
const FUZZY_FINDER_CONTEXT: &str = "FuzzyFinder";
const RENAME_PROJECT_MODAL_CONTEXT: &str = "RenameProjectModal";

pub fn init(cx: &mut App) {
    cx.bind_keys(workspace_bindings());
    cx.bind_keys(text_input_bindings());
    cx.bind_keys(picker_bindings());
    cx.bind_keys(rename_project_modal_bindings());
}

fn workspace_bindings() -> Vec<KeyBinding> {
    let mut bindings = vec![
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-t", OpenAddTerminalPicker, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-shift-n", OpenAddProjectPicker, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-alt-o", OpenProjectPicker, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new(
            "cmd-shift-r",
            OpenRenameProjectModal,
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new("cmd-w", ActiveTerminalClose, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-shift-w", ActiveProjectRemove, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-down", ActiveTerminalCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-j", ActiveTerminalCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-up", ActiveTerminalCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-k", ActiveTerminalCyclePrev, Some(WORKSPACE_CONTEXT)),
    ];

    bindings.extend(
        [
            "cmd-1", "cmd-2", "cmd-3", "cmd-4", "cmd-5", "cmd-6", "cmd-7", "cmd-8", "cmd-9",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, keystroke)| {
            KeyBinding::new(
                keystroke,
                ActiveTerminalSelectByIndex { index },
                Some(WORKSPACE_CONTEXT),
            )
        }),
    );

    bindings.extend([
        KeyBinding::new("cmd-right", ActiveProjectCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-l", ActiveProjectCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-left", ActiveProjectCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-h", ActiveProjectCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new(
            "ctrl-cmd-left",
            ActiveProjectMoveLeft,
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new("ctrl-cmd-h", ActiveProjectMoveLeft, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new(
            "ctrl-cmd-right",
            ActiveProjectMoveRight,
            Some(WORKSPACE_CONTEXT),
        ),
        KeyBinding::new(
            "ctrl-cmd-l",
            ActiveProjectMoveRight,
            Some(WORKSPACE_CONTEXT),
        ),
    ]);

    bindings.extend(
        [
            "cmd-ctrl-1",
            "cmd-ctrl-2",
            "cmd-ctrl-3",
            "cmd-ctrl-4",
            "cmd-ctrl-5",
            "cmd-ctrl-6",
            "cmd-ctrl-7",
            "cmd-ctrl-8",
            "cmd-ctrl-9",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, keystroke)| {
            KeyBinding::new(
                keystroke,
                ActiveProjectSelectByIndex { index },
                Some(WORKSPACE_CONTEXT),
            )
        }),
    );

    bindings
}

fn text_input_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("left", TextInputMoveLeft, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-b", TextInputMoveLeft, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new(
            "shift-left",
            TextInputMoveLeftSelecting,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new("right", TextInputMoveRight, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-f", TextInputMoveRight, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new(
            "shift-right",
            TextInputMoveRightSelecting,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new("cmd-left", TextInputMoveToStart, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-a", TextInputMoveToStart, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new(
            "cmd-shift-left",
            TextInputMoveToStartSelecting,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new("cmd-right", TextInputMoveToEnd, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-e", TextInputMoveToEnd, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new(
            "cmd-shift-right",
            TextInputMoveToEndSelecting,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new("alt-left", TextInputMoveWordLeft, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new(
            "alt-shift-left",
            TextInputMoveWordLeftSelecting,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new(
            "alt-right",
            TextInputMoveWordRight,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new(
            "alt-shift-right",
            TextInputMoveWordRightSelecting,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new("backspace", TextInputBackspace, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-h", TextInputBackspace, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("delete", TextInputDelete, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-d", TextInputDelete, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new(
            "alt-backspace",
            TextInputDeletePreviousWord,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new(
            "ctrl-w",
            TextInputDeletePreviousWord,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new(
            "alt-delete",
            TextInputDeleteNextWord,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new(
            "cmd-backspace",
            TextInputDeleteToStart,
            Some(TEXT_INPUT_CONTEXT),
        ),
        KeyBinding::new("ctrl-u", TextInputDeleteToStart, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("cmd-delete", TextInputDeleteToEnd, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("ctrl-k", TextInputDeleteToEnd, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("cmd-a", TextInputSelectAll, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("cmd-c", TextInputCopy, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("cmd-x", TextInputCut, Some(TEXT_INPUT_CONTEXT)),
        KeyBinding::new("cmd-v", TextInputPaste, Some(TEXT_INPUT_CONTEXT)),
    ]
}

fn picker_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("up", menu::SelectPrevious, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("ctrl-p", menu::SelectPrevious, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new(
            "shift-tab",
            menu::SelectPrevious,
            Some(FUZZY_FINDER_CONTEXT),
        ),
        KeyBinding::new("down", menu::SelectNext, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("ctrl-n", menu::SelectNext, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("tab", menu::SelectNext, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("home", menu::SelectFirst, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("cmd-up", menu::SelectFirst, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("pageup", menu::SelectPageUp, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("end", menu::SelectLast, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("cmd-down", menu::SelectLast, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("pagedown", menu::SelectPageDown, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("enter", menu::Confirm, Some(FUZZY_FINDER_CONTEXT)),
        KeyBinding::new("escape", menu::Cancel, Some(FUZZY_FINDER_CONTEXT)),
    ]
}

fn rename_project_modal_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new(
            "enter",
            RenameProjectConfirm,
            Some(RENAME_PROJECT_MODAL_CONTEXT),
        ),
        KeyBinding::new(
            "escape",
            RenameProjectCancel,
            Some(RENAME_PROJECT_MODAL_CONTEXT),
        ),
    ]
}
