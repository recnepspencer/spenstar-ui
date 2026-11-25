//! Primitive, semantic, and component-facing design tokens.
use egui::{Color32, Context, CornerRadius, Shadow, Stroke, Visuals};

/// Raw design values shared by both appearances.
pub mod primitives {
    use egui::Color32;

    /// Four-point spacing scale.
    pub const SPACE: [f32; 10] = [0., 4., 8., 12., 16., 20., 24., 32., 40., 48.];
    /// Neutral ramp from light to dark.
    pub const NEUTRAL: [Color32; 13] = [
        Color32::WHITE,
        Color32::from_rgb(248, 249, 252),
        Color32::from_rgb(241, 244, 248),
        Color32::from_rgb(225, 230, 237),
        Color32::from_rgb(205, 213, 225),
        Color32::from_rgb(158, 174, 194),
        Color32::from_rgb(106, 121, 143),
        Color32::from_rgb(77, 91, 111),
        Color32::from_rgb(54, 66, 85),
        Color32::from_rgb(37, 46, 62),
        Color32::from_rgb(26, 33, 46),
        Color32::from_rgb(19, 25, 36),
        Color32::from_rgb(13, 18, 28),
    ];
}

/// Light or dark appearance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Light canvas and dark text.
    #[default]
    Light,
    /// Dark canvas and light text.
    Dark,
}

/// Built-in and user-defined accent colors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Accent {
    /// Blue.
    #[default]
    Blue,
    /// Indigo.
    Indigo,
    /// Violet.
    Violet,
    /// Emerald.
    Emerald,
    /// Amber.
    Amber,
    /// Rose.
    Rose,
    /// Slate.
    Slate,
    /// A user-supplied color, composited over the surface if translucent.
    Custom(Color32),
}

impl Accent {
    /// All built-in presets.
    pub const PRESETS: [Self; 7] = [
        Self::Blue,
        Self::Indigo,
        Self::Violet,
        Self::Emerald,
        Self::Amber,
        Self::Rose,
        Self::Slate,
    ];

    /// Primitive accent color.
    pub const fn color(self) -> Color32 {
        match self {
            Self::Blue => Color32::from_rgb(37, 99, 235),
            Self::Indigo => Color32::from_rgb(79, 70, 229),
            Self::Violet => Color32::from_rgb(124, 58, 237),
            Self::Emerald => Color32::from_rgb(5, 150, 105),
            Self::Amber => Color32::from_rgb(217, 119, 6),
            Self::Rose => Color32::from_rgb(225, 29, 72),
            Self::Slate => Color32::from_rgb(71, 85, 105),
            Self::Custom(color) => color,
        }
    }
}

/// A semantic color family with derived interaction states.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Solid fill.
    pub base: Color32,
    /// Contrasting text on solid fills.
    pub foreground: Color32,
    /// Hovered solid fill.
    pub hover: Color32,
    /// Pressed solid fill.
    pub active: Color32,
    /// Subtle background.
    pub soft: Color32,
    /// Hovered subtle background.
    pub soft_hover: Color32,
    /// Focus outline.
    pub ring: Color32,
}

impl Palette {
    fn derive(base: Color32, surface: Color32) -> Self {
        let [r, g, b, a] = base.to_srgba_unmultiplied();
        let base = mix(surface, Color32::from_rgb(r, g, b), f32::from(a) / 255.);
        let (foreground, shade) = if luminance(base) > 0.179 {
            (Color32::BLACK, Color32::WHITE)
        } else {
            (Color32::WHITE, Color32::BLACK)
        };
        let ring = if luminance(surface) > 0.179 {
            mix(base, Color32::BLACK, 0.25)
        } else {
            mix(base, Color32::WHITE, 0.4)
        };
        Self {
            base,
            foreground,
            hover: mix(base, shade, 0.10),
            active: mix(base, shade, 0.20),
            soft: mix(surface, base, 0.13),
            soft_hover: mix(surface, base, 0.23),
            ring,
        }
    }
}

/// Role-based canvas, text, and feedback colors.
#[derive(Clone, Copy, Debug)]
pub struct Semantics {
    /// App canvas.
    pub background: Color32,
    /// Main panel fill.
    pub surface: Color32,
    /// Menus and raised panels.
    pub surface_elevated: Color32,
    /// Recessed areas and secondary controls.
    pub surface_subtle: Color32,
    /// Modal scrim.
    pub overlay: Color32,
    /// Control outline.
    pub border: Color32,
    /// Panel outline.
    pub border_subtle: Color32,
    /// Section separator.
    pub divider: Color32,
    /// Primary text.
    pub text_primary: Color32,
    /// Secondary text.
    pub text_secondary: Color32,
    /// Lower-emphasis text.
    pub text_muted: Color32,
    /// Neutral actions.
    pub default: Palette,
    /// Primary actions.
    pub accent: Palette,
    /// Positive feedback.
    pub success: Palette,
    /// Caution.
    pub warning: Palette,
    /// Destructive actions or errors.
    pub danger: Palette,
}

/// Corner radii in egui points, capped to half the shortest widget edge.
#[derive(Clone, Copy, Debug)]
pub enum Radius {
    /// Two points.
    Xs,
    /// Four points.
    Sm,
    /// Eight points.
    Md,
    /// Twelve points.
    Lg,
    /// Sixteen points.
    Xl,
    /// Pill when supported by egui's radius representation.
    Full,
}

