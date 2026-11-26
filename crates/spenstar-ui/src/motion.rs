//! Interaction animation and keyboard focus indicators.
use egui::{Context, CornerRadius, Event, Id, Response, Stroke, StrokeKind, Ui};

/// Standard animation duration for interaction feedback, in seconds.
pub const FAST: f32 = 0.14;
/// Standard transition duration for layout changes, in seconds.
pub const NORMAL: f32 = 0.22;

const FOCUS_RING_GAP: f32 = 2.;
const FOCUS_RING_WIDTH: f32 = 2.;
pub(crate) const FOCUS_RING_OUTSET: f32 = FOCUS_RING_GAP + FOCUS_RING_WIDTH;

/// Animate a binary state and return an eased value between zero and one.
pub fn eased(ctx: &Context, id: Id, active: bool, seconds: f32) -> f32 {
    let t = ctx.animate_value_with_time(id, f32::from(active), seconds);
    t * t * (3. - 2. * t)
}

/// Paint a visible keyboard focus outline around an interactive widget.
pub fn focus_ring(ui: &Ui, response: &Response, radius: CornerRadius) {
    let ctx = ui.ctx();
    let id = Id::unique(("spenstar-ui/keyboard-modality", ctx.viewport_id()));
    let modality = ctx.input(|input| {
        input.raw.events.iter().rev().find_map(|event| match event {
            Event::PointerButton { pressed: true, .. } => Some(false),
            Event::Key { pressed: true, .. } => Some(true),
            _ => None,
        })
    });
    let keyboard = ctx.data_mut(|data| {
        if let Some(keyboard) = modality {
            data.insert_temp(id, keyboard);
        }
        data.get_temp(id).unwrap_or(true)
    });
    if response.has_focus() && keyboard {
        let color = crate::Theme::current(ctx).focus_color();
        ui.painter().rect_stroke(
            response.rect.expand(FOCUS_RING_GAP),
            radius,
            Stroke::new(FOCUS_RING_WIDTH, color),
            StrokeKind::Outside,
        );
    }
}
