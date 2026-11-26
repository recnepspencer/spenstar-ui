use egui::{Color32, Response, Sense, Ui, vec2};

use super::Badge;
use crate::{Theme, charts::Sparkline, icon::IconSource, layout::Card, typography::Type};

/// Inline metric visualization.
pub enum KpiDetail<'a> {
    /// A small time series.
    Sparkline(Sparkline<'a>),
    /// A normalized completion fraction and optional fill color.
    Progress(f32, Option<Color32>),
}

/// Label, value, and optional trend in a rounded metric surface.
pub struct KpiCard<'a> {
    /// Name of the metric.
    pub label: &'a str,
    /// Formatted figure.
    pub value: &'a str,
    /// Icon displayed in a soft tile.
    pub icon: Option<IconSource>,
    /// Optional change indicator.
    pub trend: Option<Badge<'a>>,
    /// Supporting footer text.
    pub caption: Option<&'a str>,
    /// Optional inline visualization.
    pub detail: Option<KpiDetail<'a>>,
    /// Reduce inset and vertical spacing.
    pub compact: bool,
}

impl<'a> KpiCard<'a> {
    /// Create a metric card.
    pub fn new(label: &'a str, value: &'a str) -> Self {
        Self {
            label,
            value,
            icon: None,
            trend: None,
            caption: None,
            detail: None,
            compact: false,
        }
    }

    /// Render the card at the available width.
    pub fn show(&self, ui: &mut Ui) -> Response {
        let c = Theme::current(ui.ctx()).colors;
        Card::default()
            .padding(if self.compact { 12 } else { 18 })
            .show(ui, |card| {
                card.body(|ui| {
                    if self.compact {
                        ui.spacing_mut().item_spacing.y = 4.;
                    }
                    ui.horizontal(|ui| {
                        if let Some(icon) = &self.icon {
                            let (rect, _) = ui.allocate_exact_size(vec2(32., 32.), Sense::hover());
                            ui.painter().rect_filled(rect, 9, c.accent.soft);
                            let _ = icon.paint(ui, rect.shrink(8.), c.accent.ring);
                        }
                        ui.label(Type::Description.text(self.label).color(c.text_muted));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(Type::Metric.text(self.value).size(24.));
                        if let Some(trend) = &self.trend {
                            trend.show(ui);
                        }
                    });
                    match &self.detail {
                        Some(KpiDetail::Sparkline(sparkline)) => {
                            sparkline.show(ui);
                        }
                        Some(KpiDetail::Progress(value, color)) => {
                            let fraction = if value.is_finite() {
                                value.clamp(0., 1.)
                            } else {
                                0.
                            };
                            ui.add_space(if self.compact { 3. } else { 5. });
                            let (track, response) = ui.allocate_exact_size(
                                vec2(ui.available_width().max(0.), 5.),
                                Sense::hover(),
                            );
                            ui.painter().rect_filled(track, 3, c.surface_tertiary);
                            let mut fill = track;
                            fill.set_width(track.width() * fraction);
                            if fraction > 0. {
                                ui.painter()
                                    .rect_filled(fill, 3, color.unwrap_or(c.accent.base));
                            }
                            response.on_hover_text(format!("{:.0}%", fraction * 100.));
                            ui.add_space(if self.compact { 3. } else { 5. });
                        }
                        None => {}
                    }
                    if let Some(caption) = self.caption {
                        ui.label(Type::Small.text(caption).color(c.text_muted));
                    }
                })
            })
            .response
    }
}
