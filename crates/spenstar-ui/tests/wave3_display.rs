//! Contrast, display rendering, and interactive clipboard and chip behavior.
#[path = "wave3/support.rs"]
mod support;

use spenstar_ui::{
    Theme,
    display::{
        Avatar, AvatarGroup, AvatarStatus, Badge, BadgeSize, BadgeVariant, Chip, CodeBlock,
        DescriptionItem, DescriptionList, DiffKind, DiffLine, DiffView, Tone,
    },
    egui::{Color32, Context, OutputCommand},
    icon::Icon,
    theme::{Accent, Mode},
};

use support::{click, frame, location};

fn luminance(color: Color32) -> f32 {
    let linear = |component: u8| {
        let x = f32::from(component) / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
}

#[test]
fn semantic_badges_and_chips_meet_aa_throughout_both_palettes() {
    for mode in [Mode::Light, Mode::Dark] {
        let accents =
            Accent::PRESETS
                .into_iter()
                .chain([Accent::Custom(Color32::from_rgba_unmultiplied(
                    156, 68, 132, 121,
                ))]);
        for accent in accents {
            let theme = Theme::new(mode, accent);
            for tone in [
                Tone::Neutral,
                Tone::Accent,
                Tone::Success,
                Tone::Warning,
                Tone::Danger,
            ] {
                for variant in [
                    BadgeVariant::Solid,
                    BadgeVariant::Soft,
                    BadgeVariant::Outline,
                    BadgeVariant::Dot,
                ] {
                    let badge = Badge {
                        tone,
                        variant,
                        ..Badge::new("Contrast sample")
                    };
                    for selected in [false, true] {
                        let (fill, ink) = badge.colors(theme, selected);
                        assert!(fill.is_opaque() && ink.is_opaque());
                        let (a, b) = (luminance(fill), luminance(ink));
                        assert!(
                            (a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5,
                            "{mode:?} {tone:?} {variant:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn display_widgets_paint_in_both_themes_and_copy_the_exact_source() {
    for mode in [Mode::Light, Mode::Dark] {
        let ctx = Context::default();
        Theme::new(mode, Accent::Indigo).install(&ctx);
        let mut selected = false;
        let mut chip_rect = spenstar_ui::egui::Rect::ZERO;
        let mut copied = false;
        let mut draw = |time, events| {
            let output = frame(&ctx, time, events, |ui| {
                Badge {
                    variant: BadgeVariant::Dot,
                    size: BadgeSize::Md,
                    icon: Some(Icon::Check.into()),
                    ..Badge::new("Ready")
                }
                .show(ui);
                let mut chip = Chip::new("Priority");
                chip.avatar = Some(Avatar::new("Tess Hill"));
                chip.selectable = true;
                chip.selected = selected;
                chip.removable = true;
                let out = chip.show(ui);
                chip_rect = out.response.rect;
                if out.toggled {
                    selected = !selected;
                }
                copied = out.removed;
                let avatars = [
                    Avatar {
                        status: Some(AvatarStatus::Online),
                        ..Avatar::new("Avery Chen")
                    },
                    Avatar {
                        tone: Tone::Warning,
                        ..Avatar::new("Sam Lee")
                    },
                    Avatar::new("Mina Ford"),
                ];
                AvatarGroup {
                    limit: 2,
                    ..AvatarGroup::new(&avatars)
                }
                .show(ui);
                DescriptionList {
                    items: &[
                        DescriptionItem {
                            label: "Record ID",
                            value: "OPS-42",
                            mono: true,
                            copy: false,
                        },
                        DescriptionItem {
                            label: "Status",
                            value: "Archived",
                            mono: false,
                            copy: false,
                        },
                    ],
                }
                .show(ui);
                CodeBlock {
                    language: Some("Rust"),
                    ..CodeBlock::new("let row = 42;\nrow + 1")
                }
                .show(ui);
                DiffView {
                    lines: &[
                        DiffLine {
                            kind: DiffKind::Removed,
                            text: "old",
                        },
                        DiffLine {
                            kind: DiffKind::Added,
                            text: "new",
                        },
                        DiffLine {
                            kind: DiffKind::Context,
                            text: "same",
                        },
                    ],
                    language: Some("Diff"),
                    copy: false,
                    line_numbers: true,
                }
                .show(ui);
            });
            (output, chip_rect, selected, copied)
        };
        let (output, chip, _, _) = draw(0., vec![]);
        for expected in [
            "Ready",
            "Priority",
            "AC",
            "SL",
            "+1",
            "OPS-42",
            "Rust",
            "let row = 42;",
            "-",
            "+",
            "old",
            "new",
            "same",
        ] {
            location(&output, expected);
        }
        assert!((location(&output, "OPS-42").x - location(&output, "Archived").x).abs() < 8.);
        let (_, _, toggled, removed) = draw(0.1, click(location(&output, "Priority")));
        assert!(toggled && !removed);
        let remove_pos = spenstar_ui::egui::pos2(chip.right() - 20., chip.center().y);
        let (_, _, toggled, removed) = draw(0.2, click(remove_pos));
        assert!(toggled && removed);
        let (output, _, _, _) = draw(0.3, click(location(&output, "Copy")));
        assert!(
            output
                .platform_output
                .commands
                .iter()
                .any(|cmd| matches!(cmd,
            OutputCommand::CopyText(text) if text == "let row = 42;\nrow + 1"))
        );
        let (feedback, _, _, _) = draw(0.4, vec![]);
        location(&feedback, "Copied");
    }
}

#[test]
fn removable_solid_chips_paint_aa_remove_icons_at_rest_and_on_hover() {
    use spenstar_ui::{
        egui::{Event, FullOutput, Pos2, TextureId, epaint::Shape},
        icon::SvgIconEngine,
    };

    fn remove_paint(output: &FullOutput) -> (Pos2, TextureId, Color32) {
        let mut fills = Vec::new();
        for clipped in &output.shapes {
            match &clipped.shape {
                Shape::Rect(rect) if rect.fill.is_opaque() => fills.push(rect),
                Shape::Mesh(mesh) if mesh.texture_id != TextureId::default() => {
                    let center = mesh.calc_bounds().center();
                    let background = fills
                        .iter()
                        .rev()
                        .find(|rect| rect.rect.contains(center))
                        .unwrap_or_else(|| panic!("missing remove target background"));
                    return (center, mesh.texture_id, background.fill);
                }
                _ => {}
            }
        }
        panic!("missing painted remove icon")
    }

    for mode in [Mode::Light, Mode::Dark] {
        for (tone, accent) in [
            (Tone::Neutral, Accent::Blue),
            (Tone::Accent, Accent::Blue),
            (Tone::Success, Accent::Blue),
            (Tone::Warning, Accent::Blue),
            (Tone::Danger, Accent::Blue),
            (Tone::Accent, Accent::Custom(Color32::from_gray(117))),
        ] {
            let ctx = Context::default();
            let theme = Theme::new(mode, accent);
            theme.install(&ctx);
            let mut chip = Chip::new("Removable");
            chip.badge.variant = BadgeVariant::Solid;
            chip.badge.tone = tone;
            chip.removable = true;
            let (fill, ink) = chip.badge.colors(theme, false);
            let mut wanted = TextureId::default();
            let mut draw = |time, events| {
                frame(&ctx, time, events, |ui| {
                    chip.show(ui);
                    wanted = SvgIconEngine::texture(ui.ctx(), &Icon::X.svg(), 14., ink)
                        .map(|texture| texture.id())
                        .unwrap_or_else(|error| {
                            panic!("could not render the remove icon: {error}")
                        });
                })
            };
            let (center, rest_icon, background) = remove_paint(&draw(0., vec![]));
            assert_eq!(background, fill);
            let (_, hover_icon, hover_fill) =
                remove_paint(&draw(0.01, vec![Event::PointerMoved(center)]));
            assert_ne!(hover_fill, fill, "{mode:?} {tone:?} remove hover feedback");
            for (texture, fill) in [(rest_icon, fill), (hover_icon, hover_fill)] {
                assert_eq!(
                    texture, wanted,
                    "{mode:?} {tone:?} must paint the resolved ink"
                );
                let (a, b) = (luminance(ink), luminance(fill));
                assert!(
                    (a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5,
                    "{mode:?} {tone:?}"
                );
            }
        }
    }
}
