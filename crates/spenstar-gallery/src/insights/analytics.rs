use eframe::egui::{Color32, Ui};
use spenstar_ui::{
    Theme,
    charts::{BarChart, BarOrientation, DonutChart, DonutSlice, LineChart, Series, Sparkline},
    display::{KpiCard, KpiDetail, Tone},
    icon::Icon,
    layout::ResponsiveGrid,
    navigation::{Tab, Tabs},
};

use crate::widgets;

use super::{pair, panel, summary, trend};

#[derive(Default)]
pub(crate) struct Analytics {
    period: usize,
}

const PERIODS: [&str; 4] = ["7D", "30D", "90D", "12M"];
const PERIOD_LABELS: [[&str; 6]; 4] = [
    ["Nov 19", "Nov 20", "Nov 21", "Nov 22", "Nov 24", "Nov 25"],
    ["Oct 27", "Nov 2", "Nov 8", "Nov 14", "Nov 20", "Nov 25"],
    ["Aug 28", "Sep 15", "Oct 1", "Oct 20", "Nov 7", "Nov 25"],
    ["Dec ’25", "Feb", "Apr", "Jun", "Aug", "Nov ’26"],
];
const SESSION_HISTORY: [f32; 14] = [
    2100., 2400., 2280., 3150., 2960., 3450., 2800., 3400., 3700., 3570., 4070., 3610., 4210.,
    4610.,
];

impl Analytics {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let c = Theme::current(ui.ctx()).colors;
        widgets::toolbar(
            ui,
            620.,
            |ui| summary(ui, "Audience and acquisition across your website"),
            |ui| {
                widgets::sized_width(ui, 199., 38., |ui| {
                    Tabs::new("analytics-range").show(ui, &PERIODS.map(Tab::new), &mut self.period);
                });
            },
        );
        let factor = [1., 4.15, 11.7, 44.9][self.period];
        let sessions = SESSION_HISTORY.map(|x| x * factor);
        let unique = sessions.map(|x| x * 0.75);
        let bounce = [45., 46., 43., 44., 40., 42., 39., 36.];
        let duration = [88., 92., 91., 107., 105., 114., 119., 132.];
        let total = format!("{:.1}k", 24.8 * factor);
        let users = format!("{:.1}k", 18.4 * factor);
        let metrics = [
            (
                "Sessions",
                total.as_str(),
                "↑ 12.8%",
                &sessions[..],
                Icon::Activity,
            ),
            (
                "Unique users",
                users.as_str(),
                "↑ 8.4%",
                &unique[..],
                Icon::Users,
            ),
            (
                "Bounce rate",
                "36.2%",
                "↓ 2.1 pts",
                &bounce[..],
                Icon::ArrowDown,
            ),
            (
                "Avg. session",
                "2m 12s",
                "↑ 14 sec",
                &duration[..],
                Icon::Clock,
            ),
        ];
        let violet = Color32::from_rgb(149, 111, 243);
        let cyan = Color32::from_rgb(37, 174, 176);
        ResponsiveGrid::new(195.)
            .max_columns(4)
            .show(ui, 4, |ui, i| {
                let (label, value, change, samples, icon) = metrics[i];
                let mut metric = KpiCard::new(label, value);
                metric.compact = true;
                metric.icon = Some(icon.into());
                metric.trend = Some(trend(change, Tone::Success));
                let mut spark =
                    Sparkline::new(samples, [c.accent.base, violet, cyan, c.success.base][i]);
                spark.height = 23.;
                metric.detail = Some(KpiDetail::Sparkline(spark));
                metric.show(ui);
            });
        pair(
            ui,
            0.65,
            |ui| {
                panel(
                    ui,
                    "Sessions over time",
                    "Current period compared with the previous period",
                    |ui| {
                        let values = [
                            sessions[0],
                            sessions[3],
                            sessions[6],
                            sessions[8],
                            sessions[11],
                            sessions[13],
                        ];
                        let previous =
                            [2040., 2410., 2150., 3030., 2860., 3510.].map(|x| x * factor);
                        let series = [
                            Series::new("Current period", &values, c.accent.base),
                            Series::new("Previous period", &previous, violet),
                        ];
                        LineChart {
                            height: 170.,
                            ..LineChart::new(&PERIOD_LABELS[self.period], &series)
                        }
                        .show(ui);
                    },
                )
            },
            |ui| {
                panel(ui, "Traffic by device", "Share of website sessions", |ui| {
                    let slices = [
                        ("Desktop", 15200., c.accent.base),
                        ("Mobile", 8250., violet),
                        ("Tablet", 1350., cyan),
                    ]
                    .map(|(label, value, color)| DonutSlice {
                        label,
                        value: value * factor,
                        color,
                    });
                    DonutChart {
                        height: 170.,
                        ..DonutChart::new(&slices, "Sessions")
                    }
                    .show(ui);
                })
            },
        );
        panel(
            ui,
            "Top channels",
            "Sessions by acquisition channel",
            |ui| {
                let values = [10100., 6800., 4100., 2450., 1350.].map(|x| x * factor);
                let series = [Series::new("Sessions", &values, c.accent.base)];
                BarChart {
                    orientation: BarOrientation::Horizontal,
                    legend: false,
                    height: 172.,
                    ..BarChart::new(
                        &["Organic search", "Direct", "Referral", "Social", "Email"],
                        &series,
                    )
                }
                .show(ui);
            },
        );
    }
}
