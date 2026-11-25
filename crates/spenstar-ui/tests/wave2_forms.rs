//! Headless form editor, choice, range, tag, and palette interaction tests.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    color::ColorSwatchPicker,
    controls::{CheckState, Checkbox, RadioGroup, RadioItem, RangeSlider, Slider},
    egui::{
        Color32, Event, Key, Modifiers, Ui,
        accesskit::{Role, Toggled},
        pos2,
    },
    form::{NumberField, SearchField},
    tags::TagInput,
    theme::Mode,
};
use support::{Harness, center, key, key_with, node};

#[test]
fn number_clamps_steps_and_recovers_from_bad_text() {
    let mut h = Harness::new(Mode::Dark);
    let mut number = 5.;
    let mut render = |ui: &mut Ui| {
        ui.set_max_width(280.);
        let out = NumberField {
            min: 0.,
            max: 10.,
            step: 0.5,
            precision: 1,
            prefix: "$",
            suffix: "kg",
            ..Default::default()
        }
        .show(ui, &mut number);
        (out, number)
    };
    let (initial, (out, _)) = h.draw(vec![], &mut render);
    let (_, (_, raised)) = h.click(center(&initial, "Increase"), &mut render);
    assert_eq!(raised, 5.5);
    let (_, (_, lowered)) = h.click(center(&initial, "Decrease"), &mut render);
    assert_eq!(lowered, 5.);
    h.focus(out.response.id);
    h.draw(key_with(Key::A, Modifiers::COMMAND), &mut render);
    h.draw(vec![Event::Text("55".into())], &mut render);
    let (_, (out, value)) = h.draw(key(Key::Enter), &mut render);
    assert!(out.committed);
    assert_eq!(value, 10.);
    let (_, (_, value)) = h.draw(key(Key::ArrowDown), &mut render);
    assert_eq!(value, 9.5);
    let (_, (_, value)) = h.draw(key_with(Key::ArrowDown, Modifiers::SHIFT), &mut render);
    assert_eq!(value, 4.5);
    h.draw(key_with(Key::A, Modifiers::COMMAND), &mut render);
    h.draw(vec![Event::Text("-90.2".into())], &mut render);
    let (_, (_, value)) = h.click(pos2(400., 100.), &mut render);
    assert_eq!(value, 0.);
    h.focus(out.response.id);
    h.draw(key_with(Key::A, Modifiers::COMMAND), &mut render);
    let (_, (invalid, value)) = h.draw(vec![Event::Text("NaN".into())], &mut render);
    assert!(invalid.invalid);
    assert_eq!(value, 0.);
    let (_, (recovered, value)) = h.draw(key(Key::Enter), &mut render);
    assert!(recovered.committed && !recovered.invalid);
    assert_eq!(value, 0.);
    let (_, (_, value)) = h.draw(key(Key::ArrowUp), &mut render);
    assert_eq!(value, 0.5);
}

#[test]
fn search_submits_and_clears_with_the_input_still_focused() {
    let mut h = Harness::new(Mode::Light);
    let mut query = "shipments".to_owned();
    let mut render = |ui: &mut Ui| {
        SearchField {
            placeholder: "Search",
            shortcut: Some("/"),
            disabled: false,
        }
        .show(ui, &mut query)
    };
    let (frame, out) = h.draw(vec![], &mut render);
    h.focus(out.response.id);
    let (_, submitted) = h.draw(key(Key::Enter), &mut render);
    assert!(submitted.submitted);
    let (_, cleared) = h.click(center(&frame, "Clear text"), &mut render);
    assert!(cleared.cleared && cleared.response.changed() && cleared.response.has_focus());
    assert!(query.is_empty());
}

#[test]
fn indeterminate_checkbox_is_accessible_and_accepts_space_and_enter() {
    let mut h = Harness::new(Mode::Light);
    let mut state = CheckState::Indeterminate;
    let mut render = |ui: &mut Ui| {
        let response = Checkbox::new("Select all").show(ui, &mut state);
        (response, state)
    };
    let (output, (response, state)) = h.draw(vec![], &mut render);
    assert_eq!(state, CheckState::Indeterminate);
    let checkbox = node(&output, "Select all");
    assert_eq!(checkbox.role(), Role::CheckBox);
    assert_eq!(checkbox.toggled(), Some(Toggled::Mixed));
    h.focus(response.id);
    let (_, (checked, state)) = h.draw(key(Key::Space), &mut render);
    assert!(checked.changed());
    assert_eq!(state, CheckState::Checked);
    let (_, (unchecked, state)) = h.draw(key(Key::Enter), &mut render);
    assert!(unchecked.changed());
    assert_eq!(state, CheckState::Unchecked);
}

#[test]
fn radio_arrows_skip_disabled_items() {
    let mut h = Harness::new(Mode::Dark);
    let mut value = Some(0);
    let mut render = |ui: &mut Ui| {
        RadioGroup::default().show(
            ui,
            &mut value,
            &[
                RadioItem::new("First"),
                RadioItem {
                    disabled: true,
                    ..RadioItem::new("Disabled")
                },
                RadioItem::new("Third"),
            ],
        );
        value
    };
    let (frame, _) = h.draw(vec![], &mut render);
    h.click(center(&frame, "First"), &mut render);
    h.draw(vec![], &mut render);
    let (_, value) = h.draw(key(Key::ArrowDown), &mut render);
    assert_eq!(value, Some(2));
}

