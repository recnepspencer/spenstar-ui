use egui::{
    Align, Align2, FontId, Frame, InnerResponse, Key, Layout, Margin, Modifiers, Response, Role,
    Sense, Shape, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2,
};

use super::{FieldVariant, Size};
use crate::{Theme, icon::Icon, motion, theme::Radius, typography::Type};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EntryStyle {
    pub variant: FieldVariant,
    pub size: Size,
    pub invalid: bool,
}

impl EntryStyle {
    pub fn height(self) -> f32 {
        match self.size {
            Size::Sm => 28.,
            Size::Md => 36.,
            Size::Lg => 44.,
        }
    }

    pub fn font(self) -> FontId {
        match self.size {
            Size::Sm => Type::Small.font(),
            Size::Md => Type::Body.font(),
            Size::Lg => Type::H4.font(),
        }
    }

    pub fn show<R>(
        self,
        ui: &mut Ui,
        contents: impl FnOnce(&mut Ui) -> R,
    ) -> (InnerResponse<R>, egui::layers::ShapeIdx) {
        let background = ui.painter().add(Shape::Noop);
        let width = ui.available_width();
        let inner = Frame::new()
            .inner_margin(Margin::symmetric(9, 0))
            .show(ui, |ui| {
                ui.allocate_ui_with_layout(
                    vec2((width - 18.).max(0.), self.height()),
                    Layout::left_to_right(Align::Center),
                    contents,
                )
                .inner
            });
        (inner, background)
    }

    pub fn paint(
        self,
        ui: &Ui,
        background: egui::layers::ShapeIdx,
        rect: egui::Rect,
        response: &Response,
    ) {
        let c = Theme::current(ui.ctx()).colors;
        let radius = Radius::Md.for_size(rect.size());
        let focused = response.has_focus();
        let stroke = Stroke::new(
            1.,
            if self.invalid {
                c.danger.ring
            } else if focused {
                c.accent.ring
            } else if response.hovered() {
                c.text_muted
            } else {
                c.border
            },
        );
        let fill = match self.variant {
            FieldVariant::Outline => c.surface,
            FieldVariant::Filled => c.surface_subtle,
            FieldVariant::Underlined => egui::Color32::TRANSPARENT,
        };
        let shape = if self.variant == FieldVariant::Underlined {
            Shape::line_segment([rect.left_bottom(), rect.right_bottom()], stroke)
        } else {
            Shape::Rect(egui::epaint::RectShape::new(
                rect,
                radius,
                fill,
                stroke,
                StrokeKind::Inside,
            ))
        };
        let mut painter = ui.painter().clone();
        if !response.enabled() && ui.is_enabled() {
            painter.multiply_opacity(0.55);
        }
        painter.set(background, shape);
        if focused {
            painter.rect_stroke(
                rect.expand(2.),
                radius,
                Stroke::new(2., stroke.color),
                StrokeKind::Outside,
            );
        }
        if self.invalid {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_invalid(egui::accesskit::Invalid::True)
            });
        }
    }
}

pub(crate) fn icon_button(ui: &mut Ui, icon: Icon, name: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(22., 22.), Sense::click());
    let colors = Theme::current(ui.ctx()).colors;
    let radius = Radius::Sm.for_size(rect.size());
    if response.hovered() {
        ui.painter().rect_filled(rect, radius, colors.accent.soft);
    }
    let _ = icon.svg().paint(
        ui,
        egui::Rect::from_center_size(rect.center(), vec2(14., 14.)),
        colors.text_secondary,
    );
    response.widget_info(|| WidgetInfo::labeled(Role::Button, response.enabled(), name));
    motion::focus_ring(ui, &response, radius);
    response.on_hover_text(name)
}

pub(crate) fn step_keys(response: &Response, increase: Key, decrease: Key) -> f64 {
    if !response.enabled() || !response.has_focus() {
        return 0.;
    }
    let steps = response.ctx.input_mut(|input| {
        let mut steps = 0.;
        for (modifiers, multiplier) in [(Modifiers::SHIFT, 10.), (Modifiers::NONE, 1.)] {
            steps += multiplier
                * (input.count_and_consume_key(modifiers, increase) as f64
                    - input.count_and_consume_key(modifiers, decrease) as f64);
        }
        steps
    });
    if steps != 0. {
        response
            .ctx
            .memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
    }
    steps
}

pub(crate) fn counter(ui: &mut Ui, length: usize, limit: usize) {
    let color = Theme::current(ui.ctx()).colors.text_secondary;
    let text = format!("{length} / {limit}");
    let font = Type::Caption.font();
    let width = ui
        .painter()
        .layout_no_wrap(text.clone(), font.clone(), color)
        .size()
        .x;
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width().max(width), font.size + 3.),
        Sense::hover(),
    );
    ui.painter().text(
        pos2(rect.right(), rect.top()),
        Align2::RIGHT_TOP,
        text,
        font,
        color,
    );
}
