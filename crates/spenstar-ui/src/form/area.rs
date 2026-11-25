use egui::{Frame, Margin, Response, ScrollArea, TextBuffer, TextEdit, Ui, vec2};

use super::{EntryStyle, entry::counter};
use crate::typography::Type;

/// A multiline text editor with bounded vertical growth.
#[derive(Clone, Copy, Debug)]
pub struct TextArea<'a> {
    /// Minimum number of visible lines.
    pub min_rows: usize,
    /// Maximum number of visible lines before scrolling.
    pub max_rows: usize,
    /// Expand with the text up to the maximum height.
    pub auto_grow: bool,
    /// Hint shown when empty.
    pub placeholder: &'a str,
    /// Character limit and counter.
    pub max_len: Option<usize>,
    /// Draw invalid styling.
    pub invalid: bool,
    /// Allow selection and copying only.
    pub read_only: bool,
    /// Disable interaction.
    pub disabled: bool,
}

impl Default for TextArea<'_> {
    fn default() -> Self {
        Self {
            min_rows: 3,
            max_rows: 8,
            auto_grow: true,
            placeholder: "",
            max_len: None,
            invalid: false,
            read_only: false,
            disabled: false,
        }
    }
}

impl TextArea<'_> {
    /// Render a vertically scrollable editor.
    pub fn show(self, ui: &mut Ui, value: &mut String) -> Response {
        ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            let style = EntryStyle {
                invalid: self.invalid,
                ..Default::default()
            };
            let font = Type::Body.font();
            let row_height = ui.fonts_mut(|fonts| fonts.row_height(&font))
                + ui.spacing().extra_text_line_spacing;
            let min_rows = self.min_rows.clamp(1, 1024);
            let max_rows = self.max_rows.clamp(min_rows, 1024);
            let mut borrowed = value.as_str();
            let text: &mut dyn TextBuffer = if self.read_only { &mut borrowed } else { value };
            let background = ui.painter().add(egui::Shape::Noop);
            let id = ui.unique_id().with("multiline");
            let shell = Frame::new()
                .inner_margin(Margin::symmetric(9, 7))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_height(
                            row_height * if self.auto_grow { max_rows } else { min_rows } as f32,
                        )
                        .min_scrolled_height(row_height * min_rows as f32)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(text)
                                    .id(id)
                                    .font(font)
                                    .desired_width(ui.available_width())
                                    .desired_rows(min_rows)
                                    .min_size(vec2(0., row_height * min_rows as f32))
                                    .margin(Margin::ZERO)
                                    .hint_text(self.placeholder.to_owned())
                                    .char_limit(self.max_len.unwrap_or(usize::MAX))
                                    .frame(egui::Frame::NONE),
                            )
                        })
                        .inner
                });
            style.paint(ui, background, shell.response.rect, &shell.inner);
            if let Some(limit) = self.max_len {
                counter(ui, value.chars().count(), limit);
            }
            shell.inner
        })
        .inner
    }
}
