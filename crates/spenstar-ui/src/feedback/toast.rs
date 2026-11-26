use std::{collections::VecDeque, time::Duration};

use egui::{
    Align, Align2, Area, Context, Frame, Id, Layout, Margin, Order, Response, Stroke, Ui, vec2,
};

use crate::{
    Theme,
    button::{Button, CloseButton, Size, Variant},
    typography::Type,
};

use super::{Tone, semantic_ink, status_icon};

/// One transient notification.
#[derive(Clone)]
pub struct Toast {
    title: String,
    description: Option<String>,
    action: Option<String>,
    tone: Tone,
    duration: Duration,
}

impl Toast {
    /// Create a neutral notification that lasts five seconds.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            action: None,
            tone: Tone::Neutral,
            duration: Duration::from_secs(5),
        }
    }

    /// Add supporting text.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// Add an action. A click emits a queue event and dismisses the notification.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action = Some(label.into());
        self
    }

    /// Choose semantic color and icon.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Set the unhovered time before automatic dismissal.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// Corner or edge anchoring a notification stack.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastPosition {
    /// Stack upward from the bottom right.
    #[default]
    BottomRight,
    /// Stack downward from the top right.
    TopRight,
    /// Stack upward from the lower center.
    BottomCenter,
}

/// An action selected on a notification.
#[derive(Debug, PartialEq, Eq)]
pub struct ToastEvent {
    /// Stable ID returned by the queue when the toast was added.
    pub id: u64,
    /// Label of the selected action.
    pub action: String,
}

/// Retained notification queue. Pending notifications start timing when visible.
#[derive(Clone)]
pub struct ToastQueue {
    id: Id,
    pending: VecDeque<Entry>,
    visible: Vec<Entry>,
    next_id: u64,
    position: ToastPosition,
    cap: usize,
}

#[derive(Clone)]
struct Entry {
    id: u64,
    toast: Toast,
    remaining: f64,
    last: Option<f64>,
    progress: f32,
    hovered: bool,
    closing: bool,
}

impl Default for ToastQueue {
    fn default() -> Self {
        Self::new("spenstar-ui/toasts")
    }
}

impl ToastQueue {
    /// Create an independent stack and assign its stable UI identity.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            id: Id::unique(id),
            pending: VecDeque::new(),
            visible: Vec::new(),
            next_id: 0,
            position: ToastPosition::default(),
            cap: 3,
        }
    }

    /// Set the edge used by the stack.
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the maximum visible count, with a minimum of one.
    pub fn max_visible(mut self, cap: usize) -> Self {
        self.cap = cap.max(1);
        while self.visible.len() > self.cap {
            if let Some(entry) = self.visible.pop() {
                self.pending.push_front(entry);
            }
        }
        self
    }

    /// Add a notification and return its ID.
    pub fn push(&mut self, toast: Toast) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.pending.push_back(Entry {
            id,
            remaining: toast.duration.as_secs_f64(),
            toast,
            last: None,
            progress: 0.,
            hovered: false,
            closing: false,
        });
        id
    }

    /// Dismiss a pending or visible notification by ID.
    pub fn dismiss(&mut self, id: u64) {
        self.pending.retain(|entry| entry.id != id);
        if let Some(entry) = self.visible.iter_mut().find(|entry| entry.id == id) {
            entry.closing = true;
        }
    }

    /// Number of notifications currently displayed, including exiting ones.
    pub fn visible_count(&self) -> usize {
        self.visible.len()
    }

    /// Number of notifications waiting for a visible slot.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Enqueue a notification in the current viewport's shared stack.
    pub fn notify(ctx: &Context, toast: Toast) -> u64 {
        let queue_id = Self::context_id(ctx);
        let id = ctx.data_mut(|data| data.get_temp_mut_or_default::<Self>(queue_id).push(toast));
        ctx.request_repaint();
        id
    }

    /// Render the viewport's shared notifications. Call once near the end of the frame.
    pub fn show_context(ctx: &Context) -> Vec<ToastEvent> {
        let id = Self::context_id(ctx);
        let Some(mut queue) = ctx.data_mut(|data| data.remove_temp::<Self>(id)) else {
            return Vec::new();
        };
        let events = queue.show(ctx);
        ctx.data_mut(|data| data.insert_temp(id, queue));
        events
    }

    /// Clear queued and visible shared notifications for this viewport.
    pub fn clear_context(ctx: &Context) {
        let id = Self::context_id(ctx);
        ctx.data_mut(|data| data.remove::<Self>(id));
        ctx.request_repaint();
    }

    fn context_id(ctx: &Context) -> Id {
        Id::unique(("spenstar-ui/shared-toasts", ctx.viewport_id()))
    }

    /// Draw the stack and advance enter, hover, expiration, and exit state.
    pub fn show(&mut self, ctx: &Context) -> Vec<ToastEvent> {
        let now = ctx.input(|input| input.time);
        self.visible
            .retain(|entry| !(entry.closing && entry.progress <= 0.));
        while self.visible.len() < self.cap {
            let Some(mut entry) = self.pending.pop_front() else {
                break;
            };
            entry.last = None;
            self.visible.push(entry);
        }
        if self.visible.is_empty() {
            return Vec::new();
        }
        let (pivot, offset) = match self.position {
            ToastPosition::BottomRight => (Align2::RIGHT_BOTTOM, vec2(-16., -16.)),
            ToastPosition::TopRight => (Align2::RIGHT_TOP, vec2(-16., 16.)),
            ToastPosition::BottomCenter => (Align2::CENTER_BOTTOM, vec2(0., -16.)),
        };
        let width = (ctx.content_rect().width() - 38.).clamp(1., 324.);
        let mut events = Vec::new();
        Area::new(self.id)
            .order(Order::Tooltip)
            .anchor(pivot, offset)
            .fade_in(false)
            .default_width(width)
            .layout(Layout::top_down(Align::Min))
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = 8.;
                let top_down = self.position == ToastPosition::TopRight;
                let count = self.visible.len();
                for index in 0..count {
                    let entry = &mut self.visible[if top_down { index } else { count - 1 - index }];
                    if let Some(event) = ui.push_id(entry.id, |ui| entry.show(ui, now, width)).inner
                    {
                        events.push(event);
                    }
                }
            });
        events
    }
}

