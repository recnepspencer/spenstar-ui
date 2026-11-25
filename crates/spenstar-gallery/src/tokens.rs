use eframe::egui::{Color32, Sense, Ui, vec2};
use spenstar_ui::{
    Theme,
    layout::{Card, CardVariant, ResponsiveGrid},
    typography::Type,
};

pub(crate) fn show(ui: &mut Ui) {
    let c = Theme::current(ui.ctx()).colors;
    Card::new(CardVariant::Bordered).show(ui, |card| {
        card.header(
            "Semantic colors",
            Some("Canvas, text, and borders for the active appearance."),
        );
        card.body(|ui| {
            let tokens = [
                ("Background", c.background),
                ("Surface", c.surface),
                ("Elevated", c.surface_elevated),
                ("Subtle", c.surface_subtle),
                ("Overlay", c.overlay),
                ("Border", c.border),
                ("Border subtle", c.border_subtle),
                ("Divider", c.divider),
                ("Text primary", c.text_primary),
                ("Text secondary", c.text_secondary),
                ("Text muted", c.text_muted),
            ];
            ResponsiveGrid::new(126.).show(ui, tokens.len(), |ui, index| {
                let (name, color) = tokens[index];
                swatch(ui, name, color);
            });
        });
    });
    Card::new(CardVariant::Bordered).show(ui, |card| {
        card.header(
            "Action palettes",
            Some("Each family resolves its interaction states from a single color."),
        );
        card.body(|ui| {
            for (name, palette) in [
                ("Default", c.default),
                ("Accent", c.accent),
                ("Success", c.success),
                ("Warning", c.warning),
                ("Danger", c.danger),
            ] {
                ui.label(Type::H4.text(name));
                let states = [
                    ("Base", palette.base),
                    ("Foreground", palette.foreground),
                    ("Hover", palette.hover),
                    ("Active", palette.active),
                    ("Soft", palette.soft),
                    ("Soft hover", palette.soft_hover),
                    ("Ring", palette.ring),
                ];
                ResponsiveGrid::new(88.).max_columns(7).gap(8.).show(
                    ui,
                    states.len(),
                    |ui, index| {
                        let (name, color) = states[index];
                        swatch(ui, name, color);
                    },
                );
                ui.add_space(12.);
            }
        });
    });
    Card::new(CardVariant::Bordered).show(ui, |card| {
        card.header(
            "Typography",
            Some("Inter and JetBrains Mono are bundled for offline use."),
        );
        card.body(|ui| {
            for (role, sample) in [
                (Type::Display, "Workspace"),
                (Type::H1, "Dispatch overview"),
                (Type::H2, "Shipment volume"),
                (Type::H3, "Activity feed"),
                (Type::H4, "Recently updated"),
                (Type::Body, "Your workspace is up to date."),
                (Type::Small, "Assigned to dispatch"),
                (Type::Caption, "Last updated 4 min ago"),
                (Type::Mono, "SHIPMENT-0482"),
                (Type::Kbd, "Ctrl + K"),
            ] {
                ui.label(
                    Type::Caption
                        .text(format!("{role:?} · {} pt", role.font().size))
                        .color(c.text_muted),
                );
                ui.label(role.text(sample));
                ui.add_space(8.);
            }
        });
    });
}

fn swatch(ui: &mut Ui, name: &str, color: Color32) {
    let c = Theme::current(ui.ctx()).colors;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 30.), Sense::hover());
    ui.painter().rect_filled(rect, 4., c.surface_subtle);
    ui.painter().rect(
        rect,
        4.,
        color,
        eframe::egui::Stroke::new(1., c.border_subtle),
        eframe::egui::StrokeKind::Inside,
    );
    ui.label(Type::Small.text(name));
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let hex = if a == 255 {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    };
    ui.label(Type::Caption.text(hex).color(c.text_secondary));
}
