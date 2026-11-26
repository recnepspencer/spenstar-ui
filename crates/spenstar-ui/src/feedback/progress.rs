use std::f32::consts::{FRAC_PI_2, TAU};

use egui::{Align, Align2, Color32, Layout, Rect, Response, Sense, Shape, Stroke, Ui, pos2, vec2};

use crate::{Theme, theme::mix, typography::Type};

use super::{Size, Tone, unit};

/// Horizontal determinate or indeterminate progress.
pub struct ProgressBar<'a> {
    value: Option<f32>,
    label: Option<&'a str>,
    percentage: bool,
    tone: Tone,
    size: Size,
}

impl<'a> ProgressBar<'a> {
    /// Create a determinate progress bar. NaN becomes zero.
    pub fn new(fraction: f32) -> Self {
        Self {
            value: Some(unit(fraction)),
            label: None,
            percentage: false,
            tone: Tone::Accent,
            size: Size::Md,
        }
    }

    /// Create a sliding activity indicator.
    pub fn indeterminate() -> Self {
        Self {
            value: None,
            ..Self::new(0.)
        }
    }

    /// Add a label above the track.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Show a percentage for determinate progress.
    pub fn percentage(mut self, show: bool) -> Self {
        self.percentage = show;
        self
    }

    /// Apply a semantic tone.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Choose track thickness.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Return the clamped value, if determinate.
    pub fn fraction(&self) -> Option<f32> {
        self.value
    }

    /// Render the bar at the available width.
    pub fn show(self, ui: &mut Ui) -> Response {
        let c = Theme::current(ui.ctx()).colors;
        let caption = self.value.filter(|_| self.percentage).map(percent);
        if self.label.is_some() || caption.is_some() {
            captions(ui, self.label, caption.as_deref());
        }
        let height = match self.size {
            Size::Sm => 4.,
            Size::Md => 7.,
            Size::Lg => 10.,
        };
        track(ui, height, self.value, self.tone.palette(c).base)
    }
}

/// Determinate vector-painted circular progress.
pub struct ProgressCircle {
    value: f32,
    diameter: f32,
    percentage: bool,
    tone: Tone,
}

impl ProgressCircle {
    /// Create a ring. Values are clamped, with NaN treated as zero.
    pub fn new(fraction: f32) -> Self {
        Self {
            value: unit(fraction),
            diameter: 56.,
            percentage: false,
            tone: Tone::Accent,
        }
    }

    /// Set the outer diameter in points.
    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = if diameter.is_finite() {
            diameter.clamp(12., 512.)
        } else {
            56.
        };
        self
    }

    /// Show the value inside the circle.
    pub fn percentage(mut self, show: bool) -> Self {
        self.percentage = show;
        self
    }

    /// Apply a semantic tone.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Return the clamped value.
    pub fn fraction(&self) -> f32 {
        self.value
    }

    /// Render the progress ring.
    pub fn show(self, ui: &mut Ui) -> Response {
        let c = Theme::current(ui.ctx()).colors;
        let (rect, response) =
            ui.allocate_exact_size(vec2(self.diameter, self.diameter), Sense::hover());
        if ui.is_rect_visible(rect) {
            let stroke = (self.diameter * 0.075).clamp(2., 8.);
            let radius = (self.diameter - stroke) * 0.5;
            let center = rect.center();
            ui.painter()
                .circle_stroke(center, radius, Stroke::new(stroke, c.surface_secondary));
            arc(
                ui,
                center,
                radius,
                -FRAC_PI_2,
                self.value * TAU,
                Stroke::new(stroke, self.tone.palette(c).base),
            );
            if self.percentage {
                let mut font = Type::Badge.font();
                font.size = font.size.min(self.diameter / 3.4);
                ui.painter().text(
                    center,
                    Align2::CENTER_CENTER,
                    percent(self.value),
                    font,
                    c.text_primary,
                );
            }
        }
        response
    }
}

/// Gauge with interpolated success, caution, and danger colors.
pub struct Meter<'a> {
    value: f32,
    warning: f32,
    danger: f32,
    label: Option<&'a str>,
    caption: Option<&'a str>,
}

impl<'a> Meter<'a> {
    /// Create a gauge with anchors at 70% and 95%.
    pub fn new(fraction: f32) -> Self {
        Self {
            value: unit(fraction),
            warning: 0.7,
            danger: 0.95,
            label: None,
            caption: None,
        }
    }

    /// Configure warning and danger anchors. Reversed anchors are sorted.
    pub fn thresholds(mut self, warning: f32, danger: f32) -> Self {
        let (warning, danger) = (unit(warning), unit(danger));
        self.warning = warning.min(danger);
        self.danger = warning.max(danger);
        self
    }

