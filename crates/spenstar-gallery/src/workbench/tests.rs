use eframe::egui::{
    self, Context, Event, FullOutput, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape,
    pos2, vec2,
};

use crate::{Gallery, Page};

struct Headless {
    ctx: Context,
    gallery: Gallery,
    time: f64,
}

impl Headless {
    fn new() -> Self {
        let ctx = Context::default();
        ctx.enable_accesskit();
        let mut gallery = Gallery::new(&ctx);
        gallery.page = Page::Workbench;
        Self {
            ctx,
            gallery,
            time: 0.,
        }
    }

    fn draw(&mut self, events: Vec<Event>) -> FullOutput {
        self.time += 1. / 60.;
        let mut output = self.ctx.run_ui(
            RawInput {
                time: Some(self.time),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1280., 820.))),
                events,
                ..Default::default()
            },
            |ui| self.gallery.show(ui),
        );
        output.textures_delta.clear();
        output
    }

    fn click(&mut self, point: Pos2, button: PointerButton) -> FullOutput {
        self.draw(vec![Event::PointerMoved(point)]);
        self.draw(vec![Event::PointerButton {
            pos: point,
            pressed: true,
            button,
            modifiers: Modifiers::NONE,
        }]);
        self.draw(vec![Event::PointerButton {
            pos: point,
            pressed: false,
            button,
            modifiers: Modifiers::NONE,
        }])
    }

    fn settle(&mut self) -> FullOutput {
        for _ in 0..16 {
            self.draw(vec![]);
        }
        self.draw(vec![])
    }
}

fn target(output: &FullOutput, label: &str) -> Pos2 {
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .and_then(|update| {
            update.nodes.iter().find_map(|(_, node)| {
                (node.label() == Some(label))
                    .then(|| node.bounds())
                    .flatten()
            })
        })
        .unwrap_or_else(|| panic!("missing {label:?}"));
    pos2(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    )
}

fn painted(output: &FullOutput, expected: &str) -> bool {
    output.shapes.iter().any(|shape| match &shape.shape {
        Shape::Text(text) => text.galley.text().contains(expected),
        _ => false,
    })
}

#[test]
fn composer_accepts_text_sends_into_the_stream_and_queues_feedback() {
    let mut app = Headless::new();
    let first = app.settle();
    assert!(painted(&first, "Spenstar Code"));
    app.click(pos2(550., 715.), PointerButton::Primary);
    app.draw(vec![Event::Text(
        "Please review the staged changes.".into(),
    )]);
    let output = app.draw(vec![]);
    app.click(target(&output, "Send message"), PointerButton::Primary);
    let output = app.settle();
    assert!(painted(&output, "Please review the staged changes."));
    assert!(painted(&output, "Message sent"));
    let workbench = &app.gallery.workbench;
    assert!(workbench.draft.is_empty());
    assert_eq!(workbench.messages.len(), 1);
}

#[test]
fn changes_open_a_live_diff_drawer_and_escape_closes_it() {
    let mut app = Headless::new();
    let output = app.settle();
    app.click(
        target(&output, "Changes vs origin/main"),
        PointerButton::Primary,
    );
    let output = app.settle();
    assert!(app.gallery.workbench.git.changes);
    assert!(painted(&output, "src/orchestrator.rs"));
    assert!(painted(&output, "src/workspace.rs"));
    app.draw(vec![Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }]);
    assert!(!app.gallery.workbench.git.changes);
    assert!(!painted(&app.settle(), "src/orchestrator.rs"));
}

#[test]
fn thread_context_copy_and_archive_update_the_gallery() {
    let mut app = Headless::new();
    let output = app.settle();
    let thread = target(&output, "orchestrator fork");
    app.click(thread, PointerButton::Secondary);
    let menu = app.settle();
    let copy = app.click(target(&menu, "Copy link"), PointerButton::Primary);
    assert!(copy.platform_output.commands.iter().any(|command| {
        matches!(command, egui::OutputCommand::CopyText(text) if text == "spenstar://workbench/thread/0")
    }));
    assert!(painted(&app.settle(), "Copied to clipboard"));
    app.click(thread, PointerButton::Secondary);
    let menu = app.settle();
    app.click(target(&menu, "Archive"), PointerButton::Primary);
    assert_ne!(app.gallery.workbench.threads.active().id, 0);
    assert!(painted(&app.settle(), "Thread archived"));
}
