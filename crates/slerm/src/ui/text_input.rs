use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Context, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, IntoElement, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, PaintQuad, Pixels, Render, ShapedLine,
    SharedString, Style, TextRun, UTF16Selection, Window, actions, div, fill, point, prelude::*,
    px, relative, size,
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
        TextInputSelectAll,
        TextInputCopy,
        TextInputCut,
        TextInputMoveLeftSelecting,
        TextInputMoveRightSelecting,
    ]
);

pub struct TextInput {
    focus_handle: FocusHandle,
    text: SharedString,
    placeholder: SharedString,
    cursor: usize,
    selection_anchor: usize,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    horizontal_scroll_offset: Pixels,
    on_change: Option<Box<ChangeHandler>>,
}

impl TextInput {
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            text: "".into(),
            placeholder: placeholder.into(),
            cursor: 0,
            selection_anchor: 0,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            horizontal_scroll_offset: px(0.0),
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
        self.selection_anchor = self.cursor;
        self.marked_range = None;
        cx.notify();
    }

    fn move_left(&mut self, _: &TextInputMoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor_left(false);
        cx.notify();
    }

    fn move_right(&mut self, _: &TextInputMoveRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor_right(false);
        cx.notify();
    }

    fn move_to_start(&mut self, _: &TextInputMoveToStart, _: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor_to(0, false);
        cx.notify();
    }

    fn move_to_end(&mut self, _: &TextInputMoveToEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor_to(self.text.len(), false);
        cx.notify();
    }

    fn backspace(&mut self, _: &TextInputBackspace, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(range) = self.selected_range() {
            self.replace_range(range, "", window, cx);
            return;
        }

        let previous = self.previous_boundary(self.cursor);
        if previous != self.cursor {
            self.replace_range(previous..self.cursor, "", window, cx);
        }
    }

    fn delete(&mut self, _: &TextInputDelete, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(range) = self.selected_range() {
            self.replace_range(range, "", window, cx);
            return;
        }

        let next = self.next_boundary(self.cursor);
        if next != self.cursor {
            self.replace_range(self.cursor..next, "", window, cx);
        }
    }

    fn paste(&mut self, _: &TextInputPaste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_range(
                self.replacement_range(),
                &text.replace('\n', " "),
                window,
                cx,
            );
        }
    }

    fn select_all(&mut self, _: &TextInputSelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.selection_anchor = 0;
        self.cursor = self.text.len();
        cx.notify();
    }

    fn copy(&mut self, _: &TextInputCopy, _: &mut Window, cx: &mut Context<Self>) {
        self.copy_selected_text(cx);
    }

    fn cut(&mut self, _: &TextInputCut, window: &mut Window, cx: &mut Context<Self>) {
        if self.copy_selected_text(cx) {
            self.replace_range(self.replacement_range(), "", window, cx);
        }
    }

    fn move_left_selecting(
        &mut self,
        _: &TextInputMoveLeftSelecting,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_cursor_left(true);
        cx.notify();
    }

    fn move_right_selecting(
        &mut self,
        _: &TextInputMoveRightSelecting,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_cursor_right(true);
        cx.notify();
    }

    fn replace_range(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_range_without_notify(range, new_text);
        if let Some(on_change) = self.on_change.take() {
            on_change(&self.text, window, cx);
            self.on_change = Some(on_change);
        }
        cx.notify();
    }

    fn replace_range_without_notify(&mut self, range: Range<usize>, new_text: &str) {
        debug_assert!(self.text.is_char_boundary(range.start));
        debug_assert!(self.text.is_char_boundary(range.end));
        self.text = Self::text_replacing_range(&self.text, range.clone(), new_text);
        self.cursor = range.start + new_text.len();
        self.selection_anchor = self.cursor;
        self.marked_range = None;
    }

    fn replacement_range(&self) -> Range<usize> {
        self.marked_range
            .clone()
            .or_else(|| self.selected_range())
            .unwrap_or(self.cursor..self.cursor)
    }

    fn selected_range(&self) -> Option<Range<usize>> {
        Self::selected_range_for(self.cursor, self.selection_anchor)
    }

    fn has_reversed_selection(&self) -> bool {
        self.cursor < self.selection_anchor
    }

    fn move_cursor_left(&mut self, selecting: bool) {
        if !selecting && let Some(range) = self.selected_range() {
            self.move_cursor_to(range.start, false);
            return;
        }
        self.move_cursor_to(self.previous_boundary(self.cursor), selecting);
    }

    fn move_cursor_right(&mut self, selecting: bool) {
        if !selecting && let Some(range) = self.selected_range() {
            self.move_cursor_to(range.end, false);
            return;
        }
        self.move_cursor_to(self.next_boundary(self.cursor), selecting);
    }

    fn move_cursor_to(&mut self, offset: usize, selecting: bool) {
        debug_assert!(self.text.is_char_boundary(offset));
        self.cursor = offset;
        if !selecting {
            self.selection_anchor = self.cursor;
        }
        self.marked_range = None;
    }

    fn copy_selected_text(&self, cx: &mut Context<Self>) -> bool {
        let Some(range) = self.selected_range() else {
            return false;
        };
        cx.write_to_clipboard(ClipboardItem::new_string(self.text[range].to_string()));
        true
    }

    fn mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle);
        let offset = self.byte_offset_for_point(event.position);
        if event.click_count >= 2 {
            let range = Self::word_range_at_offset(&self.text, offset);
            self.selection_anchor = range.start;
            self.cursor = range.end;
        } else {
            self.selection_anchor = offset;
            self.cursor = offset;
        }
        self.marked_range = None;
        cx.stop_propagation();
        cx.notify();
    }

    fn mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if event.pressed_button != Some(MouseButton::Left) {
            return;
        }
        self.cursor = self.byte_offset_for_point(event.position);
        self.marked_range = None;
        cx.stop_propagation();
        cx.notify();
    }

    fn byte_offset_for_point(&self, point: gpui::Point<Pixels>) -> usize {
        let Some(bounds) = self.last_bounds else {
            return self.cursor;
        };
        let Some(line) = self.last_layout.as_ref() else {
            return self.cursor;
        };
        if self.text.is_empty() {
            return 0;
        }

        let x = point.x - bounds.left() + self.horizontal_scroll_offset;
        if x <= px(0.0) {
            return 0;
        }
        line.index_for_x(x).unwrap_or(self.text.len())
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
        Self::offset_from_utf16_in_text(&self.text, offset)
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        Self::offset_to_utf16_in_text(&self.text, offset)
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn offset_from_utf16_in_text(text: &str, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16_in_text(text: &str, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in text.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn selected_range_for(cursor: usize, selection_anchor: usize) -> Option<Range<usize>> {
        (cursor != selection_anchor)
            .then(|| cursor.min(selection_anchor)..cursor.max(selection_anchor))
    }

    fn text_replacing_range(text: &str, range: Range<usize>, new_text: &str) -> SharedString {
        (text[0..range.start].to_owned() + new_text + &text[range.end..]).into()
    }

    fn scroll_offset_for_caret(
        current_offset: Pixels,
        caret_x: Pixels,
        visible_width: Pixels,
        line_width: Pixels,
    ) -> Pixels {
        let padding = px(8.0);
        let mut offset = current_offset;
        if caret_x < offset + padding {
            offset = (caret_x - padding).max(px(0.0));
        } else if caret_x > offset + visible_width - padding {
            offset = caret_x - visible_width + padding;
        }

        let max_offset = (line_width - visible_width).max(px(0.0));
        offset.min(max_offset)
    }

    fn word_range_at_offset(text: &str, offset: usize) -> Range<usize> {
        if text.is_empty() {
            return 0..0;
        }

        let offset = if offset == text.len() {
            text[..offset]
                .char_indices()
                .last()
                .map(|(idx, _)| idx)
                .unwrap_or(0)
        } else {
            offset
        };
        let Some(ch) = text[offset..].chars().next() else {
            return text.len()..text.len();
        };
        let is_word = Self::is_word_char(ch);
        let mut start = offset;
        while let Some((idx, ch)) = text[..start].char_indices().last() {
            if Self::is_word_char(ch) != is_word {
                break;
            }
            start = idx;
        }

        let mut end = offset + ch.len_utf8();
        while end < text.len() {
            let Some(ch) = text[end..].chars().next() else {
                break;
            };
            if Self::is_word_char(ch) != is_word {
                break;
            }
            end += ch.len_utf8();
        }
        start..end
    }

    fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
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
        let range = self.selected_range().unwrap_or(self.cursor..self.cursor);
        Some(UTF16Selection {
            range: self.range_to_utf16(&range),
            reversed: self.has_reversed_selection(),
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
            .unwrap_or_else(|| self.replacement_range());
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
            .unwrap_or_else(|| self.replacement_range());
        self.replace_range(range.clone(), new_text, window, cx);
        self.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());
        if let Some(selected_range) = new_selected_range_utf16 {
            self.selection_anchor =
                range.start + Self::offset_from_utf16_in_text(new_text, selected_range.start);
            self.cursor =
                range.start + Self::offset_from_utf16_in_text(new_text, selected_range.end);
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
            point(
                bounds.left() + line.x_for_index(range.start) - self.horizontal_scroll_offset,
                bounds.top(),
            ),
            point(
                bounds.left() + line.x_for_index(range.end) - self.horizontal_scroll_offset,
                bounds.bottom(),
            ),
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
        Some(self.offset_to_utf16(
            line.index_for_x(point.x - bounds.left() + self.horizontal_scroll_offset)?,
        ))
    }
}

struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection_quads: Vec<PaintQuad>,
    horizontal_scroll_offset: Pixels,
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
            px(0.0)
        } else {
            line.x_for_index(input.cursor)
        };
        let horizontal_scroll_offset = TextInput::scroll_offset_for_caret(
            input.horizontal_scroll_offset,
            cursor_x,
            bounds.size.width,
            line.width,
        );
        let mut selection_quads = Vec::new();
        if !input.text.is_empty()
            && let Some(range) = input.selected_range()
        {
            let start_x = bounds.left() + line.x_for_index(range.start) - horizontal_scroll_offset;
            let end_x = bounds.left() + line.x_for_index(range.end) - horizontal_scroll_offset;
            let left = start_x.max(bounds.left());
            let right = end_x.min(bounds.right());
            if right > left {
                selection_quads.push(fill(
                    Bounds::from_corners(point(left, bounds.top()), point(right, bounds.bottom())),
                    theme.select_bg,
                ));
            }
        }
        PrepaintState {
            line: Some(line),
            cursor: Some(fill(
                Bounds::new(
                    point(
                        bounds.left() + cursor_x - horizontal_scroll_offset,
                        bounds.top(),
                    ),
                    size(px(1.5), bounds.bottom() - bounds.top()),
                ),
                theme.plus2,
            )),
            selection_quads,
            horizontal_scroll_offset,
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
        for selection_quad in prepaint.selection_quads.drain(..) {
            window.paint_quad(selection_quad);
        }
        let line_origin = point(
            bounds.left() - prepaint.horizontal_scroll_offset,
            bounds.top(),
        );
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            line.paint(line_origin, window.line_height(), window, cx)
                .unwrap();
            if focus_handle.is_focused(window)
                && let Some(cursor) = prepaint.cursor.take()
            {
                window.paint_quad(cursor);
            }
        });
        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            input.horizontal_scroll_offset = prepaint.horizontal_scroll_offset;
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
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::move_left_selecting))
            .on_action(cx.listener(Self::move_right_selecting))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_move(cx.listener(Self::mouse_move))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_selection_preserves_utf8_boundaries() {
        let text = "aé😄b";
        let range = TextInput::selected_range_for("aé😄".len(), 1).unwrap();

        let replaced = TextInput::text_replacing_range(text, range.clone(), "ø");

        assert_eq!(range, 1.."aé😄".len());
        assert_eq!(replaced.as_ref(), "aøb");
        assert_eq!(range.start + "ø".len(), "aø".len());
    }

    #[test]
    fn utf16_offsets_handle_multibyte_ranges() {
        let text = "a😄b";
        let range = TextInput::selected_range_for("a".len(), "a😄".len()).unwrap();
        let utf16_range = TextInput::offset_to_utf16_in_text(text, range.start)
            ..TextInput::offset_to_utf16_in_text(text, range.end);

        assert_eq!(range, "a".len().."a😄".len());
        assert_eq!(utf16_range, 1..3);
        assert_eq!(TextInput::offset_from_utf16_in_text(text, 3), "a😄".len());
    }

    #[test]
    fn selected_range_tracks_reversed_selections() {
        assert_eq!(TextInput::selected_range_for(5, 1), Some(1..5));
        assert_eq!(TextInput::selected_range_for(1, 5), Some(1..5));
        assert_eq!(TextInput::selected_range_for(3, 3), None);
    }

    #[test]
    fn scroll_offset_keeps_caret_visible() {
        assert_eq!(
            TextInput::scroll_offset_for_caret(px(0.0), px(20.0), px(100.0), px(200.0)),
            px(0.0)
        );
        assert_eq!(
            TextInput::scroll_offset_for_caret(px(0.0), px(140.0), px(100.0), px(200.0)),
            px(48.0)
        );
        assert_eq!(
            TextInput::scroll_offset_for_caret(px(80.0), px(40.0), px(100.0), px(200.0)),
            px(32.0)
        );
        assert_eq!(
            TextInput::scroll_offset_for_caret(px(90.0), px(190.0), px(100.0), px(200.0)),
            px(98.0)
        );
    }

    #[test]
    fn word_range_selects_multibyte_words() {
        let text = "alpha βeta_2!";

        assert_eq!(TextInput::word_range_at_offset(text, 2), 0.."alpha".len());
        assert_eq!(
            TextInput::word_range_at_offset(text, "alpha ".len()),
            "alpha ".len().."alpha βeta_2".len()
        );
        assert_eq!(
            TextInput::word_range_at_offset(text, text.len()),
            "alpha βeta_2".len()..text.len()
        );
    }
}
