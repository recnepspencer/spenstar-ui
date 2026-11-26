use eframe::egui::{Align, Layout, ScrollArea, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Variant},
    datetime::{CalendarDate, CalendarOptions, DatePicker, DateRange, DateRangePicker},
    form::{Field, TextArea, TextField},
    icon::Icon,
    layout::{Card, ResponsiveGrid, Separator, SplitAxis, SplitPane},
    select::{MultiSelect, SelectOption, SelectSection, SelectStyle},
    typography::Type,
};

use crate::{
    forms::section,
    shipments::{HUBS, STATES, Shipment},
    widgets,
};

const CARRIERS: &[&str] = &["Northline Freight", "Summit Logistics", "Juniper Express"];

pub(super) struct Workspace {
    query: String,
    hub: Option<String>,
    statuses: Vec<String>,
    range: Option<DateRange>,
    position: f32,
    selected: Option<usize>,
    draft: Option<Shipment>,
    date: Option<CalendarDate>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            query: String::new(),
            hub: None,
            statuses: Vec::new(),
            range: None,
            position: 390.,
            selected: Some(0),
            draft: None,
            date: date(25),
        }
    }
}

impl Workspace {
    pub(super) fn open(&mut self, index: usize, rows: &[Shipment]) {
        self.selected = Some(index);
        self.draft = rows.get(index).cloned();
        self.date = self.draft.as_ref().and_then(|draft| date(draft.day));
    }

    pub(super) fn removed(&mut self, index: usize) {
        match self.selected {
            Some(selected) if selected == index => {
                self.selected = None;
                self.draft = None;
            }
            Some(selected) if selected > index => self.selected = Some(selected - 1),
            _ => {}
        }
    }

    pub(super) fn new_shipment(&mut self, shipments: &[Shipment]) {
        self.selected = None;
        self.date = date(25);
        self.draft = Some(Shipment {
            id: format!(
                "SHP-{:04}",
                shipments
                    .iter()
                    .filter_map(|row| row.id.strip_prefix("SHP-")?.parse::<u32>().ok())
                    .max()
                    .unwrap_or(482)
                    .saturating_add(1)
            ),
            name: String::new(),
            hub: HUBS[0].into(),
            carrier: CARRIERS[0].into(),
            status: "Ready".into(),
            notes: String::new(),
            day: 25,
        });
    }

    pub(super) fn show(&mut self, ui: &mut Ui, rows: &mut Vec<Shipment>) {
        if self.draft.is_none() && self.selected.is_some() {
            self.open(self.selected.unwrap_or_default(), rows);
        }
        self.filters(ui);
        let query = self.query.trim().to_lowercase();
        let shown: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                row.matches(&query)
                    && self.hub.as_ref().is_none_or(|hub| row.hub == *hub)
                    && (self.statuses.is_empty() || self.statuses.contains(&row.status))
                    && self
                        .range
                        .is_none_or(|range| date(row.day).is_some_and(|day| range.contains(day)))
            })
            .collect();

        let c = Theme::current(ui.ctx()).colors;
        ResponsiveGrid::new(140.)
            .max_columns(3)
            .show(ui, 3, |ui, index| {
                let (title, count) = match index {
                    0 => ("Matching shipments", shown.len()),
                    1 => (
                        "In transit",
                        shown
                            .iter()
                            .filter(|(_, item)| item.status == "In transit")
                            .count(),
                    ),
                    _ => (
                        "Needs review",
                        shown
                            .iter()
                            .filter(|(_, item)| item.status == "Needs review")
                            .count(),
                    ),
                };
                Card::default().show(ui, |card| {
                    card.body(|ui| {
                        ui.label(Type::Small.text(title).color(c.text_muted));
                        ui.label(Type::Metric.text(count.to_string()));
                    })
                });
            });
        let selected = self.selected;
        let mut next = None;
        let mut save = false;
        let mut discard = false;
        let draft = &mut self.draft;
        let date = &mut self.date;
        let mut list = |ui: &mut Ui| next = queue(ui, &shown, selected);
        let mut detail = |ui: &mut Ui| {
            (save, discard) = editor(ui, draft.as_mut(), date);
        };
        if ui.available_width() >= 660. {
            SplitPane::new("dispatch-panes", SplitAxis::Horizontal)
                .limits(280., 540., 350.)
                .show(ui, 536., &mut self.position, list, detail);
        } else {
            list(ui);
            detail(ui);
        }
        if let Some(index) = next {
            self.open(index, rows);
        } else if save && let Some(mut draft) = self.draft.clone() {
            draft.day = self.date.map_or(25, |date| date.day);
            match selected.and_then(|index| rows.get_mut(index)) {
                Some(row) => *row = draft,
                None => {
                    self.selected = Some(rows.len());
                    rows.push(draft);
                }
            }
            self.draft = None;
            crate::widgets::notify(
                ui.ctx(),
                "Shipment saved",
                spenstar_ui::feedback::Tone::Success,
            );
        } else if discard {
            self.draft = None;
        }
    }

    fn filters(&mut self, ui: &mut Ui) {
        let statuses: Vec<_> = STATES
            .iter()
            .map(|label| SelectOption::new(label, label))
            .collect();
        ResponsiveGrid::new(170.)
            .max_columns(4)
            .show(ui, 4, |ui, index| match index {
                0 => {
                    widgets::search(ui, &mut self.query, "Search the queue…");
                }
                1 => {
                    widgets::select(ui, &mut self.hub, HUBS, "All facilities");
                }
                2 => {
                    MultiSelect {
                        style: SelectStyle {
                            placeholder: "All statuses",
                            ..SelectStyle::default()
                        },
                        max_visible_chips: 1,
                        ..MultiSelect::default()
                    }
                    .show(
                        ui,
                        &mut self.statuses,
                        &[SelectSection {
                            label: None,
                            options: &statuses,
                        }],
                    );
                }
                _ => {
                    DateRangePicker {
                        options: calendar_options(),
                        style: SelectStyle {
                            placeholder: "Dispatch date",
                            ..SelectStyle::default()
                        },
                        ..DateRangePicker::default()
                    }
                    .show(ui, &mut self.range);
                }
            });
    }
}

