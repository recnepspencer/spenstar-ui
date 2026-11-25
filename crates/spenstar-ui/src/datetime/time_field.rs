use crate::{
    Theme,
    button::{Button, Size, Variant},
    form::{TextField, step_keys},
    typography::Type,
};
use egui::{InnerResponse, Key, Response, Ui};

use super::{ClockFormat, ClockTime, civil::parse_component};

/// An editor for hour and minute with optional seconds.
#[derive(Clone, Copy, Debug, Default)]
pub struct TimeField {
    /// 12-hour or 24-hour presentation.
    pub format: ClockFormat,
    /// Include seconds.
    pub seconds: bool,
    /// External validation error styling.
    pub invalid: bool,
    /// Disable all time entry.
    pub disabled: bool,
}

/// Result of editing a time.
pub struct TimeResponse {
    /// Response for the entire editor.
    pub response: Response,
    /// One or more drafts is invalid or the caller marked the field invalid.
    pub invalid: bool,
}

impl TimeField {
    /// Render the time components. Enter or blur commits a valid component.
    pub fn show(self, ui: &mut Ui, value: &mut ClockTime) -> TimeResponse {
        let previous = *value;
        let mut out = ui.horizontal_wrapped(|ui| {
            if self.disabled {
                ui.disable();
            }
            let color = Theme::current(ui.ctx()).colors.text_secondary;
            let mut invalid = self.invalid;
            let mut visible = [value.hour, value.minute, value.second];
            let h12 = self.format == ClockFormat::H12;
            if h12 {
                visible[0] = value.hour12();
            }
            for (index, (name, unit)) in [("Hour", 3600), ("Minute", 60), ("Second", 1)]
                .into_iter()
                .take(if self.seconds { 3 } else { 2 })
                .enumerate()
            {
                if index > 0 {
                    ui.label(Type::Small.text(":").color(color));
                }
                let current = visible[index];
                let range = match (index, h12) {
                    (0, true) => 1..=12,
                    (0, false) => 0..=23,
                    _ => 0..=59,
                };
                let result = component(ui, name, current, range, self.invalid);
                invalid |= result.inner.invalid;
                if let Some(next) = result.inner.commit {
                    match index {
                        0 if h12 => value.hour = next % 12 + if value.hour >= 12 { 12 } else { 0 },
                        0 => value.hour = next,
                        1 => value.minute = next,
                        _ => value.second = next,
                    }
                }
                if result.inner.steps != 0 {
                    *value = value.offset(result.inner.steps * unit);
                    ui.ctx().data_mut(|d| d.remove::<Draft>(result.inner.id));
                }
            }
            if h12 {
                let label = if value.hour < 12 { "AM" } else { "PM" };
                if Button::new(label)
                    .size(Size::Sm)
                    .variant(Variant::Outline)
                    .tooltip("Toggle AM / PM")
                    .show(ui)
                    .clicked()
                {
                    *value = value.offset(43_200);
                }
            }
            invalid
        });
        if *value != previous {
            out.response.mark_changed();
        }
        TimeResponse {
            response: out.response,
            invalid: out.inner,
        }
    }
}

#[derive(Clone)]
struct Draft {
    value: u8,
    text: String,
}

struct Component {
    id: egui::Id,
    commit: Option<u8>,
    steps: i64,
    invalid: bool,
}

fn component(
    ui: &mut Ui,
    name: &str,
    current: u8,
    valid: std::ops::RangeInclusive<u8>,
    invalid: bool,
) -> InnerResponse<Component> {
    ui.allocate_ui_with_layout(
        egui::vec2(42., 28.),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_width(42.);
            let id = ui.unique_id().with("component");
            let mut draft = ui
                .ctx()
                .data(|d| d.get_temp::<Draft>(id))
                .filter(|d| d.value == current)
                .unwrap_or_else(|| Draft {
                    value: current,
                    text: format!("{current:02}"),
                });
            let parse = |text: &str| parse_component(text).filter(|n| valid.contains(n));
            let (input, ()) = TextField {
                size: Size::Sm,
                invalid: invalid || parse(&draft.text).is_none(),
                ..TextField::default()
            }
            .render(ui, &mut draft.text, None, 0., |_| {});
            ui.ctx()
                .accesskit_node_builder(input.response.id, |node| node.set_label(name));
            let steps = step_keys(&input.response, Key::ArrowUp, Key::ArrowDown) as i64;
            let parsed = parse(&draft.text);
            let commit = (input.response.enabled()
                && (input.submitted || input.response.lost_focus() || steps != 0))
                .then_some(parsed)
                .flatten();
            if let Some(value) = commit {
                draft.value = value;
                draft.text = format!("{value:02}");
            }
            ui.ctx().data_mut(|d| d.insert_temp(id, draft));
            Component {
                id,
                commit,
                steps,
                invalid: parsed.is_none() && steps == 0,
            }
        },
    )
}
