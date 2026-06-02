use gpui::{App, KeyBinding};

use crate::actions::{
    ActiveItemCycleNext, ActiveItemCyclePrev, ActiveProjectCycleNext, ActiveProjectCyclePrev, Quit,
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
        KeyBinding::new("cmd-right", ActiveProjectCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-l", ActiveProjectCycleNext, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-left", ActiveProjectCyclePrev, Some(WORKSPACE_CONTEXT)),
        KeyBinding::new("cmd-h", ActiveProjectCyclePrev, Some(WORKSPACE_CONTEXT)),
    ]);
}
