use spenstar_ui::{
    icon::Icon,
    layout::{MenuItem, MenuSection},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Page {
    #[default]
    Overview,
    Buttons,
    Icons,
    Tokens,
    Forms,
    Pickers,
}

impl Page {
    pub(super) const ALL: [Self; 6] = [
        Self::Overview,
        Self::Buttons,
        Self::Icons,
        Self::Tokens,
        Self::Forms,
        Self::Pickers,
    ];

    pub(super) fn id(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Buttons => "buttons",
            Self::Icons => "icons",
            Self::Tokens => "tokens",
            Self::Forms => "forms",
            Self::Pickers => "pickers",
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Overview => "Dashboard Overview",
            Self::Buttons => "Buttons & Triggers",
            Self::Icons => "SVG Icon Browser",
            Self::Tokens => "Tokens & Typography",
            Self::Forms => "Forms & Controls",
            Self::Pickers => "Pickers & Dates",
        }
    }

    pub(super) fn description(self) -> &'static str {
        match self {
            Self::Overview => "Monitor deliveries, activity, and exceptions across your workspace.",
            Self::Buttons => {
                "Try each treatment and see how states, icons, and grouping work together."
            }
            Self::Icons => {
                "Browse the bundled Lucide collection and preview it at any display scale."
            }
            Self::Tokens => "Inspect the colors and type styles used throughout the gallery.",
            Self::Forms => "Edit values, adjust ranges, and try a new accent color.",
            Self::Pickers => "Search and select options, dates, and times.",
        }
    }
}

const COMPONENTS: &[MenuItem<'_>] = &[
    MenuItem::new("buttons", "Buttons & Triggers", Icon::Command),
    MenuItem::new("icons", "SVG Icon Browser", Icon::Grid).badge(Icon::ALL.len()),
    MenuItem::new("tokens", "Tokens & Typography", Icon::Palette),
    MenuItem::new("forms", "Forms & Controls", Icon::SlidersHorizontal),
    MenuItem::new("pickers", "Pickers & Dates", Icon::Calendar),
];
pub(super) const NAVIGATION: &[MenuSection<'_>] = &[
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
pub(super) const LIBRARY: &[MenuSection<'_>] = &[MenuSection {
    label: "Component library",
    items: COMPONENTS,
}];
