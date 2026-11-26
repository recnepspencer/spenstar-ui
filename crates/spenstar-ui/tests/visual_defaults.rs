//! Geometry and theme contracts for the default component treatments.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    Theme,
    controls::{Direction, RadioGroup, RadioItem, RadioVariant, Switch, Tone},
    egui::{Color32, CornerRadius, FontFamily, Stroke, epaint::Shape},
    form::{FieldVariant, Size, TextField},
    layout::Card,
    select::SelectStyle,
    theme::{Accent, Mode},
    typography::Type,
};
use support::{Harness, node};

#[test]
fn text_roles_use_registered_optical_weights() {
    let mut harness = Harness::new(Mode::Dark);
    harness.draw(vec![], |ui| {
        for (role, size, family) in [
            (Type::Body, 14., "inter-regular"),
            (Type::Label, 14., "inter-medium"),
            (Type::H1, 20., "inter-semibold"),
            (Type::H2, 16., "inter-medium"),
            (Type::H3, 16., "inter-medium"),
            (Type::Metric, 26., "inter-semibold"),
            (Type::Display, 40., "inter-bold"),
            (Type::Mono, 13., "mono-regular"),
            (Type::Kbd, 11., "mono-medium"),
        ] {
            let font = role.font();
            assert_eq!(font.size, size);
            assert_eq!(font.family, FontFamily::Name(family.into()));
            ui.fonts_mut(|fonts| {
                let definitions = fonts.definitions();
                assert_eq!(
                    definitions
                        .families
                        .get(&font.family)
                        .and_then(|names| names.first())
                        .map(String::as_str),
                    Some(family)
                );
                assert!(definitions.font_data.contains_key(family));
            });
        }
    });
}

#[test]
fn default_fields_inside_cards_have_soft_fill_and_no_resting_stroke() {
    assert_eq!(TextField::default().variant, FieldVariant::Filled);
    assert_eq!(SelectStyle::default().variant, FieldVariant::Filled);
    for (mode, fill) in [
        (Mode::Light, Color32::from_rgb(239, 239, 240)),
        (Mode::Dark, Color32::from_rgb(39, 39, 42)),
    ] {
        let mut harness = Harness::new(mode);
        let (frame, _) = harness.draw(vec![], |ui| {
            Card::default().show(ui, |card| {
                card.body(|ui| {
                    for size in [Size::Sm, Size::Md, Size::Lg] {
                        TextField {
                            size,
                            ..TextField::new("Label")
                        }
                        .show(ui, &mut String::new());
                    }
                });
            });
        });
        let heights: Vec<_> = frame
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                Shape::Rect(rect)
                    if rect.fill == fill && rect.corner_radius == CornerRadius::same(12) =>
                {
                    assert_eq!(rect.stroke, Stroke::NONE);
                    Some(rect.rect.height())
                }
                _ => None,
            })
            .collect();
        assert_eq!(heights, [32., 38., 44.]);
        let theme = Theme::new(mode, Accent::Blue);
        assert_eq!(theme.colors.accent.base, Color32::from_rgb(4, 133, 247));
    }
}

#[test]
fn radio_cards_stack_and_fill_the_available_width() {
    let mut harness = Harness::new(Mode::Dark);
    let (frame, _) = harness.draw(vec![], |ui| {
        ui.set_width(350.);
        RadioGroup {
            variant: RadioVariant::Card,
            direction: Direction::Horizontal,
            ..Default::default()
        }
        .show(
            ui,
            &mut Some(0),
            &[
                RadioItem::new("Ground"),
                RadioItem::new("Expedited overnight"),
            ],
        );
    });
    let first = node(&frame, "Ground").bounds();
    let second = node(&frame, "Expedited overnight").bounds();
    let (Some(first), Some(second)) = (first, second) else {
        panic!("missing radio card bounds");
    };
    assert_eq!(first.x0, second.x0);
    assert_eq!(first.x1, second.x1);
    assert_eq!(first.x1 - first.x0, 350.);
    assert_eq!(second.y0 - first.y1, 12.);
}

#[test]
fn active_colored_switches_paint_white_thumbs_in_both_modes() {
    for mode in [Mode::Light, Mode::Dark] {
        let mut harness = Harness::new(mode);
        let (frame, _) = harness.draw(vec![], |ui| {
            for tone in [Tone::Accent, Tone::Success, Tone::Warning, Tone::Danger] {
                Switch {
                    tone,
                    ..Switch::new("Enabled")
                }
                .show(ui, &mut true);
            }
        });
        let thumbs: Vec<_> = frame
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Circle(circle) => Some(circle.fill),
                _ => None,
            })
            .collect();
        assert_eq!(thumbs, [Color32::WHITE; 4]);
    }
}