fn queue(ui: &mut Ui, shown: &[(usize, &Shipment)], selected: Option<usize>) -> Option<usize> {
    let c = Theme::current(ui.ctx()).colors;
    section(ui, "Dispatch queue", |ui| {
        if shown.is_empty() {
            ui.label(
                Type::Small
                    .text("No shipments match these filters.")
                    .color(c.text_muted),
            );
        }
        ScrollArea::vertical()
            .max_height(430.)
            .show(ui, |ui| {
                let mut next = None;
                for &(index, shipment) in shown {
                    if ui
                        .push_id(index, |ui| {
                            let current = selected == Some(index);
                            let variant = if current {
                                Variant::Secondary
                            } else {
                                Variant::Ghost
                            };
                            let response = Button::new(&shipment.name)
                                .leading(Icon::Package)
                                .variant(variant)
                                .show(ui);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    Type::Caption
                                        .text(format!("{} · {}", shipment.id, shipment.hub))
                                        .color(c.text_muted),
                                );
                                widgets::tag(ui, &shipment.status, shipment.tone());
                            });
                            Separator::Horizontal.show(ui);
                            response.clicked()
                        })
                        .inner
                    {
                        next = Some(index);
                    }
                }
                next
            })
            .inner
    })
}

fn editor(
    ui: &mut Ui,
    draft: Option<&mut Shipment>,
    selected_date: &mut Option<CalendarDate>,
) -> (bool, bool) {
    let Some(draft) = draft else {
        section(ui, "Shipment details", |ui| {
            ui.label(Type::Body.text("Select a shipment in the dispatch queue."))
        });
        return (false, false);
    };
    section(ui, "Shipment details", |ui| {
        let c = Theme::current(ui.ctx()).colors;
        ui.horizontal(|ui| {
            ui.label(Type::Mono.text(&draft.id).color(c.text_muted));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::tag(ui, &draft.status, draft.tone())
            });
        });
        Field::new("Shipment name").show(ui, |ui, _| {
            TextField::new("Enter a name").show(ui, &mut draft.name)
        });
        ResponsiveGrid::new(145.)
            .max_columns(2)
            .show(ui, 4, |ui, index| match index {
                0 => choice(ui, "Facility", &mut draft.hub, HUBS),
                1 => choice(ui, "Carrier", &mut draft.carrier, CARRIERS),
                2 => choice(ui, "Status", &mut draft.status, STATES),
                _ => {
                    Field::new("Dispatch date").show(ui, |ui, _| {
                        let mut picker = DatePicker::default();
                        picker.field.options = calendar_options();
                        picker.show(ui, selected_date)
                    });
                }
            });
        Field::new("Handling notes").show(ui, |ui, _| {
            TextArea {
                min_rows: 2,
                placeholder: "Add handling instructions…",
                ..TextArea::default()
            }
            .show(ui, &mut draft.notes)
        });
        Separator::Horizontal.show(ui);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let save = Button::new("Save shipment")
                .leading(Icon::Check)
                .enabled(!draft.name.trim().is_empty() && selected_date.is_some())
                .show(ui)
                .clicked();
            let discard = Button::new("Discard")
                .variant(Variant::Secondary)
                .show(ui)
                .clicked();
            (save, discard)
        })
        .inner
    })
}

fn choice(ui: &mut Ui, label: &str, value: &mut String, options: &[&str]) {
    Field::new(label).show(ui, |ui, _| {
        let mut selected = Some(value.clone());
        if widgets::select(ui, &mut selected, options, label).changed()
            && let Some(selected) = selected
        {
            *value = selected;
        }
    });
}

fn date(day: u8) -> Option<CalendarDate> {
    CalendarDate::new(2026, 11, day)
}

fn calendar_options() -> CalendarOptions<'static> {
    CalendarOptions {
        today: date(25),
        min_date: date(1),
        max_date: date(30),
        ..CalendarOptions::default()
    }
}
