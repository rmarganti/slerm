use gpui::{App, KeyBinding};

use crate::actions::{
    ActiveItemCycleNext, ActiveItemCyclePrev, ActiveItemSelectByIndex, ActiveProjectCycleNext,
    ActiveProjectCyclePrev, Quit,
};

const WORKSPACE_CONTEXT: &str = "workspace";

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("ctrl-q", Quit, None),
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
    ]);
}
