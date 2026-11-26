use egui::{
    Align2, Key, Modifiers, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, pos2,
    vec2,
};

use crate::{
    Theme,
    button::{Button, Size, Variant},
    icon::Icon,
    input::lock_arrows,
    motion,
    typography::Type,
};

use super::{CalendarDate, CalendarOptions, CalendarResponse, DateRange};

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const ROW: f32 = 36.;

#[derive(Clone, Copy)]
struct View {
    cursor: CalendarDate,
    selection: Option<CalendarDate>,
    keyboard: bool,
}

pub(super) fn show(
    ui: &mut Ui,
    options: CalendarOptions<'_>,
    selection: Option<CalendarDate>,
    selected: Option<DateRange>,
    pending: Option<CalendarDate>,
) -> CalendarResponse {
    let out = ui.scope(|ui| {
        if options.disabled {
            ui.disable();
        }
        let id = ui.unique_id().with("month-view");
        let initial = options.bounded(selection.unwrap_or_else(|| options.today()));
        let mut view = ui
            .ctx()
            .data(|data| data.get_temp::<View>(id))
            .filter(|view| view.selection == selection)
            .unwrap_or(View {
                cursor: initial,
                selection,
                keyboard: false,
            });
        view.cursor = options.bounded(view.cursor);
        let focus = ui.memory(|m| m.focused());
        let focus_day = focus.and_then(|focused| {
            month_days(view.cursor)
                .flatten()
                .find(|day| id.with(day) == focused)
        });
        let width = ui.available_width().clamp(0., 336.);
        let month_delta = header(ui, width, view.cursor, options);
        if month_delta != 0
            && let Some(target) = view.cursor.add_months(month_delta)
        {
            view.cursor = options.bounded(target);
            view.keyboard = false;
        }
        let mut request_focus = false;
        if let Some(day) = focus_day
            && ui.is_enabled()
        {
            let delta = ui.input_mut(|input| {
                [
                    (Key::ArrowLeft, -1),
                    (Key::ArrowRight, 1),
                    (Key::ArrowUp, -7),
                    (Key::ArrowDown, 7),
                ]
                .into_iter()
                .map(|(key, days)| days * input.count_and_consume_key(Modifiers::NONE, key) as i64)
                .sum::<i64>()
            });
            if delta != 0 {
                ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
                view.cursor = next_day(day, delta, options);
                view.keyboard = true;
                request_focus = true;
            }
        }
        let dates = month_days(view.cursor);
        let (rect, _) = ui.allocate_exact_size(
            vec2(width, ROW * (1 + dates.len() / 7) as f32),
            Sense::hover(),
        );
        let column = width / 7.;
        let c = Theme::current(ui.ctx()).colors;
        for (index, label) in WEEKDAYS.into_iter().enumerate() {
            let x = rect.left() + (index as f32 + 0.5) * column;
            ui.painter().text(
                pos2(x, rect.top() + ROW * 0.5),
                Align2::CENTER_CENTER,
                label,
                Type::Small.font(),
                c.text_muted,
            );
        }
        let disabled = ui.new_child(egui::UiBuilder::new().disabled());
        let days = dates
            .enumerate()
            .filter_map(|(index, date)| {
                let date = date?;
                let cell = Rect::from_min_size(
                    pos2(
                        rect.left() + (index % 7) as f32 * column,
                        rect.top() + (index / 7 + 1) as f32 * ROW,
                    ),
                    vec2(column, ROW),
                );
                let target = if options.allows(date) {
                    &*ui
                } else {
                    &disabled
                };
                let response = target.interact(cell, id.with(date), Sense::click());
                lock_arrows(&response, true, true);
                if request_focus && date == view.cursor {
                    response.request_focus();
                }
                Some((index, date, response))
            })
            .collect::<Vec<_>>();

        let today = options.today();
        let preview = pending.map(|start| {
            let hover = days
                .iter()
                .find(|(_, _, r)| r.hovered() && r.enabled())
                .map(|(_, day, _)| *day);
            let end = hover
                .or(view.keyboard.then_some(view.cursor))
                .unwrap_or(start);
            DateRange::new(start, end)
        });
        let range = preview.or(selected);
        let mut activated = None;
        for (index, date, response) in days {
            paint_day(
                ui,
                &response,
                DayStyle {
                    date,
                    today,
                    range,
                    preview: preview.is_some(),
                    left_edge: index % 7 == 0,
                    adjacent: (date.year, date.month) != (view.cursor.year, view.cursor.month),
                    right_edge: index % 7 == 6,
                },
            );
            if response.clicked() {
                if !response.has_focus() {
                    response.request_focus();
                }
                view.cursor = date;
                view.keyboard = response.interact_pointer_pos().is_none();
                activated = Some(date);
            }
        }
        ui.ctx().data_mut(|d| d.insert_temp(id, view));
        activated
    });
    CalendarResponse {
        response: out.response,
        activated: out.inner,
        completed: out.inner.is_some(),
    }
}

fn month_days(cursor: CalendarDate) -> impl ExactSizeIterator<Item = Option<CalendarDate>> {
    let first = CalendarDate { day: 1, ..cursor };
    let offset = i64::from((first.weekday() + 1) % 7);
    let days = i64::from(CalendarDate::days_in_month(first.year, first.month));
    let cells = ((offset + days + 6) / 7 * 7) as usize;
    (0..cells).map(move |index| first.add_days(index as i64 - offset))
}

