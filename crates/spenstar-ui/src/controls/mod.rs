//! Selection, toggle, and range controls.
mod choice;
mod radio;
mod slider;
mod toggle;

pub use crate::button::Size;
pub use radio::{RadioGroup, RadioItem, RadioVariant};
pub use slider::{RangeSlider, Slider, SliderOptions};
pub use toggle::{CheckState, Checkbox, CheckboxGroup, CheckboxValue, Switch};

use crate::theme::{Palette, Semantics};

/// Flow direction for grouped choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    /// Stack choices.
    #[default]
    Vertical,
    /// Wrap choices across rows.
    Horizontal,
}

impl Direction {
    fn show<R>(
        self,
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        match self {
            Self::Vertical => ui.vertical(content),
            Self::Horizontal => ui.horizontal_wrapped(content),
        }
    }
}

/// Semantic color treatment for controls and tags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// Neutral surface.
    Neutral,
    /// Theme accent.
    #[default]
    Accent,
    /// Positive feedback.
    Success,
    /// Caution.
    Warning,
    /// Destructive or invalid state.
    Danger,
}

impl Tone {
    pub(crate) fn palette(self, c: Semantics) -> Palette {
        match self {
            Self::Neutral => c.default,
            Self::Accent => c.accent,
            Self::Success => c.success,
            Self::Warning => c.warning,
            Self::Danger => c.danger,
        }
    }
}
