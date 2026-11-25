//! Dropdown selection, chip-based multi-selection, and autocomplete.
mod combo;
mod list;
pub(crate) mod trigger;

pub use crate::form::{FieldVariant, Size};
pub use combo::{ComboBox, ComboResponse};

use crate::{
    icon::IconSource,
    popover,
    tags::{Tag, chip},
    typography::Type,
};
use egui::{Response, Ui};
use list::{Menu, Row};
use trigger::Trigger;

/// A choice with a stable value and a display label.
#[derive(Clone, Debug)]
pub struct SelectOption<'a> {
    /// Value returned by the widget. Use unique values within a menu.
    pub value: &'a str,
    /// Visible option name and autocomplete search text.
    pub label: &'a str,
    /// Optional leading icon.
    pub icon: Option<IconSource>,
    /// Exclude this option from activation and keyboard navigation.
    pub disabled: bool,
}

impl<'a> SelectOption<'a> {
    /// Construct an enabled, labeled option.
    pub fn new(value: &'a str, label: &'a str) -> Self {
        Self {
            value,
            label,
            icon: None,
            disabled: false,
        }
    }
}

/// A labeled or unlabeled section of options.
#[derive(Clone, Copy, Debug)]
pub struct SelectSection<'a> {
    /// Optional heading.
    pub label: Option<&'a str>,
    /// Choices in display order.
    pub options: &'a [SelectOption<'a>],
}

/// Shared visual settings for selector triggers.
#[derive(Clone, Debug)]
pub struct SelectStyle<'a> {
    /// Hint when there is no selection.
    pub placeholder: &'a str,
    /// Trigger height.
    pub size: Size,
    /// Trigger treatment.
    pub variant: FieldVariant,
    /// Leading icon.
    pub leading: Option<IconSource>,
    /// Draw error styling.
    pub invalid: bool,
    /// Disable interaction.
    pub disabled: bool,
}

impl Default for SelectStyle<'_> {
    fn default() -> Self {
        Self {
            placeholder: "Choose an option…",
            size: Size::Md,
            variant: FieldVariant::Outline,
            leading: None,
            invalid: false,
            disabled: false,
        }
    }
}

/// A grouped, single-value dropdown.
#[derive(Clone, Debug, Default)]
pub struct Select<'a> {
    /// Trigger options.
    pub style: SelectStyle<'a>,
    /// Offer a clear action when a value is selected.
    pub clearable: bool,
}

/// Dropdown trigger and visibility state.
pub struct SelectResponse {
    /// Trigger response. Changed indicates a selection change.
    pub response: Response,
    /// Whether the popup is open.
    pub open: bool,
}

impl Select<'_> {
    /// Display a dropdown and update the selected option value.
    pub fn show(
        self,
        ui: &mut Ui,
        value: &mut Option<String>,
        sections: &[SelectSection<'_>],
    ) -> SelectResponse {
        let chosen = sections
            .iter()
            .flat_map(|s| s.options)
            .find(|option| value.as_deref() == Some(option.value));
        let label = chosen
            .map(|item| item.label)
            .or(value.as_deref())
            .unwrap_or(self.style.placeholder);
        let clear = self.clearable && value.is_some();
        let (mut response, cleared) = Trigger::new(&self.style, label, value.is_none())
            .clearable(clear)
            .show(ui);
        if cleared {
            value.take();
            response.mark_changed();
        }
        let menu = Menu::new(sections);
        if let Some(Row::Option(option)) = menu.show(&response, |id| value.as_deref() == Some(id)) {
            if value.as_deref() != Some(option.value) {
                *value = Some(option.value.to_owned());
                response.mark_changed();
            }
            popover::close(&response, true);
        }
        let open = popover::is_open(&response);
        SelectResponse { response, open }
    }
}

/// A dropdown displaying selected options as removable chips.
#[derive(Clone, Debug)]
pub struct MultiSelect<'a> {
    /// Trigger options.
    pub style: SelectStyle<'a>,
    /// Maximum number of individually displayed chips.
    pub max_visible_chips: usize,
    /// Include a clear-all row in the menu.
    pub clear_all: bool,
    /// Include the selection count in the menu.
    pub show_count: bool,
}

impl Default for MultiSelect<'_> {
    fn default() -> Self {
        Self {
            style: SelectStyle::default(),
            max_visible_chips: 3,
            clear_all: true,
            show_count: false,
        }
    }
}

impl MultiSelect<'_> {
    /// Display a dropdown and update the selected option values.
    pub fn show(
        self,
        ui: &mut Ui,
        values: &mut Vec<String>,
        sections: &[SelectSection<'_>],
    ) -> SelectResponse {
        let name = if values.is_empty() {
            self.style.placeholder.to_owned()
        } else {
            format!("{} selected", values.len())
        };
        let (mut response, removed) = Trigger::new(&self.style, &name, values.is_empty())
            .show_custom(ui, |ui| {
                if values.is_empty() {
                    trigger::label(ui, self.style.placeholder, true, self.style.size);
                    return None;
                }
                let width = ui.available_width();
                let visible = self
                    .max_visible_chips
                    .min(values.len())
                    .min((width / 76.).max(0.) as usize);
                let overflow = values.len() - visible;
                let reserve = if overflow > 0 { 48. } else { 0. };
                let chip_width =
                    (width - reserve) / visible.max(1) as f32 - ui.spacing().item_spacing.x;
                let mut removed = None;
                for (index, value) in values.iter().take(visible).enumerate() {
                    let label = sections
                        .iter()
                        .flat_map(|s| s.options)
                        .find(|option| option.value == value)
                        .map_or(value.as_str(), |option| option.label);
                    let tag = Tag {
                        removable: true,
                        ..Tag::new(label)
                    };
                    if chip(ui, &tag, chip_width) {
                        removed = Some(index);
                    }
                }
                if overflow > 0 {
                    chip(ui, &Tag::new(&format!("+{overflow}")), reserve);
                }
                removed
            });
        if let Some(index) = removed {
            values.remove(index);
            response.mark_changed();
        }
        let mut menu = Menu::new(sections);
        menu.clear = self.clear_all && !values.is_empty();
        let count = format!("{} selected", values.len());
        if self.show_count {
            menu.caption = Some(&count);
        }
        if let Some(row) = menu.show(&response, |id| values.iter().any(|value| value == id)) {
            match row {
                Row::Option(option) => {
                    if let Some(index) = values.iter().position(|value| value == option.value) {
                        values.remove(index);
                    } else {
                        values.push(option.value.to_owned());
                    }
                    response.request_focus();
                }
                Row::Clear => {
                    values.clear();
                    response.request_focus();
                }
                Row::Create(_) => {}
            }
            response.mark_changed();
        }
        let open = popover::is_open(&response);
        SelectResponse { response, open }
    }
}

fn empty_state(ui: &mut Ui) {
    ui.add_space(6.);
    ui.label(
        Type::Small
            .text("No matches")
            .color(crate::Theme::current(ui.ctx()).colors.text_secondary),
    );
    ui.add_space(6.);
}
