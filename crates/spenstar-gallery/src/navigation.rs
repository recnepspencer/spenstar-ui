use spenstar_ui::{
    icon::Icon,
    layout::{MenuItem, MenuSection},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Page {
    #[default]
    Overview,
    Orders,
    Tracker,
    Inbox,
    Settings,
    Workspace,
    Buttons,
    Icons,
    Tokens,
    Forms,
    Pickers,
    Crm,
    Portfolio,
    Analytics,
    Data,
}

impl Page {
    pub(super) const ALL: [Self; 15] = [
        Self::Overview,
        Self::Orders,
        Self::Tracker,
        Self::Inbox,
        Self::Settings,
        Self::Workspace,
        Self::Buttons,
        Self::Icons,
        Self::Tokens,
        Self::Forms,
        Self::Pickers,
        Self::Crm,
        Self::Portfolio,
        Self::Analytics,
        Self::Data,
    ];

    pub(super) fn id(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Orders => "orders",
            Self::Tracker => "tracker",
            Self::Inbox => "inbox",
            Self::Settings => "settings",
            Self::Workspace => "workspace",
            Self::Buttons => "buttons",
            Self::Icons => "icons",
            Self::Tokens => "tokens",
            Self::Forms => "forms",
            Self::Pickers => "pickers",
            Self::Crm => "crm",
            Self::Portfolio => "portfolio",
            Self::Analytics => "analytics",
            Self::Data => "data",
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Overview => "Good morning, Alex",
            Self::Orders => "Orders",
            Self::Tracker => "Tracker",
            Self::Inbox => "Inbox",
            Self::Settings => "Workspace settings",
            Self::Workspace => "Dispatch workspace",
            Self::Buttons => "Buttons & Triggers",
            Self::Icons => "SVG Icon Browser",
            Self::Tokens => "Tokens & Typography",
            Self::Forms => "Forms & Controls",
            Self::Pickers => "Pickers & Dates",
            Self::Crm => "CRM Pipeline",
            Self::Portfolio => "Portfolio",
            Self::Analytics => "Analytics",
            Self::Data => "Data & Charts",
        }
    }
}

pub(super) const NAVIGATION: &[MenuSection<'_>] = &[
    MenuSection {
        label: "Templates",
        items: &[
            MenuItem::new("overview", "Dashboard", Icon::LayoutDashboard),
            MenuItem::new("orders", "Orders", Icon::ClipboardList),
            MenuItem::new("crm", "CRM Pipeline", Icon::Kanban),
            MenuItem::new("portfolio", "Portfolio", Icon::Layers),
            MenuItem::new("analytics", "Analytics", Icon::LineChart),
            MenuItem::new("tracker", "Tracker", Icon::List),
            MenuItem::new("inbox", "Inbox", Icon::Mail),
            MenuItem::new("settings", "Settings", Icon::Settings),
            MenuItem::new("workspace", "Workspace", Icon::Truck),
        ],
    },
    MenuSection {
        label: "Components",
        items: &[
            MenuItem::new("buttons", "Buttons & Triggers", Icon::Command),
            MenuItem::new("icons", "SVG Icon Browser", Icon::Grid).badge(Icon::ALL.len()),
            MenuItem::new("tokens", "Tokens & Typography", Icon::Palette),
            MenuItem::new("forms", "Forms & Controls", Icon::SlidersHorizontal),
            MenuItem::new("pickers", "Pickers & Dates", Icon::Calendar),
            MenuItem::new("data", "Data & Charts", Icon::BarChart3),
        ],
    },
];
