use egui::{Key, Response, Ui};

use crate::{
    button::{Button, Variant},
    form::{FieldVariant, Size, TextField, icon_button},
    icon::Icon,
    popover,
    select::{SelectStyle, trigger::Trigger},
};

use super::{Calendar, CalendarDate, CalendarOptions, DateRange, RangeCalendar};

/// Typed date editor with Gregorian validation and an optional date window.
#[derive(Clone, Copy, Default)]
pub struct DateField<'a> {
    /// Accepted dates.
    pub options: CalendarOptions<'a>,
    /// Field treatment.
    pub variant: FieldVariant,
    /// Field height.
    pub size: Size,
    /// Draw error styling from external validation.
    pub invalid: bool,
    /// Allow reading and copying.
    pub read_only: bool,
}

/// Output from a typed date field or picker.
pub struct DateFieldResponse {
    /// Text editor response.
    pub response: Response,
    /// Draft is invalid or the caller marked the field invalid.
    pub invalid: bool,
    /// Whether the calendar is open.
    pub open: bool,
}

#[derive(Clone)]
struct Draft {
    date: Option<CalendarDate>,
    text: String,
}

impl DateField<'_> {
    /// Render an editor that validates and commits the date on Enter or focus loss.
    pub fn show(self, ui: &mut Ui, date: &mut Option<CalendarDate>) -> DateFieldResponse {
        self.render(ui, date, false)
    }

    fn render(
        self,
        ui: &mut Ui,
        date: &mut Option<CalendarDate>,
        picker: bool,
    ) -> DateFieldResponse {
        ui.scope(|ui| {
            let id = ui.unique_id().with("date-draft");
            let previous = *date;
            let mut draft = ui
                .ctx()
                .data(|d| d.get_temp::<Draft>(id))
                .filter(|draft| draft.date == previous)
                .unwrap_or_else(|| Draft {
                    date: previous,
                    text: previous.map_or_else(String::new, |d| d.to_string()),
                });
            let parse = |text: &str| {
                let text = text.trim();
                if text.is_empty() {
                    return Some(None);
                }
                let date = text.parse::<CalendarDate>().ok()?;
                self.options.allows(date).then_some(Some(date))
            };
            let calendar = picker && !self.read_only;
            let width = if calendar {
                22. + ui.spacing().item_spacing.x
            } else {
                0.
            };
            let (input, toggle) = TextField {
                variant: self.variant,
                size: self.size,
                placeholder: "YYYY-MM-DD",
                read_only: self.read_only,
                disabled: self.options.disabled,
                invalid: self.invalid || parse(&draft.text).is_none(),
                ..TextField::default()
            }
            .render(ui, &mut draft.text, None, width, |ui| {
                calendar && icon_button(ui, Icon::Calendar, "Open calendar").clicked()
            });
            let mut response = input.response;
            let parsed = parse(&draft.text);
            if response.enabled()
                && !self.read_only
                && (input.submitted || response.lost_focus())
                && let Some(value) = parsed
            {
                *date = value;
                draft.date = value;
                draft.text = value.map_or_else(String::new, |d| d.to_string());
            }
            let mut anchor = response.clone();
            anchor.rect = input.frame;
            anchor.interact_rect = input.frame.intersect(ui.clip_rect());
            if toggle {
                popover::toggle(&anchor);
            }
            if calendar
                && response.has_focus()
                && response.enabled()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::ALT, Key::ArrowDown))
            {
                popover::open(&anchor);
            }
            let mut invalid = self.invalid || parsed.is_none();
            if calendar {
                if let Some(popup) = popover::show(&anchor, 336., "Date picker", |ui| {
                    let out = Calendar {
                        options: self.options,
                    }
                    .show(ui, date);
                    let today = self.options.today();
                    if ui
                        .add_enabled_ui(self.options.allows(today), |ui| {
                            Button::new("Today")
                                .size(Size::Sm)
                                .variant(Variant::Ghost)
                                .show(ui)
                                .clicked()
                        })
                        .inner
                    {
                        *date = Some(today);
                        true
                    } else {
                        out.completed
                    }
                }) && popup.inner
                {
                    draft.date = *date;
                    draft.text = date.map_or_else(String::new, |d| d.to_string());
                    invalid = self.invalid;
                    popover::close(&anchor, true);
                }
            } else {
                popover::close(&anchor, false);
            }
            if previous != *date {
                response.mark_changed();
            }
            ui.ctx().data_mut(|d| d.insert_temp(id, draft));
            DateFieldResponse {
                response,
                invalid,
                open: popover::is_open(&anchor),
            }
        })
        .inner
    }
}

