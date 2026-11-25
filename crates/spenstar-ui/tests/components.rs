//! Headless interaction and rendering coverage for the public components.
use spenstar_ui::{
    Theme,
    button::{
        Button, ButtonGroup, CloseButton, Link, Size, ToggleButton, ToggleButtonGroup, Variant,
    },
    egui::{self, Color32, Context, Event, FullOutput, Modifiers, Pos2, RawInput, Rect, Ui, vec2},
    icon::Icon,
    layout::{
        AppShell, Card, CardVariant, EmptyState, Inspector, MenuItem, MenuSection, Navbar,
        PageHeader, ResponsiveGrid, ScrollShadow, SearchTrigger, Separator, SidebarPage,
        SidebarVariant, SplitAxis, SplitPane, Surface, Toolbar,
    },
    theme::{Accent, Mode, Radius},
    typography::Type,
};

fn frame(ctx: &Context, events: Vec<Event>, mut show: impl FnMut(&mut Ui)) -> FullOutput {
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1100., 840.))),
            events,
            ..Default::default()
        },
        |ui| show(ui),
    );
    output.textures_delta.clear();
    output
}

const ITEMS: &[MenuItem<'_>] = &[
    MenuItem::new("overview", "Overview", Icon::LayoutDashboard).badge(4),
    MenuItem::new("group", "Library", Icon::Layers).group(&[MenuItem::new(
        "icons",
        "Icons",
        Icon::Grid,
    )]),
];

