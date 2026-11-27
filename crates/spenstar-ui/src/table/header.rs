use egui::{
    Align, Color32, CursorIcon, Label, Layout, Rect, Response, Sense, Stroke, Ui, UiBuilder, pos2,
    vec2,
};

use super::{Column, SortDirection, TableState};
use crate::{Theme, icon::Icon, motion, theme::Mode, typography::Type};

pub(super) const CHECK_WIDTH: f32 = 42.;
pub(super) const INSET: f32 = 12.;
pub(super) const SCROLL_GUTTER: f32 = 8.;

pub(super) fn cell_layout(align: Align) -> Layout {
    match align {
        Align::Min => Layout::left_to_right(Align::Center),
        Align::Center => Layout::left_to_right(Align::Center)
            .with_main_align(Align::Center)
            .with_main_justify(true),
        Align::Max => Layout::right_to_left(Align::Center),
    }
}

pub(super) fn widths(state: &mut TableState, columns: &[Column<'_>], available: f32) -> Vec<f32> {
    state.widths.resize(columns.len(), None);
    let weight = |column: &Column<'_>| {
        if column.weight.is_finite() {
            column.weight.clamp(0., 100.)
        } else {
            1.
        }
    };
    let (occupied, total_weight) =
        columns
            .iter()
            .zip(&state.widths)
            .fold((0., 0.), |(sum, total), (column, saved)| {
                (
                    sum + saved.unwrap_or(min_width(column)).max(min_width(column)),
                    total + if saved.is_none() { weight(column) } else { 0. },
                )
            });
    let extra = (available - occupied).max(0.);
    columns
        .iter()
        .zip(&state.widths)
        .map(|(column, saved)| {
            saved.map_or_else(
                || min_width(column) + extra * weight(column) / total_weight.max(1.),
                |width| width.max(min_width(column)),
            )
        })
        .collect()
}

fn min_width(column: &Column<'_>) -> f32 {
    if column.min_width.is_finite() {
        column.min_width.clamp(28., 2048.)
    } else {
        72.
    }
}

