use egui::{Frame, InnerResponse, Margin, Role, Sense, Shadow, Stroke, Ui, UiBuilder, WidgetInfo};

use crate::{
    Theme, motion,
    theme::{Mode, Radius},
    typography::Type,
};

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
    padding: Option<i8>,
}

impl<'a> Card<'a> {
    /// Create a card in the given style.
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            label: None,
            padding: None,
        }
    }

    /// Set the name announced for an interactive card.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the internal padding in points.
    pub fn padding(mut self, points: i8) -> Self {
        self.padding = Some(points.max(0));
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
                CardVariant::Interactive if prior.hovered() => c.accent.soft,
                _ => c.surface,
            };
            let border = if self.variant == CardVariant::Flat {
                Stroke::NONE
            } else {
                Stroke::new(
                    1.,
                    if theme.mode == Mode::Light {
                        egui::Color32::from_black_alpha(13)
                    } else {
                        c.border
                    },
                )
            };
            let shadow = if self.variant != CardVariant::Flat && theme.mode == Mode::Light {
                theme.shadow()
            } else {
                Shadow::NONE
            };
            let frame = Frame::new()
                .fill(fill)
                .stroke(border)
                .corner_radius(16)
                .shadow(shadow)
                .inner_margin(Margin::same(self.padding.unwrap_or(18)));
            frame
                .show(ui, |ui| {
                    ui.visuals_mut().extreme_bg_color = c.surface_secondary;
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
                Radius::Xl.for_size(result.response.rect.size()),
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
        self.ui
            .add_space((12. - self.ui.spacing().item_spacing.y).max(0.));
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
