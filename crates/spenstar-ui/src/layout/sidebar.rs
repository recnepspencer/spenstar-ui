use egui::{Frame, Margin, Panel, Stroke, Ui};

use crate::{Theme, icon::Icon, motion, theme::Mode, typography::Type};

use super::{
    ScrollShadow,
    sidebar_row::{RowStyle, row},
};

/// The sidebar's relationship to the canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarVariant {
    /// Full-height panel adjacent to the page.
    #[default]
    Standard,
    /// Bordered, rounded panel with canvas padding.
    Inset,
    /// Detached panel with a soft shadow.
    Floating,
}

/// Sidebar visibility and density.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollapseMode {
    /// Icons, labels, groups, and badges.
    #[default]
    Expanded,
    /// Slim icon-only navigation with tooltips.
    IconRail,
    /// Fully hidden.
    Offcanvas,
}

impl CollapseMode {
    /// Next mode in the expanded, rail, hidden cycle.
    pub const fn next(self) -> Self {
        match self {
            Self::Expanded => Self::IconRail,
            Self::IconRail => Self::Offcanvas,
            Self::Offcanvas => Self::Expanded,
        }
    }

    /// Resting width in egui points.
    pub const fn width(self) -> f32 {
        match self {
            Self::Expanded => 244.,
            Self::IconRail => 68.,
            Self::Offcanvas => 0.,
        }
    }
}

/// A selectable destination or a disclosure group.
#[derive(Clone, Copy, Debug)]
pub struct MenuItem<'a> {
    /// Unique page-local ID.
    pub id: &'a str,
    /// Visible and accessible title.
    pub label: &'a str,
    /// Leading icon, also used in the rail.
    pub icon: Icon,
    /// Optional numeric badge for destinations.
    pub badge: Option<usize>,
    /// Group children. An empty slice makes this row a destination.
    pub children: &'a [Self],
}

impl<'a> MenuItem<'a> {
    /// Create a destination without a badge.
    pub const fn new(id: &'a str, label: &'a str, icon: Icon) -> Self {
        Self {
            id,
            label,
            icon,
            badge: None,
            children: &[],
        }
    }

    /// Display a badge count with a destination.
    pub const fn badge(mut self, count: usize) -> Self {
        self.badge = Some(count);
        self
    }

    /// Turn this row into a disclosure group.
    pub const fn group(mut self, children: &'a [Self]) -> Self {
        self.children = children;
        self
    }
}

/// Heading and entries in a scrollable navigation section.
pub struct MenuSection<'a> {
    /// Section title, hidden in the rail.
    pub label: &'a str,
    /// Navigation rows.
    pub items: &'a [MenuItem<'a>],
}

/// Current navigation page, including optional drill-down controls.
pub struct SidebarPage<'a> {
    /// Stable page key used to scope disclosure and scroll state.
    pub id: &'a str,
    /// Optional parent destination ID for the back control.
    pub parent: Option<&'a str>,
    /// Navigation sections.
    pub sections: &'a [MenuSection<'a>],
    /// Currently selected destination.
    pub active: Option<&'a str>,
}

/// An action dispatched from sidebar navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarEvent<'a> {
    /// A destination was activated.
    Navigate(&'a str),
    /// The drill-down back control was activated.
    Back(&'a str),
    /// The appearance control was activated.
    Theme(Mode),
}

/// Dashboard sidebar settings and animated navigation view.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sidebar {
    /// The current positioning style.
    pub variant: SidebarVariant,
    /// The current collapse mode.
    pub mode: CollapseMode,
}

impl Sidebar {
    /// Advance to the next collapse mode.
    pub fn cycle(&mut self) {
        self.mode = self.mode.next();
    }

    /// Current animated width, including any outer inset.
    pub fn animated_width(self, ctx: &egui::Context, id: egui::Id) -> f32 {
        let visible = motion::eased(
            ctx,
            id.with("open"),
            self.mode != CollapseMode::Offcanvas,
            motion::NORMAL,
        );
        let expanded = motion::eased(
            ctx,
            id.with("labels"),
            self.mode == CollapseMode::Expanded,
            motion::NORMAL,
        );
        visible
            * egui::lerp(
                CollapseMode::IconRail.width()..=CollapseMode::Expanded.width(),
                expanded,
            )
    }

