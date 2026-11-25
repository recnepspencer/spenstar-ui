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

#[test]
fn combo_opening_arrow_keeps_the_first_option_active() {
    for (opening_key, next_key, expected) in [
        (Key::ArrowDown, None, "north"),
        (Key::ArrowUp, None, "north"),
        (Key::ArrowDown, Some(Key::ArrowDown), "west"),
    ] {
        let mut h = Harness::new(Mode::Light);
        let options = choices();
        let sections = [SelectSection {
            label: None,
            options: &options,
        }];
        let mut query = String::new();
        let mut render = |ui: &mut Ui| ComboBox::default().show(ui, &mut query, &sections);
        let (_, closed) = h.draw(vec![], &mut render);
        h.focus(closed.response.id);
        let (_, open) = h.draw(key(opening_key), &mut render);
        assert!(open.open);
        if let Some(next_key) = next_key {
            h.draw(key(next_key), &mut render);
        }
        let (_, picked) = h.draw(key(Key::Enter), &mut render);
        assert_eq!(
            picked.selected.as_deref(),
            Some(expected),
            "{opening_key:?}"
        );
        assert!(!picked.open);
    }
}

#[test]
fn select_enter_activates_the_popup_row_focused_with_tab() {
    assert_eq!(select_from_tab_focus(None).as_deref(), Some("west"));
    assert_eq!(
        select_from_tab_focus(Some(Key::ArrowDown)).as_deref(),
        Some("east")
    );
}

fn select_from_tab_focus(next_key: Option<Key>) -> Option<String> {
    let mut h = Harness::new(Mode::Light);
    let options = choices();
    let sections = [SelectSection {
        label: None,
        options: &options,
    }];
    let mut selected = None;
    let mut render = |ui: &mut Ui| {
        let out = Select::default().show(ui, &mut selected, &sections);
        (out, selected.clone())
    };
    let (_, (closed, _)) = h.draw(vec![], &mut render);
    let (_, (opened, _)) = h.click(closed.response.rect.center(), &mut render);
    assert!(opened.open);
    let (frame, _) = h.draw(vec![], &mut render);
    let west = frame
        .platform_output
        .accesskit_update
        .as_ref()
        .and_then(|update| {
            update
                .nodes
                .iter()
                .find_map(|(id, node)| (node.label() == Some("West harbor")).then_some(*id))
        });
    let Some(west) = west else {
        panic!("the popup row was missing");
    };
    let west_focused = |h: &Harness| {
        h.ctx
            .memory(|m| m.focused().is_some_and(|id| id.accesskit_id() == west))
    };
    for _ in 0..6 {
        h.draw(key(Key::Tab), &mut render);
        if west_focused(&h) {
            break;
        }
    }
    assert!(west_focused(&h));
    if let Some(next_key) = next_key {
        h.draw(key(next_key), &mut render);
    }
    let (_, (picked, value)) = h.draw(key(Key::Enter), &mut render);
    assert!(!picked.open);
    value
}
