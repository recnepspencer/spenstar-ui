mod records;

pub(super) use records::RecordAction;

use eframe::egui::{self, Align2, Rect, Sense, Stroke, Ui, pos2, vec2};
use spenstar_ui::{
    Theme,
    button::{Button, Size, ToggleButtonGroup},
    controls::Tone,
    form::FieldVariant,
    icon::Icon,
    layout::{Card, ResponsiveGrid, Separator, Toolbar},
    typography::Type,
};

use crate::{shipments::Shipment, widgets};

pub(super) struct Dashboard {
    tab: usize,
    period: Option<String>,
    query: String,
    exceptions: bool,
    sorted: bool,
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
            sorted: false,
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
                    self.sorted = false;
                    self.notice = None;
                }
            },
        );
        if download_requested {
            self.notice = Some(match self.export(shipments) {
                Ok(path) => format!("Saved {}", path.display()),
                Err(err) => format!("Could not download records: {err}"),
            });
        }
        if let Some(notice) = &self.notice {
            ui.label(
                Type::Small
                    .text(notice)
                    .color(Theme::current(ui.ctx()).colors.text_muted),
            );
        }
        let palette = Theme::current(ui.ctx()).colors;
        ResponsiveGrid::new(200.)
            .max_columns(4)
            .show(ui, 4, |ui, index| {
                let (label, value, trend, tone) = METRICS[self.tab][index];
                Card::default().show(ui, |card| {
                    card.body(|ui| {
                        ui.label(Type::Description.text(label).color(palette.text_secondary));
                        ui.add_space(2.);
                        ui.horizontal(|ui| {
                            ui.label(Type::Metric.text(value));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    widgets::tag(ui, trend, tone);
                                },
                            );
                        });
                    });
                });
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
    Card::default().show(ui, |card| {
        card.header(title, None);
        card.body(|ui| {
            ui.horizontal(|ui| {
                ui.label(Type::H3.text(["1,248 dispatched", "$82,416 sold", "$28,235 spent"][tab]));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for (label, color) in [("Plan", c.accent.soft_hover), ("Actual", c.accent.base)]
                    {
                        ui.label(Type::Caption.text(label).color(c.text_muted));
                        let (dot, _) = ui.allocate_exact_size(vec2(7., 7.), Sense::hover());
                        ui.painter().circle_filled(dot.center(), 3.5, color);
                    }
                });
            });
        });
        card.body(|ui| {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 165.), Sense::hover());
            let chart = Rect::from_min_max(rect.min + vec2(0., 8.), rect.max - vec2(0., 22.));
            let col = chart.width() / 7.;
            for ratio in [0., 0.25, 0.5, 0.75, 1.] {
                let y = chart.bottom() - chart.height() * ratio;
                ui.painter()
                    .hline(chart.x_range(), y, Stroke::new(1., c.divider));
            }
            for (i, (actual, label)) in values
                .into_iter()
                .zip(["1", "5", "9", "13", "17", "21", "25"])
                .enumerate()
            {
                let x = chart.left() + col * (i as f32 + 0.5);
                let width = (col * 0.20).min(12.);
                let bar = |left, value, color| {
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            pos2(left, chart.bottom() - chart.height() * value),
                            vec2(width, chart.height() * value),
                        ),
                        3,
                        color,
                    );
                };
                bar(x - width - 2., actual, c.accent.base);
                bar(
                    x + 2.,
                    (actual - 0.10 + (i % 3) as f32 * 0.07).max(0.),
                    c.accent.soft_hover,
                );
                ui.painter().text(
                    pos2(x, rect.bottom()),
                    Align2::CENTER_BOTTOM,
                    label,
                    Type::Caption.font(),
                    c.text_muted,
                );
            }
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
                    let (rect, _) = ui.allocate_exact_size(vec2(34., 34.), Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 17., c.surface_secondary);
                    let _ = Icon::Truck.svg().paint(
                        ui,
                        Rect::from_center_size(rect.center(), vec2(16., 16.)),
                        c.text_secondary,
                    );
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
