use egui::{
    Align2, Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, TextureId, Ui, pos2,
    vec2,
};

use super::{Badge, badge::readable};
use crate::{Theme, controls::Tone, icon::IconSource, typography::Type};

/// Availability indicator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarStatus {
    /// Available.
    Online,
    /// Occupied.
    Busy,
    /// Temporarily away.
    Away,
    /// Disconnected.
    Offline,
}

/// A named avatar with initials, a vector, or a texture.
#[derive(Clone, Debug)]
pub struct Avatar<'a> {
    /// Full name used for initials and the accessible label.
    pub name: &'a str,
    /// Pastel background family.
    pub tone: Tone,
    /// Logical extent.
    pub size: u8,
    /// Use a rounded square.
    pub square: bool,
    /// Optional vector artwork.
    pub icon: Option<IconSource>,
    /// Optional square image, taking precedence over the vector.
    pub image: Option<TextureId>,
    /// Optional availability indicator.
    pub status: Option<AvatarStatus>,
}

impl<'a> Avatar<'a> {
    /// Create a neutral circular avatar.
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            tone: Tone::Neutral,
            size: 32,
            square: false,
            icon: None,
            image: None,
            status: None,
        }
    }

    /// Render the avatar and name tooltip.
    pub fn show(&self, ui: &mut Ui) -> Response {
        self.show_sized(ui, f32::from(self.size.clamp(12, 128)))
    }

    pub(super) fn show_sized(&self, ui: &mut Ui, size: f32) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
        self.paint(ui, rect, None);
        self.describe(response)
    }

    fn describe(&self, response: Response) -> Response {
        let status = match self.status {
            Some(AvatarStatus::Online) => " · Online",
            Some(AvatarStatus::Busy) => " · Busy",
            Some(AvatarStatus::Away) => " · Away",
            Some(AvatarStatus::Offline) => " · Offline",
            None => "",
        };
        let label = format!("{}{status}", self.name);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Image, response.enabled(), &label)
        });
        response.on_hover_text(label)
    }

    fn paint(&self, ui: &Ui, rect: Rect, ring: Option<Color32>) {
        if !ui.is_rect_visible(rect) {
            return;
        }
        let c = Theme::current(ui.ctx()).colors;
        let size = rect.width();
        let radius = CornerRadius::same(if self.square {
            (size * 0.24) as u8
        } else {
            (size / 2.).ceil() as u8
        });
        let palette = self.tone.palette(c);
        let fill = palette.soft_hover;
        let ink = readable(fill, palette.ring);
        let p = ui.painter();
        p.rect_filled(rect, radius, fill);
        if let Some(image) = self.image {
            p.add(
                egui::epaint::RectShape::filled(rect, radius, Color32::WHITE)
                    .with_texture(image, Rect::from_min_max(pos2(0., 0.), pos2(1., 1.))),
            );
        } else if let Some(icon) = &self.icon {
            let _ = icon.paint(ui, rect.shrink(size * 0.24), ink);
        } else {
            let initials: String = self
                .name
                .split_whitespace()
                .filter_map(|part| part.chars().next())
                .take(2)
                .flat_map(char::to_uppercase)
                .collect();
            let mut font = Type::Brand.font();
            font.size = (size * 0.38).clamp(8., 34.);
            p.text(rect.center(), Align2::CENTER_CENTER, initials, font, ink);
        }
        if let Some(ring) = ring {
            p.rect_stroke(rect, radius, Stroke::new(2., ring), StrokeKind::Outside);
        }
        if let Some(status) = self.status {
            let color = match status {
                AvatarStatus::Online => c.success.base,
                AvatarStatus::Busy => c.danger.base,
                AvatarStatus::Away => c.warning.base,
                AvatarStatus::Offline => c.text_muted,
            };
            let dot = (size * 0.125).clamp(2., 7.);
            let center = rect.right_bottom() - vec2(dot, dot);
            p.circle(
                center,
                dot,
                color,
                Stroke::new(1.5, ring.unwrap_or(c.surface)),
            );
        }
    }
}

/// Overlapping avatars with a surface-matched outline and overflow count.
pub struct AvatarGroup<'a> {
    /// Avatars in display order.
    pub avatars: &'a [Avatar<'a>],
    /// Maximum number of visible avatars.
    pub limit: usize,
    /// Shared logical extent for the group.
    pub size: u8,
    /// Override the cutout color when placed on a different surface.
    pub surface: Option<Color32>,
}

impl<'a> AvatarGroup<'a> {
    /// Create a group with at most four visible avatars.
    pub fn new(avatars: &'a [Avatar<'a>]) -> Self {
        Self {
            avatars,
            limit: 4,
            size: 32,
            surface: None,
        }
    }

    /// Render the overlapping group.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let size = f32::from(self.size.clamp(12, 128));
        let shown = self.limit.min(self.avatars.len());
        let cutout = self
            .surface
            .unwrap_or_else(|| Theme::current(ui.ctx()).colors.surface);
        ui.horizontal(|ui| {
            if shown > 0 {
                let step = size * 0.73;
                let (bounds, response) = ui.allocate_exact_size(
                    vec2(size + step * shown.saturating_sub(1) as f32 + 4., size + 4.),
                    Sense::hover(),
                );
                for (index, avatar) in self.avatars[..shown].iter().enumerate() {
                    let rect = Rect::from_min_size(
                        bounds.min + vec2(2. + index as f32 * step, 2.),
                        vec2(size, size),
                    );
                    avatar.paint(ui, rect, Some(cutout));
                    let area = if index + 1 == shown {
                        rect
                    } else {
                        Rect::from_min_size(rect.min, vec2(step, size))
                    };
                    avatar.describe(ui.interact(area, response.id.with(index), Sense::hover()));
                }
            }
            if shown < self.avatars.len() {
                Badge::new(&format!("+{}", self.avatars.len() - shown)).show(ui);
            }
        })
        .response
    }
}
