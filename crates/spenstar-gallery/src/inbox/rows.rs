use eframe::egui::{self, Align2, Color32, Frame, Margin, Rect, Sense, Ui, pos2, vec2};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    icon::Icon,
    typography::Type,
};

use super::{LABELS, Thread};
use crate::orders::initials;

pub(super) fn folder_row(
    ui: &mut Ui,
    label: &str,
    icon: Option<Icon>,
    ink: Color32,
    count: Option<usize>,
    selected: bool,
) -> bool {
    let c = Theme::current(ui.ctx()).colors;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 34.), Sense::click());
    if selected || response.hovered() {
        ui.painter().rect_filled(
            rect,
            17,
            if selected {
                c.default.base
            } else {
                c.surface_tertiary
            },
        );
    }
    let marker = pos2(rect.left() + 17., rect.center().y);
    if let Some(icon) = icon {
        let _ = icon
            .svg()
            .paint(ui, Rect::from_center_size(marker, vec2(15., 15.)), ink);
    } else {
        ui.painter().circle_filled(marker, 4., ink);
    }
    ui.painter().text(
        marker + vec2(15., 0.),
        Align2::LEFT_CENTER,
        label,
        if selected {
            Type::Label.font()
        } else {
            Type::Body.font()
        },
        c.text_primary,
    );
    if let Some(count) = count.filter(|count| *count > 0) {
        ui.painter().text(
            pos2(rect.right() - 11., marker.y),
            Align2::RIGHT_CENTER,
            count.to_string(),
            Type::Badge.font(),
            c.text_muted,
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Button, response.enabled(), selected, label)
    });
    spenstar_ui::motion::focus_ring(ui, &response, 17.into());
    response.clicked()
}

pub(super) fn thread_row(ui: &mut Ui, thread: &mut Thread, selected: bool) -> bool {
    let Some(message) = thread.messages.first() else {
        return false;
    };
    let theme = Theme::current(ui.ctx());
    let c = theme.colors;
    let color = LABELS
        .iter()
        .find(|(label, _)| *label == thread.label)
        .map_or(c.accent.base, |(_, color)| *color);
    let frame = Frame::new()
        .inner_margin(Margin::same(12))
        .corner_radius(16)
        .fill(if selected {
            c.surface
        } else {
            Color32::TRANSPARENT
        })
        .shadow(if selected {
            theme.shadow()
        } else {
            egui::Shadow::NONE
        });
    let row = ui
        .scope_builder(egui::UiBuilder::new().sense(Sense::click()), |ui| {
            frame.show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.style_mut().interaction.selectable_labels = false;
                ui.spacing_mut().item_spacing.y = 5.;
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(34., 34.), Sense::hover());
                    initials(
                        ui,
                        rect,
                        &message.sender,
                        color.gamma_multiply(0.22),
                        c.text_primary,
                    );
                    if thread.unread {
                        let (dot, _) = ui.allocate_exact_size(vec2(7., 7.), Sense::hover());
                        ui.painter().circle_filled(dot.center(), 3.5, c.accent.base);
                    }
                    let sender = if thread.unread {
                        Type::Brand
                    } else {
                        Type::Label
                    };
                    ui.label(sender.text(&message.sender));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Button::new(if thread.starred {
                            "Unstar conversation"
                        } else {
                            "Star conversation"
                        })
                        .icon_only(Icon::Star)
                        .size(Size::Sm)
                        .variant(if thread.starred {
                            Variant::Soft
                        } else {
                            Variant::Ghost
                        })
                        .show(ui)
                        .clicked()
                        {
                            thread.starred = !thread.starred;
                        }
                        if ui.available_width() >= 54. {
                            let date = message.time.split(',').next().unwrap_or(&message.time);
                            ui.label(Type::Caption.text(date).color(c.text_muted));
                        }
                    });
                });
                ui.add(egui::Label::new(Type::Label.text(&thread.subject)).truncate());
                ui.add(
                    egui::Label::new(
                        Type::Small
                            .text(message.body.lines().next().unwrap_or_default())
                            .color(c.text_muted),
                    )
                    .truncate(),
                );
            });
        })
        .response;
    row.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Button, row.enabled(), selected, &thread.subject)
    });
    spenstar_ui::motion::focus_ring(ui, &row, 12.into());
    row.clicked()
}
