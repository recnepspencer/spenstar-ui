//! Headless interaction tests for Wave 4 overlays.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    button::Button,
    egui::{Event, Key, Modifiers, PointerButton, Rect, Ui, pos2},
    feedback::Tone,
    input::ShortcutGuard,
    overlay::{
        AlertDialog, AlertDialogResponse, ContextMenu, Drawer, DrawerSide, DropdownMenu, MenuItem,
        Modal, Popover, Tooltip,
    },
    theme::Mode,
};
use support::{Harness, center, key};

#[test]
fn modal_centers_adopts_input_surface_isolates_and_repaints_on_dismiss() {
    let mut h = Harness::new(Mode::Dark);
    for outside in [false, true] {
        let mut open = true;
        let ctx = h.ctx.clone();
        let mut render = |ui: &mut Ui| {
            let modal = Modal::new("modal", "Edit settings")
                .subtitle("Workspace")
                .dismiss_on_click_outside(outside)
                .show(
                    &ctx,
                    &mut open,
                    |ui| {
                        assert_eq!(
                            ui.visuals().extreme_bg_color,
                            spenstar_ui::Theme::current(&ctx).colors.surface_secondary
                        );
                        ui.label("Modal body");
                    },
                    |ui| Button::new("Save").show(ui),
                );
            (
                open,
                modal.map(|output| output.response.rect),
                ShortcutGuard::new(ui).allows(),
            )
        };
        h.draw(vec![], &mut render);
        h.draw(vec![], &mut render);
        let (_, (open, bounds, allows)) = h.draw(vec![], &mut render);
        assert!(open && !allows);
        assert!(
            bounds.is_some_and(|bounds| bounds.center().distance(pos2(300., 240.)) <= 3.),
            "{bounds:?}"
        );
        let (_, (still_open, ..)) = h.click(pos2(20., 20.), &mut render);
        assert_eq!(still_open, !outside);
        if still_open {
            let (output, (open, ..)) = h.draw(key(Key::Escape), &mut render);
            assert!(!open);
            assert!(
                output
                    .viewport_output
                    .values()
                    .any(|output| output.repaint_delay.is_zero())
            );
        }
    }
}

#[test]
fn destructive_dialog_confirms_with_its_action_and_escape_cancels() {
    let mut h = Harness::new(Mode::Light);
    let mut open = true;
    let ctx = h.ctx.clone();
    let mut render = |_: &mut Ui| {
        let outcome = AlertDialog::new(
            "delete",
            "Delete workspace?",
            "This permanently removes the workspace.",
        )
        .tone(Tone::Danger)
        .actions("Cancel", "Delete workspace")
        .show(&ctx, &mut open);
        (outcome, open)
    };
    h.draw(vec![], &mut render);
    let (output, _) = h.draw(vec![], &mut render);
    let (_, outcome) = h.click(center(&output, "Delete workspace"), &mut render);
    assert_eq!(outcome, (AlertDialogResponse::Confirmed, false));

    open = true;
    let mut render = |_: &mut Ui| {
        AlertDialog::new("warning", "Discard draft?", "Changes are unsaved.")
            .tone(Tone::Warning)
            .show(&ctx, &mut open)
    };
    h.draw(vec![], &mut render);
    h.draw(vec![], &mut render);
    assert_eq!(
        h.draw(key(Key::Escape), &mut render).1,
        AlertDialogResponse::Cancelled
    );
}