impl Entry {
    fn show(&mut self, ui: &mut Ui, now: f64, width: f32) -> Option<ToastEvent> {
        let dt = self.last.map_or(0., |last| (now - last).max(0.));
        self.last = Some(now);
        let step = (dt / 0.18) as f32;
        self.progress = if self.closing {
            (self.progress - step).max(0.)
        } else {
            (self.progress + step).min(1.)
        };
        let progress = self.progress;
        let fade = progress * progress * (3. - 2. * progress);
        let (response, close, action) = ui
            .horizontal(|ui| {
                ui.set_opacity(fade);
                if self.closing {
                    ui.disable();
                }
                ui.spacing_mut().item_spacing.x = 0.;
                ui.add_space(14. * (1. - fade));
                ui.vertical(|ui| self.draw(ui, width)).inner
            })
            .inner;
        let hovered = ui
            .ctx()
            .pointer_hover_pos()
            .is_some_and(|point| response.rect.contains(point));
        if !self.hovered && !hovered {
            self.remaining -= dt;
        }
        self.hovered = hovered;
        self.closing |= close || action || self.remaining <= 0.;
        if self.closing || self.progress < 1. {
            ui.ctx().request_repaint();
        } else if !hovered {
            ui.ctx().request_repaint_after_secs(self.remaining as f32);
        }
        if action {
            self.toast.action.clone().map(|action| ToastEvent {
                id: self.id,
                action,
            })
        } else {
            None
        }
    }

    fn draw(&self, ui: &mut Ui, width: f32) -> (Response, bool, bool) {
        let theme = Theme::current(ui.ctx());
        let c = theme.colors;
        let p = self.toast.tone.palette(c);
        let output = Frame::new()
            .fill(c.overlay)
            .stroke(Stroke::new(1., c.overlay_border))
            .shadow(theme.shadow())
            .corner_radius(12)
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_width((width - 22.).max(1.));
                ui.spacing_mut().item_spacing.x = 6.;
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 32.),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        let close = CloseButton::show(ui).clicked();
                        let action = self.toast.action.as_ref().is_some_and(|label| {
                            Button::new(label)
                                .size(Size::Sm)
                                .variant(Variant::Soft)
                                .show(ui)
                                .clicked()
                        });
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            status_icon(self.toast.tone).show(
                                ui,
                                17.,
                                semantic_ink(p.ring, c.overlay, c.text_primary),
                            );
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.;
                                ui.label(Type::Label.text(&self.toast.title).color(c.text_primary));
                                if let Some(detail) = &self.toast.description {
                                    ui.label(
                                        Type::Description.text(detail).color(c.text_secondary),
                                    );
                                }
                            });
                        });
                        (close, action)
                    },
                )
                .inner
            });
        (output.response, output.inner.0, output.inner.1)
    }
}
