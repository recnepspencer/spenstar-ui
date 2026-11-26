use std::hash::{DefaultHasher, Hash, Hasher};

use eframe::egui::{self, Align, Color32, Id, Layout, Rect, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, CloseButton, Variant},
    datetime::{CalendarDate, CalendarOptions, DateRange, DateRangePicker},
    display::{Avatar, Badge, Tone},
    form::{Field, FieldVariant, TextField},
    icon::Icon,
    select::SelectStyle,
    table::{Column, DataTable, TableRows, TableState},
    typography::Type,
};

use crate::widgets;

const STATUSES: &[&str] = &["Paid", "Pending", "Refunded", "Failed"];

#[derive(Clone)]
struct Order {
    id: u32,
    customer: String,
    email: String,
    status: Option<String>,
    total: &'static str,
    day: u8,
}

enum Action {
    View,
    Edit,
    Remove,
}

pub(super) struct Orders {
    query: String,
    status: Option<String>,
    range: Option<DateRange>,
    rows: Vec<Order>,
    active: Option<(Action, Order)>,
    table: TableState,
}

impl Default for Orders {
    fn default() -> Self {
        Self {
            query: String::new(),
            status: Some("All statuses".into()),
            range: None,
            rows: [
                ("Avery Chen", "avery", "Paid", "$1,284.50", 25),
                ("Maya Ford", "maya", "Pending", "$429.00", 24),
                ("Noah Reed", "noah", "Refunded", "$99.00", 23),
                ("Leila Diaz", "leila", "Paid", "$2,495.00", 22),
                ("Sam Patel", "sam", "Failed", "$312.90", 21),
            ]
            .into_iter()
            .enumerate()
            .map(|(index, (name, local, status, total, day))| Order {
                id: 48291 - index as u32,
                customer: name.into(),
                email: format!("{local}@example.com"),
                status: Some(status.into()),
                total,
                day,
            })
            .collect(),
            active: None,
            table: TableState::default(),
        }
    }
}

