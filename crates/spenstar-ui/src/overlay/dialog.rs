use egui::{Context, Frame, Margin};

use crate::{
    Theme,
    button::{Button, Variant},
    controls::Tone,
    feedback::{semantic_ink, status_icon},
    typography::Type,
};

use super::Modal;

/// Result of a confirmation prompt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertDialogResponse {
    /// No action in this frame.
    #[default]
    Pending,
    /// Cancelled using a button, Escape, or the close control.
    Cancelled,
    /// Confirmed by the user.
    Confirmed,
}

/// Structured confirmation prompt with semantic emphasis.
pub struct AlertDialog<'a> {
    modal: Modal<'a>,
    message: &'a str,
    tone: Tone,
    confirm: &'a str,
    cancel: &'a str,
}

impl<'a> AlertDialog<'a> {
    /// Create an accent confirmation dialog.
    pub fn new(
        id: impl std::hash::Hash + std::fmt::Debug,
        title: &'a str,
        message: &'a str,
    ) -> Self {
        Self {
            modal: Modal::new(id, title).dismiss_on_click_outside(false),
            message,
            tone: Tone::Accent,
            confirm: "Confirm",
            cancel: "Cancel",
        }
    }

    /// Apply a semantic tone. Danger uses a destructive confirmation button.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Customize the two action labels.
    pub fn actions(mut self, cancel: &'a str, confirm: &'a str) -> Self {
        self.cancel = cancel;
        self.confirm = confirm;
        self
    }

    /// Show the prompt while open and return the user's decision.
    pub fn show(self, ctx: &Context, open: &mut bool) -> AlertDialogResponse {
        use AlertDialogResponse::{Cancelled, Confirmed, Pending};
        let c = Theme::current(ctx).colors;
        let palette = self.tone.palette(c);
        let result = self.modal.show(
            ctx,
            open,
            |ui| {
                ui.horizontal_top(|ui| {
                    Frame::new()
                        .fill(palette.soft)
                        .corner_radius(8)
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            status_icon(self.tone).show(
                                ui,
                                18.,
                                semantic_ink(palette.ring, palette.soft, c.text_primary),
                            )
                        });
                    ui.add(
                        egui::Label::new(Type::Body.text(self.message).color(c.text_primary))
                            .wrap(),
                    );
                });
            },
            |ui| {
                let variant = match self.tone {
                    Tone::Danger => Variant::DangerSoft,
                    Tone::Warning => Variant::Secondary,
                    _ => Variant::Soft,
                };
                if Button::new(self.confirm)
                    .variant(variant)
                    .show(ui)
                    .clicked()
                {
                    Confirmed
                } else if Button::new(self.cancel)
                    .variant(Variant::Ghost)
                    .show(ui)
                    .clicked()
                {
                    Cancelled
                } else {
                    Pending
                }
            },
        );
        let Some(result) = result else { return Pending };
        if result.footer != Pending {
            *open = false;
            ctx.request_repaint();
            result.footer
        } else if !*open {
            Cancelled
        } else {
            Pending
        }
    }
}
