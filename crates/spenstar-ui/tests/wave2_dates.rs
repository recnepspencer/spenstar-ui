//! Gregorian arithmetic and headless date and time interaction tests.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    datetime::{
        CalendarDate, CalendarOptions, ClockFormat, ClockTime, DateField, DatePicker, DatePreset,
        DateRange, DateRangePicker, TimeField,
    },
    egui::{Event, Key, Modifiers, Ui, pos2},
    theme::Mode,
};
use support::{Harness, center, key, key_with, node};

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    let Some(date) = CalendarDate::new(year, month, day) else {
        panic!("invalid test date");
    };
    date
}

#[test]
fn gregorian_math_round_trips_and_handles_era_boundaries() {
    assert_eq!(CalendarDate::new(1900, 2, 29), None);
    assert_eq!(date(1900, 2, 28).add_days(1), Some(date(1900, 3, 1)));
    assert_eq!(date(2000, 2, 28).add_days(1), Some(date(2000, 2, 29)));
    assert_eq!(date(2024, 2, 29).add_days(1), Some(date(2024, 3, 1)));
    assert_eq!(date(2024, 1, 31).add_months(1), Some(date(2024, 2, 29)));
    assert_eq!(date(2023, 12, 31).add_months(2), Some(date(2024, 2, 29)));
    assert_eq!(date(2024, 3, 31).add_months(-1), Some(date(2024, 2, 29)));
    assert_eq!(date(0, 1, 1).add_days(-1), Some(date(-1, 12, 31)));
    assert_eq!(date(1970, 1, 1).weekday(), 3);
    assert_eq!(date(2000, 1, 1).weekday(), 5);
    assert_eq!(date(2024, 2, 29).weekday(), 3);
    assert_eq!(CalendarDate::MIN.add_days(-1), None);
    assert_eq!(CalendarDate::MAX.add_days(1), None);
    assert_eq!(CalendarDate::MAX.add_months(i64::MAX), None);
    for value in [
        CalendarDate::MIN,
        date(-400, 2, 29),
        date(0, 1, 1),
        date(2024, 2, 29),
        CalendarDate::MAX,
    ] {
        assert_eq!(value.to_string().parse(), Ok(value));
        assert_eq!(value.add_days(0), Some(value));
    }
    for invalid in [
        "1900-02-29",
        "2025-02-29",
        "2024-2-29",
        "2024-13-01",
        "2024-01-00",
        "2024-01-32",
        "2024/01/01",
    ] {
        assert!(invalid.parse::<CalendarDate>().is_err());
    }
}

#[test]
fn clock_formats_cover_midnight_noon_seconds_and_invalid_values() {
    let midnight = ClockTime::default();
    let noon = ClockTime::new(12, 0, 0);
    assert_eq!(midnight.format(ClockFormat::H12, false), "12:00 AM");
    assert_eq!(ClockTime::parse("12:00 am", ClockFormat::H12), Ok(midnight));
    assert_eq!(ClockTime::parse("12:00 PM", ClockFormat::H12).ok(), noon);
    assert_eq!(
        ClockTime::parse("11:59:58 pm", ClockFormat::H12).ok(),
        ClockTime::new(23, 59, 58)
    );
    assert!("24:00".parse::<ClockTime>().is_err());
    assert!("23:60".parse::<ClockTime>().is_err());
    assert!("22:20:60".parse::<ClockTime>().is_err());
    assert!(ClockTime::parse("00:00 AM", ClockFormat::H12).is_err());
}

#[test]
fn range_picker_rejects_outside_days_and_orders_reverse_endpoints() {
    let mut h = Harness::new(Mode::Dark);
    let mut range = None;
    let options = CalendarOptions {
        today: Some(date(2024, 2, 15)),
        min_date: Some(date(2024, 2, 10)),
        max_date: Some(date(2024, 2, 20)),
        disabled_date: Some(&|value| value.day == 13),
        ..Default::default()
    };
    let mut render = |ui: &mut Ui| {
        ui.set_max_width(310.);
        let picker = DateRangePicker {
            options,
            presets: &[DatePreset::Last7Days],
            ..Default::default()
        };
        let out = picker.show(ui, &mut range);
        (out, range)
    };
    let (_, (closed, _)) = h.draw(vec![], &mut render);
    let (_, (open, _)) = h.click(closed.response.rect.center(), &mut render);
    assert!(open.open);
    let (output, _) = h.draw(vec![], &mut render);
    for day in ["2024-02-09", "2024-02-13", "2024-02-21"] {
        assert!(node(&output, day).is_disabled());
        let (_, (out, selected)) = h.click(center(&output, day), &mut render);
        assert!(out.open);
        assert_eq!(selected, None);
    }
    let (_, (first, selected)) = h.click(center(&output, "2024-02-19"), &mut render);
    assert!(first.open);
    assert_eq!(selected, None);
    let (_, (last, selected)) = h.click(center(&output, "2024-02-11"), &mut render);
    assert!(!last.open && last.response.changed());
    assert_eq!(
        selected,
        Some(DateRange::new(date(2024, 2, 11), date(2024, 2, 19)))
    );
    assert_eq!(
        DatePreset::Last7Days.range(options),
        Some(DateRange::new(date(2024, 2, 10), date(2024, 2, 15)))
    );
}

