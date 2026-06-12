use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};

use crate::{actions, keymap, storage, terminal, ui::root::SlermApp};

pub fn run() {
    Application::new().run(|cx: &mut App| {
        actions::init(cx);
        keymap::init(cx);
        terminal::init::init();

        let bounds = Bounds::centered(None, size(px(900.0), px(600.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                focus: true,
                app_id: Some("dev.slerm.Slerm".to_string()),
                ..Default::default()
            },
            |_, cx| {
                let workspace = storage::load_or_default();
                cx.new(|cx| SlermApp::new(workspace, cx))
            },
        )
        .expect("failed to open Slerm window");

        cx.activate(true);
    });
}
