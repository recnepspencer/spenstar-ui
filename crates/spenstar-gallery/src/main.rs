//! Interactive gallery for the spenstar-ui workspace.
#![forbid(unsafe_code)]
mod buttons;
mod dashboard;
mod icons;
#[cfg(test)]
mod tests;
mod tokens;

use eframe::egui::{
    self, Align, ComboBox, Context, Id, Key, KeyboardShortcut, Modifiers, ScrollArea, TextEdit, Ui,
};
use spenstar_ui::{
    Theme,
    button::{Button, CloseButton, Size, Variant},
    icon::Icon,
    input::ShortcutGuard,
    layout::{
        AppShell, CollapseMode, Inspector, MenuItem, MenuSection, PageHeader, SearchTrigger,
        Separator, SidebarEvent, SidebarPage, SidebarVariant, Toolbar,
    },
    theme::{Accent, Mode},
    typography::Type,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Overview,
    Buttons,
    Icons,
    Tokens,
}

impl Page {
    const ALL: [Self; 4] = [Self::Overview, Self::Buttons, Self::Icons, Self::Tokens];

    fn id(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Buttons => "buttons",
            Self::Icons => "icons",
            Self::Tokens => "tokens",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Dashboard Overview",
            Self::Buttons => "Buttons & Triggers",
            Self::Icons => "SVG Icon Browser",
            Self::Tokens => "Tokens & Typography",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Overview => "Monitor deliveries, activity, and exceptions across your workspace.",
            Self::Buttons => {
                "Try each treatment and see how states, icons, and grouping work together."
            }
            Self::Icons => {
                "Browse the bundled Lucide collection and preview it at any display scale."
            }
            Self::Tokens => "Inspect the colors and type styles used throughout the gallery.",
        }
    }
}

const COMPONENTS: &[MenuItem<'_>] = &[
    MenuItem::new("buttons", "Buttons & Triggers", Icon::Command),
    MenuItem::new("icons", "SVG Icon Browser", Icon::Grid).badge(Icon::ALL.len()),
    MenuItem::new("tokens", "Tokens & Typography", Icon::Palette),
];
const NAVIGATION: &[MenuSection<'_>] = &[
    MenuSection {
        label: "Workspace",
        items: &[
            MenuItem::new("overview", "Overview", Icon::LayoutDashboard),
            MenuItem::new("library", "Component library", Icon::Layers),
        ],
    },
    MenuSection {
        label: "Gallery",
        items: &[MenuItem::new("components", "Components", Icon::Package).group(COMPONENTS)],
    },
];
const LIBRARY: &[MenuSection<'_>] = &[MenuSection {
    label: "Component library",
    items: COMPONENTS,
}];

struct Gallery {
    shell: AppShell,
    theme: Theme,
    page: Page,
    drill_down: bool,
    inspector_open: bool,
    command_open: bool,
    command_query: String,
    buttons: buttons::Demo,
    icons: icons::Browser,
    dashboard: dashboard::Dashboard,
}

impl Gallery {
    fn new(ctx: &Context) -> Self {
        let theme = Theme::new(Mode::Dark, Accent::Blue);
        theme.install(ctx);
        Self {
            shell: AppShell::new("gallery"),
            theme,
            page: Page::Overview,
            drill_down: false,
            inspector_open: false,
            command_open: false,
            command_query: String::new(),
            buttons: buttons::Demo::default(),
            icons: icons::Browser::default(),
            dashboard: dashboard::Dashboard::default(),
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let c = self.theme.colors;
        let mut sidebar_event = None;
        let mut sidebar_settings = self.shell.sidebar;
        let mut requested_theme = None;
        let mut open_command = false;
        let mut requested_mode = None;
        let mut next_page = None;
        let mut settings = |ui: &mut Ui| {
            ui.label(Type::H3.text("Appearance"));
            ui.label(
                Type::Small
                    .text("Change the live shell and its tokens.")
                    .color(c.text_secondary),
            );
            Separator::Horizontal.show(ui);
            selector(
                ui,
                "Sidebar style",
                &mut sidebar_settings.variant,
                &[
                    SidebarVariant::Standard,
                    SidebarVariant::Inset,
                    SidebarVariant::Floating,
                ],
            );
            selector(
                ui,
                "Sidebar mode",
                &mut sidebar_settings.mode,
                &[
                    CollapseMode::Expanded,
                    CollapseMode::IconRail,
                    CollapseMode::Offcanvas,
                ],
            );
            let mut mode = self.theme.mode;
            let mut accent = self.theme.accent;
            selector(ui, "Appearance", &mut mode, &[Mode::Light, Mode::Dark]);
            selector(ui, "Accent", &mut accent, &Accent::PRESETS);
            requested_theme = Some(Theme::new(mode, accent));
        };
        let configured_sidebar = self.shell.sidebar;
        let inspector = self
            .inspector_open
            .then(|| Inspector::new(216., &mut settings));
        let page = self.page;
        self.shell.show(
            ui,
            |ui, sidebar| {
                sidebar_event = sidebar.show(
                    ui,
                    SidebarPage {
                        id: if self.drill_down {
                            "library"
                        } else {
                            "workspace"
                        },
                        parent: self.drill_down.then_some("workspace"),
                        sections: if self.drill_down { LIBRARY } else { NAVIGATION },
                        active: Some(page.id()),
                    },
                    |ui, compact| {
                        if compact {
                            ui.vertical_centered(|ui| Icon::Layers.show(ui, 22., c.accent.ring));
                        } else {
                            Toolbar::show(ui, |ui| {
                                Icon::Layers.show(ui, 22., c.accent.ring);
                                ui.vertical(|ui| {
                                    ui.label(Type::H4.text("spenstar-ui"));
                                    ui.label(
                                        Type::Caption
                                            .text("Component gallery")
                                            .color(c.text_secondary),
                                    );
                                });
                            });
                        }
                    },
                    |ui, compact| {
                        if !compact {
                            ui.horizontal(|ui| {
                                Icon::User.show(ui, 16., c.text_secondary);
                                ui.label(Type::Small.text("Demo workspace"));
                            });
                        }
                    },
                );
            },
            |ui| {
                let hint = if cfg!(target_os = "macos") {
                    "⌘ K"
                } else {
                    "Ctrl K"
                };
                let header = PageHeader::new(page.title())
                    .subtitle(page.description())
                    .search(SearchTrigger {
                        label: "Quick jump",
                        shortcut: Some(hint),
                    });
                open_command = header
                    .show(
                        ui,
                        |ui| {
                            ui.label(Type::Caption.text("Gallery").color(c.text_secondary));
                            Icon::ChevronRight.show(ui, 12., c.text_muted);
                            ui.label(Type::Caption.text(page.title()));
                        },
                        |ui| {
                            if Button::new("Appearance settings")
                                .icon_only(Icon::SlidersHorizontal)
                                .size(Size::Sm)
                                .variant(Variant::Outline)
                                .show(ui)
                                .clicked()
                            {
                                self.inspector_open = !self.inspector_open;
                            }
                            if Button::new("Cycle sidebar")
                                .icon_only(Icon::PanelLeft)
                                .size(Size::Sm)
                                .variant(Variant::Ghost)
                                .show(ui)
                                .clicked()
                            {
                                requested_mode = Some(configured_sidebar.mode.next());
                            }
                        },
                    )
                    .inner
                    .is_some_and(|response| response.clicked());
            },
            |ui| {
                ScrollArea::vertical()
                    .id_salt(("page", page.id()))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match page {
                            Page::Overview => next_page = self.dashboard.show(ui),
                            Page::Buttons => self.buttons.show(ui),
                            Page::Icons => self.icons.show(ui),
                            Page::Tokens => tokens::show(ui),
                        }
                        ui.add_space(12.);
                    });
            },
            inspector,
        );

