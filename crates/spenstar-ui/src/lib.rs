//! Dashboard layouts, controls, typography, and HiDPI SVG icons for egui.
#![forbid(unsafe_code)]

pub mod button;
pub mod icon;
pub mod input;
pub mod layout;
pub mod motion;
pub mod theme;
pub mod typography;

pub use egui;
pub use theme::Theme;

/// Install offline fonts and the default theme.
pub fn install(ctx: &egui::Context) {
    Theme::default().install(ctx);
}
