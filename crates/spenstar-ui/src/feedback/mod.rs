//! Semantic status, transient notifications, and loading indicators.
mod alert;
mod progress;
mod skeleton;
mod toast;

pub use alert::{Alert, AlertResponse, Banner};
pub use progress::{Meter, ProgressBar, ProgressCircle, Spinner};
pub use skeleton::Skeleton;
pub use toast::{Toast, ToastEvent, ToastPosition, ToastQueue};

pub use crate::controls::{Size, Tone};

use egui::Color32;

use crate::{icon::Icon, theme::luminance};

pub(crate) fn status_icon(tone: Tone) -> Icon {
    match tone {
        Tone::Neutral => Icon::Info,
        Tone::Accent => Icon::Sparkles,
        Tone::Success => Icon::CheckCircle2,
        Tone::Warning => Icon::AlertTriangle,
        Tone::Danger => Icon::AlertCircle,
    }
}

pub(crate) fn semantic_ink(preferred: Color32, fill: Color32, fallback: Color32) -> Color32 {
    let a = luminance(preferred);
    let b = luminance(fill);
    if (a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5 {
        preferred
    } else {
        fallback
    }
}

fn unit(value: f32) -> f32 {
    if value.is_nan() {
        0.
    } else {
        value.clamp(0., 1.)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Theme,
        theme::{Accent, Mode},
    };

    use super::*;

    #[test]
    fn semantic_foregrounds_keep_aa_contrast_across_both_themes() {
        fn contrast(a: Color32, b: Color32) -> f32 {
            let (a, b) = (luminance(a), luminance(b));
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        }

        for mode in [Mode::Light, Mode::Dark] {
            for accent in Accent::PRESETS {
                let c = Theme::new(mode, accent).colors;
                for tone in [
                    Tone::Neutral,
                    Tone::Accent,
                    Tone::Success,
                    Tone::Warning,
                    Tone::Danger,
                ] {
                    let palette = tone.palette(c);
                    for surface in [c.overlay, c.surface_secondary, c.field_hover, palette.soft] {
                        for preferred in [palette.ring, c.text_secondary] {
                            let ink = semantic_ink(preferred, surface, c.text_primary);
                            assert!(
                                contrast(ink, surface) >= 4.5,
                                "{mode:?} {accent:?} {tone:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}
