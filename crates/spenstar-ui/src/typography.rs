//! Semantic text styles backed by bundled Inter and JetBrains Mono.
use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, RichText, TextStyle};

const FONTS: [(&str, &[u8]); 6] = [
    (
        "Inter Regular",
        include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    ),
    (
        "Inter Medium",
        include_bytes!("../assets/fonts/Inter-Medium.ttf"),
    ),
    (
        "Inter SemiBold",
        include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
    ),
    (
        "Inter Bold",
        include_bytes!("../assets/fonts/Inter-Bold.ttf"),
    ),
    (
        "JetBrains Mono Regular",
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    ),
    (
        "JetBrains Mono Medium",
        include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf"),
    ),
];

/// A text role with a bundled font weight and point size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    /// Large display title.
    Display,
    /// Page title.
    H1,
    /// Section title.
    H2,
    /// Card title.
    H3,
    /// Subheading.
    H4,
    /// Default prose.
    Body,
    /// Compact label.
    Small,
    /// Caption or metadata.
    Caption,
    /// Monospace text.
    Mono,
    /// Keyboard shortcut.
    Kbd,
}

impl Type {
    /// Resolve the bundled face and point size.
    pub fn font(self) -> FontId {
        let (size, family) = match self {
            Self::Display => (40., named("Inter Bold")),
            Self::H1 => (30., named("Inter Bold")),
            Self::H2 => (23., named("Inter SemiBold")),
            Self::H3 => (18., named("Inter SemiBold")),
            Self::H4 => (15., named("Inter Medium")),
            Self::Body => (14., FontFamily::Proportional),
            Self::Small => (12., FontFamily::Proportional),
            Self::Caption => (11., FontFamily::Proportional),
            Self::Mono => (13., FontFamily::Monospace),
            Self::Kbd => (11., named("JetBrains Mono Medium")),
        };
        FontId::new(size, family)
    }

    /// Create an egui label with this style.
    pub fn text(self, text: impl Into<String>) -> RichText {
        RichText::new(text).font(self.font())
    }
}

fn named(name: &str) -> FontFamily {
    FontFamily::Name(Arc::from(name))
}

pub(crate) fn install(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    for (name, bytes) in FONTS {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
        let base = if name.starts_with("Inter") {
            FontFamily::Proportional
        } else {
            FontFamily::Monospace
        };
        if name.ends_with("Regular") {
            fonts
                .families
                .entry(base)
                .or_default()
                .insert(0, name.to_owned());
        } else {
            let mut fallback = fonts.families.get(&base).cloned().unwrap_or_default();
            fallback.insert(0, name.to_owned());
            fonts.families.insert(named(name), fallback);
        }
    }
    ctx.set_fonts(fonts);
    ctx.all_styles_mut(|style| {
        for (slot, role) in [
            (TextStyle::Heading, Type::H2),
            (TextStyle::Body, Type::Body),
            (TextStyle::Button, Type::H4),
            (TextStyle::Small, Type::Small),
            (TextStyle::Monospace, Type::Mono),
        ] {
            style.text_styles.insert(slot, role.font());
        }
    });
}
