mod analytics;
mod crm;
mod portfolio;

pub(super) use analytics::Analytics;
pub(super) use crm::Crm;
pub(super) use portfolio::Portfolio;

use eframe::egui::{Align, Layout, Ui, vec2};
use spenstar_ui::{
    Theme,
    display::{Badge, Tone},
    layout::Card,
    typography::Type,
};

fn pair(ui: &mut Ui, fraction: f32, first: impl FnOnce(&mut Ui), second: impl FnOnce(&mut Ui)) {
    let width = ui.available_width();
    if width < 680. {
        first(ui);
        second(ui);
        return;
    }
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 12.;
        let width = width - 12.;
        panel_width(ui, width * fraction, first);
        panel_width(ui, width * (1. - fraction), second);
    });
}

fn panel_width(ui: &mut Ui, width: f32, content: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(vec2(width, 0.), Layout::top_down(Align::Min), |ui| {
        ui.set_width(width);
        content(ui);
    });
}

fn panel(ui: &mut Ui, title: &str, subtitle: &str, content: impl FnOnce(&mut Ui)) {
    Card::default().padding(14).show(ui, |card| {
        card.body(|ui| {
            ui.spacing_mut().item_spacing.y = 6.;
            let c = Theme::current(ui.ctx()).colors;
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 3.;
                ui.label(Type::H3.text(title));
                ui.label(Type::Small.text(subtitle).color(c.text_secondary));
            });
            content(ui);
        });
    });
}

fn summary(ui: &mut Ui, text: &str) {
    ui.label(
        Type::Description
            .text(text)
            .color(Theme::current(ui.ctx()).colors.text_secondary),
    );
}

fn trend(label: &str, tone: Tone) -> Badge<'_> {
    Badge {
        tone,
        ..Badge::new(label)
    }
}
