//! Interactive gallery for the spenstar-ui workspace.
#![forbid(unsafe_code)]
mod appearance;
mod buttons;
mod capture;
mod capture_scenes;
mod dashboard;
mod data;
mod forms;
mod gallery_command;
mod icons;
mod inbox;
mod insights;
mod invite;
mod navigation;
mod orders;
mod overlays;
mod pickers;
mod settings;
mod shipments;
#[cfg(test)]
mod tests;
mod tokens;
mod tracker;
mod widgets;
mod workbench;
mod workspace;

use eframe::egui::{self, Context, ScrollArea, Sense, Ui, vec2};
use spenstar_ui::{
    CommandPalette, Theme,
    button::{Button, Variant},
    feedback::{ToastQueue, Tone},
    icon::{Icon, SvgIcon},
    layout::{AppShell, Inspector, PageHeader, SearchTrigger, SidebarEvent, SidebarPage, Toolbar},
    theme::{Accent, Mode},
    typography::Type,
};

use navigation::{NAVIGATION, Page};

struct Gallery {
    shell: AppShell,
    theme: Theme,
    page: Page,
    inspector_open: bool,
    appearance: appearance::Appearance,
    command: CommandPalette,
    crm: insights::Crm,
    portfolio: insights::Portfolio,
    analytics: insights::Analytics,
    data: data::DataDemo,
    buttons: buttons::Demo,
    icons: icons::Browser,
    dashboard: dashboard::Dashboard,
    orders: orders::Orders,
    invitations: invite::Invitations,
    inbox: inbox::Inbox,
    tracker: tracker::Tracker,
    settings: settings::Settings,
    workspace: workspace::Workspace,
    shipments: Vec<shipments::Shipment>,
    forms: forms::Demo,
    pickers: pickers::Demo,
    capture: Option<capture::Capture>,
    overlays: overlays::Showcase,
    workbench: workbench::Workbench,
}

