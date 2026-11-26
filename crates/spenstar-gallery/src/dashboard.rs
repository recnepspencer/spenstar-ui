mod records;

pub(super) use records::RecordAction;

use eframe::egui::{self, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Size, ToggleButtonGroup},
    charts::{BarChart, Series},
    display::{Avatar, Badge, KpiCard, Tone},
    form::FieldVariant,
    icon::Icon,
    layout::{Card, ResponsiveGrid, Separator, Toolbar},
    table::TableState,
    typography::Type,
};

use crate::{shipments::Shipment, widgets};

pub(super) struct Dashboard {
    tab: usize,
    period: Option<String>,
    query: String,
    exceptions: bool,
    table: TableState,
    show_carrier: bool,
    notice: Option<String>,
    remove: Option<String>,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            tab: 0,
            period: Some("Monthly".into()),
            query: String::new(),
            exceptions: false,
            table: TableState::default(),
            show_carrier: true,
            notice: None,
            remove: None,
        }
    }
}

const METRICS: [[(&str, &str, &str, Tone); 4]; 3] = [
    [
        ("Open shipments", "128", "↑ 9.4%", Tone::Success),
        ("On-time delivery", "98.2%", "↑ 2.4 pts", Tone::Success),
        ("Active carriers", "24", "+3", Tone::Success),
        ("Exceptions", "3", "↑ 1", Tone::Danger),
    ],
    [
        ("Gross sales", "$82,416", "+8.3%", Tone::Success),
        ("Orders", "364", "+3.2%", Tone::Success),
        ("Average order", "$226", "+4.8%", Tone::Success),
        ("Returns", "12", "+1", Tone::Danger),
    ],
    [
        ("Freight costs", "$28,235", "+1.2%", Tone::Danger),
        ("Fuel surcharge", "$3,426", "−3.4%", Tone::Success),
        ("Cost per shipment", "$77.57", "−1.9%", Tone::Success),
        ("Credits", "$520", "+1.0%", Tone::Success),
    ],
];

impl Dashboard {
    pub fn show(&mut self, ui: &mut Ui, shipments: &[Shipment]) -> Option<RecordAction> {
        let mut download_requested = false;
        widgets::toolbar(
            ui,
            600.,
            |ui| {
                ToggleButtonGroup::new(
                    "dashboard-period-view",
                    ["Overview", "Sales", "Expenses"]
                        .map(|label| Button::new(label).size(Size::Sm)),
                )
                .show(ui, &mut self.tab);
            },
            |ui| {
                download_requested = Button::new("Download")
                    .leading(Icon::Download)
                    .size(Size::Sm)
                    .show(ui)
                    .clicked();
                widgets::sized_width(ui, 126., 38., |ui| {
                    widgets::select_styled(
                        ui,
                        &mut self.period,
                        &["Weekly", "Monthly", "Quarterly"],
                        "Period",
                        FieldVariant::Capsule,
                    );
                });
                if widgets::action(ui, "Reset report filters", Icon::RefreshCw).clicked() {
                    self.period = Some("Monthly".into());
                    self.query.clear();
                    self.exceptions = false;
                    self.table.sort = None;
                    self.notice = None;
                }
            },
        );
        if download_requested {
            self.notice = Some(match self.export(shipments) {
                Ok(path) => {
                    crate::widgets::notify(
                        ui.ctx(),
                        "Shipment records saved",
                        spenstar_ui::feedback::Tone::Success,
                    );
                    format!("Saved {}", path.display())
                }
                Err(err) => {
                    crate::widgets::notify(
                        ui.ctx(),
                        "Download failed",
                        spenstar_ui::feedback::Tone::Danger,
                    );
                    format!("Could not download records: {err}")
                }
            });
        }
        if let Some(notice) = &self.notice {
            ui.label(
                Type::Small
                    .text(notice)
                    .color(Theme::current(ui.ctx()).colors.text_muted),
            );
        }
        ResponsiveGrid::new(200.)
            .max_columns(4)
            .show(ui, 4, |ui, index| {
                let (label, value, trend, tone) = METRICS[self.tab][index];
                let mut metric = KpiCard::new(label, value);
                metric.trend = Some(Badge {
                    tone,
                    ..Badge::new(trend)
                });
                metric.show(ui);
            });
        ResponsiveGrid::new(300.)
            .max_columns(2)
            .show(ui, 2, |ui, index| {
                if index == 0 {
                    volume(ui, self.tab, self.period.as_deref().unwrap_or("Monthly"));
                } else {
                    activity(ui, shipments);
                }
            });
        self.records(ui, shipments)
    }

