use crate::{Theme, icon::Icon, typography::Type};
use egui::{Key, Response, Ui};

use super::{TextField, icon_button, step_keys};

/// Output from a numeric input.
pub struct NumberResponse {
    /// The numeric text editor. Changed indicates a draft or numeric value change.
    pub response: Response,
    /// Current draft cannot be parsed as a finite number.
    pub invalid: bool,
    /// A step or edit was committed.
    pub committed: bool,
}

/// An editable number with an independent draft and integrated stepper.
#[derive(Clone, Copy, Debug)]
pub struct NumberField<'a> {
    /// Positive amount for each step.
    pub step: f64,
    /// Decimal places used for display and commit (up to 15).
    pub precision: usize,
    /// Minimum accepted value.
    pub min: f64,
    /// Maximum accepted value.
    pub max: f64,
    /// Label before the value, such as a currency sign.
    pub prefix: &'a str,
    /// Label after the value, such as a unit.
    pub suffix: &'a str,
    /// Disable interaction.
    pub disabled: bool,
}

impl Default for NumberField<'_> {
    fn default() -> Self {
        Self {
            step: 1.,
            precision: 0,
            min: -f64::MAX,
            max: f64::MAX,
            prefix: "",
            suffix: "",
            disabled: false,
        }
    }
}

#[derive(Clone)]
struct Draft {
    text: String,
    editing: bool,
    value_bits: u64,
}

impl NumberField<'_> {
    /// Render the input. Enter, focus loss, and step operations commit the draft.
    pub fn show(self, ui: &mut Ui, value: &mut f64) -> NumberResponse {
        ui.scope(|ui| {
            let id = ui.unique_id().with("number");
            let precision = self.precision.min(15);
            let format = |value: f64| format!("{value:.precision$}");
            let mut draft = ui
                .ctx()
                .data(|d| d.get_temp::<Draft>(id))
                .unwrap_or_else(|| Draft {
                    text: format(*value),
                    editing: false,
                    value_bits: value.to_bits(),
                });
            if !draft.editing || draft.value_bits != value.to_bits() {
                draft.text = format(*value);
                draft.value_bits = value.to_bits();
            }
            let color = Theme::current(ui.ctx()).colors.text_secondary;
            let spacing = ui.spacing().item_spacing.x;
            let suffix =
                ui.painter()
                    .layout_no_wrap(self.suffix.to_owned(), Type::Small.font(), color);
            let reserve = 2. * (22. + spacing)
                + if self.suffix.is_empty() {
                    0.
                } else {
                    suffix.size().x + spacing
                };
            // TextEdit's prefix is part of its own layout. The unit and buttons stay outside.
            let (mut input, button_steps) = TextField {
                disabled: self.disabled,
                invalid: parse(&draft.text).is_none(),
                ..TextField::default()
            }
            .render_affixed(ui, &mut draft.text, self.prefix, reserve, |ui| {
                if !self.suffix.is_empty() {
                    ui.label(Type::Small.text(self.suffix).color(color));
                }
                let down = icon_button(ui, Icon::Minus, "Decrease").clicked();
                let up = icon_button(ui, Icon::Plus, "Increase").clicked();
                f64::from(up) - f64::from(down)
            });
            let steps = button_steps + step_keys(&input.response, Key::ArrowUp, Key::ArrowDown);
            let commit = input.response.enabled()
                && (input.submitted || input.response.lost_focus() || steps != 0.);
            let parsed = parse(&draft.text);
            let old = *value;
            if commit {
                let a = if self.min.is_nan() {
                    -f64::MAX
                } else {
                    self.min.clamp(-f64::MAX, f64::MAX)
                };
                let b = if self.max.is_nan() {
                    f64::MAX
                } else {
                    self.max.clamp(-f64::MAX, f64::MAX)
                };
                let (min, max) = (a.min(b), a.max(b));
                let step = if self.step.is_finite() && self.step > 0. {
                    self.step
                } else {
                    1.
                };
                let edited = parsed.filter(|_| draft.text != format(old));
                let base = edited
                    .or_else(|| old.is_finite().then_some(old))
                    .unwrap_or(0.);
                *value = if edited.is_some() || steps != 0. {
                    let next = (base + steps * step).clamp(min, max);
                    parse(&format(next)).unwrap_or(next).clamp(min, max)
                } else {
                    // An untouched display may round an externally supplied value.
                    base.clamp(min, max)
                };
                draft.value_bits = value.to_bits();
                let formatted = format(*value);
                if draft.text != formatted {
                    draft.text = formatted;
                    ui.ctx().request_repaint();
                }
            }
            draft.editing = input.response.has_focus();
            let invalid = !commit && draft.editing && parsed.is_none();
            if old.to_bits() != value.to_bits() {
                input.response.mark_changed();
            }
            ui.ctx().accesskit_node_builder(input.response.id, |node| {
                node.set_role(egui::Role::SpinButton);
                if value.is_finite() {
                    node.set_numeric_value(*value);
                }
            });
            ui.ctx().data_mut(|d| d.insert_temp(id, draft));
            NumberResponse {
                response: input.response,
                invalid,
                committed: commit,
            }
        })
        .inner
    }
}

fn parse(text: &str) -> Option<f64> {
    text.trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}
