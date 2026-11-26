use egui::{Frame, Margin, Response, Sense, Stroke, Ui, Vec2, vec2};

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
        let width = ui.available_width();
        Frame::new()
            .fill(c.surface)
            .stroke(Stroke::new(1., c.border_subtle))
            .inner_margin(Margin::same(12))
            .corner_radius(12)
            .show(ui, |ui| {
                ui.set_width((width - 26.).max(1.));
                ui.horizontal(|ui| {
                    Self::circle(30.).show(ui);
                    ui.vertical(|ui| {
                        Self::line((width * 0.43).min(ui.available_width())).show(ui);
                        Self::line((width * 0.26).min(ui.available_width())).show(ui);
                    });
                });
                ui.add_space(3.);
                Self::line((width * 0.76).min(ui.available_width())).show(ui);
            })
            .response
    }

    /// Draw the placeholder and animate it while visible.
    pub fn show(self, ui: &mut Ui) -> Response {
        let safe = |value: f32| if value.is_finite() { value.max(0.) } else { 0. };
        let size = vec2(
            safe(self.size.x).min(ui.available_width().max(0.)),
            safe(self.size.y),
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if rect.is_positive() && ui.is_rect_visible(rect) {
            let c = Theme::current(ui.ctx()).colors;
            let radius = safe(self.radius).min(size.min_elem() * 0.5);
            ui.painter().rect_filled(rect, radius, c.surface_secondary);
            let phase = ui.input(|input| (input.time % 1.8 / 1.8) as f32);
            let angle = (phase * 2. - 1.) * std::f32::consts::PI;
            let tint = mix(
                c.surface_secondary,
                c.text_primary,
                0.06 * (1. + angle.cos()),
            );
            // Clip the moving highlight by repainting the original rounded shape.
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
        response
    }
}
