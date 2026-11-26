//! Regression tests for popup opening clicks and nested menu layering.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    button::Button,
    egui::{Event, FullOutput, Id, Key, LayerId, Order, Shape, Ui, pos2},
    input::ShortcutGuard,
    overlay::{DropdownMenu, MenuItem, Popover},
    theme::Mode,
};
use support::{Harness, center, key};

#[test]
fn hovered_submenu_keeps_keyboard_navigation_after_its_parent_row_is_clicked() {
    let mut h = Harness::new(Mode::Light);
    let children = [
        MenuItem::action("csv", "Export CSV"),
        MenuItem::action("pdf", "Export PDF"),
    ];
    let items = [
        MenuItem::submenu("Export as", &children),
        MenuItem::action("close", "Close workspace"),
    ];
    let mut draw = |ui: &mut Ui| {
        let anchor = Button::new("Actions").show(ui);
        let chosen = DropdownMenu::new("actions").show(&anchor, &items);
        (anchor.rect, chosen, ShortcutGuard::new(ui).allows())
    };

    let (_, (anchor, ..)) = h.draw(vec![], &mut draw);
    for action in [Key::ArrowDown, Key::ArrowLeft, Key::Escape] {
        h.click(anchor.center(), &mut draw);
        let (frame, _) = h.draw(vec![], &mut draw);
        let trigger = center(&frame, "Export as");
        h.draw(vec![Event::PointerMoved(trigger)], &mut draw);
        h.draw(vec![], &mut draw);
        assert!(painted(&h.draw(vec![], &mut draw).0, "Export CSV"));
        h.click(trigger, &mut draw);
        let root = Id::unique("actions");
        assert_eq!(
            h.ctx.memory(|memory| memory
                .layer_ids()
                .filter(|layer| {
                    layer.order == Order::Foreground && memory.areas().is_visible(layer)
                })
                .last()),
            Some(LayerId::new(Order::Foreground, root.with(0))),
        );
        assert!(painted(&h.draw(vec![], &mut draw).0, "Export CSV"));

        h.draw(key(action), &mut draw);
        match action {
            Key::ArrowDown => {
                assert_eq!(h.draw(key(Key::Enter), &mut draw).1.1, Some("pdf"));
            }
            Key::ArrowLeft => {
                let (frame, _) = h.draw(vec![], &mut draw);
                assert!(painted(&frame, "Close workspace"));
                assert!(!painted(&frame, "Export CSV"));
                h.draw(key(Key::ArrowDown), &mut draw);
                assert_eq!(h.draw(key(Key::Enter), &mut draw).1.1, Some("close"));
            }
            Key::Escape => {}
            _ => unreachable!(),
        }
        h.draw(vec![], &mut draw);
        let (frame, (_, _, allows_background)) = h.draw(vec![], &mut draw);
        assert!(!painted(&frame, "Close workspace"));
        assert!(!painted(&frame, "Export CSV"));
        assert!(allows_background);
    }
}

#[test]
fn a_separate_button_can_open_a_popover_without_dismissing_it() {
    let mut h = Harness::new(Mode::Dark);
    let mut open = false;
    let mut draw = |ui: &mut Ui| {
        let opener = Button::new("Show details").show(ui);
        let anchor = Button::new("Details anchor").show(ui);
        open |= opener.clicked();
        Popover::new("details").show(&anchor, &mut open, |ui| ui.label("Workspace details"));
        (opener.rect, open)
    };

    let (_, (opener, _)) = h.draw(vec![], &mut draw);
    assert!(h.click(opener.center(), &mut draw).1.1);
    h.draw(vec![], &mut draw);
    let (frame, (_, open)) = h.draw(vec![], &mut draw);
    assert!(open && painted(&frame, "Workspace details"));
    assert!(!h.click(pos2(500., 200.), &mut draw).1.1);
}

#[test]
fn a_separate_button_can_open_a_dropdown_without_dismissing_it() {
    let mut h = Harness::new(Mode::Dark);
    let items = [MenuItem::action("export", "Export workspace")];
    let mut draw = |ui: &mut Ui| {
        let opener = Button::new("Show actions").show(ui);
        let anchor = Button::new("Menu anchor").show(ui);
        let menu = DropdownMenu::new("actions");
        if opener.clicked() {
            menu.open(ui.ctx());
        }
        let choice = menu.show(&anchor, &items);
        (opener.rect, choice)
    };

    let (_, (opener, _)) = h.draw(vec![], &mut draw);
    h.click(opener.center(), &mut draw);
    h.draw(vec![], &mut draw);
    assert!(painted(&h.draw(vec![], &mut draw).0, "Export workspace"));
    h.click(pos2(500., 200.), &mut draw);
    assert!(!painted(&h.draw(vec![], &mut draw).0, "Export workspace"));
}

fn painted(output: &FullOutput, label: &str) -> bool {
    output.shapes.iter().any(|shape| match &shape.shape {
        Shape::Text(text) => text.galley.text() == label,
        _ => false,
    })
}
