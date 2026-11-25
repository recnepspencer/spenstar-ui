//! Buttons, segmented controls, and links.
mod group;
mod trigger;

pub use group::{ButtonGroup, ToggleButton, ToggleButtonGroup};
pub use trigger::{CloseButton, Link};

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, CursorIcon, Galley, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui,
    Widget, WidgetInfo, pos2, vec2,
};

use crate::{
    Theme,
    icon::IconSource,
    motion,
    theme::{Radius, mix, primitives},
    typography::Type,
};

/// The visual treatment for a button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// High-emphasis accent.
    #[default]
    Accent,
    /// Filled neutral.
    Secondary,
    /// Tonal accent.
    Soft,
    /// Bordered neutral.
    Outline,
    /// Transparent until interaction.
    Ghost,
    /// Destructive action.
    Danger,
}

/// Button height and padding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size {
    /// Compact, 28 points tall.
    Sm,
    /// Standard, 36 points tall.
    #[default]
    Md,
    /// Large, 44 points tall.
    Lg,
}

impl Size {
    fn dimensions(self) -> (f32, f32, f32, f32) {
        match self {
            Self::Sm => (28., 10., 14., 12.),
            Self::Md => (36., 14., 16., 13.),
            Self::Lg => (44., 18., 20., 15.),
        }
    }
}

/// A text or icon trigger. The label also supplies its accessible name.
#[derive(Clone, Debug)]
pub struct Button {
    label: String,
    variant: Variant,
    size: Size,
    leading: Option<IconSource>,
    trailing: Option<IconSource>,
    icon_only: bool,
    enabled: bool,
    loading: bool,
    tooltip: Option<String>,
    radius: Radius,
}

