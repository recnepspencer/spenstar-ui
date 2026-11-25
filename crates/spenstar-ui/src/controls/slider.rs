use std::ops::RangeInclusive;

use egui::{Key, Rect, Response, Sense, Stroke, Ui, WidgetInfo, pos2, vec2};

use crate::{Theme, form::step_keys, motion, theme::Radius, typography::Type};

/// Appearance and interaction options shared by both sliders.
#[derive(Clone, Copy, Debug)]
pub struct SliderOptions<'a> {
    /// Accessible name.
    pub label: &'a str,
    /// Positive step. Zero uses continuous pointer input.
    pub step: f64,
    /// Number of evenly spaced tick marks, capped at 100.
    pub ticks: usize,
    /// Show the value over an active thumb.
    pub value_badge: bool,
    /// Disable interaction.
    pub disabled: bool,
}

impl Default for SliderOptions<'_> {
    fn default() -> Self {
        Self {
            label: "Value",
            step: 0.,
            ticks: 0,
            value_badge: true,
            disabled: false,
        }
    }
}

/// A single value on a horizontal track.
pub struct Slider<'a> {
    bounds: RangeInclusive<f64>,
    /// Shared appearance and input settings.
    pub options: SliderOptions<'a>,
}

impl Slider<'_> {
    /// Create a slider with inclusive bounds.
    pub fn new(bounds: RangeInclusive<f64>) -> Self {
        Self {
            bounds,
            options: SliderOptions::default(),
        }
    }

    /// Render the slider and update its value.
    pub fn show(self, ui: &mut Ui, value: &mut f64) -> Response {
        let mut values = [*value];
        let response = draw(ui, self.bounds, self.options, &mut values);
        *value = values[0];
        response
    }
}

/// A horizontal range with independently focusable, non-crossing thumbs.
pub struct RangeSlider<'a> {
    bounds: RangeInclusive<f64>,
    /// Shared appearance and input settings.
    pub options: SliderOptions<'a>,
}

impl RangeSlider<'_> {
    /// Create a range slider with inclusive bounds.
    pub fn new(bounds: RangeInclusive<f64>) -> Self {
        Self {
            bounds,
            options: SliderOptions::default(),
        }
    }

    /// Render the track and update its inclusive range.
    pub fn show(self, ui: &mut Ui, value: &mut RangeInclusive<f64>) -> Response {
        let mut values = [*value.start(), *value.end()];
        let response = draw(ui, self.bounds, self.options, &mut values);
        *value = values[0]..=values[1];
        response
    }
}

struct Scale {
    min: f64,
    max: f64,
    step: f64,
}

impl Scale {
    fn fraction(&self, value: f64) -> f32 {
        if self.min == self.max {
            return 0.;
        }
        let span = self.max - self.min;
        if span.is_finite() {
            ((value - self.min) / span) as f32
        } else {
            ((value * 0.5 - self.min * 0.5) / (self.max * 0.5 - self.min * 0.5)) as f32
        }
    }

    fn at(&self, t: f64) -> f64 {
        self.min * (1. - t) + self.max * t
    }

    fn stepped(&self, value: f64, steps: f64) -> f64 {
        if self.step == 0. {
            let increment = self.max * 0.01 - self.min * 0.01;
            return value + steps * increment;
        }
        let offset = (value - self.min) / self.step;
        if !offset.is_finite() {
            return value + steps * self.step;
        }
        // Start from the neighboring grid interval when a bound cuts a step short.
        let rounded = offset.round();
        let grid = if (offset - rounded).abs() <= offset.abs().max(1.) * f64::EPSILON * 4. {
            rounded
        } else if steps > 0. {
            offset.floor()
        } else {
            offset.ceil()
        };
        (grid + steps).mul_add(self.step, self.min)
    }

    fn snap(&self, value: f64) -> f64 {
        if self.step <= 0. || value <= self.min || value >= self.max {
            return value.clamp(self.min, self.max);
        }
        let steps = (value - self.min) / self.step;
        if steps.is_finite() {
            steps
                .round()
                .mul_add(self.step, self.min)
                .clamp(self.min, self.max)
        } else {
            value
        }
    }
}

