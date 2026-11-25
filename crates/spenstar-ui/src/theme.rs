//! Primitive, semantic, and component-facing design tokens.
use egui::{Color32, Context, CornerRadius, Shadow, Stroke, Visuals};

/// Raw design values shared by both appearances.
pub mod primitives {
    use egui::Color32;

    /// Four-point spacing scale.
    pub const SPACE: [f32; 10] = [0., 4., 8., 12., 16., 20., 24., 32., 40., 48.];
    /// Neutral ramp from light to dark.
    pub const NEUTRAL: [Color32; 13] = [
        Color32::from_rgb(255, 255, 255),
        Color32::from_rgb(245, 245, 245),
        Color32::from_rgb(239, 239, 240),
        Color32::from_rgb(228, 228, 231),
        Color32::from_rgb(221, 222, 224),
        Color32::from_rgb(159, 159, 169),
        Color32::from_rgb(113, 113, 122),
        Color32::from_rgb(82, 82, 91),
        Color32::from_rgb(40, 40, 44),
        Color32::from_rgb(39, 39, 42),
        Color32::from_rgb(35, 35, 37),
        Color32::from_rgb(24, 24, 27),
        Color32::from_rgb(6, 6, 7),
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
            Self::Blue => Color32::from_rgb(4, 133, 247),
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
            soft: mix(surface, base, 0.14),
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
    /// Secondary control fill.
    pub surface_secondary: Color32,
    /// Tertiary control fill.
    pub surface_tertiary: Color32,
    /// Filled control hover state.
    pub field_hover: Color32,
    /// Popover surface.
    pub overlay: Color32,
    /// Popover perimeter.
    pub overlay_border: Color32,
    /// Modal backdrop.
    pub scrim: Color32,
    /// Segmented control track.
    pub segmented_track: Color32,
    /// Unselected radio indicator.
    pub radio_fill: Color32,
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
        let light = mode == Mode::Light;
        let (background, surface, border, primary, secondary, muted) = if light {
            (n[1], n[0], n[4], n[11], n[7], n[6])
        } else {
            (n[12], n[11], n[8], rgb(0xFCFCFC), rgb(0xD4D4D8), n[5])
        };
        let elevated = if light { surface } else { n[10] };
        let secondary_surface = if light { n[2] } else { n[9] };
        let subtle = if light { background } else { rgb(0x121215) };
        let line = if light { n[3] } else { rgb(0x212124) };
        let neutral = if light { rgb(0xEBEBEC) } else { n[9] };
        let palette = |base| Palette::derive(base, surface);
        let mut primary_action = palette(accent.color());
        if accent == Accent::Blue {
            primary_action.foreground = rgb(0xFCFCFC);
            primary_action.ring = primary_action.base;
        }
        let mut success = palette(rgb(0x17C964));
        success.soft = mix(surface, rgb(0x14C965), 0.15);
        success.ring = if light { rgb(0x2B7745) } else { rgb(0x7AE8AD) };
        let mut danger = palette(rgb(if light { 0xFF383C } else { 0xDB3B3E }));
        danger.ring = if light { rgb(0xA91F29) } else { rgb(0xFF8185) };
        Self {
            mode,
            accent,
            colors: Semantics {
                background,
                surface,
                surface_elevated: elevated,
                surface_subtle: subtle,
                surface_secondary: secondary_surface,
                surface_tertiary: if light { rgb(0xEAEAEB) } else { rgb(0x2E2E32) },
                field_hover: rgb(if light { 0xE8E8EA } else { 0x2E2E32 }),
                overlay: surface,
                overlay_border: if light { border } else { rgb(0x2E2E32) },
                scrim: Color32::from_black_alpha(if light { 104 } else { 168 }),
                segmented_track: if light { neutral } else { subtle },
                radio_fill: if light { n[3] } else { n[9] },
                border,
                border_subtle: line,
                divider: line,
                text_primary: primary,
                text_secondary: secondary,
                text_muted: muted,
                default: Palette {
                    foreground: primary,
                    ..palette(neutral)
                },
                accent: primary_action,
                success,
                warning: palette(rgb(if light { 0xF5A524 } else { 0xF7B750 })),
                danger,
            },
        }
    }

    /// Soft shadow for an elevated surface.
    pub fn shadow(self) -> Shadow {
        Shadow {
            offset: [0, 2],
            blur: 8,
            spread: 0,
            color: Color32::from_black_alpha(if self.mode == Mode::Dark { 64 } else { 12 }),
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
        visuals.window_fill = c.overlay;
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
        widgets.noninteractive.bg_stroke = Stroke::new(1., c.divider);
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

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
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