impl Button {
    /// Create a labeled accent button.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: Variant::Accent,
            size: Size::Md,
            leading: None,
            trailing: None,
            icon_only: false,
            enabled: true,
            loading: false,
            tooltip: None,
            radius: Radius::Md,
        }
    }

    /// Set the button's visual treatment.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the height and padding.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set the corner radius token.
    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    /// Set a leading Lucide or custom SVG icon.
    pub fn leading(mut self, icon: impl Into<IconSource>) -> Self {
        self.leading = Some(icon.into());
        self
    }

    /// Set a trailing Lucide or custom SVG icon.
    pub fn trailing(mut self, icon: impl Into<IconSource>) -> Self {
        self.trailing = Some(icon.into());
        self
    }

    /// Show the icon visually and use the label for accessibility and the tooltip.
    pub fn icon_only(mut self, icon: impl Into<IconSource>) -> Self {
        self.leading = Some(icon.into());
        self.trailing = None;
        self.icon_only = true;
        self
    }

    /// Disable pointer and keyboard activation while retaining the normal layout.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Show a spinner in the leading slot and suppress activation.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Provide a hover hint, including when the button is disabled.
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Render the button.
    pub fn show(self, ui: &mut Ui) -> Response {
        self.ui(ui)
    }

    fn render(self, ui: &mut Ui, segment: Segment, selected: Option<bool>) -> Response {
        let enabled = self.enabled && !self.loading;
        let tooltip = self
            .tooltip
            .clone()
            .or_else(|| self.icon_only.then(|| self.label.clone()));
        let response = if enabled {
            self.draw(ui, segment, selected)
        } else {
            ui.add_enabled_ui(false, |ui| self.draw(ui, segment, selected))
                .inner
        };
        if let Some(tooltip) = tooltip {
            if response.enabled() {
                response.on_hover_text(tooltip)
            } else {
                response.on_disabled_hover_text(tooltip)
            }
        } else {
            response
        }
    }

    fn label_galley(&self, ui: &Ui) -> Option<Arc<Galley>> {
        (!self.icon_only).then(|| {
            let mut font = Type::H4.font();
            font.size = self.size.dimensions().3;
            ui.painter()
                .layout_no_wrap(self.label.clone(), font, Color32::PLACEHOLDER)
        })
    }

    fn icon_slot_width(&self, present: bool) -> f32 {
        if present {
            self.size.dimensions().2 + primitives::SPACE[2]
        } else {
            0.
        }
    }

    fn width(&self, label: Option<&Galley>) -> f32 {
        let (height, padding, _, _) = self.size.dimensions();
        if self.icon_only {
            height
        } else {
            label.map_or(0., |text| text.size().x)
                + 2. * padding
                + self.icon_slot_width(self.leading.is_some() || self.loading)
                + self.icon_slot_width(self.trailing.is_some())
        }
    }

    pub(crate) fn natural_width(&self, ui: &Ui) -> f32 {
        self.width(self.label_galley(ui).as_deref())
    }

    fn draw(self, ui: &mut Ui, segment: Segment, selected: Option<bool>) -> Response {
        let (height, padding, icon_size, _) = self.size.dimensions();
        let galley = self.label_galley(ui);
        let width = self.width(galley.as_deref());
        let leading_width = self.icon_slot_width(self.leading.is_some() || self.loading);
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        response.widget_info(|| {
            let label = if self.loading {
                format!("{} (loading)", self.label)
            } else {
                self.label.clone()
            };
            if let Some(selected) = selected {
                WidgetInfo::selected(Role::Button, response.enabled(), selected, label)
            } else {
                WidgetInfo::labeled(Role::Button, response.enabled(), label)
            }
        });
        if !ui.is_rect_visible(rect) {
            return response;
        }
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        let colors = Theme::current(ui.ctx()).colors;
        let [base, hover, active, text, border] = match self.variant {
            Variant::Accent | Variant::Danger | Variant::Secondary => {
                let palette = match self.variant {
                    Variant::Accent => colors.accent,
                    Variant::Danger => colors.danger,
                    _ => colors.default,
                };
                [
                    palette.base,
                    palette.hover,
                    palette.active,
                    palette.foreground,
                    Color32::TRANSPARENT,
                ]
            }
            Variant::Soft => [
                colors.accent.soft,
                colors.accent.soft_hover,
                colors.accent.soft_hover,
                colors.text_primary,
                Color32::TRANSPARENT,
            ],
            // Translucent feedback leaves the sliding selection visible underneath.
            Variant::Ghost if selected.is_some() => [
                Color32::TRANSPARENT,
                colors.text_primary.gamma_multiply(0.08),
                colors.text_primary.gamma_multiply(0.14),
                colors.text_primary,
                Color32::TRANSPARENT,
            ],
            Variant::Outline | Variant::Ghost => [
                Color32::TRANSPARENT,
                colors.surface_subtle,
                colors.accent.soft_hover,
                colors.text_primary,
                if self.variant == Variant::Outline {
                    colors.border
                } else {
                    Color32::TRANSPARENT
                },
            ],
        };
        let over = motion::eased(
            ui.ctx(),
            response.id.with("hover"),
            response.hovered(),
            motion::FAST,
        );
        let press = motion::eased(
            ui.ctx(),
            response.id.with("press"),
            response.is_pointer_button_down_on(),
            motion::FAST,
        );
        let fill = mix(mix(base, hover, over), active, press);
        let radius = segment.radius(self.radius.for_size(rect.size()));
        let stroke = if segment == Segment::Single && border != Color32::TRANSPARENT {
            Stroke::new(1., border)
        } else {
            Stroke::NONE
        };
        ui.painter()
            .rect(rect, radius, fill, stroke, StrokeKind::Inside);

        let x = if self.icon_only {
            rect.center().x - icon_size * 0.5
        } else {
            rect.left() + padding
        };
        let icon_rect = |x| {
            Rect::from_min_size(
                pos2(x, rect.center().y - icon_size * 0.5),
                vec2(icon_size, icon_size),
            )
        };
        if self.loading {
            egui::Spinner::new()
                .size(icon_size)
                .color(text)
                .paint_at(ui, icon_rect(x));
        } else if let Some(icon) = self.leading {
            let _ = icon.paint(ui, icon_rect(x), text);
        }
        if let Some(galley) = galley {
            ui.painter().galley(
                pos2(x + leading_width, rect.center().y - galley.size().y * 0.5),
                galley,
                text,
            );
        }
        if let Some(icon) = self.trailing {
            let _ = icon.paint(ui, icon_rect(rect.right() - padding - icon_size), text);
        }
        motion::focus_ring(ui, &response, radius);
        response
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        self.render(ui, Segment::Single, None)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Segment {
    Single,
    First,
    Middle,
    Last,
}

impl Segment {
    fn at(index: usize, count: usize) -> Self {
        if count == 1 {
            Self::Single
        } else if index == 0 {
            Self::First
        } else if index + 1 == count {
            Self::Last
        } else {
            Self::Middle
        }
    }

    fn radius(self, radius: CornerRadius) -> CornerRadius {
        match self {
            Self::Single => radius,
            Self::First => CornerRadius {
                ne: 0,
                se: 0,
                ..radius
            },
            Self::Middle => CornerRadius::ZERO,
            Self::Last => CornerRadius {
                nw: 0,
                sw: 0,
                ..radius
            },
        }
    }
}
