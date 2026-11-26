use egui::{Key, Modifiers, Response, Role, Ui, WidgetInfo, vec2};

use crate::{Theme, icon::IconSource, motion};

use super::{Direction, choice::Choice};

/// Visual layout for radio choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioVariant {
    /// Circular indicator next to the label.
    #[default]
    Standard,
    /// Bordered card with optional leading icon.
    Card,
}

/// One item in a radio group.
#[derive(Clone, Debug)]
pub struct RadioItem<'a> {
    /// Visible title.
    pub label: &'a str,
    /// Secondary copy.
    pub description: Option<&'a str>,
    /// Leading icon in the card variant.
    pub icon: Option<IconSource>,
    /// Skip this item for activation and arrow navigation.
    pub disabled: bool,
}

impl<'a> RadioItem<'a> {
    /// Create an enabled choice.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            description: None,
            icon: None,
            disabled: false,
        }
    }
}

/// A mutually exclusive set of choices selected by index.
#[derive(Clone, Copy, Debug, Default)]
pub struct RadioGroup<'a> {
    /// Label above the choice list.
    pub label: &'a str,
    /// Presentation of each choice.
    pub variant: RadioVariant,
    /// Layout direction.
    pub direction: Direction,
    /// Disable all choices.
    pub disabled: bool,
}

impl RadioGroup<'_> {
    /// Render the choices and update the selected item index.
    pub fn show(
        self,
        ui: &mut Ui,
        selected: &mut Option<usize>,
        items: &[RadioItem<'_>],
    ) -> Response {
        let mut out = ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            if !self.label.is_empty() {
                ui.label(crate::typography::Type::Label.text(self.label));
            }
            let direction = if self.variant == RadioVariant::Card {
                Direction::Vertical
            } else {
                self.direction
            };
            direction
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.;
                    let previous = *selected;
                    let responses: Vec<_> = items
                        .iter()
                        .enumerate()
                        .map(|(index, item)| self.item(ui, item, previous == Some(index)))
                        .collect();
                    if let Some(current) = responses.iter().position(|r| r.clicked()) {
                        *selected = Some(current);
                        if !responses[current].has_focus() {
                            responses[current].request_focus();
                        }
                    } else if let Some(current) =
                        responses.iter().position(|r| r.enabled() && r.has_focus())
                    {
                        navigate(ui, current, &responses, selected);
                    }
                    previous != *selected
                })
                .inner
        });
        if out.inner {
            out.response.mark_changed();
        }
        out.response.widget_info(|| {
            WidgetInfo::labeled(Role::RadioGroup, out.response.enabled(), self.label)
        });
        out.response
    }

    fn item(self, ui: &mut Ui, item: &RadioItem<'_>, selected: bool) -> Response {
        ui.scope(|ui| {
            if item.disabled {
                ui.disable();
            }
            let card = self.variant == RadioVariant::Card;
            let choice = Choice::new(ui, item.label, item.description, vec2(18., 18.), card);
            choice.paint(ui, selected, card);
            let c = Theme::current(ui.ctx()).colors;
            if let Some(icon) = item.icon.as_ref().filter(|_| card) {
                let _ = icon.paint(
                    ui,
                    choice.indicator,
                    if selected {
                        c.accent.ring
                    } else {
                        c.text_secondary
                    },
                );
            } else {
                let center = choice.indicator.center();
                let tint = if selected {
                    c.accent.base
                } else {
                    c.radio_fill
                };
                ui.painter().circle_filled(center, 9., tint);
                let progress = motion::eased(
                    ui.ctx(),
                    choice.response.id.with("dot"),
                    selected,
                    motion::FAST,
                );
                if selected && progress > 0. {
                    ui.painter()
                        .circle_filled(center, 3.5 * progress, egui::Color32::WHITE);
                }
            }
            let r = choice.response;
            crate::input::lock_arrows(&r, true, true);
            r.widget_info(|| {
                WidgetInfo::selected(Role::RadioButton, r.enabled(), selected, item.label)
            });
            ui.ctx().accesskit_node_builder(r.id, |node| {
                node.set_selected(selected);
                if let Some(description) = item.description {
                    node.set_description(description);
                }
            });
            r
        })
        .inner
    }
}

fn navigate(ui: &Ui, current: usize, responses: &[Response], selected: &mut Option<usize>) {
    let direction = ui.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::ArrowRight)
            || input.consume_key(Modifiers::NONE, Key::ArrowDown)
        {
            1
        } else if input.consume_key(Modifiers::NONE, Key::ArrowLeft)
            || input.consume_key(Modifiers::NONE, Key::ArrowUp)
        {
            -1
        } else {
            0
        }
    });
    if direction == 0 {
        return;
    }
    let count = responses.len();
    let next = (1..=count)
        .map(|offset| {
            if direction > 0 {
                (current + offset) % count
            } else {
                (current + count - offset) % count
            }
        })
        .find(|&index| responses[index].enabled());
    if let Some(index) = next {
        *selected = Some(index);
        responses[index].request_focus();
        ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
    }
}
