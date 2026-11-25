use egui::{
    Align, Align2, FontId, Frame, InnerResponse, Key, Layout, Margin, Modifiers, Response, Role,
    Sense, Shape, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2,
};

use super::{FieldVariant, Size};
use crate::{
    Theme,
    icon::Icon,
    motion,
    theme::{Mode, Radius, mix},
    typography::Type,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EntryStyle {
    pub variant: FieldVariant,
    pub size: Size,
    pub invalid: bool,
}

impl EntryStyle {
    pub fn height(self) -> f32 {
        match self.size {
            Size::Sm => 32.,
            Size::Md => 38.,
            Size::Lg => 44.,
        }
    }

    pub fn font(self) -> FontId {
        match self.size {
            Size::Sm => Type::Description.font(),
            Size::Md | Size::Lg => Type::Body.font(),
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
            .inner_margin(Margin::symmetric(12, 0))
            .show(ui, |ui| {
                ui.allocate_ui_with_layout(
                    vec2((width - 24.).max(0.), self.height()),
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
        let theme = Theme::current(ui.ctx());
        let c = theme.colors;
        let radius = if matches!(self.variant, FieldVariant::Capsule | FieldVariant::Tonal) {
            Radius::Full
        } else {
            Radius::Lg
        }
        .for_size(rect.size());
        let focus = motion::eased(
            ui.ctx(),
            response.id.with("entry-focus"),
            response.has_focus(),
            motion::FAST,
        );
        let hover = motion::eased(
            ui.ctx(),
            response.id.with("entry-hover"),
            response.hovered(),
            motion::FAST,
        );
        let canvas = ui.visuals().extreme_bg_color == c.surface;
        let canvas_hairline =
            canvas && theme.mode == Mode::Light && self.variant == FieldVariant::Filled;
        let stroke = match (self.invalid, self.variant) {
            (true, _) => Stroke::new(1.5, c.danger.base),
            (false, FieldVariant::Filled) if canvas => {
                Stroke::new(1., egui::Color32::from_black_alpha(15))
            }
            (false, FieldVariant::Filled | FieldVariant::Capsule | FieldVariant::Tonal) => {
                Stroke::NONE
            }
            (false, _) => Stroke::new(1., mix(c.border, c.text_muted, hover * 0.3)),
        };
        let fill = match self.variant {
            FieldVariant::Outline => c.surface,
            FieldVariant::Filled => {
                let brighter = if theme.mode == Mode::Light {
                    c.surface
                } else {
                    c.field_hover
                };
                let base = if canvas {
                    c.surface
                } else {
                    c.surface_secondary
                };
                let hovered = match (canvas, theme.mode) {
                    (true, Mode::Light) => c.surface_subtle,
                    (true, Mode::Dark) => c.surface_elevated,
                    _ => c.field_hover,
                };
                mix(mix(base, hovered, hover), brighter, focus * 0.6)
            }
            FieldVariant::Underlined => egui::Color32::TRANSPARENT,
            FieldVariant::Capsule => mix(c.default.base, c.default.hover, hover.max(focus)),
            FieldVariant::Tonal => mix(c.accent.soft, c.accent.soft_hover, hover.max(focus)),
        };
        let shape = if self.variant == FieldVariant::Underlined {
            Shape::line_segment([rect.left_bottom(), rect.right_bottom()], stroke)
        } else {
            let fill = if self.invalid {
                mix(fill, c.danger.base, 0.07)
            } else {
                fill
            };
            Shape::Rect(egui::epaint::RectShape::new(
                rect,
                radius,
                fill,
                stroke,
                StrokeKind::Inside,
            ))
        };
        let shape = if canvas_hairline {
            let shadow = egui::Shadow {
                offset: [0, 1],
                blur: 2,
                spread: 0,
                color: egui::Color32::from_black_alpha(15),
            };
            Shape::Vec(vec![shadow.as_shape(rect, radius).into(), shape])
        } else {
            shape
        };
        let mut painter = ui.painter().clone();
        if !response.enabled() && ui.is_enabled() {
            painter.multiply_opacity(0.55);
        }
        painter.set(background, shape);
        if focus > 0. {
            let ink = if self.invalid {
                c.danger.base
            } else {
                c.accent.ring
            };
            painter.rect_stroke(
                rect.expand(1.),
                radius,
                Stroke::new(2., ink.gamma_multiply(focus)),
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