impl Gallery {
    fn new(ctx: &Context) -> Self {
        let theme = Theme::new(Mode::Dark, Accent::Blue);
        theme.install(ctx);
        Self {
            shell: AppShell::new("gallery"),
            theme,
            page: Page::Overview,
            inspector_open: false,
            appearance: appearance::Appearance::default(),
            command: CommandPalette::new("spenstar-gallery/commands"),
            crm: Default::default(),
            portfolio: Default::default(),
            analytics: Default::default(),
            data: Default::default(),
            buttons: buttons::Demo::default(),
            icons: icons::Browser::default(),
            dashboard: dashboard::Dashboard::default(),
            orders: orders::Orders::default(),
            invitations: invite::Invitations::default(),
            inbox: inbox::Inbox::default(),
            tracker: tracker::Tracker::default(),
            settings: settings::Settings::default(),
            workspace: workspace::Workspace::default(),
            shipments: shipments::samples(),
            forms: forms::Demo::default(),
            pickers: pickers::Demo::default(),
            capture: None,
            overlays: Default::default(),
            workbench: Default::default(),
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        if self.page == Page::Workbench {
            self.show_workbench(ui);
        } else {
            self.show_gallery(ui);
        }
        if ui.ctx().output(|output| {
            output
                .commands
                .iter()
                .any(|command| matches!(command, egui::OutputCommand::CopyText(_)))
        }) {
            widgets::notify(ui.ctx(), "Copied to clipboard", Tone::Success);
        }
        if !ToastQueue::show_context(ui.ctx()).is_empty() {
            widgets::notify(ui.ctx(), "Retry requested", Tone::Accent);
        }
    }

    fn show_gallery(&mut self, ui: &mut Ui) {
        let page = self.page;
        let theme = self.theme;
        let mut requested_theme = theme;
        let mut sidebar_event = None;
        let mut header = HeaderEvents::default();
        let mut form_accent = None;
        let configured_sidebar = self.shell.sidebar;
        let mut sidebar = configured_sidebar;
        let mut settings =
            |ui: &mut Ui| appearance::controls(ui, &mut requested_theme, &mut sidebar);
        let inspector = self.inspector_open.then(|| {
            Inspector::new(280., &mut settings).heading("Appearance", &mut self.inspector_open)
        });
        let mut selected_shipment = None;
        let mut open_palette = false;
        let capture_picker = self
            .capture
            .as_ref()
            .is_some_and(capture::Capture::open_select);
        self.shell.show(
            ui,
            |ui, sidebar| {
                sidebar_event = sidebar.show(
                    ui,
                    SidebarPage {
                        id: "gallery",
                        parent: None,
                        sections: NAVIGATION,
                        active: Some(page.id()),
                    },
                    brand,
                    |_, _| {},
                );
            },
            |ui| header = navbar(ui, page, theme.mode),
            |ui| {
                if page == Page::Inbox {
                    self.inbox.show(ui);
                    return;
                }
                ScrollArea::vertical()
                    .id_salt(("page", page.id()))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        match page {
                            Page::Overview => {
                                selected_shipment = self.dashboard.show(ui, &self.shipments)
                            }
                            Page::Inbox | Page::Workbench => {}
                            Page::Overlays => self.overlays.show(ui),
                            Page::Crm => self.crm.show(ui),
                            Page::Portfolio => self.portfolio.show(ui),
                            Page::Analytics => self.analytics.show(ui),
                            Page::Data => open_palette = self.data.show(ui),
                            Page::Orders => self.orders.show(ui),
                            Page::Tracker => self.tracker.show(ui),
                            Page::Settings => self.settings.show(ui),
                            Page::Workspace => self.workspace.show(ui, &mut self.shipments),
                            Page::Buttons => self.buttons.show(ui),
                            Page::Icons => self.icons.show(ui),
                            Page::Tokens => tokens::show(ui),
                            Page::Forms => {
                                if let Some(accent) = self.forms.show(ui, theme.accent) {
                                    form_accent = Some(accent);
                                }
                            }
                            Page::Pickers => self.pickers.show(ui, capture_picker),
                        }
                        ui.add_space(16.);
                    });
            },
            inspector,
        );
        if let Some(anchor) = &header.appearance {
            self.appearance.show(
                anchor,
                &mut requested_theme,
                &mut sidebar,
                &mut self.inspector_open,
            );
        }
        if let Some(mode) = header.mode {
            requested_theme = Theme::new(mode, requested_theme.accent);
        }
        if let Some(accent) = form_accent {
            requested_theme = Theme::new(requested_theme.mode, accent);
        }
        if sidebar.variant != configured_sidebar.variant {
            self.shell.sidebar.variant = sidebar.variant;
        }
        if header.cycle_sidebar {
            self.shell.sidebar.cycle();
        } else if sidebar.mode != configured_sidebar.mode {
            self.shell.sidebar.mode = sidebar.mode;
        }
        match sidebar_event {
            Some(SidebarEvent::Navigate(id)) => {
                if let Some(page) = Page::ALL.into_iter().find(|page| page.id() == id) {
                    self.page = page;
                }
            }
            Some(SidebarEvent::Theme(mode)) => {
                requested_theme = Theme::new(mode, requested_theme.accent)
            }
            _ => {}
        }
        match selected_shipment {
            Some(dashboard::RecordAction::Open(index)) => {
                self.workspace.open(index, &self.shipments);
                self.page = Page::Workspace;
            }
            Some(dashboard::RecordAction::Remove(index)) => {
                self.shipments.remove(index);
                self.workspace.removed(index);
                widgets::notify(ui.ctx(), "Shipment removed", Tone::Success);
            }
            None => {}
        }
        if header.new_shipment {
            self.workspace.new_shipment(&self.shipments);
            self.page = Page::Workspace;
        }
        if requested_theme.mode != theme.mode || requested_theme.accent != theme.accent {
            self.theme = requested_theme;
            requested_theme.apply(ui.ctx());
        }
        if self.page != page
            || self.shell.sidebar.variant != configured_sidebar.variant
            || self.shell.sidebar.mode != configured_sidebar.mode
            || requested_theme.mode != theme.mode
            || requested_theme.accent != theme.accent
        {
            ui.ctx().request_repaint();
        }
        self.invitations.show(ui, header.invite);
        self.show_command(ui, open_palette || header.open_command);
    }
}

