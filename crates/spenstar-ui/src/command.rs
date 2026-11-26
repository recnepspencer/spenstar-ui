//! A modal command palette with grouped search and guarded shortcuts.
use egui::{
    Align, Frame, Key, KeyboardShortcut, Layout, Margin, Modifiers, Response, ScrollArea, Sense,
    Stroke, TextEdit, Ui, UiBuilder, vec2,
};

use crate::{
    Theme,
    icon::{Icon, IconSource},
    input::ShortcutGuard,
    typography::Type,
};

/// Action searchable by title and optional alternate keywords.
#[derive(Clone, Debug)]
pub struct CommandItem<'a> {
    /// Unique identifier returned on activation.
    pub id: &'a str,
    /// Visible title.
    pub label: &'a str,
    /// Additional search terms.
    pub keywords: &'a str,
    /// Leading icon.
    pub icon: Option<IconSource>,
    /// Optional key hint.
    pub shortcut: Option<&'a str>,
}

impl<'a> CommandItem<'a> {
    /// Create a command without decoration.
    pub const fn new(id: &'a str, label: &'a str) -> Self {
        Self {
            id,
            label,
            keywords: "",
            icon: None,
            shortcut: None,
        }
    }
}

/// A named set of related actions.
pub struct CommandGroup<'a> {
    /// Section header.
    pub label: &'a str,
    /// Actions in display order.
    pub items: &'a [CommandItem<'a>],
}

/// State and rendering for a centered command palette.
pub struct CommandPalette {
    id: egui::IdSalt,
    open: bool,
    query: String,
    active: usize,
    focus: bool,
}

impl CommandPalette {
    /// Cross-platform global activation key.
    pub const SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::K);

    /// Create a closed palette with a stable identity.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: egui::IdSalt::new(id),
            open: false,
            query: String::new(),
            active: 0,
            focus: false,
        }
    }

    /// Whether the palette is open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Current search text.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Open and reset the palette.
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.active = 0;
        self.focus = true;
    }

    /// Close the palette.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Draw the palette and return an activated command ID.
    pub fn show<'a>(&mut self, ui: &Ui, groups: &[CommandGroup<'a>]) -> Option<&'a str> {
        if !self.open && ShortcutGuard::new(ui).consume(&Self::SHORTCUT) {
            self.open();
        }
        if !self.open {
            return None;
        }
        let ctx = ui.ctx();
        let id = ui.make_persistent_id(self.id);
        let c = Theme::current(ctx).colors;
        let modal = egui::Modal::new(id).backdrop_color(c.scrim).frame(
            Frame::new()
                .fill(c.overlay)
                .stroke(Stroke::new(1., c.overlay_border))
                .corner_radius(16)
                .shadow(Theme::current(ctx).shadow())
                .inner_margin(Margin::same(12)),
        );
        let response = modal.show(ctx, |ui| {
            let width = (ctx.content_rect().width() - 36.).clamp(100., 500.);
            ui.set_width(width);
            let input_id = id.with("search");
            let owns_input = ui.is_enabled() && ctx.memory(|m| m.allows_interaction(ui.layer_id()));
            let (up, down, enter, escape) = if owns_input {
                ctx.input_mut(|i| {
                    (
                        i.count_and_consume_key(Modifiers::NONE, Key::ArrowUp),
                        i.count_and_consume_key(Modifiers::NONE, Key::ArrowDown),
                        i.consume_key(Modifiers::NONE, Key::Enter),
                        i.consume_key(Modifiers::NONE, Key::Escape),
                    )
                })
            } else {
                (0, 0, false, false)
            };
            let mut dismiss = escape;
            let query_changed = ui
                .allocate_ui_with_layout(
                    vec2(width, 34.),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 8.;
                        IconSource::from(Icon::Search).show(ui, 19., c.text_muted);
                        let input = ui.add(
                            TextEdit::singleline(&mut self.query)
                                .id(input_id)
                                .return_key(None)
                                .frame(Frame::NONE)
                                .hint_text(Type::Label.text("Search pages and actions…"))
                                .desired_width((width - 69.).max(25.)),
                        );
                        if self.focus {
                            input.request_focus();
                            self.focus = false;
                        }
                        dismiss |= key_hint(ui, "ESC", true).clicked();
                        input.changed()
                    },
                )
                .inner;
            crate::layout::Separator::Horizontal.show(ui);
            if query_changed {
                self.active = 0;
            }
            let hits = matches(&self.query, groups);
            let count = hits.len();
            self.active = self.active.min(count.saturating_sub(1));
            if count > 0 {
                self.active = (self.active + down % count + count - up % count) % count;
            }
            let max_height = (ctx.content_rect().height() * 0.54).clamp(80., 390.);
            let mut selected = enter
                .then(|| hits.get(self.active).map(|(_, item)| item.id))
                .flatten();
            ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .max_height(max_height)
                .min_scrolled_height(max_height.min(300.))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if hits.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(32.);
                            ui.label(
                                Type::Label
                                    .text("No matching commands")
                                    .color(c.text_secondary),
                            );
                        });
                        return;
                    }
                    let mut group = None;
                    for (index, &(section, item)) in hits.iter().enumerate() {
                        if group != Some(section) {
                            ui.add_space(5.);
                            ui.label(
                                Type::SectionLabel
                                    .text(groups[section].label)
                                    .color(c.text_muted),
                            );
                            group = Some(section);
                        }
                        let (row, moved) = command_row(ui, item, index == self.active);
                        if moved {
                            self.active = index;
                        }
                        if row.clicked() {
                            selected = Some(item.id);
                        }
                        if (up > 0 || down > 0 || query_changed) && index == self.active {
                            row.scroll_to_me(Some(Align::Center));
                        }
                    }
                });
            (selected, dismiss)
        });
        let (selected, close) = response.inner;
        if selected.is_some() || close || response.should_close() {
            ctx.memory_mut(|m| m.surrender_focus(id.with("search")));
            self.close();
            ctx.request_repaint();
        }
        selected
    }
}