fn draw<const N: usize>(
    ui: &mut Ui,
    bounds: RangeInclusive<f64>,
    options: SliderOptions<'_>,
    values: &mut [f64; N],
) -> Response {
    ui.scope(|ui| {
        if options.disabled {
            ui.disable();
        }
        let original = *values;
        let a = if bounds.start().is_finite() {
            *bounds.start()
        } else {
            0.
        };
        let b = if bounds.end().is_finite() {
            *bounds.end()
        } else {
            a
        };
        let scale = Scale {
            min: a.min(b),
            max: a.max(b),
            step: if options.step.is_finite() {
                options.step.max(0.)
            } else {
                0.
            },
        };
        if ui.is_enabled() {
            for value in &mut *values {
                *value = scale.snap(if value.is_nan() { scale.min } else { *value });
            }
            if N == 2 && values[0] > values[1] {
                values.swap(0, 1);
            }
        }
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(ui.available_width().max(28.), 30.), Sense::hover());
        let track = Rect::from_min_max(
            pos2(rect.left() + 10., rect.center().y - 3.),
            pos2(rect.right() - 10., rect.center().y + 3.),
        );
        let point = |value: f64| {
            let fraction = if value.is_nan() {
                0.
            } else {
                scale.fraction(value.clamp(scale.min, scale.max))
            };
            pos2(egui::lerp(track.x_range(), fraction), track.center().y)
        };
        let split = if N == 2 {
            ((point(values[0]).x + point(values[1]).x) * 0.5)
                .clamp(rect.left() + 2., rect.right() - 2.)
        } else {
            rect.right()
        };
        let thumbs: [Response; N] = std::array::from_fn(|i| {
            let mut hit = rect;
            if i == 0 {
                hit.max.x = split;
            } else {
                hit.min.x = split;
            }
            ui.interact(hit, response.id.with(i), Sense::click_and_drag())
        });
        for (i, thumb) in thumbs.iter().enumerate() {
            crate::input::lock_arrows(thumb, true, false);
            let steps = step_keys(thumb, Key::ArrowRight, Key::ArrowLeft);
            let last = values[i];
            let mut next = last;
            if thumb.enabled()
                && (thumb.is_pointer_button_down_on()
                    || thumb.clicked()
                    || thumb.dragged()
                    || thumb.drag_stopped())
                && let Some(pos) = thumb.interact_pointer_pos()
            {
                if !thumb.has_focus() {
                    thumb.request_focus();
                }
                let t = ((pos.x - track.left()) / track.width()).clamp(0., 1.);
                next = scale.at(f64::from(t));
            }
            if steps != 0. {
                next = scale.stepped(next, steps);
            }
            if thumb.enabled() && next != last && !next.is_nan() {
                let min = if i > 0 { values[i - 1] } else { scale.min };
                let max = if i + 1 < N { values[i + 1] } else { scale.max };
                values[i] = scale.snap(next).clamp(min, max);
            }
        }
        let c = Theme::current(ui.ctx()).colors;
        ui.painter().rect_filled(track, 3, c.border);
        let left = if N == 1 {
            track.left()
        } else {
            point(values[0]).x
        };
        let right = point(values[N - 1]).x;
        ui.painter().rect_filled(
            Rect::from_x_y_ranges(left.min(right)..=left.max(right), track.y_range()),
            3,
            c.accent.base,
        );
        let ticks = options.ticks.min(100);
        for i in 0..ticks {
            let x = egui::lerp(track.x_range(), i as f32 / (ticks - 1).max(1) as f32);
            ui.painter()
                .circle_filled(pos2(x, track.bottom() + 5.), 1., c.text_muted);
        }
        for (i, thumb) in thumbs.into_iter().enumerate() {
            let label = match (N, i) {
                (1, _) => options.label.to_owned(),
                (_, 0) => format!("{} start", options.label),
                _ => format!("{} end", options.label),
            };
            thumb.widget_info(|| WidgetInfo::slider(thumb.enabled(), values[i], &label));
            let center = point(values[i]);
            ui.ctx().accesskit_node_builder(thumb.id, |node| {
                node.set_min_numeric_value(scale.min);
                node.set_max_numeric_value(scale.max);
                node.set_bounds(egui::accesskit::Rect {
                    x0: f64::from(center.x - 8.),
                    x1: f64::from(center.x + 8.),
                    y0: f64::from(center.y - 8.),
                    y1: f64::from(center.y + 8.),
                });
            });
            let active = thumb.is_pointer_button_down_on() || thumb.dragged();
            let hovering = ui.rect_contains_pointer(Rect::from_center_size(center, vec2(22., 24.)))
                && ui.is_enabled();
            if active || hovering {
                ui.painter().circle_filled(center, 11., c.accent.soft_hover);
            }
            ui.painter().circle(
                center,
                8.,
                c.surface_elevated,
                Stroke::new(2., c.accent.ring),
            );
            let mut focus = thumb.clone();
            focus.rect = Rect::from_center_size(center, vec2(16., 16.));
            motion::focus_ring(ui, &focus, Radius::Full.for_size(focus.rect.size()));
            if options.value_badge && active {
                let galley = ui.painter().layout_no_wrap(
                    format!("{:.4}", values[i])
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_owned(),
                    Type::Caption.font(),
                    c.accent.foreground,
                );
                let badge =
                    Rect::from_center_size(center - vec2(0., 23.), galley.size() + vec2(10., 6.));
                ui.painter().rect_filled(badge, 5, c.accent.base);
                ui.painter().galley(
                    badge.center() - galley.size() * 0.5,
                    galley,
                    c.accent.foreground,
                );
            }
            response = response.union(thumb);
        }
        if original
            .iter()
            .zip(values.iter())
            .any(|(a, b)| a.to_bits() != b.to_bits())
        {
            response.mark_changed();
        }
        response
    })
    .inner
}
