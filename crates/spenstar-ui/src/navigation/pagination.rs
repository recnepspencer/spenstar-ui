use egui::{Align, Layout, Ui};

use crate::{
    Theme,
    button::{Button, Size, Variant},
    icon::Icon,
    theme::Radius,
    typography::Type,
};

/// An item in a condensed page strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageSlot {
    /// Zero-based page.
    Page(usize),
    /// Omitted group of pages.
    Gap,
}

/// Condense a page range into endpoints and a window around the current page.
pub fn page_slots(current: usize, count: usize) -> Vec<PageSlot> {
    if count <= 7 {
        return (0..count).map(PageSlot::Page).collect();
    }
    let current = current.min(count - 1);
    let start = current.saturating_sub(1).clamp(1, count - 4);
    let end = current.saturating_add(1).clamp(3, count - 2);
    let mut slots = vec![PageSlot::Page(0)];
    if start > 1 {
        slots.push(if start == 2 {
            PageSlot::Page(1)
        } else {
            PageSlot::Gap
        });
    }
    slots.extend((start..=end).map(PageSlot::Page));
    if end < count - 2 {
        slots.push(if end == count - 3 {
            PageSlot::Page(count - 2)
        } else {
            PageSlot::Gap
        });
    }
    slots.push(PageSlot::Page(count - 1));
    slots
}

/// Page controls for a collection with a known item count.
pub struct Pagination {
    /// Total item count.
    pub total: usize,
    /// Items per page. Zero is treated as one.
    pub per_page: usize,
    /// Show the current item interval and total.
    pub summary: bool,
}

impl Pagination {
    /// Create a page strip with an item summary.
    pub fn new(total: usize, per_page: usize) -> Self {
        Self {
            total,
            per_page,
            summary: true,
        }
    }

    /// Number of pages, zero for an empty collection.
    pub fn page_count(&self) -> usize {
        self.total.div_ceil(self.per_page.max(1))
    }

    /// Clamp and update the zero-based page. Return whether it changed.
    pub fn show(&self, ui: &mut Ui, page: &mut usize) -> bool {
        let old = *page;
        let count = self.page_count();
        *page = (*page).min(count.saturating_sub(1));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.;
            let previous = Button::new("Previous page")
                .icon_only(Icon::ChevronLeft)
                .enabled(*page > 0);
            if control(previous, false).show(ui).clicked() {
                *page -= 1;
            }
            for slot in page_slots(*page, count) {
                match slot {
                    PageSlot::Page(target) => {
                        if control(Button::new((target + 1).to_string()), target == *page)
                            .show(ui)
                            .clicked()
                        {
                            *page = target;
                        }
                    }
                    PageSlot::Gap => {
                        ui.label(Type::Small.text("…"));
                    }
                }
            }
            let next = Button::new("Next page")
                .icon_only(Icon::ChevronRight)
                .enabled(*page + 1 < count);
            if control(next, false).show(ui).clicked() {
                *page += 1;
            }
            if self.summary {
                let size = self.per_page.max(1);
                let start = (*page * size + 1).min(self.total);
                let end = start.saturating_sub(1).saturating_add(size).min(self.total);
                ui.add_space(6.);
                ui.add(
                    egui::Label::new(
                        Type::Small
                            .text(format!("Showing {start}–{end} of {}", self.total))
                            .color(Theme::current(ui.ctx()).colors.text_secondary),
                    )
                    .halign(Align::Min),
                );
            }
        });
        old != *page
    }
    /// Render an item interval at the left and compact page controls at the right.
    pub fn show_compact(&self, ui: &mut Ui, page: &mut usize) -> bool {
        let old = *page;
        let count = self.page_count().max(1);
        *page = (*page).min(count - 1);
        let size = self.per_page.max(1);
        let offset = *page * size;
        let start = (offset + 1).min(self.total);
        let end = offset.saturating_add(size).min(self.total);
        let ink = Theme::current(ui.ctx()).colors.text_secondary;
        ui.horizontal(|ui| {
            if self.summary {
                ui.label(
                    Type::Small
                        .text(format!("{start}–{end} of {}", self.total))
                        .color(ink),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let next = Button::new("Next page")
                    .icon_only(Icon::ChevronRight)
                    .enabled(*page + 1 < count);
                let previous = Button::new("Previous page")
                    .icon_only(Icon::ChevronLeft)
                    .enabled(*page > 0);
                if control(next, false).show(ui).clicked() {
                    *page += 1;
                }
                if control(previous, false).show(ui).clicked() {
                    *page -= 1;
                }
                ui.label(
                    Type::Small
                        .text(format!("Page {} of {count}", *page + 1))
                        .color(ink),
                );
            });
        });
        old != *page
    }
}

fn control(button: Button, active: bool) -> Button {
    button
        .radius(Radius::Full)
        .size(Size::Sm)
        .min_width(28.)
        .variant(if active {
            Variant::Secondary
        } else {
            Variant::Ghost
        })
}