fn matches<'a, 'b>(
    query: &str,
    groups: &'b [CommandGroup<'a>],
) -> Vec<(usize, &'b CommandItem<'a>)> {
    let query = query.to_lowercase();
    groups
        .iter()
        .enumerate()
        .flat_map(|(section, group)| {
            let query = &query;
            group.items.iter().filter_map(move |item| {
                let searchable =
                    format!("{} {} {}", item.label, item.keywords, group.label).to_lowercase();
                query
                    .split_whitespace()
                    .all(|word| {
                        let mut remaining = searchable.chars();
                        searchable.contains(word)
                            || word.chars().all(|needle| remaining.any(|c| c == needle))
                    })
                    .then_some((section, item))
            })
        })
        .collect()
}

fn command_row(ui: &mut Ui, item: &CommandItem<'_>, active: bool) -> (Response, bool) {
    let c = Theme::current(ui.ctx()).colors;
    let background = ui.painter().add(egui::Shape::Noop);
    let row = ui
        .scope_builder(
            UiBuilder::new().id_salt(item.id).sense(Sense::click()),
            |ui| {
                Frame::new()
                    .inner_margin(Margin::symmetric(9, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_min_height(26.);
                            if let Some(icon) = &item.icon {
                                icon.show(ui, 16., c.text_secondary);
                            }
                            ui.add(
                                egui::Label::new(Type::Label.text(item.label))
                                    .selectable(false)
                                    .truncate(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if let Some(key) = item.shortcut {
                                    key_hint(ui, key, false);
                                }
                            });
                        });
                    });
            },
        )
        .response;
    if active || row.hovered() {
        ui.painter().set(
            background,
            egui::epaint::RectShape::filled(
                row.rect,
                8,
                if active { c.accent.soft } else { c.field_hover },
            ),
        );
    }
    row.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Button, row.enabled(), active, item.label)
    });
    let moved = row.hovered() && ui.input(|i| i.pointer.delta() != vec2(0., 0.));
    (row, moved)
}

fn key_hint(ui: &mut Ui, text: &str, clickable: bool) -> Response {
    let c = Theme::current(ui.ctx()).colors;
    Frame::new()
        .fill(c.surface_secondary)
        .stroke(Stroke::new(1., c.border_subtle))
        .inner_margin(Margin::symmetric(5, 2))
        .corner_radius(5)
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(Type::Kbd.text(text).color(c.text_secondary))
                    .selectable(false)
                    .sense(if clickable {
                        Sense::click()
                    } else {
                        Sense::hover()
                    }),
            )
        })
        .inner
}
