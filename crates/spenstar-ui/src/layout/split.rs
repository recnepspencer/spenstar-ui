use egui::{
    Align, CursorIcon, Id, IdSalt, Layout, Rect, Response, Sense, Stroke, Ui, UiBuilder, vec2,
};

use crate::Theme;

/// The dimension along which two panes divide space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitAxis {
    /// Side-by-side panes.
    Horizontal,
    /// Stacked panes.
    Vertical,
}

/// Two clipped panes separated by a draggable divider.
///
/// The caller owns the first pane's size in logical points, so it can persist
/// the preference alongside other application state.
#[derive(Clone, Copy, Debug)]
pub struct SplitPane {
    id: IdSalt,
    axis: SplitAxis,
    min: f32,
    max: f32,
    second_min: f32,
}

impl SplitPane {
    const HANDLE: f32 = 12.;

    /// Create a split with a stable local ID and direction.
    pub fn new(id: impl egui::AsIdSalt, axis: SplitAxis) -> Self {
        Self {
            id: IdSalt::new(id),
            axis,
            min: 0.,
            max: f32::INFINITY,
            second_min: 0.,
        }
    }

    /// Set the first-pane bounds and second-pane minimum, in points.
    pub fn limits(mut self, min: f32, max: f32, second_min: f32) -> Self {
        self.min = min.max(0.);
        self.max = max.max(self.min);
        self.second_min = second_min.max(0.);
        self
    }

    /// Clamp a first-pane size within the current available extent.
    pub fn clamp_position(self, position: f32, extent: f32) -> f32 {
        let available = (extent - Self::HANDLE).max(0.);
        let max = self.max.min((available - self.second_min).max(0.));
        let min = self.min.min(max);
        if position.is_nan() {
            min
        } else {
            position.clamp(min, max)
        }
    }

    /// Render both panes in a fixed height and return the divider's response.
    pub fn show(
        self,
        ui: &mut Ui,
        height: f32,
        position: &mut f32,
        first: impl FnOnce(&mut Ui),
        second: impl FnOnce(&mut Ui),
    ) -> Response {
        let (bounds, _) = ui.allocate_exact_size(
            vec2(ui.available_width().max(0.), height.max(0.)),
            Sense::hover(),
        );
        let axis = usize::from(self.axis == SplitAxis::Vertical);
        let id = ui.make_persistent_id(self.id);
        let extent = bounds.size()[axis];
        let placement = self.clamp_position(*position, extent);
        let make_divider = |start| {
            let mut divider = bounds;
            divider.min[axis] += start;
            divider.max[axis] = (divider.min[axis] + Self::HANDLE).min(bounds.max[axis]);
            divider
        };
        let response = ui.interact(make_divider(placement), id, Sense::drag());
        if response.dragged() {
            *position = self.clamp_position(placement + response.drag_delta()[axis], extent);
        }
        let divider = make_divider(self.clamp_position(*position, extent));
        let mut before = bounds;
        before.max[axis] = divider.min[axis];
        let mut after = bounds;
        after.min[axis] = divider.max[axis];
        pane(ui, id.with("first"), before, first);
        pane(ui, id.with("second"), after, second);

        let colors = Theme::current(ui.ctx()).colors;
        let active = response.hovered() || response.dragged();
        let stroke = Stroke::new(
            1.,
            if active {
                colors.accent.ring
            } else {
                colors.divider
            },
        );
        match self.axis {
            SplitAxis::Horizontal => {
                ui.painter()
                    .vline(divider.center().x, divider.y_range(), stroke);
                response.on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal)
            }
            SplitAxis::Vertical => {
                ui.painter()
                    .hline(divider.x_range(), divider.center().y, stroke);
                response.on_hover_and_drag_cursor(CursorIcon::ResizeVertical)
            }
        }
    }
}

fn pane(ui: &mut Ui, id: Id, bounds: Rect, render: impl FnOnce(&mut Ui)) {
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(id)
            .max_rect(bounds)
            .layout(Layout::top_down(Align::Min)),
    );
    child.shrink_clip_rect(bounds);
    render(&mut child);
}
