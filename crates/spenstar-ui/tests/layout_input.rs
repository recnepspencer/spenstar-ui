//! Headless geometry, animation, and input arbitration checks.
use spenstar_ui::{
    button::Button,
    egui::{
        self, Context, Event, FullOutput, Id, Key, KeyboardShortcut, Modifiers, Pos2, RawInput,
        Rect, Sense, Ui, vec2,
    },
    input::ShortcutGuard,
    layout::{AppShell, CollapseMode, ResponsiveGrid, Sidebar, SplitAxis, SplitPane},
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

#[test]
fn grid_clips_paint_outsets_to_parent_and_column_gaps() {
    let ctx = Context::default();
    for gap in [12., 2.] {
        frame(&ctx, vec![], |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(240.);
                    if gap == 2. {
                        let clip = ui
                            .clip_rect()
                            .with_min_x(ui.available_rect_before_wrap().left() - 2.);
                        ui.shrink_clip_rect(clip);
                    }
                    let parent_clip = ui.clip_rect();
                    let mut cells = Vec::new();
                    ResponsiveGrid::new(100.).gap(gap).show(ui, 2, |ui, _| {
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(ui.available_width(), 30.), Sense::hover());
                        cells.push((rect, ui.clip_rect()));
                    });
                    let [(first, first_clip), (second, second_clip)] = cells.as_slice() else {
                        panic!("expected two grid cells");
                    };
                    assert_eq!(first_clip.right(), second_clip.left());
                    assert_eq!(first_clip.right() - first.right(), gap * 0.5);
                    assert_eq!(second.left() - second_clip.left(), gap * 0.5);
                    assert!(parent_clip.contains_rect(*first_clip));
                    assert!(parent_clip.contains_rect(*second_clip));
                    if gap == 12. {
                        assert!(first_clip.contains_rect(first.expand(4.)));
                        assert!(second_clip.contains_rect(second.expand(4.)));
                        assert!(first.left() - first_clip.left() > 4.);
                        assert!(second_clip.right() - second.right() > 4.);
                    } else {
                        assert_eq!(first_clip.left(), parent_clip.left());
                    }
                });
        });
    }
}

#[test]
fn sidebar_animates_between_all_widths() {
    let ctx = Context::default();
    let mut sidebar = Sidebar::default();
    let id = Id::unique("test/sidebar");
    let measure = |time| {
        let mut width = 0.;
        ctx.run_ui(
            RawInput {
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                width = sidebar.animated_width(ui.ctx(), id);
            },
        )
        .textures_delta
        .clear();
        width
    };
    assert_eq!(measure(0.), CollapseMode::Expanded.width());
    // Recreate the measuring closure as the sidebar's target changes.
    for (start, mode, before) in [
        (1., CollapseMode::IconRail, CollapseMode::Expanded),
        (2., CollapseMode::Offcanvas, CollapseMode::IconRail),
        (3., CollapseMode::Expanded, CollapseMode::Offcanvas),
    ] {
        sidebar.cycle();
        assert_eq!(sidebar.mode, mode);
        let mut width = 0.;
        let mut tick = |time| {
            ctx.run_ui(
                RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| {
                    width = sidebar.animated_width(ui.ctx(), id);
                },
            )
            .textures_delta
            .clear();
            width
        };
        assert_eq!(tick(start), before.width());
        let middle = tick(start + 0.09);
        let low = mode.width().min(before.width());
        let high = mode.width().max(before.width());
        assert!(middle > low && middle < high);
        assert_eq!(tick(start + 0.5), mode.width());
    }
}

