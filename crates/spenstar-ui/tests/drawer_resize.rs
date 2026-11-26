//! Headless tests for sheet resizing and scrolling.
#[path = "wave2/support.rs"]
mod support;

use spenstar_ui::{
    button::Button,
    egui::{CursorIcon, Event, Modifiers, MouseWheelUnit, Pos2, Rect, Ui, pos2, vec2},
    overlay::{Drawer, DrawerSide, PanelResponse},
    theme::Mode,
};
use support::{Harness, pointer};

#[test]
fn inward_handles_resize_within_bounds_and_retain_extent_after_reopening() {
    for side in [DrawerSide::Left, DrawerSide::Right, DrawerSide::Bottom] {
        let mut h = Harness::new(Mode::Dark);
        let ctx = h.ctx.clone();
        let open = std::cell::Cell::new(true);
        let mut render = |_: &mut Ui| {
            let mut shown = open.get();
            let output = Drawer::new("dock", "Preferences", side)
                .extent(270.)
                .resizable(220. ..=320.)
                .show(&ctx, &mut shown, |ui| ui.label("Docked sheet"));
            open.set(shown);
            output.map(|sheet| sheet.response.rect)
        };
        for _ in 0..18 {
            h.draw(vec![], &mut render);
        }
        let (_, sheet) = h.draw(vec![], &mut render);
        let Some(sheet) = sheet else {
            panic!("sheet is missing")
        };
        assert_eq!(extent(sheet, side), 270.);
        let (start, more, less, expected_cursor) = match side {
            DrawerSide::Left => (
                pos2(sheet.right() - 3., 180.),
                vec2(100., 0.),
                vec2(-140., 0.),
                CursorIcon::ResizeHorizontal,
            ),
            DrawerSide::Right => (
                pos2(sheet.left() + 3., 180.),
                vec2(-100., 0.),
                vec2(140., 0.),
                CursorIcon::ResizeHorizontal,
            ),
            DrawerSide::Bottom => (
                pos2(250., sheet.top() + 3.),
                vec2(0., -100.),
                vec2(0., 140.),
                CursorIcon::ResizeVertical,
            ),
        };
        let (output, _) = h.draw(vec![Event::PointerMoved(start)], &mut render);
        assert_eq!(output.platform_output.cursor_icon, expected_cursor);
        h.draw(pointer(start, true), &mut render);
        let (_, resized) = h.draw(vec![Event::PointerMoved(start + more)], &mut render);
        assert_eq!(resized.map(|rect| extent(rect, side)), Some(320.));
        let (_, resized) = h.draw(vec![Event::PointerMoved(start + less)], &mut render);
        assert_eq!(resized.map(|rect| extent(rect, side)), Some(220.));
        h.draw(pointer(start + less, false), &mut render);
        open.set(false);
        for _ in 0..18 {
            h.draw(vec![], &mut render);
        }
        open.set(true);
        for _ in 0..18 {
            h.draw(vec![], &mut render);
        }
        let (_, reopened) = h.draw(vec![], &mut render);
        assert_eq!(reopened.map(|rect| extent(rect, side)), Some(220.));
    }
}

fn extent(rect: Rect, side: DrawerSide) -> f32 {
    if side == DrawerSide::Bottom {
        rect.height()
    } else {
        rect.width()
    }
}

#[test]
fn drawer_wheel_scrolls_body_while_header_and_footer_stay_pinned() {
    let mut h = Harness::new(Mode::Light);
    let ctx = h.ctx.clone();
    let mut open = true;
    let mut render = |_: &mut Ui| {
        Drawer::new("scroll", "Tall content", DrawerSide::Right).show_with_footer(
            &ctx,
            &mut open,
            |ui| {
                let first = ui.label("First row").rect.min;
                for index in 1..80 {
                    ui.label(format!("Row {index}"));
                }
                (first, ui.clip_rect())
            },
            |ui| Button::new("Done").show(ui).rect,
        )
    };
    for _ in 0..18 {
        h.draw(vec![], &mut render);
    }
    let (frame, before) = h.draw(vec![], &mut render);
    let header = support::center(&frame, "Close");
    let before = unpack(before);
    let wheel_point = before.1.center();
    h.draw(vec![Event::PointerMoved(wheel_point)], &mut render);
    h.draw(
        vec![Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: vec2(0., -220.),
            modifiers: Modifiers::NONE,
            phase: spenstar_ui::egui::TouchPhase::Move,
            source: spenstar_ui::egui::MouseWheelSource::Wheel,
        }],
        &mut render,
    );
    for _ in 0..12 {
        h.draw(vec![], &mut render);
    }
    let (frame, after) = h.draw(vec![], &mut render);
    let after = unpack(after);
    assert!(after.0.y < before.0.y - 50., "{before:?} {after:?}");
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, before.2);
    assert_eq!(support::center(&frame, "Close"), header);
}

fn unpack(output: Option<PanelResponse<(Pos2, Rect), Rect>>) -> (Pos2, Rect, Rect) {
    let Some(output) = output else {
        panic!("sheet is missing")
    };
    (output.body.0, output.body.1, output.footer)
}