impl Radius {
    /// Resolve the radius for a widget of the given size.
    pub fn for_size(self, size: egui::Vec2) -> CornerRadius {
        let max = match self {
            Self::Xs => 2.,
            Self::Sm => 4.,
            Self::Md => 8.,
            Self::Lg => 12.,
            Self::Xl => 16.,
            Self::Full => 255.,
        };
        CornerRadius::same((size.x.min(size.y) * 0.5).clamp(0., max) as u8)
    }
}

/// Resolved appearance for one mode and accent.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    /// Light or dark mode.
    pub mode: Mode,
    /// Accent selection.
    pub accent: Accent,
    /// Resolved design tokens.
    pub colors: Semantics,
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(Mode::Light, Accent::Blue)
    }
}

impl Theme {
    /// Resolve a theme from its appearance and accent.
    pub fn new(mode: Mode, accent: Accent) -> Self {
        let n = primitives::NEUTRAL;
        let [
            background,
            surface,
            elevated,
            subtle,
            border,
            line,
            primary,
            secondary,
            muted,
        ] = match mode {
            Mode::Light => [n[1], n[0], n[0], n[2], n[4], n[3], n[12], n[7], n[7]],
            Mode::Dark => [n[12], n[11], n[10], n[10], n[8], n[9], n[1], n[4], n[5]],
        };
        let palette = |base| Palette::derive(base, surface);
        Self {
            mode,
            accent,
            colors: Semantics {
                background,
                surface,
                surface_elevated: elevated,
                surface_subtle: subtle,
                overlay: Color32::from_black_alpha(if mode == Mode::Dark { 168 } else { 104 }),
                border,
                border_subtle: line,
                divider: line,
                text_primary: primary,
                text_secondary: secondary,
                text_muted: muted,
                default: palette(subtle),
                accent: palette(accent.color()),
                success: palette(Color32::from_rgb(5, 150, 105)),
                warning: palette(Color32::from_rgb(217, 119, 6)),
                danger: palette(Color32::from_rgb(220, 38, 38)),
            },
        }
    }

    /// Soft shadow for an elevated surface.
    pub fn shadow(self) -> Shadow {
        Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 0,
            color: Color32::from_black_alpha(if self.mode == Mode::Dark { 90 } else { 24 }),
        }
    }

    /// Register the bundled fonts and apply the theme.
    pub fn install(self, ctx: &Context) {
        crate::typography::install(ctx);
        self.apply(ctx);
    }

    /// Synchronize egui visuals and make the theme available to components.
    pub fn apply(self, ctx: &Context) {
        let c = self.colors;
        let (mode, mut visuals) = match self.mode {
            Mode::Light => (egui::Theme::Light, Visuals::light()),
            Mode::Dark => (egui::Theme::Dark, Visuals::dark()),
        };
        visuals.override_text_color = Some(c.text_primary);
        visuals.panel_fill = c.background;
        visuals.window_fill = c.surface_elevated;
        visuals.window_stroke = Stroke::new(1., c.border_subtle);
        visuals.window_shadow = self.shadow();
        visuals.popup_shadow = self.shadow();
        visuals.extreme_bg_color = c.surface;
        visuals.faint_bg_color = c.surface_subtle;
        visuals.code_bg_color = c.surface_subtle;
        visuals.hyperlink_color = c.accent.ring;
        visuals.warn_fg_color = c.warning.ring;
        visuals.error_fg_color = c.danger.ring;
        visuals.selection.bg_fill = c.accent.soft_hover;
        visuals.selection.stroke = Stroke::new(1., c.text_primary);
        let widgets = &mut visuals.widgets;
        for (widget, fill) in [
            (&mut widgets.noninteractive, c.surface),
            (&mut widgets.inactive, c.surface_subtle),
            (&mut widgets.hovered, c.default.hover),
            (&mut widgets.active, c.accent.soft_hover),
            (&mut widgets.open, c.accent.soft),
        ] {
            widget.bg_fill = fill;
            widget.weak_bg_fill = fill;
            widget.bg_stroke = Stroke::new(1., c.border);
            widget.fg_stroke = Stroke::new(1., c.text_primary);
            widget.corner_radius = CornerRadius::same(6);
        }
        ctx.set_visuals_of(mode, visuals);
        ctx.set_theme(mode);
        ctx.data_mut(|data| data.insert_temp(egui::Id::unique("spenstar-ui/theme"), self));
    }

    /// Read the installed theme, falling back to the current egui appearance.
    pub fn current(ctx: &Context) -> Self {
        ctx.data(|data| data.get_temp(egui::Id::unique("spenstar-ui/theme")))
            .unwrap_or_else(|| {
                let mode = if ctx.theme() == egui::Theme::Dark {
                    Mode::Dark
                } else {
                    Mode::Light
                };
                Self::new(mode, Accent::Blue)
            })
    }
}

pub(crate) fn mix(a: Color32, b: Color32, factor: f32) -> Color32 {
    let [ar, ag, ab, aa] = a.to_array();
    let [br, bg, bb, ba] = b.to_array();
    let channel = |a: u8, b: u8| egui::lerp(f32::from(a)..=f32::from(b), factor).round() as u8;
    Color32::from_rgba_premultiplied(
        channel(ar, br),
        channel(ag, bg),
        channel(ab, bb),
        channel(aa, ba),
    )
}

pub(crate) fn luminance(color: Color32) -> f32 {
    let [r, g, b, _] = color.to_array().map(|v| {
        let c = f32::from(v) / 255.;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}
