use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement, Window, div, prelude::*, px,
};

use crate::{
    theme,
    ui::{menu, text_input::TextInput},
};

pub struct FuzzyFinderItem<T> {
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    pub payload: T,
}

impl<T> FuzzyFinderItem<T> {
    pub fn new(
        title: impl Into<SharedString>,
        subtitle: Option<impl Into<SharedString>>,
        payload: T,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.map(Into::into),
            payload,
        }
    }
}

#[derive(Clone, Debug)]
struct FuzzyMatch {
    item_index: usize,
    score: i64,
}

type ConfirmHandler<T> = dyn Fn(T, &mut Window, &mut Context<FuzzyFinder<T>>) + 'static;
type CancelHandler<T> = dyn Fn(&mut Window, &mut Context<FuzzyFinder<T>>) + 'static;

pub struct FuzzyFinder<T: Clone + 'static> {
    items: Vec<FuzzyFinderItem<T>>,
    filtered: Vec<FuzzyMatch>,
    selected_index: usize,
    scroll_handle: ScrollHandle,
    input: Entity<TextInput>,
    focus_handle: FocusHandle,
    on_confirm: Box<ConfirmHandler<T>>,
    on_cancel: Box<CancelHandler<T>>,
}

impl<T: Clone + 'static> FuzzyFinder<T> {
    pub fn new(
        title: impl Into<SharedString>,
        items: Vec<FuzzyFinderItem<T>>,
        on_confirm: impl Fn(T, &mut Window, &mut Context<Self>) + 'static,
        on_cancel: impl Fn(&mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        let title = title.into();
        let input = cx.new(|cx| TextInput::new(title.clone(), cx));
        input.update(cx, |input, _| {
            input.set_on_change(|query, window, cx| {
                // The owner wires this entity-specific callback in render via update below.
                let _ = (query, window, cx);
            });
        });

        let mut finder = Self {
            items,
            filtered: Vec::new(),
            selected_index: 0,
            scroll_handle: ScrollHandle::new(),
            input,
            focus_handle: cx.focus_handle(),
            on_confirm: Box::new(on_confirm),
            on_cancel: Box::new(on_cancel),
        };
        finder.refresh_matches("");
        finder
    }

    fn refresh_matches(&mut self, query: &str) {
        let mut matches = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(item_index, item)| {
                fuzzy_item_score(query, item).map(|score| FuzzyMatch { item_index, score })
            })
            .collect::<Vec<_>>();

        matches.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.item_index.cmp(&b.item_index))
        });
        self.filtered = matches;
        self.selected_index = 0;
        self.scroll_selected_item_into_view();
    }

    fn select_previous(
        &mut self,
        _: &menu::SelectPrevious,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.select_previous_match() {
            cx.notify();
        }
    }

    fn select_next(&mut self, _: &menu::SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        if self.select_next_match() {
            cx.notify();
        }
    }

    fn select_first(&mut self, _: &menu::SelectFirst, _: &mut Window, cx: &mut Context<Self>) {
        if self.select_first_match() {
            cx.notify();
        }
    }

    fn select_last(&mut self, _: &menu::SelectLast, _: &mut Window, cx: &mut Context<Self>) {
        if self.select_last_match() {
            cx.notify();
        }
    }

    fn select_page_up(&mut self, _: &menu::SelectPageUp, _: &mut Window, cx: &mut Context<Self>) {
        if self.select_first_match() {
            cx.notify();
        }
    }

    fn select_page_down(
        &mut self,
        _: &menu::SelectPageDown,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.select_last_match() {
            cx.notify();
        }
    }

    fn confirm(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        self.confirm_selected(window, cx);
    }

    fn cancel(&mut self, _: &menu::Cancel, window: &mut Window, cx: &mut Context<Self>) {
        let on_cancel = std::mem::replace(&mut self.on_cancel, Box::new(|_, _| {}));
        on_cancel(window, cx);
        self.on_cancel = on_cancel;
    }

    fn select_previous_match(&mut self) -> bool {
        let Some(selected_index) =
            previous_selection_index(self.selected_index, self.filtered.len())
        else {
            return false;
        };
        self.selected_index = selected_index;
        self.scroll_selected_item_into_view();
        true
    }

    fn select_next_match(&mut self) -> bool {
        let Some(selected_index) = next_selection_index(self.selected_index, self.filtered.len())
        else {
            return false;
        };
        self.selected_index = selected_index;
        self.scroll_selected_item_into_view();
        true
    }

    fn select_first_match(&mut self) -> bool {
        self.select_match_at(0)
    }

    fn select_last_match(&mut self) -> bool {
        let Some(last_index) = self.filtered.len().checked_sub(1) else {
            return false;
        };
        self.select_match_at(last_index)
    }

    fn select_match_at(&mut self, index: usize) -> bool {
        if index >= self.filtered.len() || self.selected_index == index {
            return false;
        }
        self.selected_index = index;
        self.scroll_selected_item_into_view();
        true
    }

    fn confirm_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(selected) = self.filtered.get(self.selected_index) else {
            return;
        };
        let payload = self.items[selected.item_index].payload.clone();
        let on_confirm = std::mem::replace(&mut self.on_confirm, Box::new(|_, _, _| {}));
        on_confirm(payload, window, cx);
        self.on_confirm = on_confirm;
    }

    fn focus_input(&self, window: &mut Window, cx: &mut App) {
        self.input.read(cx).focus_handle(cx).focus(window);
    }

    fn scroll_selected_item_into_view(&self) {
        if !self.filtered.is_empty() {
            self.scroll_handle.scroll_to_item(self.selected_index);
        }
    }
}

