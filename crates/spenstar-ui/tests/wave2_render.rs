//! Render each Wave 2 component in both appearances with the core crate alone.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    color::{ColorSwatch, ColorSwatchPicker},
    controls::{
        CheckState, Checkbox, CheckboxGroup, RadioGroup, RadioItem, RadioVariant, RangeSlider,
        Slider, Switch,
    },
    datetime::{
        Calendar, CalendarDate, ClockFormat, ClockTime, DateField, DatePicker, DateRangePicker,
        RangeCalendar, TimeField,
    },
    egui::{Color32, ScrollArea},
    form::{Field, FieldVariant, NumberField, SearchField, Size, TextArea, TextField},
    icon::Icon,
    select::{ComboBox, MultiSelect, Select, SelectOption, SelectSection},
    tags::{Tag, TagGroup, TagInput},
    theme::Mode,
};
use support::Harness;

#[test]
fn all_controls_render_in_light_and_dark_without_panics() {
    for mode in [Mode::Light, Mode::Dark] {
        let mut h = Harness::new(mode);
        let (out, ()) = h.draw(vec![], |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                let mut title = "North".to_owned();
                for (variant, size) in [
                    (FieldVariant::Outline, Size::Sm),
                    (FieldVariant::Filled, Size::Md),
                    (FieldVariant::Underlined, Size::Lg),
                ] {
                    Field {
                        required: true,
                        error: Some("Invalid"),
                        ..Field::new("Name")
                    }
                    .show(ui, |ui, invalid| {
                        TextField {
                            invalid,
                            variant,
                            size,
                            password: true,
                            clearable: true,
                            leading: Some(Icon::User.into()),
                            max_len: Some(12),
                            ..Default::default()
                        }
                        .show(ui, &mut title);
                    });
                }
                TextArea {
                    read_only: true,
                    ..Default::default()
                }
                .show(ui, &mut title);
                SearchField::default().show(ui, &mut title);
                NumberField::default().show(ui, &mut 5.);
                CheckboxGroup::new("Checks").show(ui, |ui, _| {
                    Checkbox::new("Check").show(ui, &mut CheckState::Indeterminate)
                });
                for variant in [RadioVariant::Standard, RadioVariant::Card] {
                    RadioGroup {
                        variant,
                        ..Default::default()
                    }
                    .show(ui, &mut Some(0), &[RadioItem::new("Radio")]);
                }
                Switch::new("Switch").show(ui, &mut true);
                Slider::new(0.0..=1.0).show(ui, &mut 0.5);
                RangeSlider::new(0.0..=1.0).show(ui, &mut (0.2..=0.8));
                let options = [SelectOption::new("a", "Alpha")];
                let sections = [SelectSection {
                    label: Some("Group"),
                    options: &options,
                }];
                Select::default().show(ui, &mut None, &sections);
                MultiSelect::default().show(ui, &mut vec!["a".to_owned()], &sections);
                ComboBox::default().show(ui, &mut title, &sections);
                Calendar::default().show(ui, &mut Some(CalendarDate::today_utc()));
                RangeCalendar::default().show(ui, &mut None);
                DateField::default().show(ui, &mut None);
                DatePicker::default().show(ui, &mut None);
                DateRangePicker::default().show(ui, &mut None);
                for format in [ClockFormat::H12, ClockFormat::H24] {
                    TimeField {
                        format,
                        seconds: true,
                        ..Default::default()
                    }
                    .show(ui, &mut ClockTime::default());
                }
                TagGroup::default().show(ui, &[Tag::new("Tag")]);
                TagInput::default().show(ui, &mut vec![], &mut String::new());
                ColorSwatch::new(Color32::from_white_alpha(90)).show(ui, false);
                ColorSwatchPicker::default().show(ui, &mut { Color32::BLACK }, &[Color32::BLUE]);
            });
        });
        assert!(!out.shapes.is_empty());
    }
}
