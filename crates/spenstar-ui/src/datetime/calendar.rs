use egui::{Response, Ui};

use super::{CalendarDate, grid};

/// A range of civil dates, with inclusive start and end dates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRange {
    /// Inclusive start.
    pub start: CalendarDate,
    /// Inclusive end.
    pub end: CalendarDate,
}

impl DateRange {
    /// Order the two dates to create an inclusive range.
    pub fn new(first: CalendarDate, second: CalendarDate) -> Self {
        Self {
            start: first.min(second),
            end: first.max(second),
        }
    }

    /// Whether the date is inside the inclusive range.
    pub fn contains(self, date: CalendarDate) -> bool {
        (self.start..=self.end).contains(&date)
    }
}

/// Shared date constraints and the date indicated as today.
#[derive(Clone, Copy, Default)]
pub struct CalendarOptions<'a> {
    /// Use a caller-provided local date. The default uses the current UTC date.
    pub today: Option<CalendarDate>,
    /// Earliest selectable day, inclusive.
    pub min_date: Option<CalendarDate>,
    /// Latest selectable day, inclusive.
    pub max_date: Option<CalendarDate>,
    /// Additional filter for selectable days and range endpoints.
    pub disabled_date: Option<&'a dyn Fn(CalendarDate) -> bool>,
    /// Disable interaction.
    pub disabled: bool,
}

impl CalendarOptions<'_> {
    /// Date marked as today.
    pub fn today(self) -> CalendarDate {
        self.today
            .filter(|date| valid(*date))
            .unwrap_or_else(CalendarDate::today_utc)
    }

    /// Whether the date is valid and can be selected under these constraints.
    pub fn allows(self, date: CalendarDate) -> bool {
        valid(date)
            && self.min_date.is_none_or(|min| date >= min)
            && self.max_date.is_none_or(|max| date <= max)
            && self.disabled_date.is_none_or(|filter| !filter(date))
    }

    pub(super) fn bounded(self, date: CalendarDate) -> CalendarDate {
        let date = if valid(date) { date } else { self.today() };
        let min = self
            .min_date
            .filter(|date| valid(*date))
            .unwrap_or(CalendarDate::MIN);
        let max = self
            .max_date
            .filter(|date| valid(*date))
            .unwrap_or(CalendarDate::MAX);
        date.max(min).min(max)
    }
}

fn valid(date: CalendarDate) -> bool {
    CalendarDate::new(date.year, date.month, date.day).is_some()
}

/// A single-date month calendar.
#[derive(Clone, Copy, Default)]
pub struct Calendar<'a> {
    /// Shared date constraints.
    pub options: CalendarOptions<'a>,
}

/// A two-step range calendar. Selecting the second date commits the ordered range.
#[derive(Clone, Copy, Default)]
pub struct RangeCalendar<'a> {
    /// Shared date constraints. The disabled predicate applies to endpoints.
    pub options: CalendarOptions<'a>,
}

/// Output from a single or range calendar.
pub struct CalendarResponse {
    /// Calendar response. Changed indicates a committed value change.
    pub response: Response,
    /// Date activated this frame.
    pub activated: Option<CalendarDate>,
    /// A single date or both endpoints of a range were chosen.
    pub completed: bool,
}

impl Calendar<'_> {
    /// Render a month grid and update the chosen date.
    pub fn show(self, ui: &mut Ui, selected: &mut Option<CalendarDate>) -> CalendarResponse {
        let previous = *selected;
        let mut out = grid::show(
            ui,
            self.options,
            previous,
            previous.map(|day| DateRange::new(day, day)),
            None,
        );
        if let Some(date) = out.activated {
            *selected = Some(date);
        }
        if previous != *selected {
            out.response.mark_changed();
        }
        out
    }
}

impl RangeCalendar<'_> {
    /// Render the grid and preview the range until a second endpoint is selected.
    pub fn show(self, ui: &mut Ui, selected: &mut Option<DateRange>) -> CalendarResponse {
        let previous = *selected;
        let ordered = previous.map(|range| DateRange::new(range.start, range.end));
        let id = ui.next_auto_id().with("range-anchor");
        let frame = ui.ctx().cumulative_frame_nr();
        let pending = ui
            .ctx()
            .data(|d| d.get_temp::<(u64, Option<CalendarDate>)>(id))
            .filter(|(last, _)| frame.saturating_sub(*last) <= 1)
            .and_then(|(_, date)| date)
            .filter(|&date| self.options.allows(date));
        let mut out = grid::show(ui, self.options, ordered.map(|r| r.start), ordered, pending);
        let mut next = pending;
        if let Some(date) = out.activated {
            if let Some(start) = pending {
                *selected = Some(DateRange::new(start, date));
                next = None;
            } else {
                next = Some(date);
            }
        }
        ui.ctx().data_mut(|d| d.insert_temp(id, (frame, next)));
        out.completed = out.activated.is_some() && pending.is_some();
        if previous != *selected {
            out.response.mark_changed();
        }
        out
    }
}
