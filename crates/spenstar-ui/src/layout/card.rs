use egui::{Frame, InnerResponse, Margin, Role, Sense, Shadow, Stroke, Ui, UiBuilder, WidgetInfo};

use crate::{Theme, motion, theme::Radius, typography::Type};

use super::Separator;

/// A full-width content surface with standard inset padding.
pub struct Surface;

impl Surface {
    /// Draw a surface and return its content result.
    pub fn show<R>(ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        Card::new(CardVariant::Flat).show(ui, |card| card.body(contents))
    }
}

/// Card background and elevation style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    /// Surface fill.
    Flat,
    /// Surface fill and subtle outline.
    #[default]
    Bordered,
    /// Raised surface with a soft shadow.
    Elevated,
    /// Focusable card with hover feedback and a click response.
    Interactive,
}

/// A surface for related content, with optional header and footer sections.
#[derive(Clone, Copy, Debug, Default)]
pub struct Card<'a> {
    variant: CardVariant,
    label: Option<&'a str>,
}

impl<'a> Card<'a> {
    /// Create a card in the given style.
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            label: None,
        }
    }

    /// Set the name announced for an interactive card.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Render the card and return the body result and outer response.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut CardBody<'_>) -> R,
    ) -> InnerResponse<R> {
        let sense = if self.variant == CardVariant::Interactive {
            Sense::click()
        } else {
            Sense::hover()
        };
        let result = ui.scope_builder(UiBuilder::new().sense(sense), |ui| {
            let theme = Theme::current(ui.ctx());
            let c = theme.colors;
            let prior = ui.response();
            let fill = match self.variant {
                CardVariant::Elevated => c.surface_elevated,
                CardVariant::Interactive if prior.hovered() => c.accent.soft,
                _ => c.surface,
            };
            let border = if self.variant == CardVariant::Flat {
                Stroke::NONE
            } else {
                Stroke::new(1., c.border_subtle)
            };
            let shadow = if self.variant == CardVariant::Elevated {
                theme.shadow()
            } else {
                Shadow::NONE
            };
            let frame = Frame::new()
                .fill(fill)
                .stroke(border)
                .corner_radius(12)
                .shadow(shadow)
                .inner_margin(Margin::same(16));
            frame
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    content(&mut CardBody { ui })
                })
                .inner
        });
        if self.variant == CardVariant::Interactive {
            result.response.widget_info(|| {
                WidgetInfo::labeled(
                    Role::Button,
                    result.response.enabled(),
                    self.label.unwrap_or_default(),
                )
            });
            motion::focus_ring(
                ui,
                &result.response,
                Radius::Lg.for_size(result.response.rect.size()),
            );
        }
        result
    }
}

/// Content slots inside a card.
pub struct CardBody<'a> {
    ui: &'a mut Ui,
}

impl CardBody<'_> {
    /// Render a heading and optional supporting description.
    pub fn header(&mut self, title: &str, description: Option<&str>) {
        let colors = Theme::current(self.ui.ctx()).colors;
        self.ui.label(Type::H3.text(title));
        if let Some(description) = description {
            self.ui
                .label(Type::Small.text(description).color(colors.text_secondary));
        }
        self.ui.add_space(8.);
    }

    /// Render the main content.
    pub fn body<R>(&mut self, content: impl FnOnce(&mut Ui) -> R) -> R {
        content(self.ui)
    }

    /// Add a divider and render footer content beneath it.
    pub fn footer<R>(&mut self, content: impl FnOnce(&mut Ui) -> R) -> R {
        self.ui.add_space(4.);
        Separator::Horizontal.show(self.ui);
        content(self.ui)
    }
}
