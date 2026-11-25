//! Semantic text styles backed by bundled Inter and JetBrains Mono.
use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, RichText, TextStyle};

const FONTS: [(&str, &[u8]); 6] = [
    (
        "inter-regular",
        include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    ),
    (
        "inter-medium",
        include_bytes!("../assets/fonts/Inter-Medium.ttf"),
    ),
    (
        "inter-semibold",
        include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
    ),
    (
        "inter-bold",
        include_bytes!("../assets/fonts/Inter-Bold.ttf"),
    ),
    (
        "mono-regular",
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    ),
    (
        "mono-medium",
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
    /// Key metric figure.
    Metric,
    /// Standard medium-weight label.
    Label,
    /// Compact medium-weight label or badge.
    Badge,
    /// Medium-weight navigation section label.
    SectionLabel,
    /// Workspace or account name.
    Brand,
    /// Secondary descriptive text.
    Description,
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
            Self::Display => (40., named("inter-bold")),
            Self::H1 => (20., named("inter-semibold")),
            Self::H2 => (16., named("inter-medium")),
            Self::H3 => (16., named("inter-medium")),
            Self::H4 => (15., named("inter-medium")),
            Self::Metric => (26., named("inter-semibold")),
            Self::Label => (14., named("inter-medium")),
            Self::Badge => (12., named("inter-medium")),
            Self::SectionLabel => (11., named("inter-medium")),
            Self::Brand => (14., named("inter-semibold")),
            Self::Description => (13., named("inter-regular")),
            Self::Body => (14., named("inter-regular")),
            Self::Small => (12., named("inter-regular")),
            Self::Caption => (12., named("inter-regular")),
            Self::Mono => (13., named("mono-regular")),
            Self::Kbd => (11., named("mono-medium")),
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
        let base = if name.starts_with("inter") {
            FontFamily::Proportional
        } else {
            FontFamily::Monospace
        };
        let mut fallback = fonts.families.get(&base).cloned().unwrap_or_default();
        fallback.insert(0, name.to_owned());
        if name.ends_with("-regular") {
            fonts.families.insert(base, fallback.clone());
        }
        fonts.families.insert(named(name), fallback);
    }
    ctx.set_fonts(fonts);
    ctx.all_styles_mut(|style| {
        for (slot, role) in [
            (TextStyle::Heading, Type::H2),
            (TextStyle::Body, Type::Body),
            (TextStyle::Button, Type::Label),
            (TextStyle::Small, Type::Small),
            (TextStyle::Monospace, Type::Mono),
        ] {
            style.text_styles.insert(slot, role.font());
        }
    });
}
