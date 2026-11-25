use egui::{
    Frame, Id, InnerResponse, Key, Margin, Modifiers, Popup, PopupCloseBehavior, RectAlign,
    Response, Stroke, Ui,
};

use crate::Theme;

pub(crate) fn id(anchor: &Response) -> Id {
    Popup::default_response_id(anchor)
}

pub(crate) fn is_open(anchor: &Response) -> bool {
    Popup::is_id_open(&anchor.ctx, id(anchor))
}

pub(crate) fn toggle(anchor: &Response) {
    Popup::toggle_id(&anchor.ctx, id(anchor));
}

pub(crate) fn open(anchor: &Response) {
    Popup::open_id(&anchor.ctx, id(anchor));
}

pub(crate) fn close(anchor: &Response, restore_focus: bool) {
    if is_open(anchor) {
        Popup::close_id(&anchor.ctx, id(anchor));
        anchor.ctx.request_repaint();
    }
    if restore_focus {
        anchor.request_focus();
    }
}

pub(crate) fn show<R>(
    anchor: &Response,
    width: f32,
    name: &str,
    content: impl FnOnce(&mut Ui) -> R,
) -> Option<InnerResponse<R>> {
    if !anchor.enabled() || !anchor.interact_rect.is_positive() {
        close(anchor, false);
        return None;
    }
    if !is_open(anchor) {
        return None;
    }
    let ctx = &anchor.ctx;
    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        close(anchor, true);
        return None;
    }
    let theme = Theme::current(ctx);
    let frame = Frame::new()
        .fill(theme.colors.surface_elevated)
        .stroke(Stroke::new(1., theme.colors.border))
        .corner_radius(8)
        .shadow(theme.shadow())
        .inner_margin(Margin::same(6));
    let width = width.min((ctx.content_rect().width() - 14.).max(1.));
    let output = Popup::from_response(anchor)
        .open_memory(None)
        .close_behavior(PopupCloseBehavior::IgnoreClicks)
        .align_alternatives(&[RectAlign::TOP_START])
        .gap(6.)
        .frame(frame)
        .width(width + 14.)
        .accessible_name(name)
        .show(|ui| {
            ui.set_width(width);
            content(ui)
        })?;
    let on_anchor = ctx
        .pointer_interact_pos()
        .is_some_and(|pos| anchor.interact_rect.contains(pos));
    if output.response.clicked_elsewhere() && !on_anchor {
        close(anchor, false);
    }
    Some(output)
}