#[test]
fn drawers_slide_from_each_edge_keep_the_footer_visible_and_dismiss() {
    for side in [DrawerSide::Left, DrawerSide::Right, DrawerSide::Bottom] {
        let mut h = Harness::new(Mode::Light);
        let ctx = h.ctx.clone();
        let mut open = true;
        let mut render = |ui: &mut Ui| {
            let drawer = Drawer::new("drawer", "Workspace activity", side)
                .extent(280.)
                .show_with_footer(
                    &ctx,
                    &mut open,
                    |ui| {
                        for row in 0..40 {
                            ui.label(format!("Event {row}"));
                        }
                    },
                    |ui| Button::new("Done").show(ui),
                );
            (
                open,
                drawer.map(|panel| (panel.response.rect, panel.footer.rect)),
                ShortcutGuard::new(ui).allows(),
            )
        };
        let (_, (_, initial, _)) = h.draw(vec![], &mut render);
        for _ in 0..17 {
            h.draw(vec![], &mut render);
        }
        let (_, (open, result, allows)) = h.draw(vec![], &mut render);
        assert!(open && !allows);
        let (Some((panel, footer)), Some((initial, _))) = (result, initial) else {
            panic!("missing rendered drawer geometry");
        };
        assert!(initial.min.distance(panel.min) > 120.);
        assert!(
            ctx.content_rect().contains_rect(footer),
            "{side:?} footer {footer:?}"
        );
        let viewport = ctx.content_rect();
        match side {
            DrawerSide::Left => {
                assert_eq!(panel.min, viewport.min);
                assert_eq!(panel.bottom(), viewport.bottom());
            }
            DrawerSide::Right => {
                assert_eq!(panel.max, viewport.max);
                assert_eq!(panel.top(), viewport.top());
            }
            DrawerSide::Bottom => {
                assert_eq!(panel.max, viewport.max);
                assert_eq!(panel.left(), viewport.left());
            }
        }
        let outside = if side == DrawerSide::Left {
            pos2(580., 20.)
        } else {
            pos2(20., 20.)
        };
        let (_, (open, ..)) = h.click(outside, &mut render);
        assert!(!open);
        for _ in 0..17 {
            h.draw(vec![], &mut render);
        }
        assert!(h.draw(vec![], &mut render).1.1.is_none());
    }
}

#[test]
fn popover_flips_isolates_and_dismisses_by_keyboard_or_outside() {
    let mut h = Harness::new(Mode::Dark);
    let mut open = false;
    let mut draw = |ui: &mut Ui| {
        ui.add_space(422.);
        let anchor = Button::new("Popover").show(ui);
        let popup =
            Popover::new("popover").show(&anchor, &mut open, |ui| ui.label("Anchored content"));
        (
            open,
            anchor.rect,
            popup.map(|output| output.response.rect),
            ShortcutGuard::new(ui).allows(),
        )
    };
    let (_, (_, anchor, ..)) = h.draw(vec![], &mut draw);
    h.click(anchor.center(), &mut draw);
    let (_, (open, _, popup, allows)) = h.draw(vec![], &mut draw);
    assert!(open && !allows);
    assert!(popup.is_some_and(|popup| popup.bottom() < anchor.top()));
    assert!(!h.draw(key(Key::Escape), &mut draw).1.0);
    h.click(anchor.center(), &mut draw);
    h.draw(vec![], &mut draw);
    assert!(!h.click(pos2(572., 18.), &mut draw).1.0);
}

#[test]
fn dropdown_navigates_nested_disabled_and_checked_items_by_keyboard() {
    let mut h = Harness::new(Mode::Light);
    let children = [
        MenuItem::action("csv", "CSV").shortcut("⌘ E"),
        MenuItem::action("pdf", "PDF"),
    ];
    let items = [
        MenuItem::Heading("EXPORT"),
        MenuItem::action("disabled", "Unavailable").enabled(false),
        MenuItem::submenu("Export as", &children),
        MenuItem::Separator,
        MenuItem::action("check", "Pinned").checked(true),
    ];
    let mut draw = |ui: &mut Ui| {
        let anchor = Button::new("Dropdown").show(ui);
        let chosen = DropdownMenu::new("dropdown").show(&anchor, &items);
        (anchor.rect, chosen, ShortcutGuard::new(ui).allows())
    };
    let (_, (anchor, ..)) = h.draw(vec![], &mut draw);
    h.click(anchor.center(), &mut draw);
    assert!(!h.draw(vec![], &mut draw).1.2);
    h.draw(key(Key::ArrowDown), &mut draw);
    h.draw(key(Key::ArrowRight), &mut draw);
    let (output, _) = h.draw(vec![], &mut draw);
    assert!(center(&output, "CSV").x > anchor.right());
    h.draw(key(Key::ArrowLeft), &mut draw);
    h.draw(vec![], &mut draw);
    h.draw(key(Key::ArrowUp), &mut draw);
    h.draw(key(Key::ArrowDown), &mut draw);
    h.draw(key(Key::ArrowRight), &mut draw);
    h.draw(vec![], &mut draw);
    h.draw(key(Key::ArrowDown), &mut draw);
    assert_eq!(h.draw(key(Key::Enter), &mut draw).1.1, Some("pdf"));
    h.draw(vec![], &mut draw);
    h.click(anchor.center(), &mut draw);
    h.draw(vec![], &mut draw);
    assert_eq!(h.draw(key(Key::Escape), &mut draw).1.1, None);
    h.draw(vec![], &mut draw);
    assert!(h.draw(vec![], &mut draw).1.2);
}