#[test]
fn sliders_snap_bound_and_stop_thumbs_from_crossing() {
    let mut h = Harness::new(Mode::Light);
    let mut value = 200.;
    let mut range = 20.0..=65.0;
    let mut render = |ui: &mut Ui| {
        ui.set_max_width(310.);
        let mut single = Slider::new(0.0..=100.0);
        single.options.step = 5.;
        let mut double = RangeSlider::new(0.0..=100.0);
        double.options.step = 5.;
        double.options.label = "Capacity";
        (
            single.show(ui, &mut value),
            double.show(ui, &mut range),
            value,
            range.clone(),
        )
    };
    let (frame, (single, double, value, range)) = h.draw(vec![], &mut render);
    assert!(single.changed());
    assert_eq!(value, 100.);
    assert_eq!(range, 20.0..=65.0);
    let left = center(&frame, "Capacity start");
    let right = center(&frame, "Capacity end");
    let (_, (_, _, _, range)) = h.click(left, &mut render);
    let lower = *range.start();
    assert!(lower < 65.);
    let (_, (_, _, _, range)) = h.draw(key_with(Key::ArrowRight, Modifiers::SHIFT), &mut render);
    assert_eq!(range, 65.0..=65.0);
    h.focus(double.id.with(1));
    let (_, (_, _, _, range)) = h.draw(key_with(Key::ArrowLeft, Modifiers::SHIFT), &mut render);
    assert_eq!(range, 65.0..=65.0);
    let (_, (_, _, _, range)) = h.click(right, &mut render);
    assert!(range.start() <= range.end());
    assert!(range.end() % 5. == 0.);
}

#[test]
fn tags_commit_dedupe_limit_and_remove_with_backspace() {
    let mut h = Harness::new(Mode::Dark);
    let mut tags = Vec::new();
    let mut draft = String::new();
    let mut render = |ui: &mut Ui| {
        let result = TagInput {
            max_tags: 2,
            ..Default::default()
        }
        .show(ui, &mut tags, &mut draft);
        (result, tags.clone(), draft.clone())
    };
    let (_, (result, ..)) = h.draw(vec![], &mut render);
    h.focus(result.response.id);
    let (_, (result, tags, draft)) =
        h.draw(vec![Event::Text("Blue,blue,Gray".into())], &mut render);
    assert!(result.changed);
    assert_eq!(tags, ["Blue"]);
    assert_eq!(draft, "Gray");
    let (_, (result, tags, draft)) = h.draw(key(Key::Enter), &mut render);
    assert!(result.changed);
    assert_eq!(tags, ["Blue", "Gray"]);
    assert_eq!(draft, "");
    let (_, (_, tags, _)) = h.draw(vec![Event::Text("White,".into())], &mut render);
    assert_eq!(tags, ["Blue", "Gray"]);
    let (_, (out, tags, _)) = h.draw(key(Key::Backspace), &mut render);
    assert!(out.changed);
    assert_eq!(tags, ["Blue"]);
}

#[test]
fn swatch_selection_changes_the_color() {
    let mut h = Harness::new(Mode::Dark);
    let mut color = Color32::BLUE;
    let mut render = |ui: &mut Ui| {
        let out = ColorSwatchPicker {
            circular: true,
            disabled: false,
        }
        .show(ui, &mut color, &[Color32::BLUE, Color32::RED]);
        (out, color)
    };
    let (frame, _) = h.draw(vec![], &mut render);
    let (_, (change, color)) = h.click(center(&frame, "#FF0000"), &mut render);
    assert!(change.changed());
    assert_eq!(color, Color32::RED);
    let (frame, _) = h.draw(vec![], &mut render);
    assert_eq!(node(&frame, "#FF0000").toggled(), Some(Toggled::True));
}

#[test]
fn slider_steps_back_to_the_nearest_grid_value_from_a_partial_endpoint() {
    let mut h = Harness::new(Mode::Light);
    let mut value = 10.;
    let mut render = |ui: &mut Ui| {
        let mut slider = Slider::new(0.0..=10.0);
        slider.options.step = 3.;
        (slider.show(ui, &mut value), value)
    };
    let (frame, _) = h.draw(vec![], &mut render);
    h.click(center(&frame, "Value"), &mut render);
    let (_, (_, next)) = h.draw(key(Key::ArrowLeft), &mut render);
    assert_eq!(next, 9.);
}

#[test]
fn dragging_a_range_thumb_cannot_cross_the_other_thumb() {
    use support::pointer;

    let mut h = Harness::new(Mode::Light);
    let mut range = 20.0..=60.0;
    let mut render = |ui: &mut Ui| {
        let mut slider = RangeSlider::new(0.0..=100.0);
        slider.options.step = 10.;
        slider.show(ui, &mut range);
        range.clone()
    };
    let (frame, _) = h.draw(vec![], &mut render);
    let from = center(&frame, "Value start");
    let to = center(&frame, "Value end") + spenstar_ui::egui::vec2(60., 0.);
    h.draw(vec![Event::PointerMoved(from)], &mut render);
    h.draw(pointer(from, true), &mut render);
    let (_, active) = h.draw(vec![Event::PointerMoved(to)], &mut render);
    assert_eq!(active, 60.0..=60.0);
    let (_, released) = h.draw(pointer(to, false), &mut render);
    assert_eq!(released, active);
}
