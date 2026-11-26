use eframe::egui::{Popup, Response, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Size, ToggleButtonGroup, Variant},
    color::ColorSwatchPicker,
    form::Field,
    icon::Icon,
    layout::{CollapseMode, Sidebar, SidebarVariant},
    theme::{Accent, Mode},
    typography::Type,
};

use crate::widgets;

#[derive(Default)]
pub(super) struct Appearance {
    open: bool,
}

impl Appearance {
    pub(super) fn show(
        &mut self,
        anchor: &Response,
        theme: &mut Theme,
        sidebar: &mut Sidebar,
        inspector: &mut bool,
    ) {
        let was_open = self.open;
        let child_was_open = Popup::is_any_open(&anchor.ctx);
        let popup = widgets::popover(anchor, "Appearance", &mut self.open, 264., |ui| {
            ui.label(Type::H3.text("Appearance"));
            ui.add_space(6.);
            controls(ui, theme, sidebar);
            ui.add_space(8.);
            if Button::new(if *inspector {
                "Hide inspector"
            } else {
                "Open inspector"
            })
            .leading(Icon::Sidebar)
            .variant(Variant::Secondary)
            .show(ui)
            .clicked()
            {
                *inspector = !*inspector;
                return true;
            }
            false
        });
        if let Some(popup) = popup
            && (popup.inner
                || (!anchor.clicked()
                    && !child_was_open
                    && !Popup::is_any_open(&anchor.ctx)
                    && popup.response.clicked_elsewhere()))
        {
            self.open = false;
        }
        if was_open && !self.open {
            anchor.ctx.request_repaint();
        }
    }
}

pub(super) fn controls(ui: &mut Ui, theme: &mut Theme, sidebar: &mut Sidebar) {
    ui.spacing_mut().item_spacing.y = 16.;
    let mut mode = usize::from(theme.mode == Mode::Dark);
    Field::new("Color mode").show(ui, |ui, _| {
        ToggleButtonGroup::new(
            "appearance-mode",
            [
                Button::new("Light").leading(Icon::Sun).size(Size::Sm),
                Button::new("Dark").leading(Icon::Moon).size(Size::Sm),
            ],
        )
        .show(ui, &mut mode);
    });
    Field::new("Sidebar style").show(ui, |ui, _| {
        let mut style = Some(
            match sidebar.variant {
                SidebarVariant::Standard => "Standard",
                SidebarVariant::Inset => "Inset",
                SidebarVariant::Floating => "Floating",
            }
            .into(),
        );
        if widgets::select(
            ui,
            &mut style,
            &["Standard", "Inset", "Floating"],
            "Sidebar style",
        )
        .changed()
        {
            sidebar.variant = match style.as_deref() {
                Some("Inset") => SidebarVariant::Inset,
                Some("Floating") => SidebarVariant::Floating,
                _ => SidebarVariant::Standard,
            };
        }
    });
    Field::new("Sidebar layout").show(ui, |ui, _| {
        let mut layout = Some(
            match sidebar.mode {
                CollapseMode::Expanded => "Expanded",
                CollapseMode::IconRail => "Icon rail",
                CollapseMode::Offcanvas => "Hidden",
            }
            .into(),
        );
        if widgets::select(
            ui,
            &mut layout,
            &["Expanded", "Icon rail", "Hidden"],
            "Sidebar layout",
        )
        .changed()
        {
            sidebar.mode = match layout.as_deref() {
                Some("Icon rail") => CollapseMode::IconRail,
                Some("Hidden") => CollapseMode::Offcanvas,
                _ => CollapseMode::Expanded,
            };
        }
    });
    let mut color = theme.accent.color();
    let mut accent = theme.accent;
    Field::new("Accent color").show(ui, |ui, _| {
        if (ColorSwatchPicker {
            circular: true,
            disabled: false,
        })
        .show(ui, &mut color, &Accent::PRESETS.map(Accent::color))
        .changed()
            && let Some(preset) = Accent::PRESETS
                .into_iter()
                .find(|preset| preset.color() == color)
        {
            accent = preset;
        }
    });
    *theme = Theme::new(if mode == 0 { Mode::Light } else { Mode::Dark }, accent);
}

#[cfg(test)]
mod tests {
    use eframe::egui::{Context, Event, Modifiers, PointerButton, RawInput, Rect, pos2, vec2};

    use super::{Accent, Appearance, Mode, Sidebar, Theme};

    #[test]
    fn clicking_outside_appearance_repaints_the_dismissed_popup() {
        let ctx = Context::default();
        let mut theme = Theme::new(Mode::Dark, Accent::Blue);
        theme.install(&ctx);
        let mut sidebar = Sidebar::default();
        let mut appearance = Appearance { open: true };
        let mut draw = |events| {
            let mut frame = ctx.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(800., 620.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let anchor = ui.button("Appearance");
                    appearance.show(&anchor, &mut theme, &mut sidebar, &mut false);
                },
            );
            frame.textures_delta.clear();
            appearance.open
        };
        assert!(draw(vec![]));
        let outside = pos2(690., 540.);
        assert!(draw(vec![Event::PointerMoved(outside)]));
        let click = |pressed| Event::PointerButton {
            pos: outside,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        assert!(draw(vec![click(true)]));
        assert!(!draw(vec![click(false)]));
        draw(vec![]);
        assert!(
            ctx.repaint_causes()
                .iter()
                .any(|cause| cause.file.ends_with("/appearance.rs"))
        );
    }
}
