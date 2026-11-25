use eframe::egui::{Color32, Ui};
use spenstar_ui::{
    Theme,
    color::{ColorSwatch, ColorSwatchPicker},
    controls::{
        CheckState, Checkbox, CheckboxGroup, Direction, RadioGroup, RadioItem, RadioVariant,
        RangeSlider, Slider, Switch, Tone,
    },
    form::{Field, FieldVariant, NumberField, SearchField, Size, TextArea, TextField},
    icon::Icon,
    layout::{Card, CardVariant, ResponsiveGrid},
    tags::{Tag, TagGroup, TagInput},
    theme::Accent,
    typography::Type,
};

use std::ops::RangeInclusive;

pub(crate) struct Demo {
    name: String,
    email: String,
    password: String,
    reference: String,
    disabled: String,
    notes: String,
    query: String,
    search_result: String,
    weight: f64,
    channels: [bool; 3],
    standard: Option<usize>,
    card: Option<usize>,
    switches: [bool; 4],
    volume: f64,
    window: RangeInclusive<f64>,
    tags: Vec<String>,
    tag_draft: String,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            name: "Northwest hub".into(),
            email: "dispatch".into(),
            password: "demo-password".into(),
            reference: "SHP-0148".into(),
            disabled: "Managed by your workspace".into(),
            notes: String::new(),
            query: String::new(),
            search_result: String::new(),
            weight: 24.5,
            channels: [true, false, true],
            standard: Some(0),
            card: Some(1),
            switches: [true, true, false, false],
            volume: 70.,
            window: 20.0..=65.0,
            tags: vec!["Priority".into(), "Inbound".into()],
            tag_draft: String::new(),
        }
    }
}

impl Demo {
    pub(crate) fn show(&mut self, ui: &mut Ui, accent: Accent) -> Option<Accent> {
        let mut new_accent = None;
        ResponsiveGrid::new(308.)
            .max_columns(2)
            .show(ui, 2, |ui, index| {
                if index == 0 {
                    self.text(ui);
                    self.scales(ui);
                } else {
                    self.choices(ui);
                    new_accent = self.tags_and_colors(ui, accent);
                }
            });
        new_accent
    }

    fn text(&mut self, ui: &mut Ui) {
        section(ui, "Text & validation", |ui| {
            Field {
                required: true,
                description: Some("Shown in dispatch and shipment records."),
                ..Field::new("Workspace name")
            }
            .show_with(
                ui,
                |ui| {
                    ui.label(Type::Caption.text("Public label"));
                },
                |ui, invalid, label| {
                    TextField {
                        clearable: true,
                        max_len: Some(40),
                        invalid,
                        ..TextField::new("Workspace name")
                    }
                    .show(ui, &mut self.name)
                    .labelled_by(label);
                },
            );
            let email_error =
                (!self.email.contains('@')).then_some("Enter an address that includes @.");
            Field {
                error: email_error,
                ..Field::new("Contact email")
            }
            .show_with(
                ui,
                |_| {},
                |ui, invalid, label| {
                    TextField {
                        variant: FieldVariant::Filled,
                        size: Size::Sm,
                        leading: Some(Icon::Mail.into()),
                        invalid,
                        ..TextField::new("name@example.com")
                    }
                    .show(ui, &mut self.email)
                    .labelled_by(label);
                },
            );
            Field::new("Password").show_with(
                ui,
                |_| {},
                |ui, _, label| {
                    TextField {
                        variant: FieldVariant::Underlined,
                        size: Size::Lg,
                        leading: Some(Icon::Lock.into()),
                        password: true,
                        ..TextField::new("Password")
                    }
                    .show(ui, &mut self.password)
                    .labelled_by(label);
                },
            );
            Field::new("Shipment reference").show(ui, |ui, _| {
                TextField {
                    read_only: true,
                    trailing: Some(Icon::Tag.into()),
                    ..TextField::default()
                }
                .show(ui, &mut self.reference);
            });
            Field {
                disabled: true,
                ..Field::new("Billing profile")
            }
            .show(ui, |ui, _| {
                TextField::default().show(ui, &mut self.disabled);
            });
            Field::new("Delivery notes").show(ui, |ui, invalid| {
                TextArea {
                    min_rows: 2,
                    max_rows: 5,
                    max_len: Some(160),
                    placeholder: "Add handling instructions…",
                    invalid,
                    ..TextArea::default()
                }
                .show(ui, &mut self.notes);
            });
        });
    }

