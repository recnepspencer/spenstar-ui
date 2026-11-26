use eframe::egui::{Align, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    charts::{AreaChart, LineChart, Series, Sparkline},
    display::{Avatar, Badge, KpiCard, KpiDetail, Tone},
    icon::Icon,
    layout::{ResponsiveGrid, Separator},
    navigation::{Tab, Tabs},
    table::{Column, DataTable, Density, TableRows, TableState},
    theme::Radius,
    typography::Type,
};

use crate::widgets;

use super::{pair, panel, summary, trend};

pub(crate) struct Portfolio {
    period: usize,
    activity: TableState,
    action: Option<&'static str>,
}

impl Default for Portfolio {
    fn default() -> Self {
        Self {
            period: 2,
            activity: TableState::default(),
            action: None,
        }
    }
}

const PERIODS: [&str; 6] = ["1D", "1W", "1M", "3M", "1Y", "All"];
const PRICES: [f32; 11] = [
    4690., 4740., 4830., 4750., 4980., 4880., 5060., 5030., 5270., 5190., 5427.48,
];
const AXES: [[&str; 6]; 6] = [
    ["08:00", "10:00", "12:00", "14:00", "16:00", "18:00"],
    ["Thu", "Fri", "Sat", "Sun", "Mon", "Wed"],
    ["Oct 27", "Nov 2", "Nov 8", "Nov 14", "Nov 20", "Nov 25"],
    ["Aug 28", "Sep 15", "Oct 1", "Oct 20", "Nov 7", "Nov 25"],
    ["Jan", "Mar", "May", "Jul", "Sep", "Nov"],
    ["2021", "2022", "2023", "2024", "2025", "2026"],
];

const HOLDINGS: [(&str, &str, &str, &str, Tone); 5] = [
    ("BTC", "Bitcoin", "$1,283.84", "+2.34%", Tone::Warning),
    ("USDC", "USD Coin", "$2,087.25", "+0.01%", Tone::Accent),
    ("ETH", "Ethereum", "$1,087.25", "−1.12%", Tone::Neutral),
    ("SOL", "Solana", "$195.54", "+4.72%", Tone::Success),
    ("BNB", "BNB Chain", "$165.20", "+0.92%", Tone::Warning),
];

const ACTIVITY: [(&str, &str, &str, &str, &str); 6] = [
    ("0x9a2b…05d3", "Received", "USDC", "$500.00", "Nov 25, 2026"),
    ("0xd3be…40c3", "Sent", "ETH", "$320.65", "Nov 24, 2026"),
    ("0x2ca8…18b2", "Swapped", "BTC", "$385.22", "Nov 22, 2026"),
    ("0x4f7e…216e", "Received", "SOL", "$692.16", "Nov 21, 2026"),
    ("0x3b52…157f", "Sent", "USDC", "$85.00", "Nov 20, 2026"),
    ("0x7a15…ff9b", "Swapped", "ETH", "$210.18", "Nov 19, 2026"),
];

