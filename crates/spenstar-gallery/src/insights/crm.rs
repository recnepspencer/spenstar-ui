use eframe::egui::{Color32, Ui};
use spenstar_ui::{
    Theme,
    charts::{
        AreaChart, BarChart, BarMode, FunnelChart, FunnelStage, LineChart, Series, Sparkline,
    },
    datetime::{CalendarDate, CalendarOptions, DatePreset, DateRange, DateRangePicker},
    display::{KpiCard, KpiDetail, Tone},
    form::FieldVariant,
    icon::Icon,
    layout::ResponsiveGrid,
    select::SelectStyle,
};

use crate::widgets;

use super::{pair, panel, summary, trend};

pub(crate) struct Crm {
    range: Option<DateRange>,
}

impl Default for Crm {
    fn default() -> Self {
        Self {
            range: CalendarDate::new(2026, 10, 27)
                .zip(CalendarDate::new(2026, 11, 25))
                .map(|(start, end)| DateRange::new(start, end)),
        }
    }
}

const LABELS: [&str; 6] = ["Oct 27", "Nov 2", "Nov 8", "Nov 14", "Nov 20", "Nov 25"];
const PIPELINE: [f32; 9] = [5.8, 6.2, 6.1, 6.7, 7.1, 6.9, 7.5, 7.6, 8.4];
const MEETINGS: [f32; 9] = [2., 3., 2., 4., 3., 3., 5., 4., 5.];
const LOST: [f32; 9] = [5., 4., 5., 3., 4., 3., 3., 2., 1.];

impl Crm {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let colors = Theme::current(ui.ctx()).colors;
        widgets::toolbar(
            ui,
            620.,
            |ui| summary(ui, "Deal velocity, forecast, and follow-ups"),
            |ui| {
                widgets::sized_width(ui, 250., 36., |ui| {
                    DateRangePicker {
                        options: CalendarOptions {
                            today: CalendarDate::new(2026, 11, 25),
                            ..Default::default()
                        },
                        style: SelectStyle {
                            variant: FieldVariant::Tonal,
                            ..Default::default()
                        },
                        presets: &[DatePreset::Last7Days, DatePreset::Last30Days],
                    }
                    .show_with_format(ui, &mut self.range, |range| {
                        let date =
                            |d: CalendarDate| format!("{:02}/{:02}/{:04}", d.month, d.day, d.year);
                        format!("{}  –  {}", date(range.start), date(range.end))
                    });
                });
            },
        );
        let slots: Vec<_> = [(10, 27), (11, 2), (11, 8), (11, 14), (11, 20), (11, 25)]
            .into_iter()
            .enumerate()
            .filter_map(|(i, (month, day))| {
                CalendarDate::new(2026, month, day)
                    .filter(|&date| self.range.is_none_or(|range| range.contains(date)))
                    .map(|_| i)
            })
            .collect();
        let labels: Vec<_> = slots.iter().map(|&i| LABELS[i]).collect();
        let samples =
            |values: [f32; LABELS.len()]| slots.iter().map(|&i| values[i]).collect::<Vec<_>>();
        ResponsiveGrid::new(198.)
            .max_columns(3)
            .show(ui, 6, |ui, i| {
                let (label, value, change, tone, icon) = [
                    ("Pipeline", "$8.4M", "↑ 8.0%", Tone::Success, Icon::Kanban),
                    ("Meetings today", "5", "+2", Tone::Success, Icon::Calendar),
                    (
                        "Lost this quarter",
                        "1",
                        "↓ 3",
                        Tone::Success,
                        Icon::ArrowDown,
                    ),
                    (
                        "Win rate",
                        "67%",
                        "↑ 4 pts",
                        Tone::Success,
                        Icon::CheckCircle2,
                    ),
                    ("At risk", "2", "$410k", Tone::Warning, Icon::AlertTriangle),
                    (
                        "Overdue follow-ups",
                        "5",
                        "Due now",
                        Tone::Danger,
                        Icon::Clock,
                    ),
                ][i];
                let mut card = KpiCard::new(label, value);
                card.compact = true;
                card.icon = Some(icon.into());
                card.trend = Some(trend(change, tone));
                let detail = if i < 3 {
                    let mut spark = Sparkline::new(
                        [&PIPELINE, &MEETINGS, &LOST][i],
                        [colors.accent.base, colors.success.base, colors.warning.base][i],
                    );
                    spark.height = 20.;
                    KpiDetail::Sparkline(spark)
                } else {
                    KpiDetail::Progress(
                        [0.67, 0.32, 0.49][i - 3],
                        Some([colors.success.base, colors.warning.base, colors.danger.base][i - 3]),
                    )
                };
                card.detail = Some(detail);
                card.show(ui);
            });
        let blue = colors.accent.base;
        let violet = Color32::from_rgb(148, 111, 242);
        pair(
            ui,
            0.5,
            |ui| {
                panel(ui, "Created vs closed", "Deal count by period", |ui| {
                    let created = samples([14., 25., 21., 30., 23., 34.]);
                    let closed = samples([8., 12., 10., 19., 16., 25.]);
                    let series = [
                        Series::new("Created", &created, blue),
                        Series::new("Closed", &closed, violet),
                    ];
                    BarChart {
                        height: 104.,
                        ..BarChart::new(&labels, &series)
                    }
                    .show(ui);
                })
            },
            |ui| {
                panel(
                    ui,
                    "Days to close",
                    "Average days from creation to close",
                    |ui| {
                        let days = samples([36., 31., 34., 27., 29., 23.]);
                        let series = [Series::new("Days to close", &days, colors.success.base)];
                        LineChart {
                            height: 104.,
                            ..LineChart::new(&labels, &series)
                        }
                        .show(ui);
                    },
                )
            },
        );
        pair(
            ui,
            0.5,
            |ui| {
                panel(
                    ui,
                    "Weighted forecast",
                    "Revenue weighted by close probability",
                    |ui| {
                        let commit = samples([1.6, 2.1, 2.4, 2.5, 2.8, 3.2].map(|x| x * 1e6));
                        let upside = samples([0.8, 0.6, 1.1, 1.0, 1.2, 1.6].map(|x| x * 1e6));
                        let series = [
                            Series::new("Commit", &commit, blue),
                            Series::new("Upside", &upside, violet),
                        ];
                        if ui.available_width() < 340. {
                            BarChart {
                                height: 115.,
                                mode: BarMode::Stacked,
                                ..BarChart::new(&labels, &series)
                            }
                            .show(ui);
                        } else {
                            AreaChart(LineChart {
                                height: 115.,
                                ..LineChart::new(&labels, &series)
                            })
                            .show(ui);
                        }
                    },
                )
            },
            |ui| {
                panel(
                    ui,
                    "Pipeline funnel",
                    "Conversion from first contact",
                    |ui| {
                        let stages = [
                            ("Qualified", 162.),
                            ("Discovery", 104.),
                            ("Proposal", 67.),
                            ("Closed won", 39.),
                        ]
                        .map(|(label, count)| FunnelStage {
                            label,
                            value: (count * slots.len() as f32 / LABELS.len() as f32).round(),
                        });
                        FunnelChart {
                            height: 137.,
                            ..FunnelChart::new(if slots.is_empty() { &[] } else { &stages }, violet)
                        }
                        .show(ui);
                    },
                )
            },
        );
    }
}
