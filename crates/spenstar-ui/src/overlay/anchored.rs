use egui::{
    Id, InnerResponse, Key, Popup, PopupCloseBehavior, PopupKind, RectAlign, Response, ScrollArea,
    Ui,
};

use crate::{Theme, typography::Type};

use super::{card, kbd, key, set_input_surface};

/// Elevated, viewport-aware popup anchored to a widget.
pub struct Popover {
    id: Id,
    width: f32,
    outside: bool,
}

impl Popover {
    /// Create a popup with a stable identity.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            id: Id::unique(id),
            width: 240.,
            outside: true,
        }
    }

    /// Set the preferred inner width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Control whether a click outside closes the popup.
    pub fn dismiss_on_click_outside(mut self, dismiss: bool) -> Self {
        self.outside = dismiss;
        self
    }

    /// Toggle on anchor click and render. The caller can also change the open flag.
    pub fn show<R>(
        self,
        anchor: &Response,
        open: &mut bool,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        if anchor.clicked() {
            *open = !*open;
            anchor.ctx.request_repaint();
        }
        if *open && (!anchor.enabled() || !anchor.interact_rect.is_positive()) {
            *open = false;
            anchor.ctx.request_repaint();
        }
        if !*open {
            return None;
        }
        let ctx = &anchor.ctx;
        let width = self.width.min(ctx.content_rect().width() - 24.).max(1.);
        let height = (ctx.content_rect().height() - 24.).max(1.);
        let mut escape = false;
        let output = Popup::from_response(anchor)
            .id(self.id)
            .gap(5.)
            .width(width + 18.)
            .frame(card(ctx, 12, 8))
            .close_behavior(PopupCloseBehavior::IgnoreClicks)
            .show(|ui| {
                set_input_surface(ui);
                ui.set_width(width);
                let result = ScrollArea::vertical()
                    .max_height(height)
                    .show(ui, content)
                    .inner;
                escape = key(ui, Key::Escape);
                result
            })?;
        let on_anchor = ctx
            .pointer_interact_pos()
            .is_some_and(|point| anchor.interact_rect.contains(point));
        if escape || (self.outside && !on_anchor && output.response.clicked_elsewhere()) {
            *open = false;
            if escape {
                anchor.request_focus();
            }
            ctx.request_repaint();
        }
        Some(output)
    }
}

/// Delayed, non-interactive help attached to a widget.
pub struct Tooltip<'a> {
    text: &'a str,
    shortcut: Option<&'a str>,
    delay: f64,
}

impl<'a> Tooltip<'a> {
    /// Create a tooltip with a 350 millisecond delay.
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            shortcut: None,
            delay: 0.35,
        }
    }

    /// Add a trailing keyboard shortcut pill.
    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Set the hover delay in seconds.
    pub fn delay(mut self, seconds: f64) -> Self {
        self.delay = if seconds.is_finite() {
            seconds.max(0.)
        } else {
            0.35
        };
        self
    }

    /// Show when the anchor is hovered long enough. Returns the visible tooltip response.
    pub fn show(self, anchor: &Response) -> Option<Response> {
        let ctx = &anchor.ctx;
        let id = anchor.id.with("spenstar-tooltip");
        if !anchor.hovered() || ctx.input(|input| input.pointer.any_down()) {
            ctx.data_mut(|data| data.remove::<f64>(id));
            return None;
        }
        let now = ctx.input(|input| input.time);
        let since = ctx.data_mut(|data| *data.get_temp_mut_or_insert_with(id, || now));
        let remaining = self.delay - (now - since);
        if remaining > 0. {
            ctx.request_repaint_after_secs(remaining as f32);
            return None;
        }
        let c = Theme::current(ctx).colors;
        Popup::from_response(anchor)
            .id(id)
            .kind(PopupKind::Tooltip)
            .interactable(false)
            .align(RectAlign::TOP)
            .gap(5.)
            .accessible_name(self.text)
            .frame(card(ctx, 8, 6))
            .show(|ui| {
                ui.set_max_width((ctx.content_rect().width() - 16.).clamp(1., 300.));
                ui.horizontal_wrapped(|ui| {
                    ui.label(Type::Description.text(self.text).color(c.text_primary));
                    if let Some(shortcut) = self.shortcut {
                        kbd(ui, shortcut);
                    }
                });
            })
            .map(|output| output.response)
    }
}