#[test]
fn context_menu_uses_secondary_pointer_position_and_selects_an_action() {
    let mut h = Harness::new(Mode::Dark);
    let mut draw = |ui: &mut Ui| {
        let anchor = Button::new("Context target").show(ui);
        let chosen =
            ContextMenu::new("context").show(&anchor, &[MenuItem::action("copy", "Copy link")]);
        (anchor.rect, chosen)
    };
    let (_, (target, _)) = h.draw(vec![], &mut draw);
    let point = target.center();
    h.draw(vec![Event::PointerMoved(point)], &mut draw);
    for pressed in [true, false] {
        h.draw(
            vec![Event::PointerButton {
                pos: point,
                pressed,
                button: PointerButton::Secondary,
                modifiers: Modifiers::NONE,
            }],
            &mut draw,
        );
    }
    let (output, _) = h.draw(vec![], &mut draw);
    let item = center(&output, "Copy link");
    assert!(item.x > point.x && item.y > point.y);
    assert_eq!(h.click(item, &mut draw).1.1, Some("copy"));
}

#[test]
fn tooltip_waits_for_hover_and_stays_in_viewport() {
    let mut h = Harness::new(Mode::Dark);
    let ctx = h.ctx.clone();
    let mut draw = |ui: &mut Ui| {
        let anchor = Button::new("Help").show(ui);
        let tooltip = Tooltip::new("Open workspace commands")
            .shortcut("⌘ K")
            .delay(0.06)
            .show(&anchor);
        (anchor.rect, tooltip.map(|response| response.rect))
    };
    let (_, (anchor, _)) = h.draw(vec![], &mut draw);
    assert!(
        h.draw(vec![Event::PointerMoved(anchor.center())], &mut draw)
            .1
            .1
            .is_none()
    );
    for _ in 0..5 {
        h.draw(vec![], &mut draw);
    }
    assert!(
        h.draw(vec![], &mut draw)
            .1
            .1
            .is_some_and(|rect: Rect| ctx.content_rect().contains_rect(rect))
    );
    assert!(
        h.draw(vec![Event::PointerMoved(pos2(400., 400.))], &mut draw)
            .1
            .1
            .is_none()
    );
}

#[test]
fn hover_opens_a_submenu_to_the_left_at_the_viewport_edge() {
    let mut h = Harness::new(Mode::Dark);
    let children = [MenuItem::action("export", "Export document")];
    let items = [
        MenuItem::submenu("More formats", &children),
        MenuItem::action("close", "Close menu"),
    ];
    let mut draw = |ui: &mut Ui| {
        ui.with_layout(
            spenstar_ui::egui::Layout::right_to_left(spenstar_ui::egui::Align::Min),
            |ui| {
                let anchor = Button::new("More actions").show(ui);
                let selected = DropdownMenu::new("edge-menu").show(&anchor, &items);
                (anchor.rect, selected)
            },
        )
        .inner
    };
    let (_, (anchor, _)) = h.draw(vec![], &mut draw);
    h.click(anchor.center(), &mut draw);
    let (output, _) = h.draw(vec![], &mut draw);
    let more = center(&output, "More formats");
    h.draw(vec![Event::PointerMoved(more)], &mut draw);
    h.draw(vec![], &mut draw);
    let (output, _) = h.draw(vec![], &mut draw);
    let export = center(&output, "Export document");
    assert!(export.x < more.x - 100., "{export:?} {more:?}");
    assert_eq!(h.click(export, &mut draw).1.1, Some("export"));
}
