//! Keyboard shortcut arbitration for nested surfaces.
use egui::{Context, KeyboardShortcut, LayerId, Order, Popup, Ui};

/// A shortcut handler tied to a UI layer. Process overlay actions before background actions.
#[derive(Clone, Copy)]
pub struct ShortcutGuard<'a> {
    ctx: &'a Context,
    layer: LayerId,
    overlay: bool,
}

impl<'a> ShortcutGuard<'a> {
    /// Guard a background or regular UI's shortcuts.
    pub fn new(ui: &'a Ui) -> Self {
        Self {
            ctx: ui.ctx(),
            layer: ui.layer_id(),
            overlay: false,
        }
    }

    /// Guard shortcuts owned by the currently active overlay.
    pub fn for_overlay(ui: &'a Ui) -> Self {
        Self {
            overlay: true,
            ..Self::new(ui)
        }
    }

    /// Whether this layer can handle shortcuts and no text editor has focus.
    pub fn allows(self) -> bool {
        !self.ctx.text_edit_focused() && self.owns_layer()
    }

    pub(crate) fn owns_layer(self) -> bool {
        self.owns_layer_in(&[])
    }

    pub(crate) fn owns_layer_in(self, stack: &[LayerId]) -> bool {
        self.ctx.memory(|memory| {
            let top = memory
                .layer_ids()
                .filter(|layer| {
                    layer.order == Order::Foreground && memory.areas().is_visible(layer)
                })
                .last();
            memory.allows_interaction(self.layer)
                && top.is_none_or(|layer| {
                    self.layer == layer
                        || stack.contains(&layer)
                        || memory.areas().parent_layer(self.layer) == Some(layer)
                })
        }) && (self.overlay || !Popup::is_any_open(self.ctx))
    }

    /// Consume a pressed shortcut, reserving unmodified keys for focused widgets.
    pub fn consume(self, shortcut: &KeyboardShortcut) -> bool {
        self.allows()
            && (shortcut.modifiers.any() || !self.ctx.egui_wants_keyboard_input())
            && self.ctx.input_mut(|input| input.consume_shortcut(shortcut))
    }
}

pub(crate) fn lock_arrows(response: &egui::Response, horizontal: bool, vertical: bool) {
    if response.has_focus() {
        response.ctx.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    horizontal_arrows: horizontal,
                    vertical_arrows: vertical,
                    ..Default::default()
                },
            )
        });
    }
}
