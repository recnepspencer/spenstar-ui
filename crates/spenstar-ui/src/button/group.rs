use egui::{IdSalt, Rect, Response, Shape, Stroke, StrokeKind, Ui, pos2};

use super::{Button, Segment, Variant};
use crate::{Theme, motion, theme::Radius};

/// A row of closely joined buttons with a single outer border and inner dividers.
pub struct ButtonGroup {
    buttons: Vec<Button>,
}

impl ButtonGroup {
    /// Create a group from its ordered buttons.
    pub fn new(buttons: impl IntoIterator<Item = Button>) -> Self {
        Self {
            buttons: buttons.into_iter().collect(),
        }
    }

    /// Render at the largest member size and return responses in order.
    pub fn show(self, ui: &mut Ui) -> Vec<Response> {
        let count = self.buttons.len();
        let size = self
            .buttons
            .iter()
            .map(|button| button.size)
            .max()
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.;
            let responses: Vec<_> = self
                .buttons
                .into_iter()
                .enumerate()
                .map(|(i, button)| {
                    button
                        .size(size)
                        .radius(Radius::Md)
                        .render(ui, Segment::at(i, count), None)
                })
                .collect();
            if let (Some(first), Some(last)) = (responses.first(), responses.last()) {
                let rect = first.rect.union(last.rect);
                let stroke = Stroke::new(1., Theme::current(ui.ctx()).colors.border);
                ui.painter().rect_stroke(
                    rect,
                    Radius::Md.for_size(rect.size()),
                    stroke,
                    StrokeKind::Inside,
                );
                for response in responses.iter().skip(1) {
                    let x = response.rect.left();
                    ui.painter().vline(x, rect.y_range().shrink(3.), stroke);
                }
            }
            responses
        })
        .inner
    }
}

/// An independent on/off button.
pub struct ToggleButton {
    button: Button,
}

impl ToggleButton {
    /// Turn a button into a toggle.
    pub fn new(button: Button) -> Self {
        Self { button }
    }

    /// Render the toggle and update its checked state on activation.
    pub fn show(self, ui: &mut Ui, selected: &mut bool) -> Response {
        let variant = if *selected {
            Variant::Soft
        } else {
            Variant::Outline
        };
        let mut response =
            self.button
                .variant(variant)
                .render(ui, Segment::Single, Some(*selected));
        if response.clicked() {
            *selected = !*selected;
            response.mark_changed();
        }
        response
    }
}

/// A single-selection segmented control.
pub struct ToggleButtonGroup {
    id: IdSalt,
    buttons: Vec<Button>,
}

impl ToggleButtonGroup {
    /// Create a control from an ordered list of choices and a stable UI-local ID.
    pub fn new(id: impl egui::AsIdSalt, buttons: impl IntoIterator<Item = Button>) -> Self {
        Self {
            id: IdSalt::new(id),
            buttons: buttons.into_iter().collect(),
        }
    }

    /// Render at the largest member size and update the selected index on activation.
    /// Out-of-range selections display the last choice as selected.
    pub fn show(self, ui: &mut Ui, selected: &mut usize) -> Vec<Response> {
        let id = ui.make_persistent_id(self.id);
        let count = self.buttons.len();
        let index = (*selected).min(count.saturating_sub(1));
        let size = self
            .buttons
            .iter()
            .map(|button| button.size)
            .max()
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.;
            let painter = ui.painter().clone();
            let backdrop = painter.add(Shape::Noop);
            let indicator = painter.add(Shape::Noop);
            let responses: Vec<_> = self
                .buttons
                .into_iter()
                .enumerate()
                .map(|(i, button)| {
                    let mut response = button
                        .size(size)
                        .radius(Radius::Md)
                        .variant(Variant::Ghost)
                        .render(ui, Segment::at(i, count), Some(i == index));
                    if response.clicked() && *selected != i {
                        *selected = i;
                        response.mark_changed();
                        ui.ctx().request_repaint();
                    }
                    response
                })
                .collect();
            if let (Some(first), Some(last)) = (responses.first(), responses.last()) {
                let rect = first.rect.union(last.rect);
                let colors = Theme::current(ui.ctx()).colors;
                painter.set(
                    backdrop,
                    Shape::rect_filled(
                        rect,
                        Radius::Md.for_size(rect.size()),
                        colors.surface_subtle,
                    ),
                );
                let target = responses[index].rect.shrink(2.);
                let ctx = ui.ctx();
                let origin = rect.left();
                let animate = |name, x| {
                    origin + ctx.animate_value_with_time(id.with(name), x - origin, motion::NORMAL)
                };
                let rect = Rect::from_min_max(
                    pos2(animate("left", target.left()), target.top()),
                    pos2(animate("right", target.right()), target.bottom()),
                );
                painter.set(
                    indicator,
                    Shape::rect_filled(
                        rect,
                        Radius::Sm.for_size(rect.size()),
                        colors.accent.soft_hover,
                    ),
                );
            }
            responses
        })
        .inner
    }
}
