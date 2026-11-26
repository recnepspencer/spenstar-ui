use egui::{
    Align2, CursorIcon, Rect, Response, Role, Sense, TextWrapMode, Ui, WidgetInfo, WidgetText,
    pos2, vec2,
};

use crate::{Theme, icon::Icon, motion, theme::Radius, typography::Type};

use super::MenuItem;

#[derive(Default)]
pub(super) struct RowStyle {
    pub(super) compact: bool,
    pub(super) selected: bool,
    pub(super) depth: usize,
    pub(super) disclosure: Option<Icon>,
}

pub(super) fn row(ui: &mut Ui, item: MenuItem<'_>, style: RowStyle) -> Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 36.), Sense::click());
    response.widget_info(|| {
        WidgetInfo::selected(Role::Button, response.enabled(), style.selected, item.label)
    });
    let radius = Radius::Full.for_size(rect.size());
    let colors = Theme::current(ui.ctx()).colors;
    let tint = if style.selected {
        colors.text_primary
    } else {
        colors.text_secondary
    };
    if style.selected || response.hovered() {
        let fill = if style.selected {
            colors.default.base
        } else {
            colors.surface_secondary
        };
        ui.painter().rect_filled(rect, radius, fill);
    }
    let painter = ui.painter();
    let inset = 12. + style.depth.min(6) as f32 * 10.;
    let center = if style.compact {
        rect.center()
    } else {
        pos2(rect.left() + inset + 9., rect.center().y)
    };
    let square = |center, side| Rect::from_center_size(center, vec2(side, side));
    let _ = item.icon.svg().paint(ui, square(center, 18.), tint);

    if !style.compact {
        let right = rect.right() - 10.;
        let mut text_right = right;
        if let Some(icon) = style.disclosure {
            let _ = icon
                .svg()
                .paint(ui, square(pos2(right - 6., center.y), 12.), tint);
            text_right -= 18.;
        } else if let Some(count) = item.badge {
            let count = if count > 999 {
                "999+".into()
            } else {
                count.to_string()
            };
            let badge = painter.layout_no_wrap(count, Type::Badge.font(), colors.text_secondary);
            text_right -= badge.size().x + 12.;
            let fill = Rect::from_min_max(
                pos2(text_right + 4., center.y - 9.),
                pos2(right + 2., center.y + 9.),
            );
            painter.rect_filled(
                fill,
                Radius::Full.for_size(fill.size()),
                colors.surface_secondary,
            );
            painter.galley(fill.center() - badge.size() * 0.5, badge, tint);
        }
        let text_left = center.x + 19.;
        let role = if style.selected {
            Type::Label
        } else {
            Type::Body
        };
        let label = WidgetText::from(role.text(item.label).color(tint)).into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            (text_right - text_left).max(0.),
            role.font(),
        );
        painter.galley(
            Align2::LEFT_CENTER
                .align_size_within_rect(
                    label.size(),
                    Rect::from_min_max(pos2(text_left, rect.top()), rect.max),
                )
                .min,
            label,
            colors.text_primary,
        );
    }
    motion::focus_ring(ui, &response, radius);
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    if style.compact {
        response.on_hover_text(item.label)
    } else {
        response
    }
}
