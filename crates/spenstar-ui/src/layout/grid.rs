use egui::{Align, Layout, Rect, Ui, UiBuilder, pos2, vec2};

/// A row-major grid that reflows to fit its available width.
#[derive(Clone, Copy, Debug)]
pub struct ResponsiveGrid {
    min_width: f32,
    gap: f32,
    max_columns: usize,
}

impl ResponsiveGrid {
    /// Create a grid with a minimum cell width, measured in egui points.
    pub fn new(min_width: f32) -> Self {
        Self {
            min_width: min_width.max(1.),
            gap: 12.,
            max_columns: usize::MAX,
        }
    }

    /// Set the spacing between cells, measured in egui points.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = if gap.is_finite() { gap.max(0.) } else { 0. };
        self
    }

    /// Cap the number of columns.
    pub fn max_columns(mut self, count: usize) -> Self {
        self.max_columns = count.max(1);
        self
    }

    /// Compute the column count, including gaps, for the supplied width.
    pub fn columns(self, width: f32) -> usize {
        if !width.is_finite() {
            return 1;
        }
        (((width.max(0.) + self.gap) / (self.min_width + self.gap)).floor() as usize)
            .clamp(1, self.max_columns)
    }

    /// Render each index exactly once, left-to-right then top-to-bottom.
    pub fn show(self, ui: &mut Ui, items: usize, mut element: impl FnMut(&mut Ui, usize)) {
        let width = ui.available_width();
        let columns = self.columns(width);
        let cell = ((width - self.gap * (columns - 1) as f32) / columns as f32).max(0.);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = self.gap;
            for row in 0..items.div_ceil(columns) {
                let area = ui.available_rect_before_wrap();
                let mut height: f32 = 0.;
                let start = row * columns;
                let count = columns.min(items - start);
                for column in 0..count {
                    let left = area.left() + column as f32 * (cell + self.gap);
                    let rect = Rect::from_min_max(
                        pos2(left, area.top()),
                        pos2(left + cell, area.bottom().max(area.top())),
                    );
                    let mut child = ui.new_child(
                        UiBuilder::new()
                            .id_salt(start + column)
                            .max_rect(rect)
                            .layout(Layout::top_down(Align::Min)),
                    );
                    child.shrink_clip_rect(rect.with_max_y(ui.clip_rect().bottom()));
                    child.set_min_width(cell);
                    element(&mut child, start + column);
                    height = height.max(child.min_rect().height());
                }
                ui.allocate_space(vec2(width, height));
            }
        });
    }
}