        if sidebar_settings.variant != configured_sidebar.variant {
            self.shell.sidebar.variant = sidebar_settings.variant;
        }
        if sidebar_settings.mode != configured_sidebar.mode {
            self.shell.sidebar.mode = sidebar_settings.mode;
        }
        if let Some(mode) = requested_mode {
            self.shell.sidebar.mode = mode;
            ui.ctx().request_repaint();
        }
        match sidebar_event {
            Some(SidebarEvent::Navigate("library")) => self.drill_down = true,
            Some(SidebarEvent::Navigate(id)) => {
                next_page = Page::ALL.into_iter().find(|page| page.id() == id)
            }
            Some(SidebarEvent::Back(_)) => self.drill_down = false,
            Some(SidebarEvent::Theme(mode)) => {
                requested_theme = Some(Theme::new(mode, self.theme.accent));
            }
            None => {}
        }
        if let Some(page) = next_page {
            self.page = page;
            ui.ctx().request_repaint();
        }
        if let Some(theme) = requested_theme
            && (theme.mode != self.theme.mode || theme.accent != self.theme.accent)
        {
            self.theme = theme;
            theme.apply(ui.ctx());
            ui.ctx().request_repaint();
        }
        if open_command
            || ShortcutGuard::new(ui).consume(&KeyboardShortcut::new(Modifiers::COMMAND, Key::K))
        {
            self.command_open = true;
            self.command_query.clear();
        }
        if self.command_open {
            self.command(ui.ctx(), open_command);
        }
    }

    fn command(&mut self, ctx: &Context, opened_with_click: bool) {
        let mut destination = None;
        let mut close = false;
        let colors = self.theme.colors;
        let modal =
            egui::Modal::new(Id::unique("spenstar-gallery/jump")).backdrop_color(colors.overlay);
        let out = modal.show(ctx, |ui| {
            ui.set_width(340.);
            ui.horizontal(|ui| {
                ui.label(Type::H3.text("Quick jump"));
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    close = CloseButton::show(ui).clicked();
                });
            });
            let input = ui.add(
                TextEdit::singleline(&mut self.command_query)
                    .hint_text("Find a page…")
                    .desired_width(f32::INFINITY),
            );
            if opened_with_click || !ctx.egui_wants_keyboard_input() {
                input.request_focus();
            }
            let query = self.command_query.to_lowercase();
            for page in Page::ALL {
                if page.title().to_lowercase().contains(&query)
                    && Button::new(page.title())
                        .variant(Variant::Ghost)
                        .show(ui)
                        .clicked()
                {
                    destination = Some(page);
                }
            }
        });
        if let Some(destination) = destination {
            self.page = destination;
        }
        if close || destination.is_some() || out.should_close() {
            self.command_open = false;
        }
    }
}

fn selector<T>(ui: &mut Ui, label: &str, selected: &mut T, choices: &[T])
where
    T: Copy + PartialEq + std::fmt::Debug,
{
    ui.label(Type::Small.text(label));
    ComboBox::from_id_salt(label)
        .width(ui.available_width().max(40.) - 12.)
        .selected_text(format!("{selected:?}"))
        .show_ui(ui, |ui| {
            for &choice in choices {
                ui.selectable_value(selected, choice, format!("{choice:?}"));
            }
        });
    ui.add_space(8.);
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "spenstar-ui Gallery",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1232., 808.])
                .with_min_inner_size([540., 420.]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Gallery::new(&cc.egui_ctx)))),
    )
}