    fn choices(&mut self, ui: &mut Ui) {
        section(ui, "Choice controls", |ui| {
            let mut parent = match self.channels.iter().filter(|&&on| on).count() {
                0 => CheckState::Unchecked,
                3 => CheckState::Checked,
                _ => CheckState::Indeterminate,
            };
            if (Checkbox {
                description: Some("Toggle every notification channel."),
                ..Checkbox::new("Select all")
            })
            .show(ui, &mut parent)
            .changed()
            {
                self.channels.fill(parent == CheckState::Checked);
            }
            CheckboxGroup {
                direction: Direction::Horizontal,
                field: Field {
                    description: Some("Choose where delivery updates appear."),
                    ..Field::new("Notifications")
                },
            }
            .show(ui, |ui, invalid| {
                for (name, enabled) in ["Email", "SMS", "In-app"]
                    .into_iter()
                    .zip(&mut self.channels)
                {
                    Checkbox {
                        invalid,
                        ..Checkbox::new(name)
                    }
                    .show(ui, enabled);
                }
            });
            let mut archived = true;
            Checkbox {
                disabled: true,
                ..Checkbox::new("Archive notifications")
            }
            .show(ui, &mut archived);
            RadioGroup {
                label: "Dispatch frequency",
                direction: Direction::Horizontal,
                ..RadioGroup::default()
            }
            .show(
                ui,
                &mut self.standard,
                &[
                    RadioItem::new("Daily"),
                    RadioItem::new("Weekly"),
                    RadioItem {
                        disabled: true,
                        ..RadioItem::new("Monthly")
                    },
                ],
            );
            RadioGroup {
                label: "Delivery speed",
                variant: RadioVariant::Card,
                ..RadioGroup::default()
            }
            .show(
                ui,
                &mut self.card,
                &[
                    RadioItem {
                        description: Some("Arrives in three to five business days."),
                        icon: Some(Icon::Truck.into()),
                        ..RadioItem::new("Ground")
                    },
                    RadioItem {
                        description: Some("Prioritized next-day delivery."),
                        icon: Some(Icon::Zap.into()),
                        ..RadioItem::new("Express")
                    },
                ],
            );
        });
    }

    fn scales(&mut self, ui: &mut Ui) {
        section(ui, "Search, numbers & ranges", |ui| {
            Field::new("Shipment search").show(ui, |ui, _| {
                let out = SearchField {
                    placeholder: "Search shipments…",
                    shortcut: Some("↵"),
                    disabled: false,
                }
                .show(ui, &mut self.query);
                if out.submitted {
                    self.search_result = format!("Searched for “{}”", self.query);
                }
                if out.cleared {
                    self.search_result.clear();
                }
                if !self.search_result.is_empty() {
                    ui.label(Type::Small.text(&self.search_result));
                }
            });
            Field {
                description: Some("Use ↑ / ↓ to step by 0.5 kg."),
                ..Field::new("Parcel weight")
            }
            .show(ui, |ui, _| {
                NumberField {
                    min: 0.,
                    max: 100.,
                    step: 0.5,
                    precision: 1,
                    suffix: "kg",
                    ..NumberField::default()
                }
                .show(ui, &mut self.weight);
            });
            let mut slider = Slider::new(0.0..=100.0);
            slider.options.label = "Notification volume";
            slider.options.step = 5.;
            slider.options.ticks = 11;
            Field::new(&format!("Notification volume · {:.0}%", self.volume))
                .show(ui, |ui, _| slider.show(ui, &mut self.volume));
            let mut slider = RangeSlider::new(0.0..=100.0);
            slider.options.label = "Capacity window";
            slider.options.step = 5.;
            Field::new(&format!(
                "Capacity window · {:.0}% – {:.0}%",
                self.window.start(),
                self.window.end()
            ))
            .show(ui, |ui, _| slider.show(ui, &mut self.window));
        });
    }

    fn tags_and_colors(&mut self, ui: &mut Ui, accent: Accent) -> Option<Accent> {
        section(ui, "Switches, tags & color", |ui| {
            for (index, (label, tone, size)) in [
                ("Auto-assign", Tone::Accent, Size::Sm),
                ("Live tracking", Tone::Success, Size::Md),
                ("Quiet hours", Tone::Warning, Size::Lg),
                ("Pause dispatch", Tone::Danger, Size::Md),
            ]
            .into_iter()
            .enumerate()
            {
                Switch {
                    tone,
                    size,
                    disabled: index == 3,
                    ..Switch::new(label)
                }
                .show(ui, &mut self.switches[index]);
            }
            TagGroup::default().show(
                ui,
                &[
                    Tag {
                        tone: Tone::Success,
                        icon: Some(Icon::Check.into()),
                        ..Tag::new("Ready")
                    },
                    Tag {
                        tone: Tone::Warning,
                        icon: Some(Icon::Clock.into()),
                        ..Tag::new("Pending")
                    },
                ],
            );
            Field {
                description: Some("Press Enter or comma to add a tag."),
                ..Field::new("Shipment tags")
            }
            .show(ui, |ui, _| {
                TagInput {
                    max_tags: 6,
                    ..TagInput::default()
                }
                .show(ui, &mut self.tags, &mut self.tag_draft);
            });
            let mut color = accent.color();
            let samples = Accent::PRESETS.map(Accent::color);
            ui.label(Type::Small.text("Gallery accent"));
            let changed = ColorSwatchPicker::default()
                .show(ui, &mut color, &samples)
                .changed();
            ui.horizontal(|ui| {
                ColorSwatch {
                    circular: true,
                    size: 20.,
                    ..ColorSwatch::new(Color32::from_rgba_unmultiplied(37, 99, 235, 105))
                }
                .show(ui, false);
                ui.label(
                    Type::Caption
                        .text("Translucent sample")
                        .color(Theme::current(ui.ctx()).colors.text_secondary),
                );
            });
            changed.then(|| {
                Accent::PRESETS
                    .into_iter()
                    .find(|preset| preset.color() == color)
                    .unwrap_or(Accent::Custom(color))
            })
        })
    }
}

pub(super) fn section<R>(ui: &mut Ui, title: &str, body: impl FnOnce(&mut Ui) -> R) -> R {
    Card::new(CardVariant::Bordered)
        .show(ui, |card| {
            card.header(title, None);
            card.body(|ui| {
                ui.spacing_mut().item_spacing.y = 16.;
                body(ui)
            })
        })
        .inner
}
