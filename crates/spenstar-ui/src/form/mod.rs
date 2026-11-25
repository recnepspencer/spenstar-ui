//! Labeled fields and text, search, and numeric editors.
mod area;
mod entry;
mod number;
mod text;

pub use crate::button::Size;
pub use area::TextArea;
pub(crate) use entry::{EntryStyle, icon_button, step_keys};
pub use number::{NumberField, NumberResponse};
pub use text::{SearchField, SearchResponse, TextField};

use crate::{Theme, typography::Type};
use egui::{InnerResponse, Ui};

/// Visual treatment for text editors and selection triggers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldVariant {
    /// Surface with a full perimeter stroke.
    #[default]
    Outline,
    /// Recessed surface with a full perimeter stroke.
    Filled,
    /// Flat surface with a bottom rule.
    Underlined,
}

/// A label, helper text, and validation wrapper for a form control.
#[derive(Clone, Copy, Debug)]
pub struct Field<'a> {
    /// Visible label.
    pub label: &'a str,
    /// Marks the label as required.
    pub required: bool,
    /// Supporting copy below the control.
    pub description: Option<&'a str>,
    /// Error feedback below the control. Also supplied to the content as invalid state.
    pub error: Option<&'a str>,
    /// Disable all interactive content.
    pub disabled: bool,
}

impl<'a> Field<'a> {
    /// Create a field with a visible label.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            required: false,
            description: None,
            error: None,
            disabled: false,
        }
    }

    /// Render the field and supply its invalid state to the control.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui, bool) -> R,
    ) -> InnerResponse<R> {
        self.show_with(ui, |_| {}, |ui, invalid, _| content(ui, invalid))
    }

    /// Render a header trailing slot and supply the label ID for accessible labeling.
    pub fn show_with<R>(
        self,
        ui: &mut Ui,
        trailing: impl FnOnce(&mut Ui),
        content: impl FnOnce(&mut Ui, bool, egui::Id) -> R,
    ) -> InnerResponse<R> {
        let c = Theme::current(ui.ctx()).colors;
        ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            let label = ui
                .horizontal(|ui| {
                    let label = ui.label(Type::Small.text(self.label));
                    if self.required {
                        ui.label(Type::Small.text("*").color(c.danger.ring));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), trailing);
                    label.id
                })
                .inner;
            let value = content(ui, self.error.is_some(), label);
            if let Some(description) = self.description {
                ui.label(Type::Small.text(description).color(c.text_secondary));
            }
            if let Some(error) = self.error {
                ui.label(Type::Small.text(error).color(c.danger.ring));
            }
            value
        })
    }
}
