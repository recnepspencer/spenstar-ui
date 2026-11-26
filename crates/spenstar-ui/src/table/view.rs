use std::{cmp::Ordering, ops::Range};

use egui::{Frame, Margin, Rect, ScrollArea, Sense, Stroke, Ui, UiBuilder, pos2, vec2};

use super::{
    Column, Density, TableRows, TableState,
    header::{self, CHECK_WIDTH, INSET, SCROLL_GUTTER},
};
use crate::{
    Theme,
    button::{Button, Size, Variant},
    icon::Icon,
    layout::EmptyState,
    navigation::Pagination,
    theme::Mode,
    typography::Type,
};

/// A sticky-header table with fixed-height, virtualized rows.
pub struct DataTable<'a> {
    /// Stable UI-local identity.
    pub id: egui::IdSalt,
    /// Column definitions.
    pub columns: &'a [Column<'a>],
    /// Fixed row height.
    pub density: Density,
    /// Maximum viewport height for rows.
    pub height: f32,
    /// Include selection controls.
    pub selectable: bool,
    /// Show loading skeletons in place of body rows.
    pub loading: bool,
    /// Show page navigation and limit the visible source window.
    pub page_size: Option<usize>,
    /// Labels for the selection action bar.
    pub bulk_actions: &'a [&'a str],
    /// Message shown when the filtered collection is empty.
    pub empty_message: &'a str,
}

/// Visible work and interactions reported by the table.
#[derive(Default)]
pub struct TableResponse {
    /// Number of rows visited by the cell renderer.
    pub rendered_rows: usize,
    /// Action label index activated in the bulk bar.
    pub bulk_action: Option<usize>,
    /// The header changed sort order.
    pub sort_changed: bool,
    /// The footer changed the current page.
    pub page_changed: bool,
}