#[derive(Default)]
struct HeaderEvents {
    appearance: Option<egui::Response>,
    cycle_sidebar: bool,
    open_command: bool,
    new_shipment: bool,
    invite: bool,
    mode: Option<Mode>,
}

fn navbar(ui: &mut Ui, page: Page, mode: Mode) -> HeaderEvents {
    let mut events = HeaderEvents::default();
    let hint = if cfg!(target_os = "macos") {
        "⌘ K"
    } else {
        "Ctrl K"
    };
    let header = PageHeader::new(page.title()).search(SearchTrigger {
        label: "Search pages and actions",
        shortcut: Some(hint),
    });
    events.open_command = header
        .show(
            ui,
            |ui| {
                events.cycle_sidebar = Button::new("Cycle sidebar")
                    .icon_only(Icon::PanelLeft)
                    .variant(Variant::Ghost)
                    .show(ui)
                    .clicked();
            },
            |ui| {
                if page == Page::Workspace {
                    events.new_shipment = Button::new("New shipment")
                        .leading(Icon::Plus)
                        .show(ui)
                        .clicked();
                } else if matches!(
                    page,
                    Page::Overview | Page::Orders | Page::Tracker | Page::Settings
                ) {
                    events.invite = Button::new("Invite").leading(Icon::Plus).show(ui).clicked();
                }
                events.appearance =
                    Some(widgets::action(ui, "Appearance", Icon::SlidersHorizontal));
                let (label, icon, tone) = match mode {
                    Mode::Light => ("Dark appearance", Icon::Moon, Mode::Dark),
                    Mode::Dark => ("Light appearance", Icon::Sun, Mode::Light),
                };
                events.mode = widgets::action(ui, label, icon).clicked().then_some(tone);
            },
        )
        .inner
        .is_some_and(|response| response.clicked());
    events
}

fn brand(ui: &mut Ui, compact: bool) {
    let c = Theme::current(ui.ctx()).colors;
    const LOGO: SvgIcon =
        SvgIcon::from_static("spenstar-mark", include_str!("../assets/spenstar-mark.svg"));
    let size = if compact { 28. } else { 30. };
    let mark = |ui: &mut Ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
        let _ = LOGO.paint(ui, rect, c.text_primary);
    };
    ui.add_space(8.);
    if compact {
        ui.vertical_centered(mark);
    } else {
        Toolbar::show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 10.;
            mark(ui);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.;
                ui.label(Type::Brand.text("Spenstar Logistics"));
                ui.label(Type::Small.text("Operations workspace").color(c.text_muted));
            });
        });
    }
    ui.add_space(4.);
}

impl eframe::App for Gallery {
    fn raw_input_hook(&mut self, _ctx: &Context, input: &mut egui::RawInput) {
        if let Some(capture) = &self.capture {
            capture.input(input);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if let Some(mut capture) = self.capture.take() {
            capture.prepare(self, ui.ctx());
            self.capture = Some(capture);
        }
        self.show(ui);
        if let Some(capture) = &mut self.capture {
            capture.after_frame(ui.ctx());
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let capture = capture::Capture::from_args()?;
    let result = capture.as_ref().map(capture::Capture::result);
    eframe::run_native(
        "spenstar-ui Gallery",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(capture::SIZE)
                .with_min_inner_size([540., 420.])
                .with_resizable(capture.is_none()),
            ..Default::default()
        },
        Box::new(move |cc| {
            let mut gallery = Gallery::new(&cc.egui_ctx);
            gallery.capture = capture;
            Ok(Box::new(gallery))
        }),
    )?;
    if let Some(result) = result {
        result
            .lock()
            .map_err(|_| "capture result is unavailable")?
            .take()
            .ok_or("capture stopped before all screenshots were written")??;
    }
    Ok(())
}
