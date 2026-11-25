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

    /// Whether this UI can consume a shortcut without taking input from a widget or overlay.
    pub fn allows(self) -> bool {
        !self.ctx.text_edit_focused()
            && !self.ctx.egui_wants_keyboard_input()
            && self.ctx.memory(|memory| {
                let top = memory
                    .layer_ids()
                    .filter(|layer| {
                        layer.order == Order::Foreground && memory.areas().is_visible(layer)
                    })
                    .last();
                memory.allows_interaction(self.layer)
                    && top.is_none_or(|layer| {
                        self.layer == layer
                            || memory.areas().parent_layer(self.layer) == Some(layer)
                    })
            })
            && (self.overlay || !Popup::is_any_open(self.ctx))
    }

    /// Consume a pressed shortcut only if this UI can claim it.
    pub fn consume(self, shortcut: &KeyboardShortcut) -> bool {
        self.allows() && self.ctx.input_mut(|input| input.consume_shortcut(shortcut))
    }
}
