use egui::{Key, Modifiers, Response, Ui};

use super::{
    SelectSection, SelectStyle,
    list::{Menu, Row},
};
use crate::{
    form::{TextField, icon_button},
    icon::Icon,
    popover,
};

/// Searchable dropdown with an optional custom-entry action.
#[derive(Clone, Debug)]
pub struct ComboBox<'a> {
    /// Editor settings.
    pub style: SelectStyle<'a>,
    /// Offer the typed text as a new value when it does not exactly match an option.
    pub allow_custom: bool,
}

impl Default for ComboBox<'_> {
    fn default() -> Self {
        Self {
            style: SelectStyle {
                placeholder: "Search options…",
                ..SelectStyle::default()
            },
            allow_custom: false,
        }
    }
}

/// Output from an autocomplete editor.
pub struct ComboResponse {
    /// The text editor response.
    pub response: Response,
    /// Whether the suggestion list is open.
    pub open: bool,
    /// Selected option value for this frame.
    pub selected: Option<String>,
    /// Custom text committed this frame.
    pub created: Option<String>,
}

impl ComboBox<'_> {
    /// Display text input, filter options, and report a committed option or custom value.
    pub fn show(
        self,
        ui: &mut Ui,
        text: &mut String,
        sections: &[SelectSection<'_>],
    ) -> ComboResponse {
        self.show_with_create(ui, text, sections, |_| {})
    }

    /// Display autocomplete and call `create` when a custom value is committed.
    pub fn show_with_create(
        self,
        ui: &mut Ui,
        text: &mut String,
        sections: &[SelectSection<'_>],
        create: impl FnOnce(&str),
    ) -> ComboResponse {
        let spacing = ui.spacing().item_spacing.x;
        let (input, toggle) = TextField {
            placeholder: self.style.placeholder,
            size: self.style.size,
            variant: self.style.variant,
            leading: self.style.leading.or_else(|| Some(Icon::Search.into())),
            disabled: self.style.disabled,
            invalid: self.style.invalid,
            ..TextField::default()
        }
        .render(ui, text, None, 22. + spacing, |ui| {
            icon_button(ui, Icon::ChevronDown, "Suggestions").clicked()
        });
        let mut response = input.response;
        // The editor owns focus, but the popup aligns to the complete field.
        let mut anchor = response.clone();
        anchor.rect = input.frame;
        anchor.interact_rect = input.frame.intersect(ui.clip_rect());
        let changed = response.changed();
        let open_by_key = response.enabled()
            && response.has_focus()
            && !popover::is_open(&anchor)
            && ui.input_mut(|input| {
                let down = input.consume_key(Modifiers::NONE, Key::ArrowDown);
                let up = input.consume_key(Modifiers::NONE, Key::ArrowUp);
                down || up
            });
        if toggle {
            popover::toggle(&anchor);
            response.request_focus();
        } else if response.enabled() && (response.clicked() || changed || open_by_key) {
            popover::open(&anchor);
        }
        let trimmed = text.trim();
        let matched = sections
            .iter()
            .flat_map(|s| s.options)
            .find(|option| option.label.to_lowercase() == trimmed.to_lowercase());
        let mut menu = Menu::new(sections);
        menu.query = trimmed;
        menu.reset = changed;
        menu.create =
            (self.allow_custom && matched.is_none() && !trimmed.is_empty()).then_some(trimmed);
        let action = menu.show_with_submit(&anchor, input.submitted, |value| {
            matched.is_some_and(|option| option.value == value)
        });
        let mut selected = None;
        let mut created = None;
        match action {
            Some(Row::Option(option)) => {
                selected = Some(option.value.to_owned());
                *text = option.label.to_owned();
            }
            Some(Row::Create(value)) => {
                let value = value.to_owned();
                create(&value);
                *text = value.clone();
                created = Some(value);
            }
            _ => {}
        }
        if selected.is_some() || created.is_some() {
            popover::close(&anchor, true);
            response.mark_changed();
        }
        let open = popover::is_open(&anchor);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::Role::EditableComboBox);
            node.set_expanded(open);
        });
        ComboResponse {
            response,
            open,
            selected,
            created,
        }
    }
}
