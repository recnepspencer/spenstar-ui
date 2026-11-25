//! Headless geometry, animation, and input arbitration checks.
use spenstar_ui::{
    egui::{self, Context, Event, FullOutput, Id, Key, Modifiers, Pos2, RawInput, Rect, Ui, vec2},
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
            ui.label("Overlay");
        });
    });
    frame(&ctx, vec![command()], |ui| {
        assert!(!ShortcutGuard::new(ui).allows());
        egui::Modal::new(modal).show(ui.ctx(), |ui| {
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
