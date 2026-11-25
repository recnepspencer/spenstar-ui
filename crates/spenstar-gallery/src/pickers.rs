use eframe::egui::Ui;
use spenstar_ui::{
    datetime::{
        Calendar, CalendarDate, CalendarOptions, ClockFormat, ClockTime, DateField, DatePicker,
        DatePreset, DateRange, DateRangePicker, TimeField,
    },
    form::{Field, FieldVariant},
    icon::Icon,
    layout::ResponsiveGrid,
    select::{ComboBox, MultiSelect, Select, SelectOption, SelectSection, SelectStyle},
    typography::Type,
};

use super::forms::section;

const PRESETS: &[DatePreset] = &[
    DatePreset::Today,
    DatePreset::Last7Days,
    DatePreset::Last30Days,
    DatePreset::ThisMonth,
];

pub(crate) struct Demo {
    select: Option<String>,
    multi: Vec<String>,
    autocomplete: String,
    custom_options: Vec<String>,
    recent: String,
    calendar: Option<CalendarDate>,
    date: Option<CalendarDate>,
    date_text: Option<CalendarDate>,
    range: Option<DateRange>,
    morning: ClockTime,
    evening: ClockTime,
}

impl Default for Demo {
    fn default() -> Self {
        let today = CalendarDate::today_utc();
        Self {
            select: Some("north".into()),
            multi: ["north", "west", "central"].map(str::to_owned).into(),
            autocomplete: String::new(),
            custom_options: Vec::new(),
            recent: String::new(),
            calendar: Some(today),
            date: None,
            date_text: None,
            range: None,
            morning: ClockTime {
                hour: 9,
                minute: 30,
                second: 0,
            },
            evening: ClockTime {
                hour: 17,
                minute: 45,
                second: 15,
            },
        }
    }
}

impl Demo {
    pub(crate) fn show(&mut self, ui: &mut Ui, open_select: bool) {
        ResponsiveGrid::new(308.)
            .max_columns(2)
            .show(ui, 4, |ui, index| match index {
                0 => self.selectors(ui, open_select),
                1 => self.calendar(ui, open_select),
                2 => self.time(ui),
                _ => self.dates(ui),
            });
    }

    fn selectors(&mut self, ui: &mut Ui, open_select: bool) {
        section(ui, "Dropdowns & autocomplete", |ui| {
            let primary = [
                SelectOption {
                    icon: Some(Icon::Truck.into()),
                    ..SelectOption::new("north", "North depot")
                },
                SelectOption {
                    icon: Some(Icon::Package.into()),
                    ..SelectOption::new("west", "West warehouse")
                },
            ];
            let partners = [
                SelectOption {
                    icon: Some(Icon::Globe.into()),
                    ..SelectOption::new("central", "Central hub")
                },
                SelectOption {
                    disabled: true,
                    ..SelectOption::new("east", "East hub (offline)")
                },
            ];
            let routes = [
                SelectSection {
                    label: Some("Owned facilities"),
                    options: &primary,
                },
                SelectSection {
                    label: Some("Partner facilities"),
                    options: &partners,
                },
            ];
            Field::new("Authorized facilities").show(ui, |ui, _| {
                MultiSelect {
                    max_visible_chips: 2,
                    show_count: true,
                    ..MultiSelect::default()
                }
                .show(ui, &mut self.multi, &routes);
            });
            let custom = self
                .custom_options
                .iter()
                .map(|name| SelectOption::new(name, name))
                .collect::<Vec<_>>();
            let sections = [
                routes[0],
                routes[1],
                SelectSection {
                    label: Some("Custom facilities"),
                    options: &custom,
                },
            ];
            let selected = Field {
                description: Some("Type to filter, or choose Create to save a facility."),
                ..Field::new("Find a facility")
            }
            .show(ui, |ui, _| {
                ComboBox {
                    allow_custom: true,
                    style: SelectStyle {
                        variant: FieldVariant::Filled,
                        ..ComboBox::default().style
                    },
                }
                .show(ui, &mut self.autocomplete, &sections)
            })
            .inner;
            if let Some(name) = selected.created {
                self.recent = format!("Added {name}");
                self.custom_options.push(name);
            } else if selected.selected.is_some() {
                self.recent = format!("Selected {}", self.autocomplete);
            }
            if !self.recent.is_empty() {
                ui.label(Type::Small.text(&self.recent));
            }
            Field::new("Primary facility").show_with(
                ui,
                |_| {},
                |ui, _, id| {
                    let selected = Select {
                        style: SelectStyle {
                            leading: Some(Icon::Folder.into()),
                            ..SelectStyle::default()
                        },
                        clearable: true,
                    }
                    .show(ui, &mut self.select, &routes);
                    if open_select && !selected.open {
                        eframe::egui::Popup::open_id(
                            ui.ctx(),
                            eframe::egui::Popup::default_response_id(&selected.response),
                        );
                    }
                    selected.response.labelled_by(id);
                },
            );
        });
    }

    fn calendar(&mut self, ui: &mut Ui, capturing: bool) {
        section(ui, "Inline calendar", |ui| {
            let mut calendar = Calendar::default();
            if capturing {
                calendar.options.today = CalendarDate::new(2026, 11, 25);
                self.calendar = calendar.options.today;
            }
            calendar.show(ui, &mut self.calendar);
            if let Some(date) = self.calendar {
                ui.label(Type::Small.text(format!("Selected {date}")));
            }
        });
    }

    fn dates(&mut self, ui: &mut Ui) {
        section(ui, "Date pickers", |ui| {
            let today = CalendarDate::today_utc();
            let options = CalendarOptions {
                today: Some(today),
                min_date: today.add_months(-3),
                max_date: today.add_months(6),
                ..CalendarOptions::default()
            };
            let mut field = DateField {
                options,
                ..DateField::default()
            };
            Field::new("Delivery date").show(ui, |ui, _| {
                if (DatePicker { field }).show(ui, &mut self.date).invalid {
                    ui.label(Type::Small.text("Enter a date within the scheduling window."));
                }
            });
            field.variant = FieldVariant::Underlined;
            Field::new("Manual date entry").show(ui, |ui, _| {
                if field.show(ui, &mut self.date_text).invalid {
                    ui.label(Type::Small.text("Use YYYY-MM-DD within the scheduling window."));
                }
            });
            Field::new("Reporting period").show(ui, |ui, _| {
                DateRangePicker {
                    options,
                    presets: PRESETS,
                    ..DateRangePicker::default()
                }
                .show(ui, &mut self.range);
            });
        });
    }

    fn time(&mut self, ui: &mut Ui) {
        section(ui, "Time fields", |ui| {
            Field::new("Pickup time · 12-hour").show(ui, |ui, _| {
                TimeField {
                    format: ClockFormat::H12,
                    ..TimeField::default()
                }
                .show(ui, &mut self.morning);
            });
            Field::new("Cutoff time · 24-hour").show(ui, |ui, _| {
                TimeField {
                    seconds: true,
                    ..TimeField::default()
                }
                .show(ui, &mut self.evening);
            });
        });
    }
}