impl Portfolio {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let c = Theme::current(ui.ctx()).colors;
        let mut next_action = None;
        widgets::toolbar(
            ui,
            610.,
            |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 3.;
                    ui.label(Type::H1.text("Welcome back, Alex"));
                    summary(ui, "Track your wallet balance and asset activity.");
                });
            },
            |ui| {
                for (label, icon, variant) in [
                    ("Send", Icon::Send, Variant::Accent),
                    ("Receive", Icon::ArrowDown, Variant::Secondary),
                    ("Swap", Icon::ArrowUpDown, Variant::Secondary),
                ] {
                    if Button::new(label)
                        .leading(icon)
                        .variant(variant)
                        .radius(Radius::Full)
                        .size(Size::Sm)
                        .show(ui)
                        .clicked()
                    {
                        next_action = Some(label);
                    }
                }
            },
        );
        if next_action.is_some() {
            self.action = next_action;
        }
        if let Some(action) = self.action {
            let modal = eframe::egui::Modal::new(ui.make_persistent_id("wallet-action"))
                .backdrop_color(c.scrim)
                .show(ui.ctx(), |ui| {
                    ui.label(Type::H2.text(action));
                    ui.label(Type::Body.text("This sample wallet uses simulated holdings."));
                    Button::new("Done").show(ui).clicked()
                });
            if modal.inner || modal.should_close() {
                self.action = None;
            }
        }
        ResponsiveGrid::new(195.)
            .max_columns(4)
            .show(ui, 4, |ui, i| {
                let (label, value, change, icon) = [
                    ("Total balance", "$5,427.48", "↑ 5.32%", Icon::Layers),
                    ("24h change", "+$120.18", "↑ 2.24%", Icon::Activity),
                    (
                        "Top performer · SOL",
                        "4.72%",
                        "↑ 4.72%",
                        Icon::ArrowUpRight,
                    ),
                    ("Holdings", "8", "Assets", Icon::PieChart),
                ][i];
                let mut metric = KpiCard::new(label, value);
                metric.icon = Some(icon.into());
                metric.trend = Some(trend(
                    change,
                    if i < 3 { Tone::Success } else { Tone::Neutral },
                ));
                metric.detail = (i == 0).then(|| {
                    let mut line = Sparkline::new(&PRICES, c.success.base);
                    line.height = 16.;
                    KpiDetail::Sparkline(line)
                });
                if i != 0 {
                    metric.caption = Some(
                        [
                            "",
                            "Since yesterday",
                            "Past 24 hours",
                            "Across four networks",
                        ][i],
                    );
                }
                metric.show(ui);
            });
        pair(
            ui,
            0.60,
            |ui| {
                panel(ui, "Portfolio value", "Balance history in USD", |ui| {
                    Tabs::new("wallet-period").show(ui, &PERIODS.map(Tab::new), &mut self.period);
                    let factor = [0.26, 0.55, 1., 1.4, 2.1, 3.2][self.period];
                    let values = [
                        PRICES[0], PRICES[2], PRICES[4], PRICES[6], PRICES[8], PRICES[10],
                    ]
                    .map(|x| 5427.48 + (x - 5427.48) * factor);
                    let series = [Series::new("Wallet balance", &values, c.accent.base)];
                    AreaChart(LineChart {
                        height: 170.,
                        legend: false,
                        ..LineChart::new(&AXES[self.period], &series)
                    })
                    .show(ui);
                })
            },
            |ui| panel(ui, "Holdings", "Top assets by wallet allocation", holdings),
        );
        self.show_activity(ui);
    }

    fn show_activity(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(Type::H2.text("Recent activity"));
            Badge::new("6 transactions").show(ui);
        });
        let columns = [
            Column::new("Transaction", 1.1),
            Column::new("Type", 1.),
            Column::new("Asset", 0.7),
            Column::new("Value", 0.8).align(Align::Max),
            Column::new("Date", 1.1),
        ];
        let table = DataTable {
            density: Density::Compact,
            height: 136.,
            ..DataTable::new("wallet-activity", &columns)
        };
        table.show(
            ui,
            &mut self.activity,
            TableRows::new(
                0,
                ACTIVITY.len(),
                |_| true,
                |i| i as u64,
                |column, a, b| {
                    let amount = |i| {
                        activity_value(i, 3)
                            .trim_start_matches('$')
                            .replace(',', "")
                            .parse::<f32>()
                            .unwrap_or_default()
                    };
                    if column == 3 {
                        amount(a).total_cmp(&amount(b))
                    } else {
                        activity_value(a, column).cmp(activity_value(b, column))
                    }
                },
            ),
            |ui, i, column| {
                let value = activity_value(i, column);
                if column == 1 {
                    trend(
                        value,
                        match value {
                            "Received" => Tone::Success,
                            "Sent" => Tone::Neutral,
                            _ => Tone::Accent,
                        },
                    )
                    .show(ui);
                } else {
                    let role = if column == 0 { Type::Mono } else { Type::Small };
                    ui.label(role.text(value));
                }
            },
        );
    }
}

fn holdings(ui: &mut Ui) {
    let c = Theme::current(ui.ctx()).colors;
    ui.spacing_mut().item_spacing.y = 4.;
    for (i, (ticker, name, value, change, tone)) in HOLDINGS.into_iter().enumerate() {
        if i > 0 {
            Separator::Horizontal.show(ui);
        }
        ui.horizontal(|ui| {
            Avatar {
                tone,
                size: 29,
                ..Avatar::new(&ticker[..1])
            }
            .show(ui);
            widgets::sized_width(ui, 80., 29., |ui| {
                ui.spacing_mut().item_spacing.y = 0.;
                ui.label(Type::Badge.text(ticker));
                ui.label(Type::Small.text(name).color(c.text_muted));
            });
            let down = change.starts_with('−');
            let samples = [2., 3., 2.6, 4.2, 3.4, 5., 4.4, 5.6].map(|x| {
                if down {
                    7. - x
                } else {
                    x + (x as usize % (i + 1)) as f32 * 0.32
                }
            });
            let color = if down { c.danger.ring } else { c.success.ring };
            let spark = Sparkline {
                width: 52.,
                height: 24.,
                ..Sparkline::new(&samples, color)
            };
            spark.show(ui);
            let remaining = ui.available_width();
            ui.allocate_ui_with_layout(
                eframe::egui::vec2(remaining, 29.),
                eframe::egui::Layout::top_down(Align::Max),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 0.;
                    ui.label(Type::Badge.text(value));
                    ui.label(Type::Small.text(change).color(color));
                },
            );
        });
    }
}

fn activity_value(index: usize, column: usize) -> &'static str {
    let (id, kind, asset, amount, date) = ACTIVITY[index];
    [id, kind, asset, amount, date]
        .get(column)
        .copied()
        .unwrap_or_default()
}