#[test]
fn typed_date_rejects_bad_text_and_date_picker_today_repairs_it() {
    let mut h = Harness::new(Mode::Light);
    let today = date(2024, 2, 29);
    let mut value = None;
    let mut render = |ui: &mut Ui| {
        let field = DateField {
            options: CalendarOptions {
                today: Some(today),
                ..Default::default()
            },
            ..Default::default()
        };
        let out = DatePicker { field }.show(ui, &mut value);
        (out, value)
    };
    let (_, (out, _)) = h.draw(vec![], &mut render);
    h.focus(out.response.id);
    h.draw(vec![Event::Text("2023-02-29".into())], &mut render);
    let (_, (invalid, value)) = h.draw(key(Key::Enter), &mut render);
    assert!(invalid.invalid);
    assert_eq!(value, None);
    h.draw(key_with(Key::A, Modifiers::COMMAND), &mut render);
    h.draw(vec![Event::Text("2000-02-29".into())], &mut render);
    let (_, (valid, value)) = h.draw(key(Key::Enter), &mut render);
    assert!(!valid.invalid);
    assert_eq!(value, Some(date(2000, 2, 29)));
    let (output, _) = h.draw(vec![], &mut render);
    h.click(center(&output, "Open calendar"), &mut render);
    let (output, _) = h.draw(vec![], &mut render);
    let (_, (today_out, value)) = h.click(center(&output, "Today"), &mut render);
    assert!(!today_out.open && !today_out.invalid);
    assert_eq!(value, Some(today));
}

#[test]
fn time_steps_at_midnight_and_toggles_meridiem() {
    let mut h = Harness::new(Mode::Light);
    let mut time = ClockTime {
        hour: 23,
        minute: 59,
        second: 0,
    };
    let mut render = |ui: &mut Ui| {
        let out = TimeField {
            format: ClockFormat::H12,
            ..Default::default()
        }
        .show(ui, &mut time);
        (out, time)
    };
    let (frame, _) = h.draw(vec![], &mut render);
    h.click(center(&frame, "Minute"), &mut render);
    let (_, (_, value)) = h.draw(key(Key::ArrowUp), &mut render);
    assert_eq!(value, ClockTime::default());
    let (frame, _) = h.draw(vec![], &mut render);
    let (_, (_, value)) = h.click(center(&frame, "AM"), &mut render);
    assert_eq!(Some(value), ClockTime::new(12, 0, 0));
    h.click(pos2(450., 50.), &mut render);
}

#[test]
fn calendar_arrows_cross_months_and_skip_disabled_days() {
    use spenstar_ui::datetime::Calendar;

    let mut h = Harness::new(Mode::Light);
    let today = date(2024, 3, 1);
    let options = CalendarOptions {
        today: Some(today),
        disabled_date: Some(&|day| day == date(2024, 2, 29)),
        ..Default::default()
    };
    let mut selected = Some(today);
    let mut render = |ui: &mut Ui| {
        Calendar { options }.show(ui, &mut selected);
        selected
    };
    let (frame, _) = h.draw(vec![], &mut render);
    h.click(center(&frame, "2024-03-01"), &mut render);
    let (_, still_selected) = h.draw(key(Key::ArrowLeft), &mut render);
    assert_eq!(still_selected, Some(today));
    let (_, picked) = h.draw(key(Key::Enter), &mut render);
    assert_eq!(picked, Some(date(2024, 2, 28)));
}

#[test]
fn calendar_aligns_adjacent_dates_to_the_sunday_first_grid() {
    use spenstar_ui::datetime::Calendar;

    let mut harness = Harness::new(Mode::Light);
    let today = date(2026, 11, 25);
    let mut selected = Some(today);
    let (frame, _) = harness.draw(vec![], |ui| {
        Calendar {
            options: CalendarOptions {
                today: Some(today),
                ..Default::default()
            },
        }
        .show(ui, &mut selected)
    });
    let first = center(&frame, "2026-11-01");
    let second = center(&frame, "2026-11-02");
    let adjacent = center(&frame, "2026-12-05");
    assert!(first.x < second.x);
    assert_eq!(first.y, second.y);
    assert_eq!(center(&frame, "2026-11-07").x, adjacent.x);
    assert_eq!(adjacent.y - first.y, 4. * 36.);
    let menu = center(&frame, "Next month");
    assert!(menu.x > center(&frame, "2026-11-06").x && menu.y < first.y);
}
