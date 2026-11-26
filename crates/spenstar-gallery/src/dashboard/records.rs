use eframe::egui::{self, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    controls::Tone,
    icon::Icon,
    layout::{Card, Separator},
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
        if self.sorted {
            rows.sort_by_key(|(_, item)| item.name.as_str());
        }
        rows
    }

    pub(super) fn records(&mut self, ui: &mut Ui, shipments: &[Shipment]) -> Option<RecordAction> {
        let c = Theme::current(ui.ctx()).colors;
        ui.horizontal(|ui| {
            ui.label(Type::H2.text("Shipment records"));
            widgets::tag(ui, &shipments.len().to_string(), Tone::Neutral);
        });
        widgets::toolbar(
            ui,
            550.,
            |ui| {
                if widgets::pill(ui, "Filter", Icon::Filter, self.exceptions).clicked() {
                    self.exceptions = !self.exceptions;
                }
                if widgets::pill(ui, "Sort", Icon::ArrowUpDown, self.sorted).clicked() {
                    self.sorted = !self.sorted;
                }
                if widgets::pill(ui, "Columns", Icon::Table, !self.show_carrier).clicked() {
                    self.show_carrier = !self.show_carrier;
                }
            },
            |ui| {
                widgets::sized_width(ui, 230., 38., |ui| {
                    widgets::search(ui, &mut self.query, "Search shipments…")
                });
            },
        );
        let rows = self.filtered(shipments);
        let open = Card::default()
            .show(ui, |card| {
                card.body(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.;
                    let available = ui.available_width();
                    let carrier = self.show_carrier && available >= 620.;
                    let status = available >= 430.;
                    let id_width = if available >= 360. { 104. } else { 0. };
                    let name_width = (available
                        - id_width
                        - 126.
                        - if carrier { 148. } else { 0. }
                        - if status { 126. } else { 0. })
                    .max(42.);
                    let widths = [
                        id_width,
                        name_width,
                        if carrier { 140. } else { 0. },
                        if status { 118. } else { 0. },
                        108.,
                    ];
                    let band = ui.painter().add(egui::Shape::Noop);
                    let header = ui.horizontal(|ui| {
                        for (heading, width) in
                            ["Reference", "Shipment", "Carrier", "Status", "Actions"]
                                .into_iter()
                                .zip(widths)
                        {
                            if width > 0. {
                                widgets::width(ui, width, |ui| {
                                    ui.add_space(3.);
                                    ui.label(Type::Badge.text(heading).color(c.text_muted));
                                    ui.add_space(3.);
                                });
                            }
                        }
                    });
                    ui.painter().set(
                        band,
                        egui::Shape::rect_filled(header.response.rect, 8, c.surface_subtle),
                    );
                    let mut selected = None;
                    if rows.is_empty() {
                        ui.label(Type::Body.text("No matching shipments").color(c.text_muted));
                    }
                    for (index, shipment) in rows {
                        Separator::Horizontal.show(ui);
                        ui.horizontal(|ui| {
                            for (index, (value, width)) in
                                [&shipment.id, &shipment.name, &shipment.carrier]
                                    .into_iter()
                                    .zip(widths)
                                    .enumerate()
                            {
                                if width > 0. {
                                    widgets::width(ui, width, |ui| {
                                        let role = if index == 0 { Type::Mono } else { Type::Body };
                                        ui.add(egui::Label::new(role.text(value)).truncate());
                                    });
                                }
                            }
                            if status {
                                widgets::width(ui, widths[3], |ui| {
                                    widgets::tag(ui, &shipment.status, shipment.tone())
                                });
                            }
                            ui.spacing_mut().item_spacing.x = 4.;
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
                                        self.remove = Some(shipment.id.clone());
                                    } else {
                                        selected = Some(RecordAction::Open(index));
                                    }
                                }
                            }
                        });
                    }
                    selected
                })
            })
            .inner;
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
