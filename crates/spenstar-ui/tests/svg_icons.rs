//! End-to-end SVG rasterization and context texture cache checks.
use spenstar_ui::{
    egui::{self, Color32, Context, RawInput, ViewportId},
    icon::{Icon, SvgIcon, SvgIconEngine, SvgIconError},
};

const BRAND: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
  <defs><linearGradient id="currentColor"><stop stop-color="#6263d5"/></linearGradient></defs>
  <style>.tinted { fill: currentColor; }</style>
  <rect width="10" height="20" class="tinted"/>
  <rect x="10" width="10" height="20" fill="url(#currentColor)"/>
</svg>"##;
const SQUARE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
  <rect width="20" height="20" fill="currentColor"/>
</svg>"#;

fn dpi(ppp: f32) -> RawInput {
    let mut raw = RawInput::default();
    if let Some(viewport) = raw.viewports.get_mut(&ViewportId::ROOT) {
        viewport.native_pixels_per_point = Some(ppp);
    }
    raw
}

#[test]
fn real_textures_use_physical_pixels_and_reuse_handles() -> Result<(), SvgIconError> {
    let ctx = Context::default();
    let icon = Icon::Truck.svg();
    let tint = Color32::from_rgb(12, 124, 224);
    let mut previous = None;

    for (ppp, expected) in [(1., 17), (2., 33)] {
        let mut rendered = Err(SvgIconError::Rasterization);
        let mut output = ctx.run_ui(dpi(ppp), |ui| {
            rendered = (|| {
                let texture = SvgIconEngine::texture(ui.ctx(), &icon, 16.25, tint)?;
                let cached = SvgIconEngine::texture(ui.ctx(), &icon, 16.25, tint)?;
                assert_eq!(texture.id(), cached.id());
                assert_eq!(texture.size(), [expected, expected]);
                Ok(texture.id())
            })();
        });
        let uploads = rendered
            .ok()
            .and_then(|id| output.textures_delta.set.remove(&id));
        output.textures_delta.clear();
        let id = rendered?;
        assert_ne!(previous, Some(id));
        previous = Some(id);
        let Some(uploads) = uploads else {
            panic!("missing texture upload")
        };
        assert_eq!(uploads.len(), 1);
        let egui::ImageData::Color(image) = &uploads[0].image;
        assert_eq!(image.size, [expected, expected]);
        assert!(image.pixels.contains(&tint));
        assert!(
            image
                .pixels
                .iter()
                .any(|pixel| (1..255).contains(&pixel.a()))
        );
    }
    let id = previous;
    let mut output = ctx.run_ui(dpi(2.), |ui| {
        assert_eq!(
            SvgIconEngine::texture(ui.ctx(), &icon, 16.25, tint)
                .ok()
                .map(|t| t.id()),
            id
        );
    });
    let uploaded_again = id.is_some_and(|id| output.textures_delta.set.contains_key(&id));
    output.textures_delta.clear();
    assert!(!uploaded_again);
    Ok(())
}

#[test]
fn indexed_icons_are_real_visible_svg_artwork() -> Result<(), SvgIconError> {
    assert_eq!(Icon::ALL.len(), 109);
    let mut names = std::collections::BTreeSet::new();
    for &icon in Icon::ALL {
        assert!(names.insert(icon.name()));
        let image = icon.svg().rasterize(24, Color32::WHITE)?;
        assert!(
            image.pixels.iter().any(|pixel| pixel.a() > 0),
            "{}",
            icon.name()
        );
    }
    Ok(())
}

#[test]
fn tint_respects_alpha_css_and_explicit_brand_paint() -> Result<(), SvgIconError> {
    let svg = SvgIcon::from_bytes("brand", BRAND)?;
    let tint = Color32::from_rgba_unmultiplied(200, 40, 80, 128);
    let image = svg.rasterize(20, tint)?;
    assert_eq!(image.pixels[5], tint);
    assert_eq!(image.pixels[15], Color32::from_rgb(98, 99, 213));
    Ok(())
}

