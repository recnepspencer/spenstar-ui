use eframe::egui::{self, Align, Color32, Id, Layout, Rect, Sense, Ui, vec2};
use spenstar_ui::{
    Theme,
    button::{Button, CloseButton, Variant},
    controls::Tone,
    datetime::{CalendarDate, CalendarOptions, DateRange, DateRangePicker},
    form::{Field, FieldVariant, TextField},
    icon::Icon,
    layout::{Card, Separator},
    select::SelectStyle,
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
        let shown = self.rows.iter().filter(|order| {
            let id = format!("#ORD-{}", order.id).to_lowercase();
            (id.contains(&query)
                || order.customer.to_lowercase().contains(&query)
                || order.email.contains(&query))
                && (self.status.as_deref() == Some("All statuses") || order.status == self.status)
                && self
                    .range
                    .is_none_or(|range| date(order.day).is_some_and(|day| range.contains(day)))
        });
        Card::default().show(ui, |card| {
            card.body(|ui| {
                ui.spacing_mut().item_spacing.y = 4.;
                let width = ui.available_width();
                let expanded = width >= 740.;
                let compact = width < 465.;
                let id_width = if compact { 0. } else { 106. };
                let customer_width =
                    (width - 134. - id_width - if expanded { 298. } else { 0. }).max(60.);
                let widths = [
                    id_width,
                    customer_width,
                    if expanded { 108. } else { 0. },
                    if expanded { 84. } else { 0. },
                    if expanded { 90. } else { 0. },
                    108.,
                ];
                let band = ui.painter().add(egui::Shape::Noop);
                ui.spacing_mut().interact_size.y = 28.;
                let header = ui.horizontal(|ui| {
                    for (caption, width) in
                        ["Order", "Customer", "Status", "Total", "Date", "Actions"]
                            .into_iter()
                            .zip(widths)
                    {
                        if width > 0. {
                            widgets::sized_width(ui, width, 28., |ui| {
                                ui.add_space(4.);
                                ui.label(Type::Badge.text(caption).color(c.text_muted));
                            });
                        }
                    }
                });
                ui.painter().set(
                    band,
                    egui::Shape::rect_filled(header.response.rect, 8, c.surface_subtle),
                );
                let mut count = 0;
                for order in shown {
                    if count > 0 {
                        Separator::Horizontal.show(ui);
                    }
                    count += 1;
                    ui.push_id(order.id, |ui| {
                        ui.spacing_mut().interact_size.y = 36.;
                        ui.horizontal(|ui| {
                            cell(ui, widths[0], |ui| {
                                ui.label(Type::Mono.text(format!("#ORD-{}", order.id)));
                            });
                            cell(ui, widths[1], |ui| customer(ui, order));
                            cell(ui, widths[2], |ui| {
                                let status = order.status.as_deref().unwrap_or("");
                                widgets::tag(
                                    ui,
                                    status,
                                    match status {
                                        "Paid" => Tone::Success,
                                        "Pending" => Tone::Warning,
                                        "Failed" => Tone::Danger,
                                        _ => Tone::Neutral,
                                    },
                                );
                            });
                            cell(ui, widths[3], |ui| {
                                ui.label(Type::Body.text(order.total));
                            });
                            cell(ui, widths[4], |ui| {
                                ui.label(
                                    Type::Small
                                        .text(format!("Nov {}, 2026", order.day))
                                        .color(c.text_muted),
                                );
                            });
                            ui.spacing_mut().item_spacing.x = 4.;
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
                                    self.active = Some((action, order.clone()));
                                }
                            }
                        });
                    });
                }
                if count == 0 {
                    ui.label(
                        Type::Body
                            .text("No orders match these filters.")
                            .color(c.text_muted),
                    );
                }
            })
        });
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
                    }
                }
                Action::Remove => self.rows.retain(|current| current.id != order.id),
                Action::View => {}
            }
        }
        if close || apply || outcome.should_close() {
            self.active = None;
        }
    }
}

fn cell(ui: &mut Ui, width: f32, content: impl FnOnce(&mut Ui)) {
    if width > 0. {
        widgets::width(ui, width, content);
    }
}

fn date(day: u8) -> Option<CalendarDate> {
    CalendarDate::new(2026, 11, day)
}

fn customer(ui: &mut Ui, order: &Order) {
    let c = Theme::current(ui.ctx()).colors;
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(32., 32.), Sense::hover());
        let (fill, ink) = [
            (c.accent.soft_hover, c.text_primary),
            (c.success.soft, c.success.ring),
            (c.danger.soft, c.danger.ring),
            (c.warning.soft, c.warning.ring),
            (c.default.base, c.text_primary),
        ][order.id as usize % 5];
        initials(ui, rect, &order.customer, fill, ink);
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