pub(super) fn draw(
    ui: &mut Ui,
    state: &mut TableState,
    columns: &[Column<'_>],
    widths: &[f32],
    full: f32,
    (selection, flush): (bool, bool),
) -> bool {
    let theme = Theme::current(ui.ctx());
    let c = theme.colors;
    let fill = match (flush, theme.mode) {
        (true, _) => c.surface_subtle,
        (false, Mode::Light) => Color32::from_rgb(244, 244, 245),
        (false, Mode::Dark) => Color32::from_rgb(20, 20, 22),
    };
    let (rect, _) = ui.allocate_exact_size(vec2(full, 36.), Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius {
            nw: if flush { 0 } else { 15 },
            ne: if flush { 0 } else { 15 },
            ..Default::default()
        },
        fill,
    );
    let mut x = rect.left();
    if selection {
        let (all, partial) = state.selection_status();
        let cell = Rect::from_min_size(rect.min, vec2(CHECK_WIDTH, rect.height()));
        ui.scope_builder(UiBuilder::new(), |ui| {
            if state.filtered_count() == 0 {
                ui.disable();
            }
            if checkbox(
                ui,
                cell,
                ui.make_persistent_id("select-all"),
                all,
                partial,
                "Select all matching rows",
            )
            .clicked()
            {
                state.toggle_all();
            }
        });
        x += CHECK_WIDTH;
    }
    let mut sort = None;
    for (index, (column, &width)) in columns.iter().zip(widths).enumerate() {
        let cell = Rect::from_min_size(pos2(x, rect.top()), vec2(width, rect.height()));
        if flush && x > rect.left() {
            ui.painter().vline(
                x,
                (rect.top() + 8.)..=(rect.bottom() - 8.),
                Stroke::new(1., c.divider),
            );
        }
        let handle = Rect::from_center_size(
            pos2(cell.right() - 3., cell.center().y),
            vec2(8., cell.height()),
        );
        let click = Rect::from_min_max(cell.min, pos2(handle.left().max(x), cell.bottom()));
        let response = ui.interact(
            click,
            ui.make_persistent_id(("sort", index)),
            if column.sortable {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        if column.sortable {
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Button, response.enabled(), column.title)
            });
            motion::focus_ring(ui, &response, 4.into());
            if response.clicked() {
                sort = Some(index);
            }
        }
        if response.hovered() && column.sortable {
            ui.painter().rect_filled(click.shrink(2.), 5, c.field_hover);
        }
        let direction = state
            .sort
            .filter(|sort| sort.column == index)
            .map(|sort| sort.direction);
        let mut text = cell.shrink2(vec2(INSET.min(width * 0.2), 0.));
        let sort_icon = match direction {
            Some(SortDirection::Asc) => Some(Icon::ChevronUp),
            Some(SortDirection::Desc) => Some(Icon::ChevronDown),
            None if flush && column.sortable => Some(Icon::ChevronsUpDown),
            None => None,
        };
        if sort_icon.is_some() && text.width() > 16. {
            text.max.x -= 16.;
        }
        let mut header = ui.new_child(
            UiBuilder::new()
                .max_rect(text)
                .layout(cell_layout(column.align)),
        );
        header.set_clip_rect(ui.clip_rect().intersect(text));
        header.add(
            Label::new(Type::Badge.text(column.title).color(c.text_secondary))
                .selectable(false)
                .truncate(),
        );
        if let Some(icon) = sort_icon {
            let _ = icon.svg().paint(
                ui,
                Rect::from_center_size(pos2(click.right() - 8., click.center().y), vec2(13., 13.)),
                c.text_secondary,
            );
        }
        let resize = ui
            .interact(
                handle,
                ui.make_persistent_id(("resize", index)),
                Sense::drag(),
            )
            .on_hover_cursor(CursorIcon::ResizeHorizontal);
        if resize.drag_started() {
            state.drag = Some((index, width));
        }
        if resize.dragged()
            && let Some((i, original)) = state.drag
            && i == index
        {
            state.widths[index] = Some(
                (original + resize.total_drag_delta().map_or(0., |delta| delta.x))
                    .clamp(min_width(column), 4096.),
            );
            ui.ctx().request_repaint();
        }
        if resize.drag_stopped() && state.drag.is_some_and(|(i, _)| i == index) {
            state.drag = None;
        }
        if resize.hovered() || resize.dragged() {
            ui.painter().vline(
                handle.center().x,
                (rect.top() + 7.)..=(rect.bottom() - 7.),
                Stroke::new(1.5, c.accent.ring),
            );
        }
        x += width;
    }
    if let Some(index) = sort {
        state.cycle_sort(index);
    }
    sort.is_some()
}

pub(super) fn checkbox(
    ui: &Ui,
    cell: Rect,
    id: egui::Id,
    checked: bool,
    partial: bool,
    label: &str,
) -> Response {
    let c = Theme::current(ui.ctx()).colors;
    let response = ui.interact(cell, id, Sense::click());
    let mark = Rect::from_center_size(cell.center(), vec2(15., 15.));
    let active = checked || partial;
    let fill = if active { c.accent.base } else { c.surface };
    let outline = if active {
        c.accent.base
    } else if response.hovered() {
        c.accent.ring
    } else {
        c.text_muted
    };
    let painter = ui.painter();
    painter.rect(
        mark,
        4,
        fill,
        Stroke::new(1., outline),
        egui::StrokeKind::Inside,
    );
    if partial {
        painter.hline(
            (mark.left() + 4.)..=(mark.right() - 4.),
            mark.center().y,
            Stroke::new(2., c.accent.foreground),
        );
    } else if checked {
        let _ = Icon::Check
            .svg()
            .paint(ui, mark.shrink(2.), c.accent.foreground);
    }
    response.widget_info(|| {
        if partial {
            egui::WidgetInfo::labeled(egui::Role::CheckBox, response.enabled(), label)
        } else {
            egui::WidgetInfo::selected(egui::Role::CheckBox, response.enabled(), checked, label)
        }
    });
    if response.clicked() {
        response.request_focus();
    }
    motion::focus_ring(ui, &response, 5.into());
    response
}
