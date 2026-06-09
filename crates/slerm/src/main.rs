mod actions;
mod app;
mod keymap;
mod project;
pub mod runtime;
mod storage;
mod terminal;
mod theme;
mod ui;
mod workspace;

fn main() {
    app::run();
}
