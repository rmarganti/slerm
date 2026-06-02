use gpui::{App, actions};

actions!(slerm, [Quit]);

pub fn init(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
}