#[test]
fn grid_reflows_at_cell_boundaries_and_split_enforces_limits() {
    let grid = ResponsiveGrid::new(180.).gap(12.).max_columns(3);
    assert_eq!(grid.columns(0.), 1);
    assert_eq!(grid.columns(371.), 1);
    assert_eq!(grid.columns(372.), 2);
    assert_eq!(grid.columns(564.), 3);
    assert_eq!(grid.columns(1800.), 3);

    let ctx = Context::default();
    let split = SplitPane::new("drag", SplitAxis::Horizontal).limits(120., 340., 150.);
    assert_eq!(split.clamp_position(10., 600.), 120.);
    assert_eq!(split.clamp_position(500., 600.), 340.);
    assert_eq!(split.clamp_position(340., 300.), 138.);
    assert_eq!(split.clamp_position(100., 100.), 0.);
    let mut position = 180.;
    let mut divider = Rect::NOTHING;
    let mut show = |ui: &mut Ui| {
        divider = split.show(ui, 100., &mut position, |_| {}, |_| {}).rect;
    };
    frame(&ctx, vec![], &mut show);
    let center = divider.center();
    let pointer = |pos, pressed| Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    let mut show = |ui: &mut Ui| {
        split.show(ui, 100., &mut position, |_| {}, |_| {});
    };
    frame(
        &ctx,
        vec![Event::PointerMoved(center), pointer(center, true)],
        &mut show,
    );
    frame(
        &ctx,
        vec![Event::PointerMoved(center + vec2(400., 0.))],
        &mut show,
    );
    assert_eq!(position, 340.);

    let vertical = SplitPane::new("vertical", SplitAxis::Vertical).limits(30., 100., 50.);
    let mut position = 80.;
    let mut divider = Rect::NOTHING;
    frame(&ctx, vec![pointer(center, false)], |ui| {
        divider = vertical.show(ui, 180., &mut position, |_| {}, |_| {}).rect;
    });
    assert_eq!(divider.height(), 12.);
    assert!(divider.width() > 100.);
}

#[test]
fn text_input_and_modals_block_background_shortcuts() {
    let ctx = Context::default();
    let mut shell = AppShell::new("shortcuts");
    let command = || Event::Key {
        key: Key::B,
        physical_key: Some(Key::B),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::COMMAND,
    };
    let mut field_value = String::new();
    let mut field_id = Id::NULL;
    frame(&ctx, vec![], |ui| {
        shell.show(
            ui,
            |_, _| {},
            |_| {},
            |ui| {
                let field = ui.text_edit_singleline(&mut field_value);
                field_id = field.id;
                field.request_focus();
            },
            None,
        );
    });
    frame(&ctx, vec![command(), Event::Text("b".into())], |ui| {
        assert!(ui.ctx().text_edit_focused());
        assert!(!ShortcutGuard::new(ui).allows());
        shell.show(
            ui,
            |_, _| {},
            |_| {},
            |ui| {
                ui.text_edit_singleline(&mut field_value);
            },
            None,
        );
    });
    assert_eq!(field_value, "b");
    assert_eq!(shell.sidebar.mode, CollapseMode::Expanded);
    ctx.memory_mut(|mem| mem.surrender_focus(field_id));

    let modal = Id::unique("test/shortcut-modal");
    frame(&ctx, vec![], |ui| {
        egui::Modal::new(modal).show(ui.ctx(), |ui| {
            ui.button("Overlay action").request_focus();
        });
    });
    frame(&ctx, vec![command()], |ui| {
        assert!(!ShortcutGuard::new(ui).allows());
        egui::Modal::new(modal).show(ui.ctx(), |ui| {
            assert!(ui.button("Overlay action").has_focus());
            assert!(ShortcutGuard::for_overlay(ui).consume(&AppShell::SIDEBAR_SHORTCUT));
        });
        shell.show(ui, |_, _| {}, |_| {}, |_| {}, None);
    });
    assert_eq!(shell.sidebar.mode, CollapseMode::Expanded);

    // Let egui expire the foreground layer and modal state.
    frame(&ctx, vec![], |_| {});
    frame(&ctx, vec![command()], |ui| {
        shell.show(ui, |_, _| {}, |_| {}, |_| {}, None);
    });
    assert_eq!(shell.sidebar.mode, CollapseMode::IconRail);
}

#[test]
fn modified_shell_shortcut_works_with_a_focused_button() {
    let ctx = Context::default();
    spenstar_ui::install(&ctx);
    let mut shell = AppShell::new("focused-button-shortcuts");
    frame(&ctx, vec![], |ui| {
        shell.show(
            ui,
            |_, _| {},
            |_| {},
            |ui| Button::new("Action").show(ui).request_focus(),
            None,
        );
    });
    let pressed = |key, modifiers| Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    };
    let plain = KeyboardShortcut::new(Modifiers::NONE, Key::J);
    frame(
        &ctx,
        vec![
            pressed(Key::J, Modifiers::NONE),
            pressed(Key::B, Modifiers::COMMAND),
        ],
        |ui| {
            shell.show(
                ui,
                |_, _| {},
                |_| {},
                |ui| {
                    assert!(Button::new("Action").show(ui).has_focus());
                    let guard = ShortcutGuard::new(ui);
                    assert!(guard.allows());
                    assert!(!guard.consume(&plain));
                    assert!(ui.input(|input| input.key_pressed(plain.logical_key)));
                },
                None,
            );
        },
    );
    assert_eq!(shell.sidebar.mode, CollapseMode::IconRail);
}