impl Orders {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let c = Theme::current(ui.ctx()).colors;
        ui.label(
            Type::Body
                .text("Manage and track customer orders.")
                .color(c.text_muted),
        );
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 38.;
            ui.horizontal_wrapped(|ui| {
                widgets::sized_width(ui, 260., 38., |ui| {
                    widgets::search(ui, &mut self.query, "Search orders…");
                });
                widgets::sized_width(ui, 146., 38., |ui| {
                    widgets::select_styled(
                        ui,
                        &mut self.status,
                        &["All statuses", "Paid", "Pending", "Refunded", "Failed"],
                        "Status",
                        FieldVariant::Tonal,
                    );
                });
                widgets::sized_width(ui, 178., 38., |ui| {
                    DateRangePicker {
                        options: CalendarOptions {
                            today: date(25),
                            ..CalendarOptions::default()
                        },
                        style: SelectStyle {
                            placeholder: "Date range",
                            variant: FieldVariant::Tonal,
                            ..SelectStyle::default()
                        },
                        ..DateRangePicker::default()
                    }
                    .show(ui, &mut self.range);
                });
            });
        });

        let query = self.query.trim().to_lowercase();
        let columns = [
            Column {
                min_width: 110.,
                ..Column::new("Order", 0.8)
            },
            Column {
                min_width: 160.,
                ..Column::new("Customer", 2.7)
            },
            Column::new("Status", 0.7),
            Column {
                min_width: 92.,
                ..Column::new("Total", 0.6).align(Align::Max)
            },
            Column {
                min_width: 102.,
                ..Column::new("Date", 0.6)
            },
            Column {
                min_width: 131.,
                ..Column::new("Actions", 0.1).unsorted()
            },
        ];
        let mut hasher = DefaultHasher::new();
        (
            &query,
            &self.status,
            self.range
                .map(|range| (range.start.to_string(), range.end.to_string())),
        )
            .hash(&mut hasher);
        for row in &self.rows {
            (row.id, &row.customer, &row.status).hash(&mut hasher);
        }
        let orders = &self.rows;
        let rows = TableRows::new(
            hasher.finish(),
            orders.len(),
            |i| {
                let order = &orders[i];
                (format!("#ord-{}", order.id).contains(&query)
                    || order.customer.to_lowercase().contains(&query)
                    || order.email.contains(&query))
                    && (self.status.as_deref() == Some("All statuses")
                        || order.status == self.status)
                    && self
                        .range
                        .is_none_or(|range| date(order.day).is_some_and(|day| range.contains(day)))
            },
            |i| u64::from(orders[i].id),
            |column, a, b| {
                let (a, b) = (&orders[a], &orders[b]);
                match column {
                    1 => a.customer.cmp(&b.customer),
                    2 => a.status.cmp(&b.status),
                    3 => amount(a.total).total_cmp(&amount(b.total)),
                    4 => a.day.cmp(&b.day),
                    _ => a.id.cmp(&b.id),
                }
            },
        );
        let mut active = None;
        DataTable::new("orders", &columns).show(ui, &mut self.table, rows, |ui, i, column| {
            let order = &orders[i];
            match column {
                0 => {
                    ui.label(Type::Mono.text(format!("#ORD-{}", order.id)));
                }
                1 => customer(ui, order),
                2 => {
                    let status = order.status.as_deref().unwrap_or("Unassigned");
                    let tone = match status {
                        "Paid" => Tone::Success,
                        "Pending" => Tone::Warning,
                        "Failed" => Tone::Danger,
                        _ => Tone::Neutral,
                    };
                    Badge {
                        tone,
                        ..Badge::new(status)
                    }
                    .show(ui);
                }
                3 => {
                    ui.label(Type::Body.text(order.total));
                }
                4 => {
                    ui.label(
                        Type::Small
                            .text(format!("Nov {}, 2026", order.day))
                            .color(c.text_muted),
                    );
                }
                _ => {
                    ui.spacing_mut().item_spacing.x = 3.;
                    for (label, icon, action) in [
                        ("View order", Icon::Eye, Action::View),
                        ("Edit order", Icon::Pencil, Action::Edit),
                        ("Remove order", Icon::Trash2, Action::Remove),
                    ] {
                        let variant = if matches!(action, Action::Remove) {
                            Variant::DangerSoft
                        } else {
                            Variant::Secondary
                        };
                        if Button::new(label)
                            .icon_only(icon)
                            .size(spenstar_ui::button::Size::Sm)
                            .variant(variant)
                            .show(ui)
                            .clicked()
                        {
                            active = Some((action, order.clone()));
                        }
                    }
                }
            }
        });
        if active.is_some() {
            self.active = active;
        }
        self.dialog(ui);
    }

    fn dialog(&mut self, ui: &Ui) {
        let Some((action, order)) = &mut self.active else {
            return;
        };
        let c = Theme::current(ui.ctx()).colors;
        let outcome = egui::Modal::new(Id::unique("spenstar-gallery/order-dialog"))
            .backdrop_color(c.scrim)
            .show(ui.ctx(), |ui| {
                ui.set_width(340.);
                let title = match action {
                    Action::View => "Order details",
                    Action::Edit => "Edit order",
                    Action::Remove => "Remove order?",
                };
                let close = ui
                    .horizontal(|ui| {
                        ui.label(Type::H2.text(title));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            CloseButton::show(ui).clicked()
                        })
                        .inner
                    })
                    .inner;
                ui.label(
                    Type::Mono
                        .text(format!("#ORD-{}", order.id))
                        .color(c.text_muted),
                );
                match action {
                    Action::View => {
                        customer(ui, order);
                        ui.label(Type::Body.text(format!(
                            "{} · {}",
                            order.total,
                            order.status.as_deref().unwrap_or("")
                        )));
                        (close, false)
                    }
                    Action::Edit => {
                        Field::new("Customer").show(ui, |ui, _| {
                            TextField::new("Customer name").show(ui, &mut order.customer)
                        });
                        Field::new("Status").show(ui, |ui, _| {
                            widgets::select(ui, &mut order.status, STATUSES, "Status")
                        });
                        (
                            close,
                            Button::new("Save order")
                                .enabled(!order.customer.trim().is_empty())
                                .show(ui)
                                .clicked(),
                        )
                    }
                    Action::Remove => {
                        ui.label(Type::Body.text("Remove this order from the gallery list?"));
                        (
                            close,
                            Button::new("Remove order")
                                .variant(Variant::Danger)
                                .show(ui)
                                .clicked(),
                        )
                    }
                }
            });
        let (close, apply) = outcome.inner;
        if apply {
            match action {
                Action::Edit => {
                    if let Some(current) =
                        self.rows.iter_mut().find(|current| current.id == order.id)
                    {
                        *current = order.clone();
                        crate::widgets::notify(
                            ui.ctx(),
                            "Order saved",
                            spenstar_ui::feedback::Tone::Success,
                        );
                    }
                }
                Action::Remove => {
                    self.rows.retain(|current| current.id != order.id);
                    crate::widgets::notify(
                        ui.ctx(),
                        "Order removed",
                        spenstar_ui::feedback::Tone::Success,
                    );
                }
                Action::View => {}
            }
        }
        if close || apply || outcome.should_close() {
            self.active = None;
        }
    }
}

fn amount(text: &str) -> f32 {
    text.trim_start_matches('$')
        .replace(',', "")
        .parse()
        .unwrap_or_default()
}

fn date(day: u8) -> Option<CalendarDate> {
    CalendarDate::new(2026, 11, day)
}

fn customer(ui: &mut Ui, order: &Order) {
    let c = Theme::current(ui.ctx()).colors;
    ui.horizontal(|ui| {
        Avatar {
            tone: [
                Tone::Accent,
                Tone::Success,
                Tone::Danger,
                Tone::Warning,
                Tone::Neutral,
            ][order.id as usize % 5],
            ..Avatar::new(&order.customer)
        }
        .show(ui);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.;
            ui.add(egui::Label::new(Type::Label.text(&order.customer)).truncate());
            ui.add(egui::Label::new(Type::Small.text(&order.email).color(c.text_muted)).truncate());
        });
    });
}

pub(super) fn initials(ui: &Ui, rect: Rect, name: &str, fill: Color32, ink: Color32) {
    let initials: String = name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect();
    ui.painter()
        .circle_filled(rect.center(), rect.width() * 0.5, fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        initials,
        Type::SectionLabel.font(),
        ink,
    );
}
