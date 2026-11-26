use std::{
    cmp::Ordering,
    hash::{DefaultHasher, Hash, Hasher},
};

use eframe::egui::{self, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    display::Badge,
    icon::Icon,
    table::{Column, DataTable, Density, Sort, SortDirection, TableRows},
    typography::Type,
};

use crate::{shipments::Shipment, widgets};

use super::Dashboard;

pub(crate) enum RecordAction {
    Open(usize),
    Remove(usize),
}

impl Dashboard {
    pub(super) fn filtered<'a>(&self, shipments: &'a [Shipment]) -> Vec<(usize, &'a Shipment)> {
        let query = self.query.trim().to_lowercase();
        let mut rows: Vec<_> = shipments
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.matches(&query) && (!self.exceptions || item.status == "Needs review")
            })
            .collect();
        if let Some(sort) = self.table.sort {
            let columns = columns(self.show_carrier);
            if let Some(column) = columns.get(sort.column) {
                rows.sort_by(|(_, a), (_, b)| {
                    let order = compare(column.title, a, b);
                    if sort.direction == SortDirection::Asc {
                        order
                    } else {
                        order.reverse()
                    }
                });
            }
        }
        rows
    }

    pub(super) fn records(&mut self, ui: &mut Ui, shipments: &[Shipment]) -> Option<RecordAction> {
        ui.horizontal(|ui| {
            ui.label(Type::H2.text("Shipment records"));
            Badge::new(&shipments.len().to_string()).show(ui);
        });
        widgets::toolbar(
            ui,
            550.,
            |ui| {
                if widgets::pill(ui, "Filter", Icon::Filter, self.exceptions).clicked() {
                    self.exceptions = !self.exceptions;
                }
                if widgets::pill(ui, "Sort", Icon::ArrowUpDown, self.table.sort.is_some()).clicked()
                {
                    self.table.sort = if self.table.sort.is_some() {
                        None
                    } else {
                        Some(Sort {
                            column: 1,
                            direction: SortDirection::Asc,
                        })
                    };
                }
                if widgets::pill(ui, "Columns", Icon::Table, !self.show_carrier).clicked() {
                    self.show_carrier = !self.show_carrier;
                    self.table = Default::default();
                }
            },
            |ui| {
                widgets::sized_width(ui, 230., 38., |ui| {
                    widgets::search(ui, &mut self.query, "Search shipments…")
                });
            },
        );
        let query = self.query.trim().to_lowercase();
        let exceptions = self.exceptions;
        let columns = columns(self.show_carrier);
        let mut hasher = DefaultHasher::new();
        (&query, exceptions, self.show_carrier).hash(&mut hasher);
        for row in shipments {
            (&row.id, &row.name, &row.carrier, &row.hub, &row.status).hash(&mut hasher);
        }
        let rows = TableRows::new(
            hasher.finish(),
            shipments.len(),
            |i| {
                shipments[i].matches(&query)
                    && (!exceptions || shipments[i].status == "Needs review")
            },
            |i| i as u64,
            |column, a, b| compare(columns[column].title, &shipments[a], &shipments[b]),
        );
        let mut open = None;
        let table = DataTable {
            density: Density::Compact,
            height: 170.,
            ..DataTable::new("shipments", &columns)
        };
        table.show(ui, &mut self.table, rows, |ui, index, column| {
            let row = &shipments[index];
            match columns[column].title {
                "Status" => {
                    Badge {
                        tone: row.tone(),
                        ..Badge::new(&row.status)
                    }
                    .show(ui);
                }
                "Actions" => {
                    ui.spacing_mut().item_spacing.x = 3.;
                    for (label, icon, variant) in [
                        ("View shipment", Icon::Eye, Variant::Secondary),
                        ("Edit shipment", Icon::Pencil, Variant::Secondary),
                        ("Remove shipment", Icon::Trash2, Variant::DangerSoft),
                    ] {
                        if Button::new(label)
                            .icon_only(icon)
                            .variant(variant)
                            .size(Size::Sm)
                            .show(ui)
                            .clicked()
                        {
                            if variant == Variant::DangerSoft {
                                self.remove = Some(row.id.clone());
                            } else {
                                open = Some(RecordAction::Open(index));
                            }
                        }
                    }
                }
                title => {
                    let (role, value) = match title {
                        "Reference" => (Type::Mono, &row.id),
                        "Carrier" => (Type::Small, &row.carrier),
                        _ => (Type::Label, &row.name),
                    };
                    ui.add(egui::Label::new(role.text(value)).truncate());
                }
            }
        });
        self.confirm_remove(ui, shipments).or(open)
    }

    fn confirm_remove(&mut self, ui: &Ui, shipments: &[Shipment]) -> Option<RecordAction> {
        let id = self.remove.as_deref()?;
        let index = shipments.iter().position(|row| row.id == id)?;
        let theme = Theme::current(ui.ctx());
        let modal = egui::Modal::new(egui::Id::unique("spenstar-gallery/remove-shipment"))
            .backdrop_color(theme.colors.scrim)
            .show(ui.ctx(), |ui| {
                ui.label(Type::H2.text("Remove shipment?"));
                ui.label(
                    Type::Body.text(format!("{id} will be removed from the gallery records.")),
                );
                ui.horizontal(|ui| {
                    let cancel = Button::new("Cancel")
                        .variant(Variant::Secondary)
                        .show(ui)
                        .clicked();
                    let remove = Button::new("Remove")
                        .variant(Variant::Danger)
                        .show(ui)
                        .clicked();
                    (cancel, remove)
                })
                .inner
            });
        let (cancel, remove) = modal.inner;
        if cancel || remove || modal.should_close() {
            self.remove = None;
        }
        remove.then_some(RecordAction::Remove(index))
    }
}

fn columns(carrier: bool) -> Vec<Column<'static>> {
    let mut columns = vec![
        Column {
            min_width: 99.,
            ..Column::new("Reference", 0.6)
        },
        Column::new("Shipment", 2.2),
    ];
    if carrier {
        columns.push(Column::new("Carrier", 1.1));
    }
    columns.push(Column {
        min_width: 112.,
        ..Column::new("Status", 0.6)
    });
    columns.push(Column {
        min_width: 131.,
        ..Column::new("Actions", 0.1).unsorted()
    });
    columns
}

fn compare(column: &str, a: &Shipment, b: &Shipment) -> Ordering {
    match column {
        "Reference" => a.id.cmp(&b.id),
        "Carrier" => a.carrier.cmp(&b.carrier),
        "Status" => a.status.cmp(&b.status),
        _ => a.name.cmp(&b.name),
    }
}
