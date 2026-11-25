//! Lucide and custom vector icons rendered at the display's physical scale.
mod cache;
mod lucide;
mod svg;

pub use cache::{IconCacheStats, SvgIconEngine};
pub use lucide::{Icon, LUCIDE_LICENSE};
pub use svg::{SvgIcon, SvgIconError};

use egui::{Color32, Rect, Response, Sense, Ui, Vec2};

/// An icon that can be passed to a layout or control.
#[derive(Clone, Debug)]
pub enum IconSource {
    /// An embedded Lucide icon.
    BuiltIn(Icon),
    /// A custom SVG document.
    Svg(SvgIcon),
}

impl IconSource {
    /// Paint centered in a square or rectangular logical destination.
    pub fn paint(&self, ui: &Ui, rect: Rect, tint: Color32) -> Result<(), SvgIconError> {
        match self {
            Self::BuiltIn(icon) => icon.svg().paint(ui, rect, tint),
            Self::Svg(svg) => svg.paint(ui, rect, tint),
        }
    }

    /// Allocate an icon and paint it when visible.
    pub fn show(&self, ui: &mut Ui, size: f32, tint: Color32) -> Response {
        let size = if size.is_finite() { size.max(0.) } else { 0. };
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        let _ = self.paint(ui, rect, tint);
        response
    }
}

impl From<Icon> for IconSource {
    fn from(icon: Icon) -> Self {
        Self::BuiltIn(icon)
    }
}

impl From<SvgIcon> for IconSource {
    fn from(svg: SvgIcon) -> Self {
        Self::Svg(svg)
    }
}