    fn export(&self, shipments: &[Shipment]) -> std::io::Result<std::path::PathBuf> {
        use std::{
            fs::OpenOptions,
            io::Write,
            path::PathBuf,
            time::{SystemTime, UNIX_EPOCH},
        };
        let dir = std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|path| path.join("Downloads"))
            .filter(|path| path.is_dir())
            .unwrap_or_else(std::env::temp_dir);
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let path = dir.join(format!("spenstar-shipments-{suffix}.csv"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        writeln!(file, "Reference,Shipment,Carrier,Status")?;
        for (_, item) in self.filtered(shipments) {
            let cells = [&item.id, &item.name, &item.carrier, &item.status].map(|cell| {
                let prefix = if cell.trim_start().starts_with(['=', '+', '-', '@']) {
                    "'"
                } else {
                    ""
                };
                format!("\"{prefix}{}\"", cell.replace('"', "\"\""))
            });
            writeln!(file, "{}", cells.join(","))?;
        }
        Ok(path)
    }
}

fn volume(ui: &mut Ui, tab: usize, period: &str) {
    let c = Theme::current(ui.ctx()).colors;
    let title = ["Shipment volume", "Sales performance", "Freight spending"][tab];
    let values = match period {
        "Weekly" => [0.32, 0.60, 0.44, 0.68, 0.52, 0.71, 0.59],
        "Quarterly" => [0.44, 0.39, 0.61, 0.52, 0.71, 0.62, 0.81],
        _ => [0.38, 0.57, 0.46, 0.69, 0.59, 0.82, 0.75],
    };
    let actual = values.map(|value| value * [300., 20000., 6500.][tab]);
    let plan = actual.map(|value| value * 0.82 + [9., 600., 195.][tab]);
    let series = [
        Series::new("Actual", &actual, c.accent.base),
        Series::new("Plan", &plan, c.accent.ring.gamma_multiply(0.48)),
    ];
    Card::default().show(ui, |card| {
        card.header(
            title,
            Some(["1,248 dispatched", "$82,416 sold", "$28,235 spent"][tab]),
        );
        card.body(|ui| {
            BarChart {
                height: 134.,
                ..BarChart::new(&["1", "5", "9", "13", "17", "21", "25"], &series)
            }
            .show(ui);
        });
    });
}

fn activity(ui: &mut Ui, shipments: &[Shipment]) {
    let c = Theme::current(ui.ctx()).colors;
    Card::default().show(ui, |card| {
        card.header("Recent activity", Some("Latest updates across the network"));
        card.body(|ui| {
            ui.set_min_height(166.);
            ui.spacing_mut().item_spacing.y = 6.;
            for (index, row) in shipments.iter().take(3).enumerate() {
                if index > 0 {
                    Separator::Horizontal.show(ui);
                }
                Toolbar::show(ui, |ui| {
                    Avatar {
                        size: 34,
                        tone: row.tone(),
                        icon: Some(Icon::Truck.into()),
                        ..Avatar::new(&row.carrier)
                    }
                    .show(ui);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.;
                        ui.add(egui::Label::new(Type::Label.text(&row.name)).truncate());
                        ui.label(
                            Type::Small
                                .text(format!("{} · {}", row.id, row.hub))
                                .color(c.text_muted),
                        );
                    });
                });
            }
        });
    });
}
