use super::capture::Scene;
use crate::{Page, overlays::OverlayScene};
use spenstar_ui::theme::Mode;

use Mode::{Dark, Light};

pub(super) const LEGACY_SCENES: &[Scene] = &[
    Scene::new("01-overview-dark.png", Page::Overview, Dark),
    Scene::new("02-overview-light.png", Page::Overview, Light),
    Scene {
        inspector: true,
        ..Scene::new("03-overview-inspector-dark.png", Page::Overview, Dark)
    },
    Scene::new("04-buttons-dark.png", Page::Buttons, Dark),
    Scene::new("05-buttons-light.png", Page::Buttons, Light),
    Scene::new("06-icons-dark.png", Page::Icons, Dark),
    Scene::new("07-icons-light.png", Page::Icons, Light),
    Scene::new("08-tokens-dark.png", Page::Tokens, Dark),
    Scene::new("09-forms-dark.png", Page::Forms, Dark),
    Scene::new("10-forms-light.png", Page::Forms, Light),
    Scene::new("11-pickers-dark.png", Page::Pickers, Dark),
    Scene::new("12-pickers-light.png", Page::Pickers, Light),
    Scene::new("13-inbox-light.png", Page::Inbox, Light),
    Scene::new("14-inbox-dark.png", Page::Inbox, Dark),
    Scene::new("15-settings-dark.png", Page::Settings, Dark),
    Scene::new("16-settings-light.png", Page::Settings, Light),
    Scene::new("17-workspace-dark.png", Page::Workspace, Dark),
    Scene::new("18-orders-light.png", Page::Orders, Light),
    Scene::new("19-tracker-light.png", Page::Tracker, Light),
];

pub(super) const WAVE3_SCENES: &[Scene] = &[
    Scene::new("01-dashboard-light.png", Page::Overview, Light),
    Scene::new("02-dashboard-dark.png", Page::Overview, Dark),
    Scene::new("03-crm-dark.png", Page::Crm, Dark),
    Scene::new("04-crm-light.png", Page::Crm, Light),
    Scene::new("05-portfolio-light.png", Page::Portfolio, Light),
    Scene::new("06-portfolio-dark.png", Page::Portfolio, Dark),
    Scene::new("07-analytics-light.png", Page::Analytics, Light),
    Scene::new("08-analytics-dark.png", Page::Analytics, Dark),
    Scene::new("09-data-charts-dark.png", Page::Data, Dark),
    Scene::new("10-data-charts-light.png", Page::Data, Light),
    Scene {
        command: true,
        ..Scene::new("11-command-palette-dark.png", Page::Analytics, Dark)
    },
    Scene {
        command: true,
        ..Scene::new("12-command-palette-light.png", Page::Analytics, Light)
    },
];

pub(super) const WAVE4_SCENES: &[Scene] = &[
    Scene::w4(
        "01-agent-workbench-dark.png",
        Page::Workbench,
        Dark,
        OverlayScene::None,
    ),
    Scene::w4(
        "02-agent-workbench-light.png",
        Page::Workbench,
        Light,
        OverlayScene::None,
    ),
    Scene::w4(
        "03-agent-workbench-drawer-dark.png",
        Page::Workbench,
        Dark,
        OverlayScene::WorkbenchDrawer,
    ),
    Scene::w4(
        "04-overlays-dark.png",
        Page::Overlays,
        Dark,
        OverlayScene::Toasts,
    ),
    Scene::w4(
        "05-overlays-light.png",
        Page::Overlays,
        Light,
        OverlayScene::Toasts,
    ),
    Scene::w4(
        "06-modal-dark.png",
        Page::Overlays,
        Dark,
        OverlayScene::Alert,
    ),
    Scene::w4(
        "07-modal-light.png",
        Page::Overlays,
        Light,
        OverlayScene::Modal,
    ),
    Scene::w4(
        "08-drawer-light.png",
        Page::Overlays,
        Light,
        OverlayScene::Drawer,
    ),
    Scene::w4(
        "09-dropdown-dark.png",
        Page::Overlays,
        Dark,
        OverlayScene::Dropdown,
    ),
    Scene::w4(
        "10-dropdown-light.png",
        Page::Overlays,
        Light,
        OverlayScene::Dropdown,
    ),
    Scene::w4(
        "11-dashboard-light.png",
        Page::Overview,
        Light,
        OverlayScene::None,
    ),
    Scene::w4(
        "12-dashboard-dark.png",
        Page::Overview,
        Dark,
        OverlayScene::None,
    ),
];
