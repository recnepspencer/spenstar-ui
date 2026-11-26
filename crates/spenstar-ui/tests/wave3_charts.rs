//! Finite rendering and hover tests for painter-backed charts.
#[path = "wave3/support.rs"]
mod support;

use spenstar_ui::{
    Theme,
    charts::{
        AreaChart, BarChart, BarMode, BarOrientation, DonutChart, DonutSlice, FunnelChart,
        FunnelStage, LineChart, Series, Sparkline,
    },
    egui::{Color32, Context, Event, Pos2, Response, Ui, epaint::Primitive, pos2},
};

use support::{frame, location};

fn draw(ui: &mut Ui, kind: usize, values: &[f32]) -> Response {
    let blue = Color32::from_rgb(33, 112, 218);
    let purple = Color32::from_rgb(147, 97, 215);
    let labels = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"];
    let mut inverse = values.to_vec();
    inverse.reverse();
    let series = [
        Series::new("Current", values, blue),
        Series::new("Previous", &inverse, purple),
    ];
    match kind {
        0 => Sparkline {
            width: 300.,
            ..Sparkline::new(values, blue)
        }
        .show(ui),
        1 => LineChart::new(&labels[..values.len().min(labels.len())], &series).show(ui),
        2 => AreaChart::new(&labels[..values.len().min(labels.len())], &series).show(ui),
        3..=6 => BarChart {
            orientation: if kind % 2 == 1 {
                BarOrientation::Vertical
            } else {
                BarOrientation::Horizontal
            },
            mode: if kind <= 4 {
                BarMode::Grouped
            } else {
                BarMode::Stacked
            },
            ..BarChart::new(&labels[..values.len().min(labels.len())], &series)
        }
        .show(ui),
        7 => {
            let slices = labels
                .iter()
                .zip(values)
                .enumerate()
                .map(|(i, (&label, &value))| DonutSlice {
                    label,
                    value,
                    color: if i % 2 == 0 { blue } else { purple },
                })
                .collect::<Vec<_>>();
            DonutChart::new(&slices, "Total").show(ui)
        }
        _ => {
            let stages = labels
                .iter()
                .zip(values)
                .map(|(&label, &value)| FunnelStage { label, value })
                .collect::<Vec<_>>();
            FunnelChart::new(&stages, blue).show(ui)
        }
    }
}

#[test]
fn every_chart_tessellates_edge_cases_without_nonfinite_geometry() {
    let cases: &[&[f32]] = &[
        &[],
        &[0.],
        &[5.],
        &[3., 3., 3.],
        &[-6., -3., -9.],
        &[f32::NAN, f32::INFINITY],
        &[-f32::MAX, 0., f32::MAX, f32::NAN, f32::NEG_INFINITY],
        &[5., f32::NAN, -3., 0., 5.],
    ];
    let ctx = Context::default();
    Theme::default().install(&ctx);
    for kind in 0..9 {
        for (index, values) in cases.iter().enumerate() {
            let output = frame(&ctx, (kind * cases.len() + index) as f64, vec![], |ui| {
                ui.set_width(480.);
                let response = draw(ui, kind, values);
                assert!(response.rect.is_finite() && response.rect.is_positive());
            });
            for clipped in ctx.tessellate(output.shapes, output.pixels_per_point) {
                assert!(!clipped.clip_rect.any_nan());
                if let Primitive::Mesh(mesh) = clipped.primitive {
                    assert!(mesh.is_valid());
                    assert!(
                        mesh.vertices.iter().all(|v| v.pos.is_finite()),
                        "case {index} chart {kind}"
                    );
                }
            }
        }
    }
}

#[test]
fn hover_shows_chart_specific_tooltips() {
    let ctx = Context::default();
    Theme::default().install(&ctx);
    let samples = [30., 20., 10.];
    for kind in [0, 1, 2, 3, 4, 7, 8] {
        let mut pointer = Pos2::ZERO;
        let start = 2. * kind as f64;
        frame(&ctx, start, vec![], |ui| {
            ui.set_width(480.);
            let rect = draw(ui, kind, &samples).rect;
            pointer = match kind {
                0 => rect.center(),
                1 | 2 => pos2(rect.center().x + 20., rect.center().y),
                3 => pos2(rect.left() + 101., rect.bottom() - 45.),
                4 => pos2(rect.left() + 145., rect.top() + 42.),
                7 => pos2(rect.center().x + 72., rect.center().y),
                _ => pos2(rect.left() + 50., rect.top() + 22.),
            };
        });
        let target = match kind {
            0 => "Sample 2",
            1 | 2 => "Beta",
            _ => "Alpha",
        };
        let render = |time| {
            frame(&ctx, time, vec![Event::PointerMoved(pointer)], |ui| {
                ui.set_width(480.);
                draw(ui, kind, &samples);
            })
        };
        render(start + 0.1);
        let output = render(start + 0.2);
        let hits = support::texts(&output)
            .iter()
            .filter(|(text, _)| *text == target)
            .count();
        let minimum = if matches!(kind, 0 | 7) { 1 } else { 2 };
        assert!(
            hits >= minimum,
            "chart {kind}, target {target}, {pointer:?}: {:?}",
            support::texts(&output)
        );
        location(&output, target);
    }
}