    pub(super) fn panel(self, ui: &mut Ui, id: egui::Id, contents: impl FnOnce(&mut Ui)) {
        let width = self
            .animated_width(ui.ctx(), id)
            .min(ui.available_width() * 0.58);
        if width < 1. {
            return;
        }
        let theme = Theme::current(ui.ctx());
        let c = theme.colors;
        let inset = (width / 8.).clamp(0., 8.) as i8;
        let frame = Frame::new().fill(c.surface);
        let frame = match self.variant {
            SidebarVariant::Standard => frame,
            SidebarVariant::Inset => frame
                .outer_margin(Margin::same(inset))
                .corner_radius(12)
                .stroke(Stroke::new(1., c.border_subtle)),
            SidebarVariant::Floating => frame
                .outer_margin(Margin::same(inset))
                .corner_radius(12)
                .stroke(Stroke::new(1., c.border_subtle))
                .shadow(theme.shadow()),
        };
        Panel::left(id)
            .resizable(false)
            .show_separator_line(self.variant == SidebarVariant::Standard)
            .exact_size(width)
            .frame(frame)
            .show(ui, contents);
    }

    /// Render header and footer slots around the scrollable navigation.
    ///
    /// The slot callbacks receive whether the current width requires the icon rail.
    pub fn show<'a>(
        self,
        ui: &mut Ui,
        page: SidebarPage<'a>,
        header: impl FnOnce(&mut Ui, bool),
        footer: impl FnOnce(&mut Ui, bool),
    ) -> Option<SidebarEvent<'a>> {
        let compact = ui.available_width() < 180.;
        if ui.available_width() < 36. {
            return None;
        }
        let colors = Theme::current(ui.ctx()).colors;
        let top = Frame::new().inner_margin(Margin::same(8));
        Panel::top("sidebar-header")
            .resizable(false)
            .frame(top)
            .show(ui, |ui| header(ui, compact));

        let bottom = Frame::new().inner_margin(Margin::same(8));
        let theme = Panel::bottom("sidebar-footer")
            .resizable(false)
            .frame(bottom)
            .show(ui, |ui| {
                footer(ui, compact);
                let current = Theme::current(ui.ctx()).mode;
                let (mode, label, icon) = match current {
                    Mode::Light => (Mode::Dark, "Dark appearance", Icon::Moon),
                    Mode::Dark => (Mode::Light, "Light appearance", Icon::Sun),
                };
                row(
                    ui,
                    MenuItem::new("theme", label, icon),
                    RowStyle {
                        compact,
                        ..Default::default()
                    },
                )
                .clicked()
                .then_some(SidebarEvent::Theme(mode))
            })
            .inner;

        let navigation =
            ScrollShadow::new(("sidebar-native-scroll", page.id), ui.available_height())
                .fill(colors.surface)
                .show(ui, |ui| {
                    Frame::new()
                        .inner_margin(Margin::symmetric(8, 2))
                        .show(ui, |ui| {
                            let mut event = None;
                            if let Some(parent) = page.parent
                                && row(
                                    ui,
                                    MenuItem::new("back", "Back", Icon::ArrowLeft),
                                    RowStyle {
                                        compact,
                                        ..Default::default()
                                    },
                                )
                                .clicked()
                            {
                                event = Some(SidebarEvent::Back(parent));
                            }
                            for section in page.sections {
                                if !compact && !section.label.is_empty() {
                                    ui.add_space(8.);
                                    ui.label(
                                        Type::Caption
                                            .text(section.label)
                                            .color(colors.text_secondary),
                                    );
                                }
                                for item in section.items {
                                    let next = menu_item(ui, *item, &page, compact, 0);
                                    event = next.or(event);
                                }
                            }
                            event
                        })
                        .inner
                });
        theme.or(navigation)
    }
}

fn menu_item<'a>(
    ui: &mut Ui,
    item: MenuItem<'a>,
    page: &SidebarPage<'a>,
    compact: bool,
    depth: usize,
) -> Option<SidebarEvent<'a>> {
    let id = ui.make_persistent_id(("sidebar-group", page.id, item.id));
    let group = !item.children.is_empty();
    let selected = page.active == Some(item.id);
    let mut open = group
        && ui
            .ctx()
            .data(|data| data.get_temp::<bool>(id))
            .unwrap_or(true);
    let disclosure = group.then_some(if open {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    });
    let response = row(
        ui,
        item,
        RowStyle {
            compact,
            selected,
            depth,
            disclosure,
        },
    );
    if response.clicked() {
        if !group {
            return Some(SidebarEvent::Navigate(item.id));
        }
        open = !open;
        ui.ctx().data_mut(|data| data.insert_temp(id, open));
    }
    if open {
        let mut event = None;
        for child in item.children {
            let next = menu_item(ui, *child, page, compact, depth + 1);
            event = next.or(event);
        }
        return event;
    }
    None
}
