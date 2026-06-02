use gpui::{Context, FontWeight, IntoElement, Render, Window, div, prelude::*};

use crate::theme;

pub struct SlermApp;

impl Render for SlermApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::active();

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .bg(theme.bg)
            .text_color(theme.fg)
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Slerm"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.minus1)
                    .child("A GPUI workspace ready to build on."),
            )
    }
}
