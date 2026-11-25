use std::{error::Error, fmt, sync::Arc};

use egui::{Color32, ColorImage, Rect, Ui, Vec2, pos2};
use resvg::{tiny_skia, usvg};

use super::SvgIconEngine;

/// A bounded SVG parsing or rendering failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SvgIconError {
    /// The document exceeds the input byte limit.
    TooLarge,
    /// The SVG is not UTF-8.
    InvalidUtf8,
    /// The document is malformed or cannot be parsed as SVG.
    InvalidSvg,
    /// The requested physical size is zero, non-finite, or above the renderer's limit.
    InvalidSize,
    /// A raster buffer could not be allocated.
    Rasterization,
}

impl fmt::Display for SvgIconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "SVG exceeds the input byte limit",
            Self::InvalidUtf8 => "SVG must be UTF-8",
            Self::InvalidSvg => "invalid SVG document",
            Self::InvalidSize => "physical icon size is outside the supported range",
            Self::Rasterization => "could not allocate the icon raster",
        })
    }
}

impl Error for SvgIconError {}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Text {
    Static(&'static str),
    Owned(Arc<str>),
}

impl Text {
    fn as_str(&self) -> &str {
        match self {
            Self::Static(text) => text,
            Self::Owned(text) => text,
        }
    }
}

/// A self-contained vector SVG and its caller-defined identifier.
///
/// Vector-only icons are rendered offline. External resources, embedded raster
/// images, and text are omitted. Convert lettering into outlines before embedding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SvgIcon {
    id: Text,
    source: Text,
}

impl SvgIcon {
    /// Maximum input size, in bytes.
    pub const MAX_INPUT_BYTES: usize = 64 * 1024;
    /// Maximum raster edge length, in physical pixels.
    pub const MAX_PHYSICAL_PX: u32 = 512;

    /// Borrow a static document. Parsing is deferred until first render.
    pub const fn from_static(id: &'static str, svg: &'static str) -> Self {
        Self {
            id: Text::Static(id),
            source: Text::Static(svg),
        }
    }

    /// Copy and validate a UTF-8 SVG.
    pub fn from_bytes(id: impl Into<String>, svg: &[u8]) -> Result<Self, SvgIconError> {
        if svg.len() > Self::MAX_INPUT_BYTES {
            return Err(SvgIconError::TooLarge);
        }
        let source = std::str::from_utf8(svg).map_err(|_| SvgIconError::InvalidUtf8)?;
        parse(source)?;
        Ok(Self {
            id: Text::Owned(Arc::from(id.into())),
            source: Text::Owned(Arc::from(source)),
        })
    }

    /// Caller-defined identifier used for debugging and texture labels.
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Rasterize into a square RGBA image with the original aspect ratio.
    pub fn rasterize(&self, physical_px: u32, tint: Color32) -> Result<ColorImage, SvgIconError> {
        if !(1..=Self::MAX_PHYSICAL_PX).contains(&physical_px) {
            return Err(SvgIconError::InvalidSize);
        }
        let source = self.source.as_str();
        if source.len() > Self::MAX_INPUT_BYTES {
            return Err(SvgIconError::TooLarge);
        }
        let [r, g, b, a] = tint.to_srgba_unmultiplied();
        let color = format!("#{r:02x}{g:02x}{b:02x}{a:02x}");
        let tinted;
        let source = if source.contains("currentColor") {
            tinted = replace_current_color(source, &color)?;
            &tinted
        } else {
            source
        };
        let tree = parse(source)?;
        let mut pixmap =
            tiny_skia::Pixmap::new(physical_px, physical_px).ok_or(SvgIconError::Rasterization)?;
        let size = tree.size();
        let edge = physical_px as f32;
        let scale = edge / size.width().max(size.height());
        if !scale.is_finite() || scale <= 0. {
            return Err(SvgIconError::InvalidSvg);
        }
        let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(
            (edge - size.width() * scale) * 0.5,
            (edge - size.height() * scale) * 0.5,
        );
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        Ok(ColorImage::from_rgba_premultiplied(
            [physical_px as usize; 2],
            pixmap.data(),
        ))
    }

    /// Draw the icon, resolving a context-local HiDPI texture when visible.
    pub fn paint(&self, ui: &Ui, rect: Rect, tint: Color32) -> Result<(), SvgIconError> {
        if !ui.is_rect_visible(rect) {
            return Ok(());
        }
        let size = rect.width().min(rect.height());
        let texture = SvgIconEngine::texture(ui.ctx(), self, size, tint)?;
        ui.painter().image(
            texture.id(),
            Rect::from_center_size(rect.center(), Vec2::splat(size)),
            Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)),
            Color32::WHITE,
        );
        Ok(())
    }
}

fn replace_current_color(svg: &str, color: &str) -> Result<String, SvgIconError> {
    let xml = parse_xml(svg)?;
    let mut tokens = Vec::new();
    for node in xml.descendants() {
        for attribute in node.attributes() {
            if matches!(
                attribute.name(),
                "fill"
                    | "stroke"
                    | "stop-color"
                    | "flood-color"
                    | "lighting-color"
                    | "style"
                    | "filter"
            ) {
                paint_tokens(svg, attribute.range_value(), &mut tokens);
            }
        }
        if node.is_text()
            && node
                .parent()
                .is_some_and(|parent| parent.has_tag_name("style"))
        {
            paint_tokens(svg, node.range(), &mut tokens);
        }
    }
    tokens.sort_unstable_by_key(|range| range.start);
    let mut tinted = svg.to_owned();
    for range in tokens.into_iter().rev() {
        tinted.replace_range(range, color);
    }
    Ok(tinted)
}

fn paint_tokens(
    svg: &str,
    range: std::ops::Range<usize>,
    tokens: &mut Vec<std::ops::Range<usize>>,
) {
    let value = &svg[range.clone()];
    let identifier = |c: char| c.is_alphanumeric() || matches!(c, '_' | '-');
    for (start, token) in value.match_indices("currentColor") {
        let end = start + token.len();
        let before = value[..start].chars().next_back();
        let after = value[end..].chars().next();
        if !before.is_some_and(|c| identifier(c) || matches!(c, '#' | '.'))
            && !after.is_some_and(identifier)
        {
            tokens.push(range.start + start..range.start + end);
        }
    }
}

fn parse_xml(svg: &str) -> Result<roxmltree::Document<'_>, SvgIconError> {
    let doc = roxmltree::Document::parse_with_options(
        svg,
        roxmltree::ParsingOptions {
            nodes_limit: 4096,
            ..Default::default()
        },
    )
    .map_err(|_| SvgIconError::InvalidSvg)?;
    if !doc
        .root_element()
        .has_tag_name(("http://www.w3.org/2000/svg", "svg"))
    {
        return Err(SvgIconError::InvalidSvg);
    }
    Ok(doc)
}

fn parse(svg: &str) -> Result<usvg::Tree, SvgIconError> {
    let xml = parse_xml(svg)?;
    let options = usvg::Options {
        max_depth: 32,
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    usvg::Tree::from_xmltree(&xml, &options).map_err(|_| SvgIconError::InvalidSvg)
}
