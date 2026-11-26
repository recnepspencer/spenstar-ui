use eframe::egui::{self, Align, Rect, Sense, Stroke, Ui, UiBuilder, vec2};
use spenstar_ui::{Theme, icon::Icon, typography::Type};

use super::WorkbenchPalette;

pub(super) fn trigger(
    ui: &mut Ui,
    p: WorkbenchPalette,
    label: &str,
    leading: Option<Icon>,
    trailing: impl FnOnce(&mut Ui),
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 28.), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    if response.hovered() {
        ui.painter().rect_filled(rect, 5, p.active);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            5,
            Stroke::new(1., Theme::current(ui.ctx()).colors.accent.ring),
            egui::StrokeKind::Inside,
        );
    }
    split(ui, rect.shrink2(vec2(4., 0.)), trailing, |ui| {
        if let Some(icon) = leading {
            icon.show(ui, 14., p.muted);
        }
        ui.add(
            egui::Label::new(Type::Description.text(label).color(p.ink))
                .truncate()
                .selectable(false),
        );
    });
    response
}

pub(super) fn split(
    ui: &mut Ui,
    rect: Rect,
    trailing: impl FnOnce(&mut Ui),
    leading: impl FnOnce(&mut Ui),
) {
    let mut right = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(Align::Center)),
    );
    right.spacing_mut().item_spacing.x = 4.;
    trailing(&mut right);

    let mut left_rect = rect;
    left_rect.max.x = (right.available_rect_before_wrap().right() - 4.).max(rect.left());
    let mut left = ui.new_child(
        UiBuilder::new()
            .max_rect(left_rect)
            .layout(egui::Layout::left_to_right(Align::Center)),
    );
    left.shrink_clip_rect(left_rect);
    left.spacing_mut().item_spacing.x = 6.;
    leading(&mut left);
}
