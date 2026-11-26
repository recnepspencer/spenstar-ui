use egui::{Align2, CornerRadius, Rect, Response, Ui, pos2};

use super::{
    Series,
    plot::{self, Range},
};

/// Direction of the numeric axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarOrientation {
    /// Categories from left to right.
    #[default]
    Vertical,
    /// Categories from top to bottom.
    Horizontal,
}

/// Multi-series arrangement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarMode {
    /// Series side by side.
    #[default]
    Grouped,
    /// Stack positive and negative values on their respective sides.
    Stacked,
}

/// A grouped or stacked bar chart.
pub struct BarChart<'a> {
    /// Category labels.
    pub labels: &'a [&'a str],
    /// Named series with one value per category.
    pub series: &'a [Series<'a>],
    /// Direction of the bars.
    pub orientation: BarOrientation,
    /// Arrangement of series.
    pub mode: BarMode,
    /// Logical height.
    pub height: f32,
    /// Show series legend.
    pub legend: bool,
}

impl<'a> BarChart<'a> {
    /// Make a vertical grouped chart.
    pub fn new(labels: &'a [&'a str], series: &'a [Series<'a>]) -> Self {
        Self {
            labels,
            series,
            orientation: BarOrientation::Vertical,
            mode: BarMode::Grouped,
            height: 210.,
            legend: true,
        }
    }

    /// Paint bars, axes, and hover details.
    pub fn show(&self, ui: &mut Ui) -> Response {
        if self.legend {
            plot::legend(ui, self.series.iter().map(|s| (s.color, s.name)));
        }
        let (rect, response) = plot::canvas(ui, 0., self.height, 210.);
        let Some(count) = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .filter(|&n| n > 0)
        else {
            plot::empty(ui, rect);
            return response;
        };
        if !self
            .series
            .iter()
            .any(|s| s.values.iter().any(|x| x.is_finite()))
        {
            plot::empty(ui, rect);
            return response;
        }
        let horizontal = self.orientation == BarOrientation::Horizontal;
        let bounds = plot::plot_rect(rect, if horizontal { 108. } else { 47. });
        let count = count.max(self.labels.len());
        let stacked = self.mode == BarMode::Stacked;
        let range = if stacked {
            Range::new((0..count).flat_map(|i| {
                self.series
                    .iter()
                    .filter_map(|s| value(*s, i))
                    .fold([0., 0.], |[negative, positive], v| {
                        [negative + v.min(0.), positive + v.max(0.)]
                    })
            }))
        } else {
            Range::new(
                std::iter::once(0.).chain(
                    self.series
                        .iter()
                        .flat_map(|s| s.values.iter().map(|&x| f64::from(x))),
                ),
            )
        };
        let range = range.with_grid();
        plot::grid(ui, bounds, &range, horizontal);
        let slot = if horizontal {
            bounds.height()
        } else {
            bounds.width()
        } / count as f32;
        let center = |i| if horizontal { bounds.top() } else { bounds.left() } + (i as f32 + 0.5) * slot;
        let pointer = response.hover_pos().filter(|p| bounds.contains(*p));
        let mut hit = None;
        let label_step =
            ((if horizontal { 15. } else { 56. } / slot.max(f32::EPSILON)).ceil() as usize).max(1);
        for i in 0..count {
            let at = center(i);
            if i % label_step == 0
                && let Some(label) = self.labels.get(i)
            {
                let (pos, align) = if horizontal {
                    (pos2(bounds.left() - 8., at), Align2::RIGHT_CENTER)
                } else {
                    (pos2(at, bounds.bottom() + 7.), Align2::CENTER_TOP)
                };
                plot::label(ui, pos, align, label);
            }
            let group_width = slot * 0.66;
            let thickness = if stacked {
                group_width
            } else {
                group_width / self.series.len() as f32
            };
            let mut sums = [0., 0.];
            let mut remaining = [0, 0];
            if stacked {
                for s in self
                    .series
                    .iter()
                    .filter_map(|s| value(*s, i))
                    .filter(|&v| v != 0.)
                {
                    remaining[usize::from(s >= 0.)] += 1;
                }
            }
            for (j, &series) in self.series.iter().enumerate() {
                let Some(v) = value(series, i).filter(|&v| v != 0.) else {
                    continue;
                };
                let sign = usize::from(v >= 0.);
                let start = sums[sign];
                let end = start + v;
                let mut position = at;
                if stacked {
                    sums[sign] = end;
                    remaining[sign] -= 1;
                } else {
                    position += (j as f32 + 0.5) * thickness - group_width / 2.;
                }
                let radius = if stacked && remaining[sign] > 0 {
                    0
                } else {
                    (thickness * 0.45).min(5.) as u8
                };
                let corners = outer_corners(horizontal, v > 0., radius);
                let half = thickness * if stacked { 0.5 } else { 0.44 };
                let bar = if horizontal {
                    Rect::from_two_pos(
                        pos2(range.x(bounds, start), position - half),
                        pos2(range.x(bounds, end), position + half),
                    )
                } else {
                    Rect::from_two_pos(
                        pos2(position - half, range.y(bounds, start)),
                        pos2(position + half, range.y(bounds, end)),
                    )
                };
                let hovered = pointer.is_some_and(|p| bar.contains(p));
                if hovered {
                    hit = Some((i, series, v));
                }
                ui.painter().rect_filled(
                    bar,
                    corners,
                    if hovered {
                        series.color.gamma_multiply(0.76)
                    } else {
                        series.color
                    },
                );
            }
        }
        if let Some((index, series, v)) = hit {
            let title = self.labels.get(index).map_or_else(
                || format!("Category {}", index + 1),
                |label| (*label).to_owned(),
            );
            plot::tooltip(
                ui,
                &response,
                &title,
                [(series.color, series.name.to_owned(), plot::number(v))],
            );
        }
        response
    }
}

fn value(series: Series<'_>, index: usize) -> Option<f64> {
    series
        .values
        .get(index)
        .copied()
        .filter(|v| v.is_finite())
        .map(f64::from)
}

fn outer_corners(horizontal: bool, positive: bool, r: u8) -> CornerRadius {
    CornerRadius {
        nw: if horizontal != positive { r } else { 0 },
        ne: if positive { r } else { 0 },
        sw: if !positive { r } else { 0 },
        se: if horizontal == positive { r } else { 0 },
    }
}
