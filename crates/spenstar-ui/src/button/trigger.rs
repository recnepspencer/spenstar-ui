use egui::{Response, Ui};

use super::{Button, Size, Variant};
use crate::icon::Icon;

/// A compact, accessible close or dismiss control.
pub struct CloseButton;

impl CloseButton {
    /// Render an icon-only close button.
    pub fn show(ui: &mut Ui) -> Response {
        Button::new("Close")
            .icon_only(Icon::X)
            .size(Size::Sm)
            .variant(Variant::Ghost)
            .show(ui)
    }
}

/// A themed link that opens a URL through the native egui integration.
pub struct Link {
    label: String,
    url: String,
}

impl Link {
    /// Make a link with the given label and destination.
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            url: url.into(),
        }
    }

    /// Display the link.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.hyperlink_to(self.label, self.url)
    }
}
