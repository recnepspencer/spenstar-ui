use egui::{Color32, Frame, Label, Margin, Response, ScrollArea, Stroke, Ui};

use super::Badge;
use crate::{
    Theme,
    button::{Button, Size, Variant},
    icon::Icon,
    layout::Separator,
    theme::{Mode, mix},
    typography::Type,
};

/// A labeled value and its display options.
pub struct DescriptionItem<'a> {
    /// Key for the value.
    pub label: &'a str,
    /// Value to display and copy.
    pub value: &'a str,
    /// Use the bundled monospace typeface.
    pub mono: bool,
    /// Include a copy action.
    pub copy: bool,
}

/// Structured key-value rows.
pub struct DescriptionList<'a> {
    /// Rows in display order.
    pub items: &'a [DescriptionItem<'a>],
}

impl DescriptionList<'_> {
    /// Render rows and per-value copy controls.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let c = Theme::current(ui.ctx()).colors;
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.;
            for (index, item) in self.items.iter().enumerate() {
                ui.push_id(index, |ui| {
                    if index > 0 {
                        Separator::Horizontal.show(ui);
                    }
                    ui.horizontal(|ui| {
                        let width = (ui.available_width() * 0.32).min(170.);
                        ui.allocate_ui_with_layout(
                            egui::vec2(width, 24.),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.set_min_width(width);
                                ui.add(
                                    Label::new(
                                        Type::Description.text(item.label).color(c.text_secondary),
                                    )
                                    .truncate(),
                                );
                            },
                        );
                        let style = if item.mono {
                            Type::Mono
                        } else {
                            Type::Description
                        };
                        if item.copy {
                            let value_width = (ui.available_width() - 83.).max(0.);
                            ui.allocate_ui_with_layout(
                                egui::vec2(value_width, 24.),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.set_min_width(value_width);
                                    ui.add(
                                        Label::new(style.text(item.value).color(c.text_primary))
                                            .truncate(),
                                    );
                                },
                            );
                            copy(ui, item.value);
                        } else {
                            ui.label(style.text(item.value).color(c.text_primary));
                        }
                    });
                });
            }
        })
        .response
    }
}

/// A readable source listing with optional language, copy, and line numbers.
pub struct CodeBlock<'a> {
    /// Source text.
    pub text: &'a str,
    /// Header language label.
    pub language: Option<&'a str>,
    /// Include the copy action.
    pub copy: bool,
    /// Include line numbers.
    pub line_numbers: bool,
}

impl<'a> CodeBlock<'a> {
    /// Create a numbered source listing.
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            language: None,
            copy: true,
            line_numbers: true,
        }
    }

    /// Render the listing.
    pub fn show(&self, ui: &mut Ui) -> Response {
        source(
            ui,
            self.language,
            self.copy.then_some(self.text),
            self.line_numbers,
            self.text.split('\n').map(|line| DiffLine {
                kind: DiffKind::Context,
                text: line,
            }),
            false,
        )
    }
}

/// Line treatment in a diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffKind {
    /// Unmodified context.
    Context,
    /// Newly added content.
    Added,
    /// Removed content.
    Removed,
}

/// A source line with a diff treatment.
#[derive(Clone, Copy, Debug)]
pub struct DiffLine<'a> {
    /// Line treatment.
    pub kind: DiffKind,
    /// Line contents without the diff prefix.
    pub text: &'a str,
}

/// A unified diff listing.
pub struct DiffView<'a> {
    /// Source lines.
    pub lines: &'a [DiffLine<'a>],
    /// Header language label.
    pub language: Option<&'a str>,
    /// Include the copy action.
    pub copy: bool,
    /// Include line numbers.
    pub line_numbers: bool,
}

impl DiffView<'_> {
    /// Render the diff.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let text = self.copy.then(|| {
            self.lines
                .iter()
                .map(|line| format!("{}{}", prefix(line.kind), line.text))
                .collect::<Vec<_>>()
                .join("\n")
        });
        source(
            ui,
            self.language,
            text.as_deref(),
            self.line_numbers,
            self.lines.iter().copied(),
            true,
        )
    }
}

fn prefix(kind: DiffKind) -> char {
    match kind {
        DiffKind::Added => '+',
        DiffKind::Removed => '-',
        DiffKind::Context => ' ',
    }
}

fn source<'a>(
    ui: &mut Ui,
    language: Option<&str>,
    clipboard: Option<&str>,
    numbers: bool,
    lines: impl Iterator<Item = DiffLine<'a>>,
    diff: bool,
) -> Response {
    let theme = Theme::current(ui.ctx());
    let c = theme.colors;
    let fill = match theme.mode {
        Mode::Light => Color32::from_rgb(244, 244, 245),
        Mode::Dark => Color32::from_rgb(18, 18, 21),
    };
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1., c.border_subtle))
        .corner_radius(10)
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 5.;
            ui.set_min_width(ui.available_width());
            if language.is_some() || clipboard.is_some() {
                ui.horizontal(|ui| {
                    if let Some(language) = language {
                        Badge::new(language).show(ui);
                    }
                    if let Some(text) = clipboard {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            copy(ui, text)
                        });
                    }
                });
                Separator::Horizontal.show(ui);
            }
            let width = ui.available_width();
            let scroll_id = ui.next_auto_id();
            ScrollArea::horizontal().id_salt(scroll_id).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.;
                for (index, line) in lines.enumerate() {
                    let (background, ink) = match line.kind {
                        DiffKind::Added => (mix(fill, c.success.base, 0.13), c.success.ring),
                        DiffKind::Removed => (mix(fill, c.danger.base, 0.13), c.danger.ring),
                        DiffKind::Context => (fill, c.text_primary),
                    };
                    let ink = super::badge::readable(background, ink);
                    Frame::new()
                        .fill(background)
                        .inner_margin(Margin::symmetric(5, 2))
                        .show(ui, |ui| {
                            ui.set_min_width((width - 10.).max(0.));
                            ui.horizontal(|ui| {
                                if numbers {
                                    ui.label(
                                        Type::Kbd
                                            .text(format!("{:>3}", index + 1))
                                            .color(c.text_muted),
                                    );
                                }
                                if diff {
                                    ui.label(
                                        Type::Mono.text(prefix(line.kind).to_string()).color(ink),
                                    );
                                }
                                ui.add(
                                    Label::new(Type::Mono.text(line.text).color(ink))
                                        .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            });
                        });
                }
            });
        })
        .response
}

fn copy(ui: &mut Ui, text: &str) {
    let id = ui.next_auto_id().with("copied");
    let time = ui.input(|input| input.time);
    let copied = ui
        .ctx()
        .data(|data| data.get_temp::<f64>(id))
        .is_some_and(|until| until > time);
    if copied {
        ui.ctx().request_repaint_after_secs(1.5);
    }
    let response = Button::new(if copied { "Copied" } else { "Copy" })
        .leading(if copied { Icon::Check } else { Icon::Copy })
        .size(Size::Sm)
        .variant(Variant::Ghost)
        .show(ui);
    if response.clicked() {
        ui.ctx().copy_text(text.to_owned());
        ui.ctx().data_mut(|data| data.insert_temp(id, time + 1.5));
        ui.ctx().request_repaint();
    }
}
