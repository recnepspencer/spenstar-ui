use egui::{Align, Frame, Layout, Margin, Response, Stroke, Ui, vec2};

use crate::{
    Theme,
    button::{Button, CloseButton, Size, Variant},
    theme::mix,
    typography::Type,
};

use super::{Tone, semantic_ink, status_icon};

/// Responses from an inline callout.
pub struct AlertResponse {
    /// The whole card.
    pub response: Response,
    /// Whether the optional action was pressed.
    pub action_clicked: bool,
    /// Whether the optional close control was pressed.
    pub dismissed: bool,
}

/// Inline semantic status callout.
pub struct Alert<'a> {
    title: &'a str,
    description: Option<&'a str>,
    action: Option<&'a str>,
    tone: Tone,
    dismissible: bool,
}

/// A full-width inline alert.
pub type Banner<'a> = Alert<'a>;

impl<'a> Alert<'a> {
    /// Create a neutral callout.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
            action: None,
            tone: Tone::Neutral,
            dismissible: false,
        }
    }

    /// Add a detail line.
    pub fn description(mut self, text: &'a str) -> Self {
        self.description = Some(text);
        self
    }

    /// Set semantic appearance and status icon.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Add a trailing action button.
    pub fn action(mut self, label: &'a str) -> Self {
        self.action = Some(label);
        self
    }

    /// Add a close control.
    pub fn dismissible(mut self, enabled: bool) -> Self {
        self.dismissible = enabled;
        self
    }

    /// Render a full-width alert and report any action or dismiss click.
    pub fn show(self, ui: &mut Ui) -> AlertResponse {
        let c = Theme::current(ui.ctx()).colors;
        let palette = self.tone.palette(c);
        let width = ui.available_width();
        let result = Frame::new()
            .fill(palette.soft)
            .stroke(Stroke::new(1., mix(palette.soft, palette.ring, 0.35)))
            .corner_radius(12)
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width((width - 26.).max(1.));
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 32.),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        let dismissed = self.dismissible && CloseButton::show(ui).clicked();
                        let action_clicked = self.action.is_some_and(|label| {
                            Button::new(label)
                                .variant(Variant::Ghost)
                                .size(Size::Sm)
                                .show(ui)
                                .clicked()
                        });
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            status_icon(self.tone).show(
                                ui,
                                17.,
                                semantic_ink(palette.ring, palette.soft, c.text_primary),
                            );
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.;
                                ui.label(Type::Label.text(self.title).color(c.text_primary));
                                if let Some(text) = self.description {
                                    ui.label(Type::Description.text(text).color(semantic_ink(
                                        c.text_secondary,
                                        palette.soft,
                                        c.text_primary,
                                    )));
                                }
                            });
                        });
                        (action_clicked, dismissed)
                    },
                )
                .inner
            });
        let (action_clicked, dismissed) = result.inner;
        AlertResponse {
            response: result.response,
            action_clicked,
            dismissed,
        }
    }
}
