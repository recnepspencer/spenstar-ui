use eframe::egui::Ui;
use spenstar_ui::{
    Theme,
    command::{CommandGroup, CommandItem},
    icon::Icon,
    theme::{Accent, Mode},
};

use crate::{
    Gallery,
    navigation::{NAVIGATION, Page},
};

impl Gallery {
    pub(super) fn show_workbench(&mut self, ui: &mut Ui) {
        let events = self.workbench.show(ui);
        if events.theme {
            let mode = match self.theme.mode {
                Mode::Light => Mode::Dark,
                Mode::Dark => Mode::Light,
            };
            self.theme = Theme::new(mode, self.theme.accent);
            self.theme.apply(ui.ctx());
            ui.ctx().request_repaint();
        }
        if events.gallery {
            self.page = Page::Overview;
            ui.ctx().request_repaint();
        }
        self.show_command(ui, events.commands);
    }

    pub(super) fn show_command(&mut self, ui: &Ui, activated: bool) {
        if activated {
            self.command.open();
        }
        let pages: Vec<Vec<_>> = NAVIGATION
            .iter()
            .map(|section| {
                section
                    .items
                    .iter()
                    .map(|item| CommandItem {
                        icon: Some(item.icon.into()),
                        ..CommandItem::new(item.id, item.label)
                    })
                    .collect()
            })
            .collect();
        let actions = [
            CommandItem {
                icon: Some(Icon::Sun.into()),
                keywords: "theme appearance",
                ..CommandItem::new("light", "Use light appearance")
            },
            CommandItem {
                icon: Some(Icon::Moon.into()),
                keywords: "theme appearance",
                ..CommandItem::new("dark", "Use dark appearance")
            },
            CommandItem {
                icon: Some(Icon::PanelLeft.into()),
                shortcut: Some(if cfg!(target_os = "macos") {
                    "⌘ B"
                } else {
                    "Ctrl B"
                }),
                ..CommandItem::new("sidebar", "Cycle sidebar")
            },
            CommandItem {
                icon: Some(Icon::Palette.into()),
                keywords: "theme color appearance",
                ..CommandItem::new("accent", "Cycle accent color")
            },
        ];
        let mut groups = vec![CommandGroup {
            label: "Appearance",
            items: &actions,
        }];
        groups.extend(
            NAVIGATION
                .iter()
                .zip(&pages)
                .map(|(section, items)| CommandGroup {
                    label: section.label,
                    items,
                }),
        );
        if let Some(id) = self.command.show(ui, &groups) {
            match id {
                "light" => self.theme = Theme::new(Mode::Light, self.theme.accent),
                "dark" => self.theme = Theme::new(Mode::Dark, self.theme.accent),
                "sidebar" => self.shell.sidebar.cycle(),
                "accent" => {
                    let presets = Accent::PRESETS;
                    let index = presets
                        .iter()
                        .position(|&accent| accent == self.theme.accent);
                    let next = index.map_or(0, |i| (i + 1) % presets.len());
                    self.theme = Theme::new(self.theme.mode, presets[next]);
                }
                _ => {
                    if let Some(page) = Page::ALL.into_iter().find(|page| page.id() == id) {
                        self.page = page;
                    }
                }
            }
            self.theme.apply(ui.ctx());
            ui.ctx().request_repaint();
        }
    }
}