    /// Set the label and optional custom value caption.
    pub fn labels(mut self, label: &'a str, caption: Option<&'a str>) -> Self {
        self.label = Some(label);
        self.caption = caption;
        self
    }

    /// Return the clamped value.
    pub fn fraction(&self) -> f32 {
        self.value
    }

    /// Resolve the interpolated indicator color.
    pub fn color(&self, theme: Theme) -> Color32 {
        let c = theme.colors;
        if self.value >= self.danger {
            c.danger.base
        } else if self.value >= self.warning {
            mix(
                c.warning.base,
                c.danger.base,
                (self.value - self.warning) / (self.danger - self.warning),
            )
        } else {
            let start = self.warning * 0.65;
            mix(
                c.success.base,
                c.warning.base,
                unit((self.value - start) / (self.warning - start)),
            )
        }
    }

    /// Render the gauge at the available width.
    pub fn show(self, ui: &mut Ui) -> Response {
        let label = percent(self.value);
        captions(ui, self.label, Some(self.caption.unwrap_or(&label)));
        track(
            ui,
            7.,
            Some(self.value),
            self.color(Theme::current(ui.ctx())),
        )
    }
}

/// Rotating vector arc for an operation without a known duration.
pub struct Spinner {
    size: Size,
    tone: Tone,
}

impl Spinner {
    /// Create an accent spinner of the selected size.
    pub fn new(size: Size) -> Self {
        Self {
            size,
            tone: Tone::Accent,
        }
    }

    /// Apply a semantic tone.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Paint and schedule animation only when inside the clip rectangle.
    pub fn show(self, ui: &mut Ui) -> Response {
        let (size, stroke) = match self.size {
            Size::Sm => (12., 1.5),
            Size::Md => (20., 2.),
            Size::Lg => (28., 2.5),
        };
        let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
        if ui.is_rect_visible(rect) {
            let color = self.tone.palette(Theme::current(ui.ctx()).colors).base;
            let start = ui.input(|input| (input.time % 0.85 / 0.85) as f32 * TAU);
            arc(
                ui,
                rect.center(),
                (size - stroke) * 0.5,
                start,
                TAU * 0.72,
                Stroke::new(stroke, color),
            );
            ui.ctx().request_repaint();
        }
        response
    }
}

fn arc(ui: &Ui, center: egui::Pos2, radius: f32, start: f32, sweep: f32, stroke: Stroke) {
    if sweep <= 0. {
        return;
    }
    let segments = (sweep * radius.sqrt() * 2.).ceil().max(2.) as usize;
    let points: Vec<_> = (0..=segments)
        .map(|i| {
            let angle = start + sweep * i as f32 / segments as f32;
            center + vec2(angle.cos(), angle.sin()) * radius
        })
        .collect();
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        ui.painter()
            .circle_filled(*first, stroke.width * 0.5, stroke.color);
        ui.painter()
            .circle_filled(*last, stroke.width * 0.5, stroke.color);
    }
    ui.painter().add(Shape::line(points, stroke));
}

fn track(ui: &mut Ui, height: f32, value: Option<f32>, color: Color32) -> Response {
    let c = Theme::current(ui.ctx()).colors;
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    if ui.is_rect_visible(rect) {
        let radius = height * 0.5;
        ui.painter().rect_filled(rect, radius, c.surface_secondary);
        let (start, end) = if let Some(value) = value {
            (0., value)
        } else {
            ui.ctx().request_repaint();
            let phase = ui.input(|input| (input.time % 1.45 / 1.45) as f32);
            let start = (1. - (phase * TAU).cos()) * 0.36;
            (start, start + 0.28)
        };
        if end > start {
            let fill = Rect::from_min_max(
                pos2(rect.left() + rect.width() * start, rect.top()),
                pos2(rect.left() + rect.width() * end, rect.bottom()),
            );
            ui.painter()
                .rect_filled(fill, radius.min(fill.width() * 0.5), color);
        }
    }
    response
}

fn captions(ui: &mut Ui, label: Option<&str>, value: Option<&str>) {
    let c = Theme::current(ui.ctx()).colors;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 18.),
        Layout::right_to_left(Align::Center),
        |ui| {
            if let Some(value) = value {
                ui.label(Type::Small.text(value).color(c.text_secondary));
            }
            if let Some(label) = label {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.label(Type::Label.text(label).color(c.text_primary));
                });
            }
        },
    );
}

fn percent(value: f32) -> String {
    format!("{:.0}%", value * 100.)
}
