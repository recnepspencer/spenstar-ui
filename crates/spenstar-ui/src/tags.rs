//! Wrapping tags and inline tag entry.
use egui::{Frame, Key, Margin, Modifiers, Response, Stroke, TextEdit, Ui};

use crate::{
    Theme,
    controls::Tone,
    form::icon_button,
    icon::{Icon, IconSource},
    theme::luminance,
    typography::Type,
};

/// Display information for a tag.
#[derive(Clone, Debug)]
pub struct Tag<'a> {
    /// Visible text.
    pub label: &'a str,
    /// Semantic color.
    pub tone: Tone,
    /// Optional leading icon.
    pub icon: Option<IconSource>,
    /// Display an individual remove action.
    pub removable: bool,
}

impl<'a> Tag<'a> {
    /// Make a neutral tag.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            tone: Tone::Neutral,
            icon: None,
            removable: false,
        }
    }
}

/// A wrapping collection of tags.
#[derive(Clone, Copy, Debug, Default)]
pub struct TagGroup {
    /// Disable remove actions.
    pub disabled: bool,
}

impl TagGroup {
    /// Display tags and return the indices whose remove actions were activated.
    pub fn show(self, ui: &mut Ui, tags: &[Tag<'_>]) -> Vec<usize> {
        ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            ui.horizontal_wrapped(|ui| {
                tags.iter()
                    .enumerate()
                    .filter_map(|(i, tag)| chip(ui, tag, ui.available_width()).then_some(i))
                    .collect()
            })
            .inner
        })
        .inner
    }
}

pub(crate) fn chip(ui: &mut Ui, tag: &Tag<'_>, width: f32) -> bool {
    let colors = Theme::current(ui.ctx()).colors;
    let tone = tag.tone.palette(colors);
    let (ink_luminance, fill_luminance) = (luminance(tone.ring), luminance(tone.soft_hover));
    let contrast =
        (ink_luminance.max(fill_luminance) + 0.05) / (ink_luminance.min(fill_luminance) + 0.05);
    let ink = if tag.tone == Tone::Neutral || contrast < 4.5 {
        colors.text_primary
    } else {
        tone.ring
    };
    Frame::new()
        .fill(tone.soft_hover)
        .corner_radius(12)
        .inner_margin(Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.;
                if let Some(icon) = &tag.icon {
                    icon.show(ui, 12., ink);
                }
                let decorations = 12.
                    + if tag.removable { 25. } else { 0. }
                    + if tag.icon.is_some() { 15. } else { 0. };
                let text_width = ui
                    .painter()
                    .layout_no_wrap(tag.label.to_owned(), Type::Badge.font(), ink)
                    .size()
                    .x;
                let label = egui::Label::new(Type::Badge.text(tag.label).color(ink)).truncate();
                ui.add_sized([text_width.min((width - decorations).max(10.)), 20.], label);
                tag.removable
                    && icon_button(ui, Icon::X, &format!("Remove {}", tag.label)).clicked()
            })
            .inner
        })
        .inner
}

/// An inline editor with wrapping chips, delimiter handling, and an optional tag cap.
#[derive(Clone, Copy, Debug)]
pub struct TagInput<'a> {
    /// Hint for the text editor.
    pub placeholder: &'a str,
    /// Reject case-insensitive duplicates.
    pub prevent_duplicates: bool,
    /// Maximum tag count.
    pub max_tags: usize,
    /// Disable editing and removal.
    pub disabled: bool,
}

impl Default for TagInput<'_> {
    fn default() -> Self {
        Self {
            placeholder: "Add a tag…",
            prevent_duplicates: true,
            max_tags: usize::MAX,
            disabled: false,
        }
    }
}

/// The text editor and collection change state.
pub struct TagResponse {
    /// Inline editor response.
    pub response: Response,
    /// Tags were added or removed.
    pub changed: bool,
}

impl TagInput<'_> {
    /// Display tags and update the supplied collection and draft.
    pub fn show(self, ui: &mut Ui, tags: &mut Vec<String>, draft: &mut String) -> TagResponse {
        ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            let colors = Theme::current(ui.ctx()).colors;
            let id = ui.unique_id().with("tag-edit");
            let focused = ui.memory(|m| m.has_focus(id));
            let border = if focused {
                colors.accent.ring
            } else {
                colors.border
            };
            Frame::new()
                .fill(colors.surface)
                .stroke(Stroke::new(1., border))
                .corner_radius(8)
                .inner_margin(Margin::same(6))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| self.editor(ui, tags, draft, id, focused))
                        .inner
                })
                .inner
        })
        .inner
    }

    fn editor(
        self,
        ui: &mut Ui,
        tags: &mut Vec<String>,
        draft: &mut String,
        id: egui::Id,
        focused: bool,
    ) -> TagResponse {
        let mut changed = false;
        let backspace = ui.is_enabled()
            && focused
            && draft.is_empty()
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Backspace));
        if backspace {
            changed |= tags.pop().is_some();
        }
        let width = ui.max_rect().width();
        let mut index = 0;
        while index < tags.len() {
            let tag = Tag {
                removable: true,
                ..Tag::new(&tags[index])
            };
            if chip(ui, &tag, width) {
                tags.remove(index);
                changed = true;
                ui.memory_mut(|memory| memory.request_focus(id));
            } else {
                index += 1;
            }
        }
        let enter = ui.is_enabled()
            && focused
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
        let response = ui.add(
            TextEdit::singleline(draft)
                .id(id)
                .return_key(None)
                .hint_text(self.placeholder.to_owned())
                .desired_width(112.)
                .frame(Frame::NONE),
        );
        if response.enabled() && (enter || draft.contains(',')) {
            let input = std::mem::take(draft);
            let mut parts = input.split(',').peekable();
            while let Some(part) = parts.next() {
                if !enter && parts.peek().is_none() {
                    *draft = part.to_owned();
                } else {
                    let tag = part.trim();
                    let duplicate = self.prevent_duplicates
                        && tags
                            .iter()
                            .any(|value| value.to_lowercase() == tag.to_lowercase());
                    if !tag.is_empty() && !duplicate && tags.len() < self.max_tags {
                        tags.push(tag.to_owned());
                        changed = true;
                    }
                }
            }
        }
        TagResponse { response, changed }
    }
}
