//! Geometry and theme contracts for the default component treatments.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    Theme,
    controls::{Direction, RadioGroup, RadioItem, RadioVariant, Switch, Tone},
    egui::{Color32, CornerRadius, FontFamily, Id, Modal, Pos2, Stroke, Window, epaint::Shape},
    form::{FieldVariant, Size, TextField},
    layout::{Card, MainContent, MenuSection, Sidebar, SidebarPage},
    select::SelectStyle,
    tags::{Tag, TagGroup},
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

#[test]
fn filled_fields_distinguish_canvas_from_modal_and_window_surfaces() {
    fn has_fill(shape: &Shape, center: Pos2, fill: Color32) -> bool {
        match shape {
            Shape::Rect(rect) => {
                rect.fill == fill
                    && (rect.rect.height() - 38.).abs() < 0.01
                    && rect.rect.contains(center)
            }
            Shape::Vec(shapes) => shapes.iter().any(|shape| has_fill(shape, center, fill)),
            _ => false,
        }
    }

    for (mode, canvas_fill, overlay_fill) in [
        (
            Mode::Light,
            Color32::WHITE,
            Color32::from_rgb(239, 239, 240),
        ),
        (
            Mode::Dark,
            Color32::from_rgb(24, 24, 27),
            Color32::from_rgb(39, 39, 42),
        ),
    ] {
        let mut harness = Harness::new(mode);
        harness
            .ctx
            .all_styles_mut(|style| style.animation_time = 0.);
        let field = |ui: &mut spenstar_ui::egui::Ui| {
            ui.set_width(126.);
            TextField {
                variant: FieldVariant::Filled,
                ..TextField::new("Field")
            }
            .show(ui, &mut String::new())
            .rect
            .center()
        };
        let mut render = |ui: &mut spenstar_ui::egui::Ui| {
            let canvas = MainContent::show(ui, field);
            let window = Window::new("Field window")
                .default_pos([30., 120.])
                .show(ui.ctx(), field)
                .and_then(|window| window.inner);
            let modal = Modal::new(Id::unique("field-modal"))
                .show(ui.ctx(), field)
                .inner;
            (canvas, window, modal)
        };
        harness.draw(vec![], &mut render);
        let (frame, (canvas, window, modal)) = harness.draw(vec![], render);
        let Some(window) = window else {
            panic!("missing window field");
        };
        for (center, fill) in [
            (canvas, canvas_fill),
            (window, overlay_fill),
            (modal, overlay_fill),
        ] {
            assert!(
                frame
                    .shapes
                    .iter()
                    .any(|shape| has_fill(&shape.shape, center, fill)),
                "{mode:?}: field at {center:?} did not paint {fill:?}"
            );
        }
    }
}

#[test]
fn sidebar_section_labels_use_the_modes_muted_text() {
    for (mode, expected) in [
        (Mode::Light, Color32::from_rgb(113, 113, 122)),
        (Mode::Dark, Color32::from_rgb(159, 159, 169)),
    ] {
        let mut harness = Harness::new(mode);
        let (frame, _) = harness.draw(vec![], |ui| {
            Sidebar::default().show(
                ui,
                SidebarPage {
                    id: "sidebar-contrast",
                    parent: None,
                    active: None,
                    sections: &[MenuSection {
                        label: "Section",
                        items: &[],
                    }],
                },
                |_, _| {},
                |_, _| {},
            );
        });
        assert_eq!(painted_text(&frame, "Section"), Some(expected));
    }
}

#[test]
fn semantic_tag_text_has_aa_contrast_against_its_surfaces() {
    fn luminance(color: Color32) -> f64 {
        let [r, g, b, _] = color.to_srgba_unmultiplied().map(|channel| {
            let value = f64::from(channel) / 255.;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        });
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    for mode in [Mode::Light, Mode::Dark] {
        let mut harness = Harness::new(mode);
        for accent in Accent::PRESETS.into_iter().chain([
            Accent::Custom(Color32::WHITE),
            Accent::Custom(Color32::BLACK),
        ]) {
            let theme = Theme::new(mode, accent);
            theme.apply(&harness.ctx);
            let colors = theme.colors;
            for (tone, palette) in [
                (Tone::Accent, colors.accent),
                (Tone::Success, colors.success),
                (Tone::Warning, colors.warning),
                (Tone::Danger, colors.danger),
            ] {
                let (frame, _) = harness.draw(vec![], |ui| {
                    TagGroup::default().show(
                        ui,
                        &[Tag {
                            tone,
                            ..Tag::new("Tag contrast")
                        }],
                    );
                });
                let Some(ink) = painted_text(&frame, "Tag contrast") else {
                    panic!("tag did not paint");
                };
                if tone == Tone::Warning {
                    assert_eq!(
                        ink, palette.ring,
                        "warning palette must be legible on its own"
                    );
                }
                for fill in [colors.surface, palette.soft, palette.soft_hover] {
                    let a = luminance(ink);
                    let b = luminance(fill);
                    let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                    assert!(
                        ratio >= 4.5,
                        "{mode:?} {accent:?} {tone:?} on {fill:?}: {ratio:.2}:1"
                    );
                }
            }
        }
    }
}

fn painted_text(frame: &spenstar_ui::egui::FullOutput, label: &str) -> Option<Color32> {
    frame
        .shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            Shape::Text(text) if text.galley.text() == label => {
                text.override_text_color.or_else(|| {
                    text.galley
                        .job
                        .sections
                        .first()
                        .map(|section| section.format.color)
                })
            }
            _ => None,
        })
}
