use gpui::{Context, FontWeight, IntoElement, Render, Window, div, prelude::*, rgb};

pub struct SlermApp;

impl Render for SlermApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .bg(rgb(0x18181b))
            .text_color(rgb(0xf4f4f5))
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Slerm"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xa1a1aa))
                    .child("A GPUI workspace ready to build on."),
            )
    }
}
