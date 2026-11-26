use egui::{Align2, Color32, Mesh, Pos2, Rect, Response, Shape, Stroke, Ui, pos2};

use super::plot::{self, Range};
use crate::Theme;

/// A named series with one sample per x-axis slot.
#[derive(Clone, Copy, Debug)]
pub struct Series<'a> {
    /// Legend and tooltip label.
    pub name: &'a str,
    /// Samples in display order.
    pub values: &'a [f32],
    /// Line, bar, and marker color.
    pub color: Color32,
}

impl<'a> Series<'a> {
    /// Create a named series.
    pub const fn new(name: &'a str, values: &'a [f32], color: Color32) -> Self {
        Self {
            name,
            values,
            color,
        }
    }
}

/// An inline series with optional gradient fill and hover tracking.
#[derive(Clone, Copy, Debug)]
pub struct Sparkline<'a> {
    /// Samples in display order.
    pub values: &'a [f32],
    /// Line color.
    pub color: Color32,
    /// Maximum width. Zero fills available space.
    pub width: f32,
    /// Logical height.
    pub height: f32,
    /// Shade between the curve and the bottom edge.
    pub fill: bool,
    /// Show sample values on hover.
    pub hover: bool,
}

impl<'a> Sparkline<'a> {
    /// Create a filled hoverable sparkline.
    pub fn new(values: &'a [f32], color: Color32) -> Self {
        Self {
            values,
            color,
            width: 0.,
            height: 32.,
            fill: true,
            hover: true,
        }
    }

    /// Render the series.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let (rect, response) = plot::canvas(ui, self.width, self.height, 32.);
        let bounds = rect.shrink2(egui::vec2(rect.width().min(6.) / 2., 3.));
        let range = Range::new(self.values.iter().map(|&x| f64::from(x)));
        let series = Series::new("", self.values, self.color);
        paint(
            ui,
            bounds,
            &range,
            series,
            self.values.len(),
            self.fill,
            false,
        );
        if self.hover
            && let Some(pointer) = response.hover_pos()
        {
            let index = nearest(bounds, pointer.x, self.values.len());
            if let Some(&value) = self.values.get(index).filter(|x| x.is_finite()) {
                dot(
                    ui,
                    point(bounds, &range, index, self.values.len(), value),
                    self.color,
                );
                plot::tooltip(
                    ui,
                    &response,
                    &format!("Sample {}", index + 1),
                    [(self.color, String::new(), plot::number(f64::from(value)))],
                );
            }
        }
        response
    }
}

/// A multi-series Cartesian line chart.
pub struct LineChart<'a> {
    /// Shared x-axis slots. Missing labels are numbered in tooltips.
    pub labels: &'a [&'a str],
    /// Series plotted on a common y scale.
    pub series: &'a [Series<'a>],
    /// Plot height including axes.
    pub height: f32,
    /// Show the series legend.
    pub legend: bool,
}

impl<'a> LineChart<'a> {
    /// Create a line chart with a legend.
    pub fn new(labels: &'a [&'a str], series: &'a [Series<'a>]) -> Self {
        Self {
            labels,
            series,
            height: 210.,
            legend: true,
        }
    }

    /// Render the chart.
    pub fn show(&self, ui: &mut Ui) -> Response {
        self.draw(ui, false)
    }

    fn draw(&self, ui: &mut Ui, fill: bool) -> Response {
        if self.legend {
            plot::legend(ui, self.series.iter().map(|s| (s.color, s.name)));
        }
        let (rect, response) = plot::canvas(ui, 0., self.height, 210.);
        let valid = self
            .series
            .iter()
            .any(|s| s.values.iter().any(|x| x.is_finite()));
        if !valid {
            plot::empty(ui, rect);
            return response;
        }
        let bounds = plot::plot_rect(rect, 49.);
        let range = Range::new(
            self.series
                .iter()
                .flat_map(|s| s.values.iter().map(|&x| f64::from(x))),
        );
        let count = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or_default()
            .max(self.labels.len());
        let range = range.with_grid();
        plot::grid(ui, bounds, &range, false);
        let step =
            ((self.labels.len() as f32 * 56. / bounds.width().max(1.)).ceil() as usize).max(1);
        for (i, label) in self.labels.iter().enumerate().step_by(step) {
            let x = x_at(bounds, i, count);
            let pivot = if i == 0 && count > 1 {
                Align2::LEFT_TOP
            } else if i + 1 == count && count > 1 {
                Align2::RIGHT_TOP
            } else {
                Align2::CENTER_TOP
            };
            plot::label(ui, pos2(x, bounds.bottom() + 7.), pivot, label);
        }
        for &series in self.series {
            paint(ui, bounds, &range, series, count, fill, true);
        }
        if let Some(pointer) = response.hover_pos().filter(|p| bounds.contains(*p)) {
            let index = nearest(bounds, pointer.x, count);
            let x = x_at(bounds, index, count);
            let c = Theme::current(ui.ctx()).colors;
            ui.painter().line_segment(
                [pos2(x, bounds.top()), pos2(x, bounds.bottom())],
                Stroke::new(1., c.text_muted),
            );
            let rows = self.series.iter().filter_map(|series| {
                let &value = series.values.get(index).filter(|x| x.is_finite())?;
                dot(ui, pos2(x, range.y(bounds, f64::from(value))), series.color);
                Some((
                    series.color,
                    series.name.to_owned(),
                    plot::number(f64::from(value)),
                ))
            });
            let title = self.labels.get(index).map_or_else(
                || format!("Sample {}", index + 1),
                |label| (*label).to_owned(),
            );
            // Collect before rendering the overlay so the dots remain in the chart layer.
            plot::tooltip(ui, &response, &title, rows.collect::<Vec<_>>());
        }
        response
    }
}

