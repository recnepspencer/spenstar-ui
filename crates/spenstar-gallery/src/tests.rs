use eframe::egui::{Context, RawInput, Rect, epaint::Shape, pos2, vec2};
use spenstar_ui::{layout::SidebarVariant, theme::Accent};

use super::{Gallery, Mode, Page, Theme};

#[test]
fn each_gallery_page_paints_content_at_desktop_and_compact_widths() {
    for (mode, width) in [(Mode::Dark, 1232.), (Mode::Light, 540.)] {
        let ctx = Context::default();
        let mut gallery = Gallery::new(&ctx);
        gallery.theme = Theme::new(mode, Accent::Violet);
        gallery.theme.apply(&ctx);
        gallery.shell.sidebar.variant = SidebarVariant::Floating;

        for (page, sample) in [
            (
                Page::Overview,
                [
                    "Open shipments",
                    "128",
                    "Shipment volume",
                    "Recent activity",
                ],
            ),
            (
                Page::Orders,
                [
                    "Manage and track customer orders.",
                    "Search orders…",
                    "Customer",
                    "Avery Chen",
                ],
            ),
            (
                Page::Tracker,
                [
                    "Track work across your team.",
                    "Completed",
                    "In progress",
                    "Improve form validation",
                ],
            ),
            (
                Page::Inbox,
                [
                    "Dispatch dashboard review",
                    "Search messages…",
                    "Casey Imani",
                    "2 messages",
                ],
            ),
            (
                Page::Settings,
                [
                    "Organization name",
                    "Organization bio",
                    "Organization email",
                    "Workspace appearance",
                ],
            ),
            (
                Page::Workspace,
                [
                    "Search the queue…",
                    "Dispatch queue",
                    "Shipment details",
                    "Handling notes",
                ],
            ),
            (
                Page::Buttons,
                ["Variants", "Accent", "Icons & states", "Grouped actions"],
            ),
            (
                Page::Icons,
                ["Filter by icon name", "109 of 109 icons", "Primary", "20"],
            ),
            (
                Page::Tokens,
                ["Semantic colors", "Background", "Border", "Action palettes"],
            ),
            (
                Page::Forms,
                [
                    "Text & validation",
                    "Workspace name",
                    "Choice controls",
                    "Select all",
                ],
            ),
            (
                Page::Pickers,
                [
                    "Dropdowns & autocomplete",
                    "Primary facility",
                    "Inline calendar",
                    "Date pickers",
                ],
            ),
        ] {
            gallery.page = page;
            let mut frame = ctx.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(width, 808.))),
                    ..Default::default()
                },
                |ui| gallery.show(ui),
            );
            frame.textures_delta.clear();
            let required = if width < 600. {
                &sample[..1]
            } else {
                &sample[..]
            };
            for expected in std::iter::once(page.title()).chain(required.iter().copied()) {
                fn draws_text(shape: &Shape, clip: Rect, expected: &str) -> bool {
                    match shape {
                        Shape::Text(text) => {
                            text.galley.text() == expected
                                && clip.intersects(shape.visual_bounding_rect())
                        }
                        Shape::Vec(children) => {
                            children.iter().any(|s| draws_text(s, clip, expected))
                        }
                        _ => false,
                    }
                }
                assert!(
                    frame
                        .shapes
                        .iter()
                        .any(|shape| { draws_text(&shape.shape, shape.clip_rect, expected) }),
                    "{mode:?} {page:?}: missing {expected:?}"
                );
            }
        }
    }
}

#[test]
fn palette_click_updates_the_live_gallery_theme() {
    use eframe::egui::{Event, Modifiers, PointerButton};

    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut gallery = Gallery::new(&ctx);
    gallery.page = Page::Forms;
    let mut render = |events| {
        let mut frame = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(1232., 1280.))),
                events,
                ..Default::default()
            },
            |ui| gallery.show(ui),
        );
        frame.textures_delta.clear();
        frame
    };
    let frame = render(vec![]);
    let bounds = frame
        .platform_output
        .accesskit_update
        .as_ref()
        .and_then(|update| {
            update.nodes.iter().find_map(|(_, node)| {
                (node.label() == Some("#7C3AED"))
                    .then(|| node.bounds())
                    .flatten()
            })
        });
    let Some(bounds) = bounds else {
        panic!("the violet palette swatch is missing");
    };
    let at = pos2(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    render(vec![Event::PointerMoved(at)]);
    for pressed in [true, false] {
        render(vec![Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }]);
    }
    assert_eq!(gallery.theme.accent, Accent::Violet);
    assert_eq!(Theme::current(&ctx).accent, Accent::Violet);
}

#[test]
fn appearance_opens_dropdown_and_closes_optional_inspector() {
    use eframe::egui::{Event, FullOutput, Modifiers, PointerButton, Pos2};

    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut gallery = Gallery::new(&ctx);
    let mut draw = |events| {
        let mut frame = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1280., 820.))),
                events,
                ..Default::default()
            },
            |ui| gallery.show(ui),
        );
        frame.textures_delta.clear();
        frame
    };
    let find = |frame: &FullOutput, label: &str| {
        frame
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
            .map(|bounds| {
                pos2(
                    ((bounds.x0 + bounds.x1) * 0.5) as f32,
                    ((bounds.y0 + bounds.y1) * 0.5) as f32,
                )
            })
    };
    let mut click = |label| {
        let frame = draw(vec![]);
        let Some(at) = find(&frame, label) else {
            panic!("missing control {label}");
        };
        draw(vec![Event::PointerMoved(at)]);
        for pressed in [true, false] {
            draw(vec![Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }]);
        }
        draw(vec![])
    };
    let appearance = click("Appearance");
    assert!(find(&appearance, "Open inspector").is_some());
    let dropdown = click("Standard");
    assert!(find(&dropdown, "Floating").is_some());
    let appearance = click("Floating");
    assert!(find(&appearance, "Open inspector").is_some());
    let inspector = click("Open inspector");
    assert!(find(&inspector, "Close").is_some());
    let closed = click("Close");
    assert!(find(&closed, "Close").is_none());
    assert!(!gallery.inspector_open);
    assert_eq!(gallery.shell.sidebar.variant, SidebarVariant::Floating);
}
