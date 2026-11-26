use std::f32::consts::{FRAC_PI_2, TAU};

use egui::{Align2, Color32, Mesh, Pos2, Rect, Response, Shape, Stroke, Ui, pos2, vec2};

use super::plot;
use crate::{Theme, theme::mix, typography::Type};

/// One part of a donut total.
#[derive(Clone, Copy, Debug)]
pub struct DonutSlice<'a> {
    /// Category name.
    pub label: &'a str,
    /// A strictly positive, finite contribution.
    pub value: f32,
    /// Segment color.
    pub color: Color32,
}

/// A segmented ring with centered sum.
pub struct DonutChart<'a> {
    /// Contributions to the total.
    pub slices: &'a [DonutSlice<'a>],
    /// Caption beneath the total.
    pub label: &'a str,
    /// Logical ring height.
    pub height: f32,
    /// Show each percentage in a legend.
    pub legend: bool,
}

impl<'a> DonutChart<'a> {
    /// Make a donut chart with a percentage legend.
    pub fn new(slices: &'a [DonutSlice<'a>], label: &'a str) -> Self {
        Self {
            slices,
            label,
            height: 174.,
            legend: true,
        }
    }

    /// Paint the ring and return its canvas response.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let c = Theme::current(ui.ctx()).colors;
        let valid = |slice: &&DonutSlice<'_>| slice.value.is_finite() && slice.value > 0.;
        let total: f64 = self
            .slices
            .iter()
            .filter(valid)
            .map(|s| f64::from(s.value))
            .sum();
        let (rect, response) = plot::canvas(ui, 0., self.height, 174.);
        let center = rect.center();
        let outer = (rect.width().min(rect.height()) / 2. - 5.).max(0.);
        let inner = outer * 0.76;
        let mut start = 0.;
        let hover = response
            .hover_pos()
            .map(|pointer| pointer - center)
            .filter(|delta| (inner..=outer + 3.).contains(&delta.length()))
            .map(|delta| (delta.angle() + FRAC_PI_2).rem_euclid(TAU));
        let mut hit = None;
        let count = self.slices.iter().filter(valid).count();
        for slice in self.slices.iter().filter(valid) {
            let end = start + (f64::from(slice.value) / total) as f32 * TAU;
            let gap = if count > 1 {
                ((end - start) * 0.2).min(0.025)
            } else {
                0.
            };
            if end > start {
                let active = hover.is_some_and(|angle| (start + gap..end - gap).contains(&angle));
                if active {
                    hit = Some(slice);
                }
                let ring = if active { outer + 2. } else { outer };
                ui.painter().add(segment(
                    center,
                    [inner, ring],
                    [start + gap, end - gap],
                    slice.color,
                    1. / ui.ctx().pixels_per_point(),
                ));
            }
            start = end;
        }
        if total == 0. && outer > 0. {
            ui.painter().circle_stroke(
                center,
                (inner + outer) / 2.,
                Stroke::new(outer - inner, c.surface_tertiary),
            );
        }
        if outer >= 30. {
            ui.painter().text(
                center - vec2(0., 7.),
                Align2::CENTER_CENTER,
                plot::number(total),
                Type::Metric.font(),
                c.text_primary,
            );
            plot::label(
                ui,
                center + vec2(0., 12.),
                Align2::CENTER_CENTER,
                self.label,
            );
        }
        if self.legend && total > 0. {
            ui.vertical_centered(|ui| {
                plot::legend(
                    ui,
                    self.slices.iter().filter(valid).map(|slice| {
                        (
                            slice.color,
                            format!(
                                "{} {:.1}%",
                                slice.label,
                                100. * f64::from(slice.value) / total
                            ),
                        )
                    }),
                )
            });
        }
        if let Some(slice) = hit {
            plot::tooltip(
                ui,
                &response,
                slice.label,
                [(
                    slice.color,
                    self.label.to_owned(),
                    format!(
                        "{} · {:.1}%",
                        plot::number(f64::from(slice.value)),
                        100. * f64::from(slice.value) / total
                    ),
                )],
            );
        }
        response
    }
}

