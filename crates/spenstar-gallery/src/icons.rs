use eframe::egui::{Color32, TextWrapMode, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Size, ToggleButtonGroup},
    icon::Icon,
    layout::{Card, CardVariant, EmptyState, ResponsiveGrid, Toolbar},
    typography::Type,
};

const SIZES: [f32; 5] = [14., 16., 20., 24., 32.];
const TINTS: [&str; 6] = ["Primary", "Muted", "Accent", "Success", "Warning", "Danger"];

pub(crate) struct Browser {
    query: String,
    size: usize,
    tint: Option<String>,
    copied: Option<Icon>,
}

impl Default for Browser {
    fn default() -> Self {
        Self {
            query: String::new(),
            size: 2,
            tint: Some(TINTS[0].into()),
            copied: None,
        }
    }
}

impl Browser {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let c = Theme::current(ui.ctx()).colors;
        Card::new(CardVariant::Flat).show(ui, |card| {
            card.body(|ui| {
                ui.spacing_mut().interact_size.y = 38.;
                ui.horizontal_wrapped(|ui| {
                    super::widgets::sized_width(ui, 226., 38., |ui| {
                        super::widgets::search(ui, &mut self.query, "Filter by icon name");
                    });
                    ToggleButtonGroup::new(
                        "icon-size",
                        ["14", "16", "20", "24", "32"]
                            .map(|label| Button::new(label).size(Size::Sm)),
                    )
                    .show(ui, &mut self.size);
                    super::widgets::sized_width(ui, 140., 38., |ui| {
                        super::widgets::select(ui, &mut self.tint, &TINTS, "Icon tint");
                    });
                });
            });
        });
        let query = self.query.trim().to_ascii_lowercase();
        let matching: Vec<_> = Icon::ALL
            .iter()
            .copied()
            .filter(|icon| icon.name().contains(&query))
            .collect();
        Toolbar::show(ui, |ui| {
            ui.label(
                Type::Small
                    .text(format!("{} of {} icons", matching.len(), Icon::ALL.len()))
                    .color(c.text_secondary),
            );
            ui.label(
                Type::Caption
                    .text(if let Some(icon) = self.copied {
                        format!("Copied {}", icon.name())
                    } else {
                        "Click an icon to copy its name".into()
                    })
                    .color(c.text_muted),
            );
        });
        if matching.is_empty() {
            EmptyState::new(
                Icon::Search,
                "No matches",
                "Search for a name such as arrow, folder, or user.",
            )
            .show(ui);
            return;
        }
        let tint = [
            c.text_primary,
            c.text_muted,
            c.accent.ring,
            c.success.ring,
            c.warning.ring,
            c.danger.ring,
        ][TINTS
            .iter()
            .position(|tint| Some(*tint) == self.tint.as_deref())
            .unwrap_or_default()];
        ResponsiveGrid::new(110.)
            .gap(8.)
            .max_columns(10)
            .show(ui, matching.len(), |ui, index| {
                let icon = matching[index];
                if tile(ui, icon, SIZES[self.size], tint) {
                    ui.ctx().copy_text(icon.name().to_owned());
                    self.copied = Some(icon);
                }
            });
    }
}

fn tile(ui: &mut Ui, icon: Icon, size: f32, tint: Color32) -> bool {
    Card::new(CardVariant::Interactive)
        .label(icon.name())
        .show(ui, |card| {
            card.body(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space((32. - size) * 0.5);
                    icon.show(ui, size, tint);
                    ui.add_space((32. - size) * 0.5);
                    ui.add(
                        eframe::egui::Label::new(Type::Small.text(icon.name()))
                            .selectable(false)
                            .wrap_mode(TextWrapMode::Truncate),
                    );
                });
            });
        })
        .response
        .on_hover_text(icon.name())
        .clicked()
}