impl<'a> DataTable<'a> {
    /// Create a table with a 300-point body viewport.
    pub fn new(id: impl egui::AsIdSalt, columns: &'a [Column<'a>]) -> Self {
        Self {
            id: egui::IdSalt::new(id),
            columns,
            density: Density::Comfortable,
            height: 300.,
            selectable: false,
            loading: false,
            page_size: None,
            bulk_actions: &[],
            empty_message: "Try changing the current filters.",
        }
    }

    /// Render the table. The callback receives a source index and a column index.
    /// Only virtualized rows are visited. In a centered column, group multiple
    /// widgets inside a single horizontal child.
    pub fn show<F, I, C>(
        &self,
        ui: &mut Ui,
        state: &mut TableState,
        source: TableRows<F, I, C>,
        mut cell: impl FnMut(&mut Ui, usize, usize),
    ) -> TableResponse
    where
        F: FnMut(usize) -> bool,
        I: FnMut(usize) -> u64,
        C: FnMut(usize, usize, usize) -> Ordering,
    {
        ui.push_id(self.id, |ui| self.draw(ui, state, source, &mut cell))
            .inner
    }

    fn draw<F, I, C>(
        &self,
        ui: &mut Ui,
        state: &mut TableState,
        source: TableRows<F, I, C>,
        cell: &mut impl FnMut(&mut Ui, usize, usize),
    ) -> TableResponse
    where
        F: FnMut(usize) -> bool,
        I: FnMut(usize) -> u64,
        C: FnMut(usize, usize, usize) -> Ordering,
    {
        let theme = Theme::current(ui.ctx());
        let c = theme.colors;
        if state.sort.is_some_and(|sort| {
            self.columns
                .get(sort.column)
                .is_none_or(|column| !column.sortable)
        }) {
            state.sort = None;
            state.page = 0;
        }
        let updated = state.update(source);
        let range = state.page_range(self.page_size);
        let viewport_changed = state.viewport_changed(self.page_size, self.density as usize);
        let mut output = TableResponse::default();
        let border = Stroke::new(
            1.,
            if theme.mode == Mode::Dark {
                c.border
            } else {
                c.border_subtle
            },
        );
        let frame = Frame::new()
            .fill(c.surface)
            .stroke(border)
            .corner_radius(16)
            .shadow(if theme.mode == Mode::Light {
                theme.shadow()
            } else {
                egui::epaint::Shadow::NONE
            });
        frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.;
            if self.selectable && !state.selected().is_empty() && !self.loading {
                output.bulk_action = self.bulk(ui, state);
            }
            let available = ui.available_width();
            let check = if self.selectable { CHECK_WIDTH } else { 0. };
            let widths = header::widths(state, self.columns, available - check - SCROLL_GUTTER);
            let total = (widths.iter().sum::<f32>() + check + SCROLL_GUTTER).max(available);
            let mut draw = |ui: &mut Ui| {
                output.sort_changed = ui
                    .add_enabled_ui(!self.loading, |ui| {
                        header::draw(ui, state, self.columns, &widths, total, self.selectable)
                    })
                    .inner;
                if self.loading {
                    self.skeleton(ui, &widths, total);
                } else if range.is_empty() {
                    EmptyState::new(Icon::Table, "No rows to show", self.empty_message).show(ui);
                } else {
                    let mut area = ScrollArea::vertical()
                        .max_height(self.safe_height())
                        .auto_shrink([false, true]);
                    if updated || viewport_changed {
                        area = area.vertical_scroll_offset(0.);
                    }
                    output.rendered_rows = area
                        .show_rows(ui, self.density.height(), range.len(), |ui, visible| {
                            self.rows(
                                ui,
                                state,
                                range.start + visible.start..range.start + visible.end,
                                &widths,
                                total,
                                cell,
                            );
                            visible.len()
                        })
                        .inner;
                }
            };
            if total > available + 0.5 {
                ScrollArea::horizontal()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.set_width(total);
                        draw(ui);
                    });
            } else {
                draw(ui);
            }
            if let Some(size) = self.page_size.filter(|_| !self.loading) {
                crate::layout::Separator::Horizontal.show(ui);
                Frame::new()
                    .inner_margin(Margin::symmetric(12, 6))
                    .show(ui, |ui| {
                        output.page_changed =
                            Pagination::new(state.filtered_count(), size).show(ui, &mut state.page);
                    });
            }
        });
        if output.sort_changed || output.page_changed {
            ui.ctx().request_repaint();
        }
        output
    }

    fn safe_height(&self) -> f32 {
        if self.height.is_finite() {
            self.height.clamp(24., 8192.)
        } else {
            300.
        }
    }

    fn rows(
        &self,
        ui: &mut Ui,
        state: &mut TableState,
        range: Range<usize>,
        widths: &[f32],
        width: f32,
        cell: &mut impl FnMut(&mut Ui, usize, usize),
    ) {
        let c = Theme::current(ui.ctx()).colors;
        for position in range {
            let Some((index, id)) = state.row_at(position) else {
                continue;
            };
            let (rect, response) =
                ui.allocate_exact_size(vec2(width, self.density.height()), Sense::hover());
            let selected = self.selectable && state.selected().contains(&id);
            if selected || response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    0,
                    if selected {
                        c.accent.soft
                    } else {
                        c.surface_subtle
                    },
                );
            }
            ui.painter()
                .hline(rect.x_range(), rect.bottom(), Stroke::new(1., c.divider));
            let mut x = rect.left();
            if self.selectable {
                let bounds = Rect::from_min_size(rect.min, vec2(CHECK_WIDTH, rect.height()));
                if header::checkbox(
                    ui,
                    bounds,
                    ui.make_persistent_id(("row", id)),
                    selected,
                    false,
                    "Select row",
                )
                .clicked()
                {
                    state.toggle_row(position);
                }
                x += CHECK_WIDTH;
            }
            for (column, (&width, definition)) in widths.iter().zip(self.columns).enumerate() {
                let bounds = Rect::from_min_size(pos2(x, rect.top()), vec2(width, rect.height()));
                let inner = bounds.shrink2(vec2(INSET.min(width * 0.2), 1.));
                let layout = header::cell_layout(definition.align);
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .id_salt(("cell", id, column))
                        .max_rect(inner)
                        .layout(layout),
                );
                child.set_clip_rect(ui.clip_rect().intersect(inner));
                cell(&mut child, index, column);
                x += width;
            }
        }
    }

    fn skeleton(&self, ui: &mut Ui, widths: &[f32], width: f32) {
        let c = Theme::current(ui.ctx()).colors;
        let count = (self.safe_height() / self.density.height())
            .floor()
            .clamp(1., 6.) as usize;
        for row in 0..count {
            let (rect, _) =
                ui.allocate_exact_size(vec2(width, self.density.height()), Sense::hover());
            let mut x = rect.left() + if self.selectable { CHECK_WIDTH } else { 0. };
            for (i, &width) in widths.iter().enumerate() {
                let w = (width - 2. * INSET).max(2.) * [0.56, 0.72, 0.43][(row + i) % 3];
                ui.painter().rect_filled(
                    Rect::from_min_size(pos2(x + INSET, rect.center().y - 4.), vec2(w, 8.)),
                    4,
                    c.surface_tertiary,
                );
                x += width;
            }
            ui.painter()
                .hline(rect.x_range(), rect.bottom(), Stroke::new(1., c.divider));
        }
    }

    fn bulk(&self, ui: &mut Ui, state: &mut TableState) -> Option<usize> {
        let c = Theme::current(ui.ctx()).colors;
        Frame::new()
            .inner_margin(Margin::symmetric(10, 7))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        Type::Label
                            .text(format!("{} selected", state.selected().len()))
                            .color(c.text_primary),
                    );
                    let mut selected = None;
                    for (index, label) in self.bulk_actions.iter().enumerate() {
                        if Button::new(*label)
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .show(ui)
                            .clicked()
                        {
                            selected = Some(index);
                        }
                    }
                    if Button::new("Clear")
                        .variant(Variant::Ghost)
                        .size(Size::Sm)
                        .show(ui)
                        .clicked()
                    {
                        state.clear_selection();
                    }
                    selected
                })
                .inner
            })
            .inner
    }
}
