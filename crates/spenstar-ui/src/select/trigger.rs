use egui::{Key, Modifiers, Rect, Response, Role, Sense, Stroke, Ui, UiBuilder, WidgetInfo, vec2};

use crate::{
    Theme,
    form::{EntryStyle, Size, icon_button},
    icon::Icon,
    motion, popover,
};

use super::SelectStyle;

pub(crate) struct Trigger<'a> {
    style: &'a SelectStyle<'a>,
    name: &'a str,
    placeholder: bool,
    clear: bool,
    indicator: Icon,
}

impl<'a> Trigger<'a> {
    pub fn new(style: &'a SelectStyle<'a>, name: &'a str, placeholder: bool) -> Self {
        Self {
            style,
            name,
            placeholder,
            clear: false,
            indicator: Icon::ChevronDown,
        }
    }

    pub fn clearable(mut self, clear: bool) -> Self {
        self.clear = clear;
        self
    }

    pub fn indicator(mut self, icon: Icon) -> Self {
        self.indicator = icon;
        self
    }

    pub fn show(self, ui: &mut Ui) -> (Response, bool) {
        let name = self.name;
        let placeholder = self.placeholder;
        let size = self.style.size;
        let (response, (_, clear)) = self.draw(ui, |ui| label(ui, name, placeholder, size));
        (response, clear)
    }

    pub fn show_custom<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
        let (response, (content, _)) = self.draw(ui, content);
        (response, content)
    }

    fn draw<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> (Response, (R, bool)) {
        let style = EntryStyle {
            variant: self.style.variant,
            size: self.style.size,
            invalid: self.style.invalid,
        };
        let mut builder = UiBuilder::new().sense(Sense::click());
        if self.style.disabled {
            builder = builder.disabled();
        }
        let out = ui.scope_builder(builder, |ui| {
            let c = Theme::current(ui.ctx()).colors;
            let (shell, background) = style.show(ui, |ui| {
                if let Some(icon) = &self.style.leading {
                    icon.show(ui, 16., c.text_secondary);
                }
                let spacing = ui.spacing().item_spacing.x;
                let reserve = 16. + spacing + if self.clear { 22. + spacing } else { 0. };
                let width = (ui.available_width() - reserve).max(0.);
                let value = ui
                    .allocate_ui_with_layout(
                        vec2(width, style.height()),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_width(width);
                            ui.shrink_clip_rect(ui.max_rect());
                            content(ui)
                        },
                    )
                    .inner;
                let cleared = self.clear && icon_button(ui, Icon::X, "Clear selection").clicked();
                let (arrow, _) = ui.allocate_exact_size(vec2(16., 16.), Sense::hover());
                (value, cleared, arrow)
            });
            let mut response = ui.response();
            response.rect = shell.response.rect;
            response.interact_rect = response.rect.intersect(ui.clip_rect());
            let (value, cleared, arrow) = shell.inner;
            if cleared {
                popover::close(&response, true);
            } else if response.clicked()
                && (!popover::is_open(&response) || !ui.input(|i| i.key_pressed(Key::Enter)))
            {
                if !response.has_focus() {
                    response.request_focus();
                }
                if !popover::is_open(&response) {
                    ui.input_mut(|input| {
                        input.consume_key(Modifiers::NONE, Key::Enter);
                    });
                }
                popover::toggle(&response);
            } else if response.enabled()
                && response.has_focus()
                && !popover::is_open(&response)
                && ui.input(|i| i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::ArrowUp))
            {
                ui.input_mut(|i| {
                    i.consume_key(Modifiers::NONE, Key::ArrowDown);
                    i.consume_key(Modifiers::NONE, Key::ArrowUp);
                });
                popover::open(&response);
            }
            crate::input::lock_arrows(&response, false, true);
            let open = popover::is_open(&response);
            response
                .widget_info(|| WidgetInfo::labeled(Role::ComboBox, response.enabled(), self.name));
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_expanded(open));
            style.paint(ui, background, shell.response.rect, &response);
            if self.indicator == Icon::ChevronDown {
                chevron(ui, arrow, &response, open);
            } else {
                let _ = self.indicator.svg().paint(ui, arrow, c.text_secondary);
            }
            (response, (value, cleared))
        });
        out.inner
    }
}

pub(super) fn label(ui: &mut Ui, label: &str, placeholder: bool, size: Size) {
    let c = Theme::current(ui.ctx()).colors;
    let tint = if placeholder {
        c.text_muted
    } else {
        c.text_primary
    };
    ui.add(
        egui::Label::new(
            egui::RichText::new(label)
                .font(
                    EntryStyle {
                        size,
                        ..Default::default()
                    }
                    .font(),
                )
                .color(tint),
        )
        .truncate(),
    );
}

fn chevron(ui: &Ui, rect: Rect, response: &Response, open: bool) {
    let color = Theme::current(ui.ctx()).colors.text_secondary;
    let t = motion::eased(ui.ctx(), response.id.with("chevron"), open, motion::FAST);
    let rotation = egui::emath::Rot2::from_angle(std::f32::consts::PI * t);
    let points = [vec2(-4., -2.), vec2(0., 2.), vec2(4., -2.)];
    ui.painter().add(egui::Shape::line(
        points.map(|p| rect.center() + rotation * p).to_vec(),
        Stroke::new(1.6, color),
    ));
}
