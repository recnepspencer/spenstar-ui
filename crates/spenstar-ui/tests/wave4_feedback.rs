//! Headless state, hit testing, and painting checks for Wave 4 feedback.
#[path = "wave2/support.rs"]
mod support;

use std::time::Duration;

use spenstar_ui::{
    Theme,
    egui::{Event, Ui, pos2, vec2},
    feedback::{
        Banner, Meter, ProgressBar, ProgressCircle, Size, Skeleton, Spinner, Toast, ToastEvent,
        ToastPosition, ToastQueue, Tone,
    },
    theme::Mode,
};
use support::{Harness, center};

#[test]
fn toast_cap_hover_pause_close_expiration_and_pending_promotion() {
    let mut h = Harness::new(Mode::Dark);
    let mut queue = ToastQueue::new("stack")
        .position(ToastPosition::TopRight)
        .max_visible(1);
    let first = queue.push(Toast::new("First toast").duration(Duration::from_millis(360)));
    queue.push(
        Toast::new("Next toast")
            .description("Waiting for a slot")
            .duration(Duration::from_millis(360)),
    );
    let mut render = |ui: &mut Ui| {
        queue.show(ui.ctx());
        (queue.visible_count(), queue.pending_count())
    };
    h.draw(vec![], &mut render);
    for _ in 0..11 {
        h.draw(vec![], &mut render);
    }
    let (output, counts) = h.draw(vec![], &mut render);
    assert_eq!(counts, (1, 1));
    let close = center(&output, "Close");
    h.draw(vec![Event::PointerMoved(close)], &mut render);
    for _ in 0..36 {
        h.draw(vec![], &mut render);
    }
    assert_eq!(h.draw(vec![], &mut render).1, (1, 1));
    h.click(close, &mut render);
    h.draw(vec![Event::PointerMoved(pos2(5., 460.))], &mut render);
    for _ in 0..12 {
        h.draw(vec![], &mut render);
    }
    assert_eq!(h.draw(vec![], &mut render).1, (1, 0));
    for _ in 0..38 {
        h.draw(vec![], &mut render);
    }
    assert_eq!(h.draw(vec![], &mut render).1, (0, 0));
    queue.dismiss(first);
    let pending = queue.push(Toast::new("Cancel before display"));
    queue.dismiss(pending);
    assert_eq!(queue.pending_count(), 0);
}

#[test]
fn shared_toast_action_emits_once_and_all_positions_are_visible() {
    let mut h = Harness::new(Mode::Light);
    let ctx = h.ctx.clone();
    let id = ToastQueue::notify(
        &ctx,
        Toast::new("Upload failed")
            .tone(Tone::Danger)
            .action("Retry"),
    );
    let mut draw = |_: &mut Ui| ToastQueue::show_context(&ctx);
    h.draw(vec![], &mut draw);
    for _ in 0..12 {
        h.draw(vec![], &mut draw);
    }
    let (output, _) = h.draw(vec![], &mut draw);
    let retry = center(&output, "Retry");
    assert!(retry.x > 300. && retry.y > 330., "{retry:?}");
    assert_eq!(
        h.click(retry, &mut draw).1,
        vec![ToastEvent {
            id,
            action: "Retry".into()
        }]
    );
    assert!(h.draw(vec![], &mut draw).1.is_empty());
    ToastQueue::clear_context(&ctx);
    let mut centered = ToastQueue::new("centered").position(ToastPosition::BottomCenter);
    centered.push(Toast::new("Centered"));
    let mut draw = |_: &mut Ui| centered.show(&ctx);
    for _ in 0..12 {
        h.draw(vec![], &mut draw);
    }
    let close = center(&h.draw(vec![], &mut draw).0, "Close");
    assert!(close.x > 300. && close.x < 470. && close.y > 330.);
}

#[test]
fn alerts_expose_action_and_dismiss_responses() {
    let mut h = Harness::new(Mode::Light);
    let mut draw = |ui: &mut Ui| {
        Banner::new("Workspace in sync")
            .description("All changes pushed.")
            .tone(Tone::Success)
            .action("Details")
            .dismissible(true)
            .show(ui)
    };
    let (output, _) = h.draw(vec![], &mut draw);
    assert!(
        h.click(center(&output, "Details"), &mut draw)
            .1
            .action_clicked
    );
    let (output, _) = h.draw(vec![], &mut draw);
    assert!(h.click(center(&output, "Close"), &mut draw).1.dismissed);
}

#[test]
fn progress_clamps_nonfinite_inputs_and_meter_interpolates_valid_thresholds() {
    for (input, expected) in [
        (f32::NAN, 0.),
        (f32::NEG_INFINITY, 0.),
        (-7., 0.),
        (0.42, 0.42),
        (18., 1.),
        (f32::INFINITY, 1.),
    ] {
        assert_eq!(ProgressBar::new(input).fraction(), Some(expected));
        assert_eq!(ProgressCircle::new(input).fraction(), expected);
        assert_eq!(Meter::new(input).fraction(), expected);
    }
    assert_eq!(ProgressBar::indeterminate().fraction(), None);
    for mode in [Mode::Light, Mode::Dark] {
        let mut h = Harness::new(mode);
        let c = Theme::current(&h.ctx);
        assert_eq!(Meter::new(0.).color(c), c.colors.success.base);
        assert_eq!(Meter::new(0.7).color(c), c.colors.warning.base);
        assert_eq!(Meter::new(1.).color(c), c.colors.danger.base);
        assert_ne!(Meter::new(0.8).color(c), c.colors.danger.base);
        assert_eq!(
            Meter::new(0.7).thresholds(0.95, 0.7).color(c),
            c.colors.warning.base
        );
        assert_eq!(
            Meter::new(0.).thresholds(f32::NAN, f32::NAN).color(c),
            c.colors.danger.base
        );
        let (output, response) = h.draw(vec![], |ui| {
            ProgressBar::new(f32::NAN)
                .label("Deploying")
                .percentage(true)
                .tone(Tone::Success)
                .show(ui);
            ProgressBar::indeterminate().size(Size::Sm).show(ui);
            Meter::new(f32::INFINITY)
                .labels("Quota", Some("Full"))
                .show(ui);
            ProgressCircle::new(0.42)
                .percentage(true)
                .diameter(f32::NAN)
                .show(ui)
        });
        assert!(
            response.rect.is_finite() && output.shapes.len() > 12,
            "{:?} {}",
            response.rect,
            output.shapes.len()
        );
    }
}

#[test]
fn spinner_and_skeleton_paint_visible_geometry_and_repaint() {
    let mut h = Harness::new(Mode::Dark);
    let (output, bounds) = h.draw(vec![], |ui| {
        let small = Spinner::new(Size::Sm).show(ui).rect;
        let large = Spinner::new(Size::Lg).tone(Tone::Success).show(ui).rect;
        Skeleton::line(90.).show(ui);
        Skeleton::circle(24.).show(ui);
        Skeleton::rect(vec2(100., 20.)).show(ui);
        let card = Skeleton::card(ui).rect;
        (small, large, card)
    });
    assert!(bounds.1.width() > bounds.0.width() && bounds.2.is_positive());
    assert!(output.shapes.len() >= 20);
    assert!(
        output
            .viewport_output
            .values()
            .any(|output| output.repaint_delay.is_zero())
    );
}
