mod conversation;
mod inspector;
mod rail;
mod row;
#[cfg(test)]
mod tests;
mod threads;

use eframe::egui::{self, Align, Color32, Layout, Rect, Response, Ui, UiBuilder, pos2};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    feedback::Tone,
    icon::IconSource,
    overlay::Tooltip,
    theme::Mode,
};

use crate::widgets::notify;
use threads::Threads;

pub(super) struct Workbench {
    threads: Threads,
    rail: bool,
    compact_rail: bool,
    rail_width: f32,
    inspector: bool,
    trace: bool,
    messages: Vec<(u64, String)>,
    draft: String,
    model: &'static str,
    git: inspector::Git,
}

impl Default for Workbench {
    fn default() -> Self {
        Self {
            threads: Threads::default(),
            rail: true,
            compact_rail: false,
            rail_width: 228.,
            inspector: true,
            trace: false,
            messages: Vec::new(),
            draft: String::new(),
            model: "6.2 Sol Ultrafast",
            git: inspector::Git::default(),
        }
    }
}

#[derive(Default)]
pub(super) struct WorkbenchEvents {
    pub(super) commands: bool,
    pub(super) gallery: bool,
    pub(super) theme: bool,
}

impl Workbench {
    pub(super) fn capture_drawer(&mut self, open: bool) {
        self.git.changes = open;
    }

    pub(super) fn show(&mut self, ui: &mut Ui) -> WorkbenchEvents {
        let palette = WorkbenchPalette::new(Theme::current(ui.ctx()).mode);
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
        ui.painter().rect_filled(rect, 0, palette.canvas);
        let show_rail = if rect.width() < 610. {
            self.compact_rail
        } else {
            self.rail
        };
        let rail_width = if show_rail {
            self.resize_rail(ui, rect, palette)
        } else {
            0.
        };
        let inspector_width = if self.inspector && rect.width() - rail_width >= 680. {
            260.
        } else {
            0.
        };
        let mut events = WorkbenchEvents::default();
        if rail_width > 0. {
            let rail = Rect::from_min_max(rect.min, pos2(rect.left() + rail_width, rect.bottom()));
            self.rail(&mut pane(ui, "rail", rail), palette, &mut events);
        }
        let center = Rect::from_min_max(
            pos2(rect.left() + rail_width, rect.top()),
            pos2(rect.right() - inspector_width, rect.bottom()),
        );
        self.conversation(
            &mut pane(ui, "conversation", center),
            palette,
            rail_width > 0.,
            &mut events,
        );
        if inspector_width > 0. {
            let inspector = Rect::from_min_max(pos2(center.right(), rect.top()), rect.max);
            self.git
                .inspector(&mut pane(ui, "inspector", inspector), palette);
        }
        self.git.overlays(ui.ctx());
        self.threads.rename(ui.ctx());
        events
    }

    fn send(&mut self, ctx: &egui::Context) {
        let message = self.draft.trim();
        if !message.is_empty() {
            self.messages
                .push((self.threads.active().id, message.to_owned()));
            self.draft.clear();
            notify(ctx, "Message sent", Tone::Success);
        }
    }
}

#[derive(Clone, Copy)]
struct WorkbenchPalette {
    canvas: Color32,
    rail: Color32,
    card: Color32,
    active: Color32,
    bubble: Color32,
    border: Color32,
    ink: Color32,
    muted: Color32,
}

impl WorkbenchPalette {
    fn new(mode: Mode) -> Self {
        let dark = mode == Mode::Dark;
        let color = |dark_hex, light_hex| rgb(if dark { dark_hex } else { light_hex });
        Self {
            canvas: color(0x09090B, 0xFAFAFB),
            rail: color(0x0A0A0C, 0xF4F4F6),
            card: color(0x111113, 0xFFFFFF),
            active: color(0x1A1A1E, 0xE8E8EC),
            bubble: color(0x1F1F23, 0xEBEBEC),
            border: color(0x1E1E22, 0xE0E0E5),
            ink: color(0xF4F4F5, 0x18181B),
            muted: color(0xA1A1AB, 0x62626E),
        }
    }
}

fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn pane(ui: &mut Ui, salt: &'static str, rect: Rect) -> Ui {
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(salt)
            .max_rect(rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    child.set_min_width(rect.width());
    child
}

fn action(ui: &mut Ui, label: &str, icon: impl Into<IconSource>) -> Response {
    let response = Button::new(label)
        .icon_only(icon)
        .variant(Variant::Ghost)
        .size(Size::Sm)
        .show(ui);
    Tooltip::new(label).show(&response);
    response
}
