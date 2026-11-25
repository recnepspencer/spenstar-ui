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