/// A multi-series line chart with gradient area fills.
pub struct AreaChart<'a>(pub LineChart<'a>);

impl<'a> AreaChart<'a> {
    /// Create an area chart with a legend.
    pub fn new(labels: &'a [&'a str], series: &'a [Series<'a>]) -> Self {
        Self(LineChart::new(labels, series))
    }

    /// Render the chart.
    pub fn show(&self, ui: &mut Ui) -> Response {
        self.0.draw(ui, true)
    }
}

fn x_at(rect: Rect, index: usize, count: usize) -> f32 {
    if count <= 1 {
        rect.center().x
    } else {
        rect.left() + rect.width() * (index as f32 / (count - 1) as f32)
    }
}

fn nearest(rect: Rect, x: f32, count: usize) -> usize {
    let fraction = ((x - rect.left()) / rect.width().max(f32::EPSILON)).clamp(0., 1.);
    (fraction * count.saturating_sub(1) as f32).round() as usize
}

fn point(rect: Rect, range: &Range, index: usize, count: usize, value: f32) -> Pos2 {
    pos2(x_at(rect, index, count), range.y(rect, f64::from(value)))
}

fn dot(ui: &Ui, p: Pos2, color: Color32) {
    ui.painter().circle(
        p,
        4.,
        color,
        Stroke::new(1.5, Theme::current(ui.ctx()).colors.surface),
    );
}

fn paint(
    ui: &Ui,
    rect: Rect,
    range: &Range,
    series: Series<'_>,
    count: usize,
    fill: bool,
    baseline_zero: bool,
) {
    let baseline = if baseline_zero {
        range.y(rect, 0.)
    } else {
        rect.bottom()
    };
    let painter = ui.painter().with_clip_rect(rect.expand(2.));
    let mut offset = 0;
    for segment in series.values.split(|x| !x.is_finite()) {
        if segment.len() == 1 {
            painter.circle_filled(
                point(rect, range, offset, count, segment[0]),
                2.5,
                series.color,
            );
        } else if segment.len() > 1 {
            let points: Vec<_> = segment
                .iter()
                .enumerate()
                .map(|(i, &value)| point(rect, range, offset + i, count, value))
                .collect();
            if fill {
                painter.add(gradient(&points, baseline, series.color));
            }
            painter.add(Shape::line(points, Stroke::new(2., series.color)));
        }
        offset += segment.len() + 1;
    }
}

fn gradient(points: &[Pos2], baseline: f32, color: Color32) -> Mesh {
    let mut mesh = Mesh::default();
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if (a.y - baseline).signum() != (b.y - baseline).signum() && a.y != b.y {
            let cross = pos2(
                egui::lerp(a.x..=b.x, (baseline - a.y) / (b.y - a.y)),
                baseline,
            );
            quad(&mut mesh, a, cross, baseline, color);
            quad(&mut mesh, cross, b, baseline, color);
        } else {
            quad(&mut mesh, a, b, baseline, color);
        }
    }
    mesh
}

fn quad(mesh: &mut Mesh, a: Pos2, b: Pos2, baseline: f32, color: Color32) {
    let start = mesh.vertices.len() as u32;
    mesh.colored_vertex(a, color.gamma_multiply(0.26));
    mesh.colored_vertex(b, color.gamma_multiply(0.26));
    mesh.colored_vertex(pos2(b.x, baseline), Color32::TRANSPARENT);
    mesh.colored_vertex(pos2(a.x, baseline), Color32::TRANSPARENT);
    mesh.add_triangle(start, start + 1, start + 2);
    mesh.add_triangle(start, start + 2, start + 3);
}
