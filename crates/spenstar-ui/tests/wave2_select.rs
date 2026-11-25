//! Headless tests for keyboard, pointer, and popover interactions.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    egui::{Event, Key, KeyboardShortcut, Modifiers, Ui, pos2},
    input::ShortcutGuard,
    select::{ComboBox, MultiSelect, Select, SelectOption, SelectSection},
    theme::Mode,
};
use support::{Harness, center, key, key_with};

fn choices() -> [SelectOption<'static>; 4] {
    [
        SelectOption::new("north", "North hub"),
        SelectOption {
            disabled: true,
            ..SelectOption::new("south", "South hub")
        },
        SelectOption::new("west", "West harbor"),
        SelectOption::new("east", "East station"),
    ]
}

#[test]
fn select_flips_skips_disabled_options_selects_and_dismisses() {
    let mut h = Harness::new(Mode::Light);
    let options = choices();
    let sections = [SelectSection {
        label: Some("Facilities"),
        options: &options,
    }];
    let mut value = None;
    let mut render = |ui: &mut Ui| {
        ui.add_space(408.);
        ui.set_max_width(280.);
        Select {
            clearable: true,
            ..Select::default()
        }
        .show(ui, &mut value, &sections)
    };
    let (_, closed) = h.draw(vec![], &mut render);
    let click = closed.response.rect.center();
    let (_, open) = h.click(click, &mut render);
    assert!(open.open);
    let (output, _) = h.draw(vec![], &mut render);
    assert!(center(&output, "North hub").y < open.response.rect.top());
    let (_, after_down) = h.draw(key(Key::ArrowDown), &mut render);
    assert!(after_down.open);
    let (_, picked) = h.draw(key(Key::Enter), &mut render);
    assert!(picked.response.changed() && !picked.open);
    let (output, closed) = h.draw(vec![], &mut render);
    let clear = center(&output, "Clear selection");
    let (_, cleared) = h.click(clear, &mut render);
    assert!(cleared.response.changed() && !cleared.open);
    let (_, open) = h.click(closed.response.rect.center(), &mut render);
    assert!(open.open);
    let (_, escape) = h.draw(key(Key::Escape), &mut render);
    assert!(!escape.open);
    let (_, open) = h.click(click, &mut render);
    assert!(open.open);
    h.draw(vec![], &mut render);
    let (_, closed) = h.click(pos2(530., 60.), &mut render);
    assert!(!closed.open);
    assert_eq!(value, None);
}

#[test]
fn multi_select_keeps_menu_open_until_escape_and_removes_a_chip() {
    let mut h = Harness::new(Mode::Dark);
    let options = choices();
    let sections = [SelectSection {
        label: None,
        options: &options,
    }];
    let mut values = vec!["north".into()];
    let mut render = |ui: &mut Ui| {
        ui.set_max_width(350.);
        MultiSelect {
            max_visible_chips: 1,
            show_count: true,
            ..MultiSelect::default()
        }
        .show(ui, &mut values, &sections)
    };
    let (_, closed) = h.draw(vec![], &mut render);
    let (_, opened) = h.click(closed.response.rect.center(), &mut render);
    assert!(opened.open);
    h.draw(key(Key::ArrowDown), &mut render);
    let (_, picked) = h.draw(key(Key::Enter), &mut render);
    assert!(picked.open && picked.response.changed());
    let (_, escaped) = h.draw(key(Key::Escape), &mut render);
    assert!(!escaped.open);
    let (output, _) = h.draw(vec![], &mut render);
    let (_, removed) = h.click(center(&output, "Remove North hub"), &mut render);
    assert!(removed.response.changed() && !removed.open);
    assert_eq!(values, ["west"]);
}

#[test]
fn combo_filters_and_blocks_global_shortcuts_then_commits_existing_and_custom_values() {
    let mut h = Harness::new(Mode::Light);
    let options = choices();
    let sections = [SelectSection {
        label: None,
        options: &options,
    }];
    let mut query = String::new();
    let mut custom = Vec::new();
    let mut render = |ui: &mut Ui| {
        let response = ComboBox {
            allow_custom: true,
            ..ComboBox::default()
        }
        .show_with_create(ui, &mut query, &sections, |name| {
            custom.push(name.to_owned())
        });
        let guard = ShortcutGuard::new(ui);
        let global = guard.consume(&KeyboardShortcut::new(Modifiers::COMMAND, Key::K));
        (response, global, guard.allows())
    };
    let (_, (closed, ..)) = h.draw(vec![], &mut render);
    let (_, (open, _, allows)) = h.click(closed.response.rect.center(), &mut render);
    assert!(open.open && !allows);
    let (filtered, (open, _, allows)) = h.draw(vec![Event::Text("HUB".into())], &mut render);
    assert!(open.open && !allows);
    let (_, (_, fired, _)) = h.draw(key_with(Key::K, Modifiers::COMMAND), &mut render);
    assert!(!fired);
    let drawn = filtered
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            spenstar_ui::egui::Shape::Text(text) => Some(text.galley.text()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(drawn.contains(&"North hub"));
    assert!(!drawn.contains(&"West harbor"));
    h.draw(key(Key::ArrowDown), &mut render);
    h.draw(key(Key::ArrowUp), &mut render);
    let (_, (selected, ..)) = h.draw(key(Key::Enter), &mut render);
    assert_eq!(selected.selected.as_deref(), Some("north"));
    assert!(!selected.open);
    h.draw(key_with(Key::A, Modifiers::COMMAND), &mut render);
    let (_, (custom_open, ..)) = h.draw(vec![Event::Text("Remote dock".into())], &mut render);
    assert!(custom_open.open);
    let (_, (created, ..)) = h.draw(key(Key::Enter), &mut render);
    assert_eq!(created.created.as_deref(), Some("Remote dock"));
    let (_, (opened_again, ..)) = h.draw(key(Key::ArrowDown), &mut render);
    assert!(opened_again.open);
    let (_, (escaped, ..)) = h.draw(key(Key::Escape), &mut render);
    assert!(!escaped.open);
    assert_eq!(query, "Remote dock");
    assert_eq!(custom, ["Remote dock"]);
}
