use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement, Render, SharedString, Window,
    actions, div, prelude::*, px,
};

use crate::{theme, ui::text_input::TextInput};

actions!(
    slerm_fuzzy_finder,
    [
        FuzzyFinderSelectPrev,
        FuzzyFinderSelectNext,
        FuzzyFinderConfirm,
        FuzzyFinderCancel,
    ]
);

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
                fuzzy_score(query, &item.title).map(|score| FuzzyMatch { item_index, score })
            })
            .collect::<Vec<_>>();

        matches.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.item_index.cmp(&b.item_index))
        });
        self.filtered = matches;
        if self.filtered.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = self.selected_index.min(self.filtered.len() - 1);
        }
    }

    fn select_prev(&mut self, _: &FuzzyFinderSelectPrev, _: &mut Window, cx: &mut Context<Self>) {
        if !self.filtered.is_empty() {
            self.selected_index = self
                .selected_index
                .checked_sub(1)
                .unwrap_or(self.filtered.len() - 1);
            cx.notify();
        }
    }

    fn select_next(&mut self, _: &FuzzyFinderSelectNext, _: &mut Window, cx: &mut Context<Self>) {
        if !self.filtered.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered.len();
            cx.notify();
        }
    }

    fn confirm(&mut self, _: &FuzzyFinderConfirm, window: &mut Window, cx: &mut Context<Self>) {
        let Some(selected) = self.filtered.get(self.selected_index) else {
            return;
        };
        let payload = self.items[selected.item_index].payload.clone();
        let on_confirm = std::mem::replace(&mut self.on_confirm, Box::new(|_, _, _| {}));
        on_confirm(payload, window, cx);
        self.on_confirm = on_confirm;
    }

    fn cancel(&mut self, _: &FuzzyFinderCancel, window: &mut Window, cx: &mut Context<Self>) {
        let on_cancel = std::mem::replace(&mut self.on_cancel, Box::new(|_, _| {}));
        on_cancel(window, cx);
        self.on_cancel = on_cancel;
    }

    fn focus_input(&self, window: &mut Window, cx: &mut App) {
        self.input.read(cx).focus_handle(cx).focus(window);
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
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::select_next))
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
                    .border_t_1()
                    .border_color(theme.border)
                    .p_1()
                    .max_h(px(260.0))
                    .overflow_hidden()
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
                                    .px_2()
                                    .py_1()
                                    .rounded_xs()
                                    .bg(if index == self.selected_index {
                                        theme.select_bg
                                    } else {
                                        theme.float_bg
                                    })
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
    use super::fuzzy_score;

    #[test]
    fn fuzzy_score_matches_subsequences() {
        assert!(fuzzy_score("term", "Terminal").is_some());
        assert!(fuzzy_score("tml", "Terminal").is_some());
        assert!(fuzzy_score("xyz", "Terminal").is_none());
    }
}