#[test]
fn invalid_and_oversized_documents_return_typed_errors() {
    let malformed = b"<svg><not-closed>";
    assert_eq!(
        SvgIcon::from_bytes("malformed", malformed),
        Err(SvgIconError::InvalidSvg)
    );
    assert_eq!(
        SvgIcon::from_bytes("invalid-utf8", &[255]),
        Err(SvgIconError::InvalidUtf8)
    );
    let too_large = vec![b' '; SvgIcon::MAX_INPUT_BYTES + 1];
    assert_eq!(
        SvgIcon::from_bytes("large", &too_large),
        Err(SvgIconError::TooLarge)
    );
    let deep = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1">{}{}</svg>"#,
        "<g>".repeat(80),
        "</g>".repeat(80)
    );
    assert_eq!(
        SvgIcon::from_bytes("deep", deep.as_bytes()),
        Err(SvgIconError::InvalidSvg)
    );
    let dtd = br#"<!DOCTYPE svg [<!ENTITY payload SYSTEM "file:///invalid">]><svg xmlns="http://www.w3.org/2000/svg">&payload;</svg>"#;
    assert_eq!(
        SvgIcon::from_bytes("entity", dtd),
        Err(SvgIconError::InvalidSvg)
    );
    let static_invalid = SvgIcon::from_static("static-invalid", "<svg");
    assert_eq!(
        static_invalid.rasterize(12, Color32::WHITE),
        Err(SvgIconError::InvalidSvg)
    );
    assert_eq!(
        Icon::Check
            .svg()
            .rasterize(SvgIcon::MAX_PHYSICAL_PX + 1, Color32::WHITE),
        Err(SvgIconError::InvalidSize),
    );
}

#[test]
fn cache_eviction_is_lru_and_context_local() -> Result<(), SvgIconError> {
    let ctx = Context::default();
    let engine = |index: usize| {
        let tint = Color32::from_rgb(index as u8, 130, 180);
        SvgIconEngine::texture(&ctx, &Icon::CheckCircle2.svg(), 16., tint).map(|t| t.id())
    };
    let touched = engine(0)?;
    let evicted = engine(1)?;
    for index in 2..SvgIconEngine::MAX_TEXTURES {
        engine(index)?;
    }
    assert_eq!(engine(0)?, touched);
    engine(SvgIconEngine::MAX_TEXTURES)?;
    assert_eq!(
        SvgIconEngine::stats(&ctx).textures,
        SvgIconEngine::MAX_TEXTURES
    );
    assert_eq!(engine(0)?, touched);
    assert_ne!(engine(1)?, evicted);
    assert_eq!(SvgIconEngine::stats(&Context::default()).textures, 0);

    let same_id = SvgIcon::from_static("shared", SQUARE);
    let first = SvgIconEngine::texture(&ctx, &same_id, 20., Color32::WHITE)?;
    let other_source = SvgIcon::from_bytes("shared", BRAND)?;
    let second = SvgIconEngine::texture(&ctx, &other_source, 20., Color32::WHITE)?;
    assert_ne!(first.id(), second.id());
    Ok(())
}

#[test]
fn cache_respects_byte_budget_before_entry_limit() -> Result<(), SvgIconError> {
    let ctx = Context::default();
    let size = SvgIcon::MAX_PHYSICAL_PX as f32;
    let icon = SvgIcon::from_static("square", SQUARE);
    let first = SvgIconEngine::texture(&ctx, &icon, size, Color32::WHITE)?.id();
    for red in 0..8 {
        let _ = SvgIconEngine::texture(&ctx, &icon, size, Color32::from_rgb(red, 0, 0))?;
    }
    let stats = SvgIconEngine::stats(&ctx);
    assert_eq!(stats.textures, 8);
    assert_eq!(stats.bytes, SvgIconEngine::MAX_TEXTURE_BYTES);
    assert_ne!(
        SvgIconEngine::texture(&ctx, &icon, size, Color32::WHITE)?.id(),
        first
    );
    Ok(())
}
