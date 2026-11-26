//! Floating dialogs, drawers, menus, popovers, and tooltips.
mod anchored;
mod dialog;
mod menu;
mod menu_item;
mod menu_row;
mod panel;

pub use anchored::{Popover, Tooltip};
pub use dialog::{AlertDialog, AlertDialogResponse};
pub use menu::{ContextMenu, DropdownMenu, MenuItem};
pub use panel::{Drawer, DrawerSide, Modal, PanelResponse};

use egui::{Context, Frame, Key, Margin, Modifiers, Stroke, Ui};

use crate::{Theme, input::ShortcutGuard, typography::Type};

fn card(ctx: &Context, radius: u8, inset: i8) -> Frame {
    let theme = Theme::current(ctx);
    Frame::new()
        .fill(theme.colors.overlay)
        .stroke(Stroke::new(1., theme.colors.overlay_border))
        .corner_radius(radius)
        .shadow(theme.shadow())
        .inner_margin(Margin::same(inset))
}

fn set_input_surface(ui: &mut Ui) {
    ui.visuals_mut().extreme_bg_color = Theme::current(ui.ctx()).colors.surface_secondary;
}

fn key(ui: &Ui, key: Key) -> bool {
    ShortcutGuard::for_overlay(ui).owns_layer()
        && ui
            .ctx()
            .input_mut(|input| input.consume_key(Modifiers::NONE, key))
}

fn kbd(ui: &mut Ui, text: &str) {
    let c = Theme::current(ui.ctx()).colors;
    Frame::new()
        .fill(c.surface_secondary)
        .corner_radius(4)
        .inner_margin(Margin::symmetric(5, 1))
        .show(ui, |ui| {
            ui.label(Type::Kbd.text(text).color(c.text_primary));
        });
}