#[test]
fn renders_all_wave_one_controls_and_layouts_in_both_modes() {
    for mode in [Mode::Light, Mode::Dark] {
        let ctx = Context::default();
        Theme::new(mode, Accent::Indigo).install(&ctx);
        let mut shell = AppShell::new("smoke");
        let mut first_pane = 180.;
        let mut toggle = true;
        let mut segment = 1;
        for variant in [
            SidebarVariant::Standard,
            SidebarVariant::Inset,
            SidebarVariant::Floating,
        ] {
            shell.sidebar.variant = variant;
            let output = frame(&ctx, vec![], |ui| {
                let mut inspector = |ui: &mut Ui| {
                    ui.label("Inspector");
                };
                shell.show(
                    ui,
                    |ui, sidebar| {
                        sidebar.show(
                            ui,
                            SidebarPage {
                                id: "library",
                                parent: Some("workspace"),
                                sections: &[MenuSection {
                                    label: "Workspace",
                                    items: ITEMS,
                                }],
                                active: Some("overview"),
                            },
                            |ui, _| {
                                ui.label("Workspace");
                            },
                            |ui, _| {
                                ui.label("Operator");
                            },
                        );
                    },
                    |ui| {
                        PageHeader::new("Dashboard")
                            .subtitle("Recent updates")
                            .search(SearchTrigger {
                                label: "Search",
                                shortcut: Some("Ctrl K"),
                            })
                            .show(
                                ui,
                                |ui| {
                                    ui.label("Workspace");
                                },
                                |ui| {
                                    CloseButton::show(ui);
                                },
                            );
                    },
                    |ui| {
                        Surface::show(ui, |ui| {
                            Navbar::show(
                                ui,
                                None,
                                |ui| {
                                    ui.label("Components");
                                },
                                |_| {},
                            );
                            ResponsiveGrid::new(140.)
                                .max_columns(4)
                                .show(ui, 4, |ui, index| {
                                    let variant = [
                                        CardVariant::Flat,
                                        CardVariant::Bordered,
                                        CardVariant::Elevated,
                                        CardVariant::Interactive,
                                    ][index];
                                    Card::new(variant).label("Card").show(ui, |card| {
                                        card.header("Card", Some("Description"));
                                        card.body(|ui| {
                                            ui.label(Type::H4.text("Body"));
                                        });
                                        card.footer(|ui| {
                                            Link::new("Rust", "https://www.rust-lang.org/")
                                                .show(ui);
                                        });
                                    });
                                });
                        });
                        Toolbar::show(ui, |ui| {
                            for (index, variant) in [
                                Variant::Accent,
                                Variant::Secondary,
                                Variant::Soft,
                                Variant::Outline,
                                Variant::Ghost,
                                Variant::Danger,
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                Button::new("Action")
                                    .variant(variant)
                                    .size([Size::Sm, Size::Md, Size::Lg][index % 3])
                                    .leading(Icon::Plus)
                                    .trailing(Icon::ChevronRight)
                                    .show(ui);
                            }
                            Separator::Vertical(28.).show(ui);
                            Button::new("Settings")
                                .icon_only(Icon::Settings)
                                .enabled(false)
                                .tooltip("Disabled")
                                .show(ui);
                            Button::new("Saving").loading(true).show(ui);
                        });
                        Separator::Labeled("Groups").show(ui);
                        Toolbar::show(ui, |ui| {
                            ButtonGroup::new([Button::new("Left"), Button::new("Right")]).show(ui);
                            ToggleButton::new(Button::new("Star")).show(ui, &mut toggle);
                            ToggleButtonGroup::new(
                                "range",
                                [Button::new("Day"), Button::new("Month")],
                            )
                            .show(ui, &mut segment);
                        });
                        SplitPane::new("side-by-side", SplitAxis::Horizontal)
                            .limits(120., 320., 140.)
                            .show(
                                ui,
                                210.,
                                &mut first_pane,
                                |ui| {
                                    ScrollShadow::new("feed", 116.).show(ui, |ui| {
                                        for index in 0..25 {
                                            ui.label(Type::Mono.text(format!("Activity {index}")));
                                        }
                                    });
                                },
                                |ui| {
                                    EmptyState::new(
                                        Icon::Inbox,
                                        "No entries",
                                        "Create a record to start.",
                                    )
                                    .primary(Button::new("Create"))
                                    .secondary(Button::new("Help"))
                                    .show(ui);
                                },
                            );
                    },
                    Some(Inspector::new(190., &mut inspector)),
                );
            });
            assert!(output.shapes.len() > 35);
            assert!(
                !ctx.tessellate(output.shapes, output.pixels_per_point)
                    .is_empty()
            );
        }
        assert_eq!(
            ctx.style_of(ctx.theme()).visuals.dark_mode,
            mode == Mode::Dark
        );
        assert_eq!(Theme::current(&ctx).mode, mode);
    }
}

#[test]
fn interactive_card_accepts_pointer_activation() {
    let ctx = Context::default();
    let mut clicked = false;
    let rect = std::cell::Cell::new(Rect::NOTHING);
    let mut render = |ui: &mut Ui| {
        let response = Card::new(CardVariant::Interactive)
            .label("Copy")
            .show(ui, |card| {
                card.body(|ui| {
                    ui.add(egui::Label::new("Copy").selectable(false));
                });
            })
            .response;
        clicked |= response.clicked();
        rect.set(response.rect);
    };
    frame(&ctx, vec![], &mut render);
    let pos = rect.get().center();
    let pointer = |pressed| Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(
        &ctx,
        vec![Event::PointerMoved(pos), pointer(true)],
        &mut render,
    );
    frame(&ctx, vec![pointer(false)], &mut render);
    assert!(clicked);
}

#[test]
fn custom_accent_and_radius_resolve_safely() {
    let custom = Color32::from_rgb(15, 80, 90);
    let light = Theme::new(Mode::Light, Accent::Custom(custom));
    assert_eq!(light.colors.accent.base, custom);
    assert_eq!(light.colors.accent.foreground, Color32::WHITE);
    assert_ne!(light.colors.accent.base, light.colors.accent.active);
    assert_eq!(Radius::Full.for_size(vec2(5., 48.)).nw, 2);
    assert_eq!(Radius::Lg.for_size(vec2(2., 16.)).nw, 1);
}
