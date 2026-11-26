use egui::{CentralPanel, Frame, Id, IdSalt, Key, KeyboardShortcut, Margin, Modifiers, Panel, Ui};

use crate::{Theme, button::CloseButton, input::ShortcutGuard, typography::Type};

use super::Sidebar;

/// Main page area with consistent canvas padding.
pub struct MainContent;

impl MainContent {
    /// Fill the remaining shell area.
    pub fn show<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let colors = Theme::current(ui.ctx()).colors;
        CentralPanel::no_frame()
            .frame(
                Frame::new()
                    .fill(colors.background)
                    .inner_margin(Margin::symmetric(24, 0)),
            )
            .show(ui, |ui| {
                ui.visuals_mut().extreme_bg_color = colors.surface;
                content(ui)
            })
            .inner
    }
}

/// Optional right-side shell panel.
pub struct Inspector<'a> {
    width: f32,
    heading: Option<(&'a str, &'a mut bool)>,
    content: &'a mut dyn FnMut(&mut Ui),
}

impl<'a> Inspector<'a> {
    /// Create a panel with the given preferred width and content.
    pub fn new(width: f32, content: &'a mut dyn FnMut(&mut Ui)) -> Self {
        Self {
            width: width.max(0.),
            heading: None,
            content,
        }
    }

    /// Add a named header and a dismiss button that clears the supplied open state.
    pub fn heading(mut self, title: &'a str, open: &'a mut bool) -> Self {
        self.heading = Some((title, open));
        self
    }

    fn show(self, ui: &mut Ui, id: Id) {
        let width = self.width.min(ui.available_width() * 0.4);
        if width < 40. {
            return;
        }
        let colors = Theme::current(ui.ctx()).colors;
        let frame = Frame::new()
            .fill(colors.surface)
            .inner_margin(Margin::same(20));
        Panel::right(id)
            .resizable(false)
            .show_separator_line(true)
            .exact_size(width)
            .frame(frame)
            .show(ui, |ui| {
                ui.visuals_mut().extreme_bg_color = colors.surface_secondary;
                if let Some((title, open)) = self.heading {
                    ui.horizontal(|ui| {
                        ui.label(Type::H3.text(title));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if CloseButton::show(ui).clicked() {
                                *open = false;
                            }
                        });
                    });
                    ui.add_space(12.);
                }
                (self.content)(ui);
            });
    }
}

/// Dashboard chrome with left navigation, header, main content, and optional inspector.
pub struct AppShell {
    id: IdSalt,
    /// Sidebar state, which may be changed by the app or its keyboard shortcut.
    pub sidebar: Sidebar,
}

impl AppShell {
    /// Shortcut for advancing the sidebar's collapse mode (Command on macOS, Control elsewhere).
    pub const SIDEBAR_SHORTCUT: KeyboardShortcut =
        KeyboardShortcut::new(Modifiers::COMMAND, Key::B);

    /// Create a shell with a stable UI-local ID.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: IdSalt::new(id),
            sidebar: Sidebar::default(),
        }
    }

    /// Render the shell, returning the main content's result.
    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        sidebar: impl FnOnce(&mut Ui, Sidebar),
        header: impl FnOnce(&mut Ui),
        main: impl FnOnce(&mut Ui) -> R,
        inspector: Option<Inspector<'_>>,
    ) -> R {
        let id = ui.make_persistent_id(self.id);
        let colors = Theme::current(ui.ctx()).colors;
        ui.painter()
            .rect_filled(ui.available_rect_before_wrap(), 0., colors.background);
        let sidebar_state = self.sidebar;
        sidebar_state.panel(ui, id.with("sidebar"), |ui| sidebar(ui, sidebar_state));
        if let Some(inspector) = inspector {
            inspector.show(ui, id.with("inspector"));
        }
        let result = MainContent::show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 12.;
            Panel::top(id.with("header"))
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::new().outer_margin(Margin {
                    bottom: 12,
                    ..Margin::ZERO
                }))
                .show(ui, header);
            main(ui)
        });
        if ShortcutGuard::new(ui).consume(&Self::SIDEBAR_SHORTCUT) {
            self.sidebar.cycle();
            ui.ctx().request_repaint();
        }
        result
    }
}
