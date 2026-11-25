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
