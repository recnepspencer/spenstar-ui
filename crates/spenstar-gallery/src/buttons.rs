use eframe::egui::{Context, Id, Modal, Ui};
use spenstar_ui::{
    Theme,
    button::{
        Button, ButtonGroup, CloseButton, Link, Size, ToggleButton, ToggleButtonGroup, Variant,
    },
    icon::{Icon, SvgIcon},
    layout::{Card, CardVariant, Separator, Toolbar},
    typography::Type,
};

const UPLOAD: SvgIcon = SvgIcon::from_static(
    "gallery-upload",
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <path d="M4 17v3h16v-3M12 16V4m-5 5 5-5 5 5" stroke="currentColor"
            stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
        <circle cx="18" cy="15" r="2" fill="#f59e0b"/>
    </svg>"##,
);

#[derive(Default)]
pub(crate) struct Demo {
    recent: Option<&'static str>,
    loading: bool,
    starred: bool,
    period: usize,
    dialog_open: bool,
}

impl Demo {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let colors = Theme::current(ui.ctx()).colors;
        Card::new(CardVariant::Bordered).show(ui, |card| {
            card.header(
                "Variants",
                Some("Six treatments for primary, supporting, and destructive actions."),
            );
            card.body(|ui| {
                ui.horizontal_wrapped(|ui| {
                    for (name, variant) in [
                        ("Accent", Variant::Accent),
                        ("Secondary", Variant::Secondary),
                        ("Soft", Variant::Soft),
                        ("Outline", Variant::Outline),
                        ("Ghost", Variant::Ghost),
                        ("Danger", Variant::Danger),
                    ] {
                        if Button::new(name).variant(variant).show(ui).clicked() {
                            self.recent = Some(name);
                        }
                    }
                });
                ui.add_space(12.);
                Separator::Labeled("Sizes").show(ui);
                ui.horizontal_wrapped(|ui| {
                    for (name, size) in [
                        ("Small", Size::Sm),
                        ("Medium", Size::Md),
                        ("Large", Size::Lg),
                    ] {
                        if Button::new(name)
                            .size(size)
                            .variant(Variant::Outline)
                            .show(ui)
                            .clicked()
                        {
                            self.recent = Some(name);
                        }
                    }
                });
            });
        });

        Card::new(CardVariant::Bordered).show(ui, |card| {
            card.header(
                "Icons & states",
                Some("A custom SVG retains its amber detail when the button changes tint."),
            );
            card.body(|ui| {
                ui.horizontal_wrapped(|ui| {
                    for (label, button) in [
                        ("Upload", Button::new("Upload").leading(UPLOAD)),
                        (
                            "Read documentation",
                            Button::new("Read docs")
                                .trailing(Icon::ExternalLink)
                                .variant(Variant::Outline),
                        ),
                        (
                            "Next record",
                            Button::new("Next record")
                                .leading(Icon::FileText)
                                .trailing(Icon::ChevronRight)
                                .variant(Variant::Soft),
                        ),
                        (
                            "Starred",
                            Button::new("Star")
                                .icon_only(Icon::Star)
                                .variant(Variant::Ghost),
                        ),
                    ] {
                        if button.show(ui).clicked() {
                            self.recent = Some(label);
                        }
                    }
                    Button::new("Save")
                        .leading(Icon::Save)
                        .enabled(false)
                        .tooltip("Save becomes available when there are changes.")
                        .show(ui);
                });
                ui.add_space(8.);
                ui.horizontal_wrapped(|ui| {
                    if Button::new("Sync")
                        .leading(Icon::RefreshCw)
                        .loading(self.loading)
                        .show(ui)
                        .clicked()
                    {
                        self.loading = true;
                        self.recent = Some("Sync started");
                    }
                    if Button::new("Reset sync")
                        .variant(Variant::Outline)
                        .enabled(self.loading)
                        .show(ui)
                        .clicked()
                    {
                        self.loading = false;
                        self.recent = Some("Sync reset");
                    }
                });
            });
        });

        Card::new(CardVariant::Bordered).show(ui, |card| {
            card.header(
                "Grouped actions",
                Some("Joined actions, independent toggles, and a single-selection control."),
            );
            card.body(|ui| {
                ui.horizontal_wrapped(|ui| {
                    let names = ["Undo", "Redo", "Save"];
                    let responses = ButtonGroup::new(
                        names
                            .into_iter()
                            .zip([Icon::Undo2, Icon::Redo2, Icon::Save])
                            .map(|(label, icon)| {
                                Button::new(label).leading(icon).variant(Variant::Outline)
                            }),
                    )
                    .show(ui);
                    for (name, response) in names.into_iter().zip(responses) {
                        if response.clicked() {
                            self.recent = Some(name);
                        }
                    }
                    Separator::Vertical(26.).show(ui);
                    ToggleButton::new(Button::new("Star").leading(Icon::Star))
                        .show(ui, &mut self.starred);
                });
                ui.add_space(8.);
                ToggleButtonGroup::new(
                    "period",
                    ["Day", "Week", "Month"].map(|label| Button::new(label).size(Size::Sm)),
                )
                .show(ui, &mut self.period);
                ui.add_space(12.);
                ui.horizontal_wrapped(|ui| {
                    if Button::new("Open dialog")
                        .variant(Variant::Soft)
                        .show(ui)
                        .clicked()
                    {
                        self.dialog_open = true;
                    }
                    Link::new(
                        "View repository",
                        "https://github.com/recnepspencer/spenstar-ui",
                    )
                    .show(ui);
                });
            });
            card.footer(|ui| {
                let message = self
                    .recent
                    .map_or("Choose an action to see it here.".to_owned(), |name| {
                        format!("Last action: {name}")
                    });
                ui.label(Type::Small.text(message).color(colors.text_secondary));
            });
        });
        if self.dialog_open {
            self.dialog(ui.ctx());
        }
    }

    fn dialog(&mut self, ctx: &Context) {
        let colors = Theme::current(ctx).colors;
        let modal =
            Modal::new(Id::unique("spenstar-gallery/button-dialog")).backdrop_color(colors.scrim);
        let response = modal.show(ctx, |ui| {
            Toolbar::show(ui, |ui| {
                ui.label(Type::H3.text("Example dialog"));
                CloseButton::show(ui).clicked()
            })
            .inner
        });
        if response.inner || response.should_close() {
            self.dialog_open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui::{Pos2, RawInput, Rect, epaint::Shape, vec2};
    use spenstar_ui::theme::{Accent, Mode};

    use super::{Context, Demo, Theme};

    #[test]
    fn example_dialog_paints_the_translucent_scrim() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = Context::default();
            let theme = Theme::new(mode, Accent::Blue);
            theme.install(&ctx);
            ctx.all_styles_mut(|style| style.animation_time = 0.);
            let mut demo = Demo {
                dialog_open: true,
                ..Demo::default()
            };
            let bounds = Rect::from_min_size(Pos2::ZERO, vec2(640., 480.));
            let mut draw = || {
                let mut frame = ctx.run_ui(
                    RawInput {
                        screen_rect: Some(bounds),
                        ..RawInput::default()
                    },
                    |ui| demo.dialog(ui.ctx()),
                );
                frame.textures_delta.clear();
                frame
            };
            draw();
            let frame = draw();
            assert!(theme.colors.scrim.a() < 255);
            assert!(
                frame.shapes.iter().any(|clipped| {
                    matches!(&clipped.shape, Shape::Rect(rect)
                    if rect.rect == bounds && rect.fill == theme.colors.scrim)
                }),
                "{mode:?}: missing modal scrim"
            );
        }
    }
}
