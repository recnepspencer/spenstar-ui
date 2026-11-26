use egui::{Align2, Color32, Pos2, Rect, Response, Sense, Stroke, Tooltip, Ui, pos2, vec2};

use crate::{Theme, typography::Type};

pub(super) struct Range {
    min: f64,
    max: f64,
    step: f64,
}

impl Range {
    pub fn new(values: impl Iterator<Item = f64>) -> Self {
        let (mut min, mut max) = values
            .filter(|x| x.is_finite())
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), x| {
                (min.min(x), max.max(x))
            });
        if !min.is_finite() {
            return Self {
                min: 0.,
                max: 1.,
                step: 0.25,
            };
        }
        if min == max {
            let pad = (min.abs() * 0.15).max(1.);
            min -= pad;
            max += pad;
        }
        Self { min, max, step: 0. }
    }

    pub fn fraction(&self, value: f64) -> f32 {
        ((value - self.min) / (self.max - self.min)).clamp(0., 1.) as f32
    }

    pub fn y(&self, rect: Rect, value: f64) -> f32 {
        rect.bottom() - rect.height() * self.fraction(value)
    }

    pub fn x(&self, rect: Rect, value: f64) -> f32 {
        rect.left() + rect.width() * self.fraction(value)
    }

    pub fn with_grid(self) -> Self {
        let step = self.grid_step();
        Self {
            min: (self.min / step).floor() * step,
            max: (self.max / step).ceil() * step,
            step,
        }
    }

    fn grid_step(&self) -> f64 {
        let rough = (self.max - self.min) / 4.;
        let magnitude = 10_f64.powf(rough.log10().floor());
        let scale = match rough / magnitude {
            x if x < 1.5 => 1.,
            x if x < 2.25 => 2.,
            x if x < 3.75 => 2.5,
            x if x < 7.5 => 5.,
            _ => 10.,
        };
        scale * magnitude
    }
}

pub(super) fn canvas(ui: &mut Ui, width: f32, height: f32, default: f32) -> (Rect, Response) {
    let available = ui.available_width();
    let available = if available.is_finite() {
        available.max(1.)
    } else {
        320.
    };
    let width = if width.is_finite() && width > 0. {
        width.min(available)
    } else {
        available
    };
    let height = if height.is_finite() && height > 0. {
        height.clamp(8., 2048.)
    } else {
        default
    };
    ui.allocate_exact_size(vec2(width, height), Sense::hover())
}

pub(super) fn plot_rect(rect: Rect, left: f32) -> Rect {
    let left = left.min(rect.width() * 0.35);
    let bottom = 23_f32.min(rect.height() * 0.35);
    Rect::from_min_max(
        rect.left_top() + vec2(left, 8_f32.min(rect.height() * 0.25)),
        rect.right_bottom() - vec2(9_f32.min(rect.width() * 0.15), bottom),
    )
}

pub(super) fn label(ui: &Ui, position: Pos2, pivot: Align2, text: &str) {
    ui.painter().text(
        position,
        pivot,
        text,
        Type::Caption.font(),
        Theme::current(ui.ctx()).colors.text_muted,
    );
}

pub(super) fn number(x: f64) -> String {
    let abs = x.abs();
    if abs >= 1e9 || (abs != 0. && abs < 0.01) {
        format!("{x:.1e}")
    } else if abs >= 1e6 {
        abbreviate(x / 1e6, "M")
    } else if abs >= 1e3 {
        abbreviate(x / 1e3, "k")
    } else if x.fract() == 0. || abs >= 100. {
        format!("{x:.0}")
    } else {
        format!("{x:.1}")
    }
}

fn abbreviate(value: f64, unit: &str) -> String {
    let rounded = (value * 10.).round() / 10.;
    if rounded.fract() == 0. {
        format!("{rounded:.0}{unit}")
    } else {
        format!("{rounded:.1}{unit}")
    }
}

pub(super) fn grid(ui: &Ui, plot: Rect, range: &Range, horizontal: bool) {
    let c = Theme::current(ui.ctx()).colors;
    let step = range.step;
    let first = (range.min / step).round() as i64;
    let last = (range.max / step).round() as i64;
    for tick in first..=last {
        let value = tick as f64 * step;
        let (line, tick, align) = if horizontal {
            let x = range.x(plot, value);
            (
                [pos2(x, plot.top()), pos2(x, plot.bottom())],
                pos2(x, plot.bottom() + 7.),
                Align2::CENTER_TOP,
            )
        } else {
            let y = range.y(plot, value);
            (
                [pos2(plot.left(), y), pos2(plot.right(), y)],
                pos2(plot.left() - 8., y),
                Align2::RIGHT_CENTER,
            )
        };
        ui.painter().line_segment(line, Stroke::new(1., c.divider));
        label(ui, tick, align, &number(value));
    }
}

pub(super) fn legend(ui: &mut Ui, entries: impl IntoIterator<Item = (Color32, impl AsRef<str>)>) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 13.;
        for (color, title) in entries {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.;
                marker(ui, color);
                ui.label(
                    Type::Small
                        .text(title.as_ref())
                        .color(Theme::current(ui.ctx()).colors.text_secondary),
                );
            });
        }
    });
}

pub(super) fn marker(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(8., 12.), Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.5, color);
}

pub(super) fn tooltip(
    ui: &Ui,
    response: &Response,
    title: &str,
    entries: impl IntoIterator<Item = (Color32, String, String)>,
) {
    if !response.hovered() {
        return;
    }
    let c = Theme::current(ui.ctx()).colors;
    Tooltip::always_open(
        ui.ctx().clone(),
        response.layer_id,
        response.id.with("chart-tooltip"),
        egui::PopupAnchor::Pointer,
    )
    .gap(10.)
    .show(|ui| {
        ui.set_min_width(100.);
        ui.label(Type::Label.text(title).color(c.text_primary));
        for (color, name, value) in entries {
            ui.horizontal(|ui| {
                marker(ui, color);
                if !name.is_empty() {
                    ui.label(Type::Small.text(name).color(c.text_secondary));
                }
                ui.label(Type::Kbd.text(value).color(c.text_primary));
            });
        }
    });
}

pub(super) fn empty(ui: &Ui, rect: Rect) {
    label(ui, rect.center(), Align2::CENTER_CENTER, "No data");
}