fn segment(
    center: Pos2,
    [inner, outer]: [f32; 2],
    [start, end]: [f32; 2],
    color: Color32,
    feather: f32,
) -> Mesh {
    let mut mesh = Mesh::default();
    let steps = ((end - start) * outer / 2.5).ceil().clamp(2., 512.) as u32;
    for i in 0..=steps {
        let angle = egui::lerp(start..=end, i as f32 / steps as f32) - FRAC_PI_2;
        let ray = vec2(angle.cos(), angle.sin());
        let faded_edge = end - start < TAU - 0.001 && (i == 0 || i == steps);
        let ink = if faded_edge {
            Color32::TRANSPARENT
        } else {
            color
        };
        for (radius, color) in [
            ((inner - feather).max(0.), Color32::TRANSPARENT),
            (inner, ink),
            (outer, ink),
            (outer + feather, Color32::TRANSPARENT),
        ] {
            mesh.colored_vertex(center + ray * radius, color);
        }
        if i > 0 {
            for j in 0..3 {
                let v = (i - 1) * 4 + j;
                mesh.add_triangle(v, v + 4, v + 1);
                mesh.add_triangle(v + 1, v + 4, v + 5);
            }
        }
    }
    mesh
}

/// One labeled step of a conversion funnel.
#[derive(Clone, Copy, Debug)]
pub struct FunnelStage<'a> {
    /// Stage name.
    pub label: &'a str,
    /// Nonnegative, finite stage count.
    pub value: f32,
}

/// Stepped stages with counts and percentages of the first stage.
pub struct FunnelChart<'a> {
    /// Stages in conversion order.
    pub stages: &'a [FunnelStage<'a>],
    /// Starting segment color.
    pub color: Color32,
    /// Logical height.
    pub height: f32,
}

impl<'a> FunnelChart<'a> {
    /// Create a funnel.
    pub fn new(stages: &'a [FunnelStage<'a>], color: Color32) -> Self {
        Self {
            stages,
            color,
            height: 192.,
        }
    }

    /// Paint the stages and return the canvas response.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let c = Theme::current(ui.ctx()).colors;
        let (rect, response) = plot::canvas(ui, 0., self.height, 192.);
        if self.stages.is_empty() {
            plot::empty(ui, rect);
            return response;
        }
        let value = |stage: &FunnelStage<'_>| {
            if stage.value.is_finite() {
                f64::from(stage.value.max(0.))
            } else {
                0.
            }
        };
        let max = self.stages.iter().map(value).fold(0., f64::max);
        let first = self.stages.first().map_or(0., value);
        let step = rect.height() / self.stages.len() as f32;
        let (diagram, label) = rect.split_left_right_at_x(rect.left() + rect.width() * 0.52);
        let center = diagram.center().x;
        let width = |stage: &FunnelStage<'_>| {
            let ratio = if max > 0. {
                (value(stage) / max) as f32
            } else {
                0.
            };
            diagram.width() * (0.14 + 0.34 * ratio.sqrt())
        };
        for (i, stage) in self.stages.iter().enumerate() {
            let top = rect.top() + i as f32 * step;
            let row = Rect::from_min_max(pos2(rect.left(), top), pos2(rect.right(), top + step));
            let before = width(stage);
            let after = self.stages.get(i + 1).map_or(before * 0.76, width);
            let bottom = top + step - 4_f32.min(step * 0.12);
            let polygon = vec![
                pos2(center - before, top),
                pos2(center + before, top),
                pos2(center + after, bottom),
                pos2(center - after, bottom),
            ];
            let hovered = response.hover_pos().is_some_and(|p| row.contains(p));
            let color = mix(
                self.color,
                c.surface,
                (0.08 + i as f32 / self.stages.len() as f32 * 0.55)
                    * if hovered { 0.6 } else { 1. },
            );
            ui.painter()
                .add(Shape::convex_polygon(polygon, color, Stroke::NONE));
            let info = if first > 0. {
                format!(
                    "{}  ·  {:.0}%",
                    plot::number(value(stage)),
                    value(stage) / first * 100.
                )
            } else {
                plot::number(value(stage))
            };
            if step >= 24. {
                ui.painter().text(
                    pos2(label.left() + 8., row.center().y - 7.),
                    Align2::LEFT_CENTER,
                    stage.label,
                    Type::Label.font(),
                    c.text_primary,
                );
                plot::label(
                    ui,
                    pos2(label.left() + 8., row.center().y + 9.),
                    Align2::LEFT_CENTER,
                    &info,
                );
            }
            if hovered {
                plot::tooltip(ui, &response, stage.label, [(color, String::new(), info)]);
            }
        }
        response
    }
}