/// Typed input with a calendar popup and a Today action.
#[derive(Clone, Copy, Default)]
pub struct DatePicker<'a> {
    /// Editor appearance and date constraints.
    pub field: DateField<'a>,
}

impl DatePicker<'_> {
    /// Render the date input and calendar trigger.
    pub fn show(self, ui: &mut Ui, date: &mut Option<CalendarDate>) -> DateFieldResponse {
        self.field.render(ui, date, true)
    }
}

/// A date range calculated relative to the calendar's today.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePreset {
    /// The current day.
    Today,
    /// Today and the previous six days.
    Last7Days,
    /// Today and the previous 29 days.
    Last30Days,
    /// The first through last days of the current month.
    ThisMonth,
}

impl DatePreset {
    /// Visible name.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Last7Days => "Last 7 days",
            Self::Last30Days => "Last 30 days",
            Self::ThisMonth => "This month",
        }
    }

    /// Calculate the preset and clamp it to the date window.
    pub fn range(self, options: CalendarOptions<'_>) -> Option<DateRange> {
        let today = options.today();
        let (start, end) = match self {
            Self::Today => (today, today),
            Self::Last7Days => (today.add_days(-6).unwrap_or(CalendarDate::MIN), today),
            Self::Last30Days => (today.add_days(-29).unwrap_or(CalendarDate::MIN), today),
            Self::ThisMonth => (
                CalendarDate { day: 1, ..today },
                CalendarDate {
                    day: CalendarDate::days_in_month(today.year, today.month),
                    ..today
                },
            ),
        };
        // Intersect with the window so a preset entirely outside it stays unavailable.
        let start = start.max(options.min_date.unwrap_or(CalendarDate::MIN));
        let end = end.min(options.max_date.unwrap_or(CalendarDate::MAX));
        (start <= end && options.allows(start) && options.allows(end))
            .then(|| DateRange::new(start, end))
    }
}

/// A range trigger with a calendar popup and optional quick presets.
#[derive(Clone)]
pub struct DateRangePicker<'a> {
    /// Calendar constraints.
    pub options: CalendarOptions<'a>,
    /// Trigger appearance.
    pub style: SelectStyle<'a>,
    /// Presets to display in the order provided.
    pub presets: &'a [DatePreset],
}

/// Output from a date range picker.
pub struct DateRangeResponse {
    /// Trigger response. Changed indicates the range was updated.
    pub response: Response,
    /// Whether the range popup is open.
    pub open: bool,
}

impl Default for DateRangePicker<'_> {
    fn default() -> Self {
        Self {
            options: CalendarOptions::default(),
            style: SelectStyle {
                placeholder: "Choose dates…",
                ..SelectStyle::default()
            },
            presets: &[],
        }
    }
}

impl DateRangePicker<'_> {
    /// Render a calendar popup. A range or preset selection closes it.
    pub fn show(mut self, ui: &mut Ui, range: &mut Option<DateRange>) -> DateRangeResponse {
        let previous = *range;
        let label = range.map_or_else(
            || self.style.placeholder.to_owned(),
            |range| format!("{}  –  {}", range.start, range.end),
        );
        self.style.disabled |= self.options.disabled;
        let (mut response, _) = Trigger::new(&self.style, &label, range.is_none())
            .indicator(Icon::Calendar)
            .show(ui);
        if let Some(output) = popover::show(&response, 336., "Date range picker", |ui| {
            let result = RangeCalendar {
                options: self.options,
            }
            .show(ui, range);
            let chosen = ui
                .horizontal_wrapped(|ui| {
                    self.presets.iter().find_map(|&preset| {
                        let window = preset.range(self.options);
                        let pressed = Button::new(preset.label())
                            .size(Size::Sm)
                            .variant(Variant::Ghost)
                            .enabled(window.is_some())
                            .show(ui)
                            .clicked();
                        if pressed { window } else { None }
                    })
                })
                .inner;
            if let Some(chosen) = chosen {
                *range = Some(chosen);
            }
            result.completed || chosen.is_some()
        }) && output.inner
        {
            popover::close(&response, true);
        }
        if previous != *range {
            response.mark_changed();
        }
        DateRangeResponse {
            open: popover::is_open(&response),
            response,
        }
    }
}
