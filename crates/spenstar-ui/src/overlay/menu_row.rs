use egui::{
    Align, Label, Layout, Rect, Response, Role, Sense, Ui, UiBuilder, WidgetInfo, pos2, vec2,
};

use crate::{
    Theme, controls::Tone, feedback::semantic_ink, icon::Icon, input::lock_arrows, typography::Type,
};

use super::{kbd, menu::MenuItem};

pub(super) fn row(
    ui: &mut Ui,
    item: &MenuItem<'_>,
    selected: bool,
    width: f32,
) -> Option<Response> {
    let c = Theme::current(ui.ctx()).colors;
    match item {
        MenuItem::Heading(label) => {
            ui.add_space(4.);
            ui.label(Type::SectionLabel.text(*label).color(c.text_secondary));
            None
        }
        MenuItem::Separator => {
            ui.separator();
            None
        }
        MenuItem::Action {
            label,
            icon,
            shortcut,
            checked,
            enabled,
            danger,
            children,
            ..
        } => {
            let (rect, response) = ui.allocate_exact_size(
                vec2(width, 29.),
                if *enabled {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            response.widget_info(|| {
                WidgetInfo::selected(Role::MenuItem, *enabled, checked.unwrap_or(false), *label)
            });
            lock_arrows(&response, true, true);
            let selected = *enabled && (selected || response.hovered());
            let fill = if selected { c.field_hover } else { c.overlay };
            if selected {
                ui.painter().rect_filled(rect, 6, fill);
            }
            let tone = if *danger { Tone::Danger } else { Tone::Neutral };
            let ink = if *enabled {
                semantic_ink(tone.palette(c).ring, fill, c.text_primary)
            } else {
                c.text_secondary
            };
            let inset = rect.shrink2(vec2(7., 0.));
            let mut content = ui.new_child(
                UiBuilder::new()
                    .max_rect(inset)
                    .layout(Layout::right_to_left(Align::Center)),
            );
            content.set_clip_rect(ui.clip_rect().intersect(rect));
            if !children.is_empty() {
                Icon::ChevronRight.show(&mut content, 13., ink);
            }
            if let Some(shortcut) = shortcut {
                kbd(&mut content, shortcut);
            }
            let remaining = content.available_rect_before_wrap();
            let label_rect = Rect::from_min_max(inset.min, pos2(remaining.right(), inset.bottom()));
            let mut content = ui.new_child(
                UiBuilder::new()
                    .max_rect(label_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            content.set_clip_rect(ui.clip_rect().intersect(label_rect));
            if *checked == Some(true) {
                Icon::Check.show(&mut content, 14., ink);
            } else if let Some(icon) = icon {
                icon.show(&mut content, 14., ink);
            } else {
                content.add_space(14.);
            }
            content.add(
                Label::new(Type::Description.text(*label).color(ink))
                    .selectable(false)
                    .truncate(),
            );
            Some(response)
        }
    }
}
