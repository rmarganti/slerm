pub mod app;
pub mod workspace;

pub use app::*;
pub use workspace::*;

use gpui::App;

pub fn init(cx: &mut App) {
    app::init(cx);
}