impl<T: Clone + 'static> Focusable for FuzzyFinder<T> {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T: Clone + 'static> Render for FuzzyFinder<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::active();
        self.focus_input(window, cx);

        let entity = cx.entity();
        let input = self.input.clone();
        input.update(cx, move |input, _| {
            let finder = entity.clone();
            input.set_on_change(move |query, _window, cx| {
                finder.update(cx, |finder, cx| {
                    finder.refresh_matches(query);
                    cx.notify();
                });
            });
        });

        div()
            .key_context("FuzzyFinder")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::select_page_up))
            .on_action(cx.listener(Self::select_page_down))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .w(px(560.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(theme.border)
            .bg(theme.float_bg)
            .shadow_lg()
            .child(self.input.clone())
            .child(
                div()
                    .id("fuzzy-finder-results")
                    .border_t_1()
                    .border_color(theme.border)
                    .p_1()
                    .max_h(px(260.0))
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .children(if self.filtered.is_empty() {
                        vec![
                            div()
                                .px_2()
                                .py_2()
                                .text_color(theme.minus1)
                                .child("No matches")
                                .into_any_element(),
                        ]
                    } else {
                        self.filtered
                            .iter()
                            .enumerate()
                            .map(|(index, matched)| {
                                let item = &self.items[matched.item_index];
                                div()
                                    .id(("fuzzy-finder-row", index))
                                    .px_2()
                                    .py_1()
                                    .rounded_xs()
                                    .bg(if index == self.selected_index {
                                        theme.select_bg
                                    } else {
                                        theme.float_bg
                                    })
                                    .on_hover(cx.listener(move |finder, hovered: &bool, _, cx| {
                                        if *hovered && finder.select_match_at(index) {
                                            cx.notify();
                                        }
                                    }))
                                    .on_click(cx.listener(move |finder, _event, window, cx| {
                                        finder.select_match_at(index);
                                        finder.confirm_selected(window, cx);
                                    }))
                                    .child(div().text_color(theme.fg).child(item.title.clone()))
                                    .when_some(item.subtitle.clone(), |row, subtitle| {
                                        row.child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.minus1)
                                                .child(subtitle),
                                        )
                                    })
                                    .into_any_element()
                            })
                            .collect::<Vec<_>>()
                    }),
            )
    }
}

fn previous_selection_index(selected_index: usize, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(selected_index.checked_sub(1).unwrap_or(len - 1))
}

fn next_selection_index(selected_index: usize, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some((selected_index + 1) % len)
}

fn fuzzy_item_score<T>(query: &str, item: &FuzzyFinderItem<T>) -> Option<i64> {
    let title_score = fuzzy_score(query, &item.title);
    let subtitle_score = item
        .subtitle
        .as_ref()
        .and_then(|subtitle| fuzzy_score(query, subtitle))
        // Prefer title matches over equally good subtitle/path matches.
        .map(|score| score - 10);

    title_score.into_iter().chain(subtitle_score).max()
}

fn fuzzy_score(query: &str, candidate: &str) -> Option<i64> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Some(0);
    }

    let candidate_lower = candidate.to_lowercase();
    let mut score = 0i64;
    let mut last_match: Option<usize> = None;
    let mut search_start = 0usize;

    for query_char in query.chars() {
        let found = candidate_lower[search_start..]
            .char_indices()
            .find(|(_, candidate_char)| *candidate_char == query_char)
            .map(|(offset, ch)| (search_start + offset, ch));
        let (index, ch) = found?;

        score += 100;
        if index == 0 || candidate_lower[..index].ends_with([' ', '-', '_', '/']) {
            score += 25;
        }
        if let Some(last) = last_match {
            if index == last + ch.len_utf8() {
                score += 15;
            } else {
                score -= (index - last) as i64;
            }
        } else {
            score -= index as i64;
        }

        last_match = Some(index);
        search_start = index + ch.len_utf8();
    }

    Some(score - candidate_lower.len() as i64)
}

#[cfg(test)]
mod tests {
    use super::{
        FuzzyFinderItem, fuzzy_item_score, fuzzy_score, next_selection_index,
        previous_selection_index,
    };

    #[test]
    fn fuzzy_score_matches_subsequences() {
        assert!(fuzzy_score("term", "Terminal").is_some());
        assert!(fuzzy_score("tml", "Terminal").is_some());
        assert!(fuzzy_score("xyz", "Terminal").is_none());
    }

    #[test]
    fn fuzzy_item_score_matches_subtitles() {
        let item = FuzzyFinderItem::new("slerm", Some("/Users/rmarganti/code/rmarganti/slerm"), ());

        assert!(fuzzy_item_score("code", &item).is_some());
        assert!(fuzzy_item_score("rm/sl", &item).is_some());
        assert!(fuzzy_item_score("zed", &item).is_none());
    }

    #[test]
    fn selection_navigation_wraps_and_ignores_empty_results() {
        assert_eq!(previous_selection_index(0, 0), None);
        assert_eq!(next_selection_index(0, 0), None);
        assert_eq!(previous_selection_index(0, 3), Some(2));
        assert_eq!(previous_selection_index(2, 3), Some(1));
        assert_eq!(next_selection_index(2, 3), Some(0));
        assert_eq!(next_selection_index(0, 3), Some(1));
    }
}