fn header(ui: &mut Ui, width: f32, cursor: CalendarDate, options: CalendarOptions<'_>) -> i64 {
    ui.allocate_ui_with_layout(
        vec2(width, ROW),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let month = MONTHS[usize::from(cursor.month - 1)];
            let mut font = Type::Label.font();
            font.size = 15.;
            let year_controls = width >= 245.;
            let controls = if year_controls { 4. } else { 2. };
            let label_width = (width - (32. + ui.spacing().item_spacing.x) * controls).max(0.);
            ui.allocate_ui_with_layout(
                vec2(label_width, ROW),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_min_width(label_width);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!("{month} {}", cursor.year)).font(font),
                        )
                        .truncate(),
                    );
                },
            );
            let mut change = 0;
            if year_controls {
                change += nav(ui, cursor, options, -12, Icon::ArrowLeft, "Previous year");
            }
            change += nav(ui, cursor, options, -1, Icon::ChevronLeft, "Previous month");
            change += nav(ui, cursor, options, 1, Icon::ChevronRight, "Next month");
            if year_controls {
                change += nav(ui, cursor, options, 12, Icon::ArrowRight, "Next year");
            }
            change
        },
    )
    .inner
}

fn nav(
    ui: &mut Ui,
    cursor: CalendarDate,
    options: CalendarOptions<'_>,
    delta: i64,
    icon: Icon,
    name: &str,
) -> i64 {
    let enabled = cursor.add_months(delta).is_some_and(|date| {
        let bounded = options.bounded(date);
        (cursor.year, cursor.month) != (bounded.year, bounded.month)
    });
    let clicked = Button::new(name)
        .icon_only(icon)
        .variant(Variant::Ghost)
        .size(Size::Sm)
        .enabled(enabled)
        .show(ui)
        .clicked();
    if clicked { delta } else { 0 }
}

fn next_day(day: CalendarDate, delta: i64, options: CalendarOptions<'_>) -> CalendarDate {
    let Some(target) = day.add_days(delta) else {
        return day;
    };
    let mut candidate = options.bounded(target);
    // A callback may disable every date. Bound one keypress to a year of searching.
    for _ in 0..366 {
        if options.allows(candidate) {
            return candidate;
        }
        let Some(next) = candidate.add_days(delta.signum()) else {
            break;
        };
        let next = options.bounded(next);
        if next == candidate {
            break;
        }
        candidate = next;
    }
    day
}

struct DayStyle {
    date: CalendarDate,
    today: CalendarDate,
    range: Option<DateRange>,
    preview: bool,
    adjacent: bool,
    left_edge: bool,
    right_edge: bool,
}

fn paint_day(ui: &Ui, response: &Response, style: DayStyle) {
    let DayStyle {
        date,
        today,
        range,
        preview,
        adjacent,
        left_edge,
        right_edge,
    } = style;
    let c = Theme::current(ui.ctx()).colors;
    let rect = response.rect;
    let diameter = rect.width().min(ROW);
    let pill = Rect::from_center_size(rect.center(), vec2(diameter, diameter));
    let rounding = (diameter * 0.5) as u8;
    let inside = range.is_some_and(|r| r.contains(date));
    let endpoint = range.is_some_and(|r| date == r.start || date == r.end);
    if let Some(range) = range.filter(|r| r.start != r.end && inside) {
        let left = if date == range.start {
            pill.center().x
        } else if left_edge {
            pill.left()
        } else {
            rect.left()
        };
        let right = if date == range.end {
            pill.center().x
        } else if right_edge {
            pill.right()
        } else {
            rect.right()
        };
        let band = Rect::from_min_max(pos2(left, pill.top()), pos2(right, pill.bottom()));
        let radius = egui::CornerRadius {
            nw: if left_edge { 10 } else { 0 },
            sw: if left_edge { 10 } else { 0 },
            ne: if right_edge { 10 } else { 0 },
            se: if right_edge { 10 } else { 0 },
        };
        ui.painter().rect_filled(
            band,
            radius,
            if preview {
                c.accent.soft_hover
            } else {
                c.accent.soft
            },
        );
    }
    if endpoint {
        ui.painter().rect_filled(pill, rounding, c.accent.base);
    } else if response.hovered() && response.enabled() {
        ui.painter()
            .rect_filled(pill, rounding, c.accent.soft_hover);
    }
    if date == today && !endpoint {
        ui.painter().rect_stroke(
            pill.shrink(2.),
            rounding.saturating_sub(2),
            Stroke::new(1., c.accent.base),
            StrokeKind::Inside,
        );
    }
    let mut color = if endpoint {
        c.accent.foreground
    } else {
        c.text_primary
    };
    if !response.enabled() || (adjacent && !endpoint) {
        color = c
            .text_muted
            .gamma_multiply(if adjacent { 0.55 } else { 0.65 });
    }
    ui.painter().text(
        pill.center(),
        Align2::CENTER_CENTER,
        date.day.to_string(),
        Type::Small.font(),
        color,
    );
    response.widget_info(|| {
        WidgetInfo::selected(Role::Button, response.enabled(), inside, date.to_string())
    });
    let mut focus = response.clone();
    focus.rect = pill.shrink(2.);
    motion::focus_ring(ui, &focus, rounding.saturating_sub(2).into());
}
