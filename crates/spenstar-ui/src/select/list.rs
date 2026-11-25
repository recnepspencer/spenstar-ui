use egui::{
    Align2, Key, Modifiers, Rect, Response, Role, ScrollArea, Sense, Ui, WidgetInfo, pos2, vec2,
};

use crate::{Theme, icon::Icon, motion, popover, typography::Type};

use super::{SelectOption, SelectSection, empty_state};

#[derive(Clone, Copy)]
pub(super) enum Row<'a> {
    Option(&'a SelectOption<'a>),
    Create(&'a str),
    Clear,
}

impl Row<'_> {
    fn enabled(self) -> bool {
        !matches!(self, Self::Option(option) if option.disabled)
    }

    fn selected(self, is_selected: &impl Fn(&str) -> bool) -> bool {
        matches!(self, Self::Option(option) if is_selected(option.value))
    }
}

pub(super) struct Menu<'a> {
    sections: &'a [SelectSection<'a>],
    pub query: &'a str,
    pub create: Option<&'a str>,
    pub caption: Option<&'a str>,
    pub clear: bool,
    pub reset: bool,
}

impl<'a> Menu<'a> {
    pub fn new(sections: &'a [SelectSection<'a>]) -> Self {
        Self {
            sections,
            query: "",
            create: None,
            caption: None,
            clear: false,
            reset: false,
        }
    }

    pub fn show(self, anchor: &Response, is_selected: impl Fn(&str) -> bool) -> Option<Row<'a>> {
        self.show_with_submit(anchor, false, is_selected)
    }

    pub fn show_with_submit(
        self,
        anchor: &Response,
        submitted: bool,
        is_selected: impl Fn(&str) -> bool,
    ) -> Option<Row<'a>> {
        if !popover::is_open(anchor) {
            return None;
        }
        let id = popover::id(anchor).with("row");
        let query = self.query.to_lowercase();
        let mut rows = Vec::new();
        let mut headings = Vec::new();
        for section in self.sections {
            let start = rows.len();
            rows.extend(
                section
                    .options
                    .iter()
                    .filter(|item| query.is_empty() || item.label.to_lowercase().contains(&query))
                    .map(Row::Option),
            );
            if rows.len() > start
                && let Some(label) = section.label
            {
                headings.push((start, label));
            }
        }
        let no_matches = rows.is_empty();
        if let Some(label) = self.create {
            rows.push(Row::Create(label));
        }
        if self.clear {
            rows.push(Row::Clear);
        }
        let c = Theme::current(&anchor.ctx).colors;
        let output = popover::show(
            anchor,
            (anchor.rect.width() - 14.).max(168.),
            "Options",
            |ui| {
                let initial = rows
                    .iter()
                    .position(|row| row.enabled() && row.selected(&is_selected))
                    .or_else(|| rows.iter().position(|row| row.enabled()));
                let stored = (!self.reset && ui.ctx().read_response(popover::id(anchor)).is_some())
                    .then(|| ui.ctx().data(|d| d.get_temp::<usize>(id)))
                    .flatten();
                let mut active = stored
                    .filter(|&i| rows.get(i).is_some_and(|row| row.enabled()))
                    .or(initial);
                let (steps, submit) = ui.input_mut(|i| {
                    let steps = i.count_and_consume_key(Modifiers::NONE, Key::ArrowDown) as isize
                        - i.count_and_consume_key(Modifiers::NONE, Key::ArrowUp) as isize;
                    (
                        steps,
                        submitted || i.consume_key(Modifiers::NONE, Key::Enter),
                    )
                });
                if steps != 0
                    && let Some(index) = active
                {
                    ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
                    let enabled = rows
                        .iter()
                        .enumerate()
                        .filter_map(|(i, row)| row.enabled().then_some(i))
                        .collect::<Vec<_>>();
                    if let Some(index) = enabled.iter().position(|&i| i == index) {
                        active = Some(
                            enabled[(index as isize + steps).rem_euclid(enabled.len() as isize)
                                as usize],
                        );
                    }
                }
                let mut action = submit
                    .then(|| active.and_then(|i| rows.get(i).copied()))
                    .flatten();
                if let Some(caption) = self.caption {
                    ui.label(Type::Caption.text(caption).color(c.text_secondary));
                }
                let height = (anchor.ctx.content_rect().height() * 0.55).clamp(56., 244.);
                ScrollArea::vertical()
                    .max_height(height)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        if no_matches && self.create.is_none() {
                            empty_state(ui);
                        }
                        let mut headings = headings.into_iter().peekable();
                        for (index, row) in rows.iter().copied().enumerate() {
                            if headings.peek().is_some_and(|&(start, _)| start == index)
                                && let Some((_, label)) = headings.next()
                            {
                                ui.label(Type::Caption.text(label).color(c.text_secondary));
                            }
                            let highlighted = active == Some(index);
                            let response =
                                paint_row(ui, row, highlighted, row.selected(&is_selected));
                            if response.clicked() {
                                action = Some(row);
                                active = Some(index);
                            }
                            if highlighted && (steps != 0 || stored.is_none()) {
                                response.scroll_to_me(None);
                            }
                        }
                    });
                if let Some(index) = active {
                    ui.ctx().data_mut(|d| d.insert_temp(id, index));
                } else {
                    ui.ctx().data_mut(|d| d.remove::<usize>(id));
                }
                action
            },
        );
        if !popover::is_open(anchor) {
            anchor.ctx.data_mut(|d| d.remove::<usize>(id));
        }
        output.and_then(|out| out.inner)
    }
}

fn paint_row(ui: &mut Ui, row: Row<'_>, active: bool, selected: bool) -> Response {
    ui.scope(|ui| {
        if !row.enabled() {
            ui.disable();
        }
        let c = Theme::current(ui.ctx()).colors;
        let (label, icon) = match row {
            Row::Option(option) => (option.label.to_owned(), option.icon.clone()),
            Row::Create(value) => (format!("Create “{value}”"), Some(Icon::Plus.into())),
            Row::Clear => ("Clear all".into(), Some(Icon::X.into())),
        };
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 30.), Sense::click());
        if response.hovered() || (active && response.enabled()) {
            ui.painter().rect_filled(
                rect,
                5,
                if response.hovered() {
                    c.accent.soft_hover
                } else {
                    c.accent.soft
                },
            );
        }
        let mut left = rect.left() + 8.;
        if let Some(icon) = icon {
            let icon_rect =
                Rect::from_center_size(pos2(left + 8., rect.center().y), vec2(16., 16.));
            let _ = icon.paint(ui, icon_rect, c.text_secondary);
            left += 24.;
        }
        let clip = Rect::from_min_max(
            pos2(left, rect.top()),
            pos2((rect.right() - 26.).max(left), rect.bottom()),
        );
        ui.painter().with_clip_rect(clip).text(
            pos2(left, rect.center().y),
            Align2::LEFT_CENTER,
            &label,
            Type::Small.font(),
            c.text_primary,
        );
        if selected {
            let at = pos2(rect.right() - 13., rect.center().y);
            let _ = Icon::Check.svg().paint(
                ui,
                Rect::from_center_size(at, vec2(15., 15.)),
                c.accent.ring,
            );
        }
        response.widget_info(|| {
            WidgetInfo::selected(Role::ListBoxOption, response.enabled(), selected, &label)
        });
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_selected(selected));
        crate::input::lock_arrows(&response, false, true);
        motion::focus_ring(ui, &response, 5.into());
        response
    })
    .inner
}
