use egui::{Color32, InnerResponse, Response, Role, Stroke, StrokeKind, Ui, WidgetInfo, vec2};

use crate::{
    Theme,
    form::Field,
    icon::Icon,
    motion,
    theme::{Radius, mix},
};

use super::{Direction, Size, Tone, choice::Choice};

/// Three-state selection for aggregate or partial checks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckState {
    /// No selection.
    #[default]
    Unchecked,
    /// Selected.
    Checked,
    /// Part of an aggregate selection is checked.
    Indeterminate,
}

/// A value that can be shown in a checkbox.
pub trait CheckboxValue {
    /// Get the displayed selection.
    fn check_state(&self) -> CheckState;
    /// Set the displayed selection.
    fn set_check_state(&mut self, state: CheckState);
}

impl CheckboxValue for bool {
    fn check_state(&self) -> CheckState {
        if *self {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        }
    }

    fn set_check_state(&mut self, state: CheckState) {
        *self = state == CheckState::Checked;
    }
}

impl CheckboxValue for CheckState {
    fn check_state(&self) -> CheckState {
        *self
    }

    fn set_check_state(&mut self, state: CheckState) {
        *self = state;
    }
}

/// A checkbox accepting either a boolean or a three-state value.
#[derive(Clone, Copy, Debug, Default)]
pub struct Checkbox<'a> {
    /// Text next to the box.
    pub label: &'a str,
    /// Optional detail below the label.
    pub description: Option<&'a str>,
    /// Draw error styling.
    pub invalid: bool,
    /// Disable interaction.
    pub disabled: bool,
}

impl<'a> Checkbox<'a> {
    /// Create a labeled checkbox.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            ..Self::default()
        }
    }

    /// Render the checkbox. Activation selects a mixed value and toggles other values.
    pub fn show(self, ui: &mut Ui, value: &mut impl CheckboxValue) -> Response {
        ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            let mut choice = Choice::new(ui, self.label, self.description, vec2(18., 18.), false);
            let mut state = value.check_state();
            if choice.response.clicked() {
                if !choice.response.has_focus() {
                    choice.response.request_focus();
                }
                state = if state == CheckState::Checked {
                    CheckState::Unchecked
                } else {
                    CheckState::Checked
                };
                value.set_check_state(state);
                choice.response.mark_changed();
            }
            let active = state != CheckState::Unchecked;
            let c = Theme::current(ui.ctx()).colors;
            let palette = if self.invalid { c.danger } else { c.accent };
            let progress = motion::eased(
                ui.ctx(),
                choice.response.id.with("check"),
                active,
                motion::FAST,
            );
            choice.paint(ui, active, false);
            let border = if self.invalid || active || choice.response.hovered() {
                palette.ring
            } else {
                c.border
            };
            ui.painter().rect(
                choice.indicator,
                4,
                mix(c.surface, palette.base, progress),
                Stroke::new(1., border),
                StrokeKind::Inside,
            );
            if active {
                let mark = if state == CheckState::Indeterminate {
                    Icon::Minus
                } else {
                    Icon::Check
                };
                let _ = mark.svg().paint(
                    ui,
                    choice.indicator.shrink(3.),
                    palette.foreground.gamma_multiply(progress),
                );
            }
            let r = choice.response;
            r.widget_info(|| {
                if state == CheckState::Indeterminate {
                    WidgetInfo::labeled(Role::CheckBox, r.enabled(), self.label)
                } else {
                    WidgetInfo::selected(Role::CheckBox, r.enabled(), active, self.label)
                }
            });
            ui.ctx().accesskit_node_builder(r.id, |node| {
                if self.invalid {
                    node.set_invalid(egui::accesskit::Invalid::True);
                }
                if let Some(description) = self.description {
                    node.set_description(description);
                }
            });
            r
        })
        .inner
    }
}

/// Labeled checkbox collection with shared helper or error feedback.
#[derive(Clone, Copy, Debug)]
pub struct CheckboxGroup<'a> {
    /// Group label and form feedback.
    pub field: Field<'a>,
    /// Layout direction.
    pub direction: Direction,
}

impl<'a> CheckboxGroup<'a> {
    /// Create a vertically stacked group.
    pub fn new(label: &'a str) -> Self {
        Self {
            field: Field::new(label),
            direction: Direction::Vertical,
        }
    }

    /// Render the group and supply its shared invalid state to its items.
    pub fn show<R>(self, ui: &mut Ui, items: impl FnOnce(&mut Ui, bool) -> R) -> InnerResponse<R> {
        self.field.show(ui, |ui, invalid| {
            self.direction.show(ui, |ui| items(ui, invalid)).inner
        })
    }
}

/// An animated on/off switch.
#[derive(Clone, Copy, Debug, Default)]
pub struct Switch<'a> {
    /// Text next to the track.
    pub label: &'a str,
    /// Optional detail below the label.
    pub description: Option<&'a str>,
    /// Track and thumb size.
    pub size: Size,
    /// Active state color.
    pub tone: Tone,
    /// Disable interaction.
    pub disabled: bool,
}

impl<'a> Switch<'a> {
    /// Create a switch with the theme accent.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            ..Self::default()
        }
    }

    /// Render an animated switch and update its value when activated.
    pub fn show(self, ui: &mut Ui, value: &mut bool) -> Response {
        ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            let size = match self.size {
                Size::Sm => vec2(30., 16.),
                Size::Md => vec2(38., 20.),
                Size::Lg => vec2(46., 24.),
            };
            let mut choice = Choice::new(ui, self.label, self.description, size, false);
            if choice.response.clicked() {
                if !choice.response.has_focus() {
                    choice.response.request_focus();
                }
                *value = !*value;
                choice.response.mark_changed();
            }
            choice.paint(ui, *value, false);
            let c = Theme::current(ui.ctx()).colors;
            let p = self.tone.palette(c);
            let track = choice.indicator;
            let t = motion::eased(
                ui.ctx(),
                choice.response.id.with("slide"),
                *value,
                motion::NORMAL,
            );
            let fill = mix(
                c.border,
                if choice.response.hovered() {
                    p.hover
                } else {
                    p.base
                },
                t,
            );
            ui.painter()
                .rect_filled(track, Radius::Full.for_size(size), fill);
            let center = egui::lerp(
                (track.left() + size.y / 2.)..=(track.right() - size.y / 2.),
                t,
            );
            let active_thumb = if self.tone == Tone::Neutral {
                p.foreground
            } else {
                Color32::WHITE
            };
            ui.painter().circle_filled(
                egui::pos2(center, track.center().y),
                size.y / 2. - 2.,
                mix(c.surface, active_thumb, t),
            );
            let r = choice.response;
            r.widget_info(|| WidgetInfo::selected(Role::Switch, r.enabled(), *value, self.label));
            if let Some(description) = self.description {
                ui.ctx()
                    .accesskit_node_builder(r.id, |node| node.set_description(description));
            }
            r
        })
        .inner
    }
}
