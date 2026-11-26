use egui::{Color32, Frame, Label, Margin, Response, Sense, Stroke, Ui, vec2};

use super::{Avatar, Tone};
use crate::{
    Theme,
    form::icon_button_with_colors,
    icon::{Icon, IconSource},
    motion,
    theme::{luminance, mix},
    typography::Type,
};

/// Semantic label treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Solid semantic fill.
    Solid,
    /// Muted semantic fill.
    #[default]
    Soft,
    /// Colored perimeter.
    Outline,
    /// Leading semantic dot.
    Dot,
}

/// Semantic label size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeSize {
    /// Compact label.
    #[default]
    Sm,
    /// Standard label.
    Md,
}

/// A compact, high-contrast semantic label.
#[derive(Clone, Debug)]
pub struct Badge<'a> {
    /// Visible label.
    pub label: &'a str,
    /// Semantic color.
    pub tone: Tone,
    /// Surface treatment.
    pub variant: BadgeVariant,
    /// Label density.
    pub size: BadgeSize,
    /// Leading icon.
    pub icon: Option<IconSource>,
}

impl<'a> Badge<'a> {
    /// Create a small neutral label.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            tone: Tone::Neutral,
            variant: BadgeVariant::Soft,
            size: BadgeSize::Sm,
            icon: None,
        }
    }

    /// Resolve opaque background and AA foreground colors.
    pub fn colors(&self, theme: Theme, highlighted: bool) -> (Color32, Color32) {
        let c = theme.colors;
        let palette = self.tone.palette(c);
        let soft_ink = readable(
            palette.soft_hover,
            if self.tone == Tone::Neutral {
                c.text_primary
            } else {
                palette.ring
            },
        );
        let (fill, preferred) = match self.variant {
            BadgeVariant::Solid => (palette.base, palette.foreground),
            BadgeVariant::Soft if highlighted => (palette.soft_hover, soft_ink),
            BadgeVariant::Soft => (palette.soft, soft_ink),
            _ if highlighted => (palette.soft_hover, palette.ring),
            _ => (c.surface, palette.ring),
        };
        (fill, readable(fill, preferred))
    }

    /// Render the semantic label.
    pub fn show(&self, ui: &mut Ui) -> Response {
        self.content(ui, None, false, false, false, ui.available_width())
            .response
    }

    pub(crate) fn content(
        &self,
        ui: &mut Ui,
        avatar: Option<&Avatar<'_>>,
        selectable: bool,
        selected: bool,
        removable: bool,
        width: f32,
    ) -> ChipResponse {
        let theme = Theme::current(ui.ctx());
        let outline = if self.variant == BadgeVariant::Outline || (selectable && selected) {
            Stroke::new(1., self.tone.palette(theme.colors).ring)
        } else {
            Stroke::NONE
        };
        let (height, padding) = match self.size {
            BadgeSize::Sm => (18., 7),
            BadgeSize::Md => (22., 9),
        };
        let scope = ui.scope_builder(
            egui::UiBuilder::new().sense(if selectable {
                Sense::click()
            } else {
                Sense::hover()
            }),
            |ui| {
                let (fill, ink) =
                    self.colors(theme, selected || (selectable && ui.response().hovered()));
                Frame::new()
                    .fill(fill)
                    .stroke(outline)
                    .corner_radius(100)
                    .inner_margin(Margin::symmetric(padding, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 5.;
                            let start = ui.cursor().min.x;
                            if self.variant == BadgeVariant::Dot {
                                let (rect, _) =
                                    ui.allocate_exact_size(vec2(6., height), Sense::hover());
                                ui.painter().circle_filled(
                                    rect.center(),
                                    3.,
                                    self.tone.palette(theme.colors).ring,
                                );
                            }
                            if let Some(avatar) = avatar {
                                avatar.show_sized(ui, height);
                            } else if let Some(icon) = &self.icon {
                                icon.show(ui, 13., ink);
                            }
                            let used = ui.cursor().min.x - start
                                + f32::from(padding) * 2.
                                + if removable { 29. } else { 0. };
                            let label_width = ui
                                .painter()
                                .layout_no_wrap(self.label.to_owned(), Type::Badge.font(), ink)
                                .size()
                                .x;
                            ui.add_sized(
                                [label_width.min((width - used).max(5.)), height],
                                Label::new(Type::Badge.text(self.label).color(ink))
                                    .selectable(false)
                                    .truncate(),
                            );
                            removable
                                && icon_button_with_colors(
                                    ui,
                                    Icon::X,
                                    &format!("Remove {}", self.label),
                                    ink,
                                    remove_hover_fill(fill, ink),
                                )
                                .clicked()
                        })
                        .inner
                    })
                    .inner
            },
        );
        let response = scope.response;
        if selectable {
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::Role::Button,
                    response.enabled(),
                    selected,
                    self.label,
                )
            });
            motion::focus_ring(ui, &response, 100.into());
        }
        ChipResponse {
            toggled: selectable && response.clicked() && !scope.inner,
            removed: scope.inner,
            response,
        }
    }
}

fn remove_hover_fill(fill: Color32, ink: Color32) -> Color32 {
    let tint = mix(fill, ink, 0.14);
    if readable(tint, ink) == ink {
        return tint;
    }
    // If tinting toward the glyph would reduce contrast below AA, shade away from it.
    let opposite = if luminance(ink) > luminance(fill) {
        Color32::BLACK
    } else {
        Color32::WHITE
    };
    mix(fill, opposite, 0.14)
}

pub(super) fn readable(fill: Color32, preferred: Color32) -> Color32 {
    let light = luminance(fill);
    let ink = luminance(preferred);
    if (light.max(ink) + 0.05) / (light.min(ink) + 0.05) >= 4.5 {
        preferred
    } else if light >= 0.179 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// A semantic pill with optional selection, avatar, and remove action.
pub struct Chip<'a> {
    /// Shared semantic styling and label.
    pub badge: Badge<'a>,
    /// Leading avatar, taking precedence over the icon.
    pub avatar: Option<Avatar<'a>>,
    /// Show an interactive selected state.
    pub selectable: bool,
    /// Current caller-owned selection.
    pub selected: bool,
    /// Include a separate remove action.
    pub removable: bool,
}

/// Independent chip interactions.
pub struct ChipResponse {
    /// Main pill response.
    pub response: Response,
    /// The user activated the selectable pill.
    pub toggled: bool,
    /// The user activated removal.
    pub removed: bool,
}

impl<'a> Chip<'a> {
    /// Create an informational chip.
    pub fn new(label: &'a str) -> Self {
        Self {
            badge: Badge::new(label),
            avatar: None,
            selectable: false,
            selected: false,
            removable: false,
        }
    }

    /// Render the chip.
    pub fn show(&self, ui: &mut Ui) -> ChipResponse {
        self.badge.content(
            ui,
            self.avatar.as_ref(),
            self.selectable,
            self.selected,
            self.removable,
            ui.available_width(),
        )
    }
}
