use super::{
    EntryStyle, FieldVariant, Size,
    entry::{counter, icon_button},
};
use crate::{
    Theme,
    icon::{Icon, IconSource},
    typography::Type,
};
use egui::{Key, Modifiers, Response, TextBuffer, TextEdit, Ui, vec2};

/// The result of displaying a search editor.
pub struct SearchResponse {
    /// The text editor response.
    pub response: Response,
    /// Enter submitted the current query.
    pub submitted: bool,
    /// The clear action was activated.
    pub cleared: bool,
    pub(crate) frame: egui::Rect,
}

/// A themed, single-line text editor.
#[derive(Clone, Debug, Default)]
pub struct TextField<'a> {
    /// Control treatment.
    pub variant: FieldVariant,
    /// Control height and font size.
    pub size: Size,
    /// Hint shown when empty.
    pub placeholder: &'a str,
    /// Optional decorative icon on the left.
    pub leading: Option<IconSource>,
    /// Optional decorative icon on the right.
    pub trailing: Option<IconSource>,
    /// Show a clear action when there is text.
    pub clearable: bool,
    /// Mask the value and provide a visibility toggle.
    pub password: bool,
    /// Draw error styling.
    pub invalid: bool,
    /// Allow selecting and copying the value.
    pub read_only: bool,
    /// Disable interaction.
    pub disabled: bool,
    /// Character limit and visible counter.
    pub max_len: Option<usize>,
}

impl<'a> TextField<'a> {
    /// Create an editor with a placeholder.
    pub fn new(placeholder: &'a str) -> Self {
        Self {
            placeholder,
            ..Self::default()
        }
    }

    /// Render the control and return the editable widget response.
    pub fn show(self, ui: &mut Ui, value: &mut String) -> Response {
        self.render(ui, value, None, 0., |_| {}).0.response
    }

    pub(crate) fn render<R>(
        self,
        ui: &mut Ui,
        value: &mut String,
        shortcut: Option<&str>,
        extra_width: f32,
        extra: impl FnOnce(&mut Ui) -> R,
    ) -> (SearchResponse, R) {
        self.render_options(ui, value, (shortcut, ""), extra_width, extra)
    }

    pub(crate) fn render_affixed<R>(
        self,
        ui: &mut Ui,
        value: &mut String,
        prefix: &str,
        extra_width: f32,
        extra: impl FnOnce(&mut Ui) -> R,
    ) -> (SearchResponse, R) {
        self.render_options(ui, value, (None, prefix), extra_width, extra)
    }

    fn render_options<R>(
        self,
        ui: &mut Ui,
        value: &mut String,
        decorations: (Option<&str>, &str),
        extra_width: f32,
        extra: impl FnOnce(&mut Ui) -> R,
    ) -> (SearchResponse, R) {
        ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            self.render_enabled(ui, value, decorations, extra_width, extra)
        })
        .inner
    }

    fn render_enabled<R>(
        self,
        ui: &mut Ui,
        value: &mut String,
        (shortcut, prefix): (Option<&str>, &str),
        extra_width: f32,
        extra: impl FnOnce(&mut Ui) -> R,
    ) -> (SearchResponse, R) {
        let style = EntryStyle {
            variant: self.variant,
            size: self.size,
            invalid: self.invalid,
        };
        let visibility_id = ui.unique_id().with("visible-password");
        let revealed = ui
            .ctx()
            .data(|d| d.get_temp::<bool>(visibility_id))
            .unwrap_or(false);
        let colors = Theme::current(ui.ctx()).colors;
        let tint = colors.text_secondary;
        let id = ui.unique_id().with("edit");
        let hint = shortcut.map(|shortcut| {
            ui.painter()
                .layout_no_wrap(shortcut.to_owned(), Type::Kbd.font(), tint)
        });
        // Keep the edit geometry stable as text makes the clear action appear.
        let reserve_clear = self.clearable && !self.read_only;
        let (mut shell, bg) = style.show(ui, |ui| {
            let spacing = ui.spacing().item_spacing.x;
            if let Some(icon) = self.leading {
                icon.show(ui, 16., tint);
            }
            let reserved = extra_width
                + self.trailing.as_ref().map_or(0., |_| 16. + spacing)
                + f32::from(u8::from(reserve_clear) + u8::from(self.password)) * (22. + spacing)
                + hint.as_ref().map_or(0., |g| g.size().x + 10. + spacing);
            let focused = ui.memory(|mem| mem.has_focus(id));
            let submitted = ui.is_enabled()
                && focused
                && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter));
            let mut borrowed = value.as_str();
            let text: &mut dyn TextBuffer = if self.read_only { &mut borrowed } else { value };
            let mut response = ui.add(
                TextEdit::singleline(text)
                    .id(id)
                    .font(style.font())
                    .prefix(prefix.to_owned())
                    .hint_text(
                        Type::Body
                            .text(self.placeholder)
                            .font(style.font())
                            .color(colors.text_muted),
                    )
                    .password(self.password && !revealed)
                    .char_limit(self.max_len.unwrap_or(usize::MAX))
                    .return_key(None)
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::ZERO)
                    .min_size(vec2(0., style.height() - 4.))
                    .align(egui::Align2::LEFT_CENTER)
                    .desired_width((ui.available_width() - reserved).max(1.)),
            );
            let mut cleared = false;
            if reserve_clear {
                if !value.is_empty() {
                    cleared = icon_button(ui, Icon::X, "Clear text").clicked();
                    if cleared {
                        value.clear();
                        response.mark_changed();
                        response.request_focus();
                        ui.ctx().request_repaint();
                    }
                } else {
                    ui.add_space(22. + spacing);
                }
            }
            if self.password {
                let (icon, name) = if revealed {
                    (Icon::EyeOff, "Hide password")
                } else {
                    (Icon::Eye, "Show password")
                };
                if icon_button(ui, icon, name).clicked() {
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(visibility_id, !revealed));
                    response.request_focus();
                }
            }
            if let Some(icon) = self.trailing {
                icon.show(ui, 16., tint);
            }
            if let Some(galley) = hint {
                let (rect, _) =
                    ui.allocate_exact_size(galley.size() + vec2(10., 6.), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4, colors.surface_subtle);
                ui.painter()
                    .galley(rect.center() - galley.size() * 0.5, galley, tint);
            }
            (
                SearchResponse {
                    response,
                    submitted,
                    cleared,
                    frame: egui::Rect::NOTHING,
                },
                extra(ui),
            )
        });
        shell.inner.0.frame = shell.response.rect;
        style.paint(ui, bg, shell.response.rect, &shell.inner.0.response);
        if let Some(limit) = self.max_len {
            counter(ui, value.chars().count(), limit);
        }
        shell.inner
    }
}

/// Search editor with an instant clear action and optional shortcut hint.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchField<'a> {
    /// Hint shown when the query is empty.
    pub placeholder: &'a str,
    /// Shortcut label supplied by the application.
    pub shortcut: Option<&'a str>,
    /// Disable interaction.
    pub disabled: bool,
}

impl SearchField<'_> {
    /// Render the search editor and report submit and clear actions.
    pub fn show(self, ui: &mut Ui, query: &mut String) -> SearchResponse {
        TextField {
            placeholder: self.placeholder,
            disabled: self.disabled,
            leading: Some(Icon::Search.into()),
            clearable: true,
            ..TextField::default()
        }
        .render(ui, query, self.shortcut, 0., |_| {})
        .0
    }
}
