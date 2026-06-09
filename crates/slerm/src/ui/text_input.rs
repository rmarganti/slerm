use std::ops::Range;

use gpui::{
    App, Bounds, Context, Element, ElementId, ElementInputHandler, Entity, EntityInputHandler,
    FocusHandle, Focusable, GlobalElementId, IntoElement, LayoutId, PaintQuad, Pixels, Render,
    ShapedLine, SharedString, Style, TextRun, UTF16Selection, Window, actions, div, fill, point,
    prelude::*, px, relative, size,
};

use crate::theme;

type ChangeHandler = dyn Fn(&str, &mut Window, &mut Context<TextInput>) + 'static;

actions!(
    slerm_text_input,
    [
        TextInputMoveLeft,
        TextInputMoveRight,
        TextInputMoveToStart,
        TextInputMoveToEnd,
        TextInputBackspace,
        TextInputDelete,
        TextInputPaste,
    ]
);

pub struct TextInput {
    focus_handle: FocusHandle,
    text: SharedString,
    placeholder: SharedString,
    cursor: usize,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    on_change: Option<Box<ChangeHandler>>,
}

impl TextInput {
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            text: "".into(),
            placeholder: placeholder.into(),
            cursor: 0,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            on_change: None,
        }
    }

    pub fn set_on_change(
        &mut self,
        on_change: impl Fn(&str, &mut Window, &mut Context<Self>) + 'static,
    ) {
        self.on_change = Some(Box::new(on_change));
    }

    #[allow(dead_code)]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[allow(dead_code)]
    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.text = text.into();
        self.cursor = self.text.len();
        cx.notify();
    }

    fn move_left(&mut self, _: &TextInputMoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.cursor = self.previous_boundary(self.cursor);
        cx.notify();
    }

    fn move_right(&mut self, _: &TextInputMoveRight, _: &mut Window, cx: &mut Context<Self>) {
        self.cursor = self.next_boundary(self.cursor);
        cx.notify();
    }

    fn move_to_start(&mut self, _: &TextInputMoveToStart, _: &mut Window, cx: &mut Context<Self>) {
        self.cursor = 0;
        cx.notify();
    }

    fn move_to_end(&mut self, _: &TextInputMoveToEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.cursor = self.text.len();
        cx.notify();
    }

    fn backspace(&mut self, _: &TextInputBackspace, window: &mut Window, cx: &mut Context<Self>) {
        let previous = self.previous_boundary(self.cursor);
        if previous != self.cursor {
            self.replace_range(previous..self.cursor, "", window, cx);
        }
    }

    fn delete(&mut self, _: &TextInputDelete, window: &mut Window, cx: &mut Context<Self>) {
        let next = self.next_boundary(self.cursor);
        if next != self.cursor {
            self.replace_range(self.cursor..next, "", window, cx);
        }
    }

    fn paste(&mut self, _: &TextInputPaste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_range(
                self.cursor..self.cursor,
                &text.replace('\n', " "),
                window,
                cx,
            );
        }
    }

    fn replace_range(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.text =
            (self.text[0..range.start].to_owned() + new_text + &self.text[range.end..]).into();
        self.cursor = range.start + new_text.len();
        self.marked_range = None;
        if let Some(on_change) = self.on_change.take() {
            on_change(&self.text, window, cx);
            self.on_change = Some(on_change);
        }
        cx.notify();
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.text[..offset]
            .char_indices()
            .last()
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.text[offset..]
            .char_indices()
            .nth(1)
            .map(|(idx, _)| offset + idx)
            .unwrap_or(self.text.len())
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.text.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.text[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&(self.cursor..self.cursor)),
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.cursor..self.cursor);
        self.replace_range(range, new_text, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.cursor..self.cursor);
        self.replace_range(range.clone(), new_text, window, cx);
        self.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());
        if let Some(selected_range) = new_selected_range_utf16 {
            let selected_range = self.range_from_utf16(&selected_range);
            self.cursor = range.start + selected_range.end;
        }
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(bounds.left() + line.x_for_index(range.start), bounds.top()),
            point(bounds.left() + line.x_for_index(range.end), bounds.bottom()),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line = self.last_layout.as_ref()?;
        Some(self.offset_to_utf16(line.index_for_x(point.x - bounds.left())?))
    }
}

struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(_cx);
        let theme = theme::active();
        let (display_text, text_color) = if input.text.is_empty() {
            (input.placeholder.clone(), theme.minus2.into())
        } else {
            (input.text.clone(), window.text_style().color)
        };
        let run = TextRun {
            len: display_text.len(),
            font: window.text_style().font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let font_size = window.text_style().font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &[run], None);
        let cursor_x = if input.text.is_empty() {
            0.0.into()
        } else {
            line.x_for_index(input.cursor)
        };
        PrepaintState {
            line: Some(line),
            cursor: Some(fill(
                Bounds::new(
                    point(bounds.left() + cursor_x, bounds.top()),
                    size(px(1.5), bounds.bottom() - bounds.top()),
                ),
                theme.plus2,
            )),
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        let line = prepaint.line.take().unwrap();
        line.paint(bounds.origin, window.line_height(), window, cx)
            .unwrap();
        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }
        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::active();
        div()
            .key_context("TextInput")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::move_left))
            .on_action(cx.listener(Self::move_right))
            .on_action(cx.listener(Self::move_to_start))
            .on_action(cx.listener(Self::move_to_end))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::paste))
            .w_full()
            .h(px(40.0))
            .px_3()
            .flex()
            .items_center()
            .bg(theme.float_bg)
            .child(TextElement { input: cx.entity() })
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
