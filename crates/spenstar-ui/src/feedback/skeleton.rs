use egui::{Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::{Theme, theme::mix};

/// Animated placeholder for content that is loading.
#[must_use]
pub struct Skeleton {
    size: Vec2,
    radius: f32,
}

impl Skeleton {
    /// Placeholder for a text line.
    pub fn line(width: f32) -> Self {
        Self::rect(vec2(width, 11.))
    }

    /// Circular placeholder for an avatar or icon.
    pub fn circle(diameter: f32) -> Self {
        Self {
            size: Vec2::splat(diameter),
            radius: diameter * 0.5,
        }
    }

    /// Rectangular placeholder with soft corners.
    pub fn rect(size: Vec2) -> Self {
        Self { size, radius: 5. }
    }

    /// Draw a card with an avatar and three lines.
    pub fn card(ui: &mut Ui) -> Response {
        let c = Theme::current(ui.ctx()).colors;
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 82.), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }
        ui.painter().rect(
            rect,
            12,
            c.surface,
            Stroke::new(1., c.border_subtle),
            StrokeKind::Inside,
        );
        let content = rect.shrink2(vec2(12.0_f32.min(rect.width() * 0.25), 12.));
        let avatar = Rect::from_min_size(
            content.min,
            Vec2::splat(30.0_f32.min(content.width() * 0.22)),
        );
        let x = (avatar.right() + 9.).min(content.right());
        let line = |x, y, width| Rect::from_min_size(pos2(x, y), vec2(width, 9.));
        let length = (content.right() - x).max(0.);
        shimmer(ui, avatar, avatar.width() * 0.5);
        shimmer(ui, line(x, content.top() + 3., length * 0.51), 4.);
        shimmer(ui, line(x, content.top() + 17., length * 0.32), 4.);
        shimmer(
            ui,
            line(
                content.left(),
                content.bottom() - 12.,
                content.width() * 0.76,
            ),
            4.,
        );
        response
    }

    /// Draw the placeholder and animate it while visible.
    pub fn show(self, ui: &mut Ui) -> Response {
        let safe = |value: f32| if value.is_finite() { value.max(0.) } else { 0. };
        let size = vec2(
            safe(self.size.x).min(ui.available_width().max(0.)),
            safe(self.size.y),
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        shimmer(ui, rect, safe(self.radius).min(size.min_elem() * 0.5));
        response
    }
}

fn shimmer(ui: &Ui, rect: Rect, radius: f32) {
    if !rect.is_positive() || !ui.is_rect_visible(rect) {
        return;
    }
    let c = Theme::current(ui.ctx()).colors;
    ui.painter().rect_filled(rect, radius, c.surface_secondary);
    let phase = ui.input(|input| (input.time % 1.8 / 1.8) as f32);
    let angle = (phase * 2. - 1.) * std::f32::consts::PI;
    let tint = mix(
        c.surface_secondary,
        c.text_primary,
        0.06 * (1. + angle.cos()),
    );
    let x = rect.left() + rect.width() * phase;
    for band in 0..5 {
        let spread = rect.width() * (0.20 - band as f32 * 0.034);
        let mut clip = rect;
        clip.min.x = (x - spread).max(rect.left());
        clip.max.x = (x + spread).min(rect.right());
        ui.painter().with_clip_rect(clip).rect_filled(
            rect,
            radius,
            mix(c.surface_secondary, tint, (band + 1) as f32 / 5.),
        );
    }
    ui.ctx().request_repaint();
}
