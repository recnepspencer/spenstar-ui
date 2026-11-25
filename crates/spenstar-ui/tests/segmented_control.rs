//! Headless checks for segmented control painting and accessibility.
use spenstar_ui::{
    Theme,
    button::{Button, ToggleButtonGroup},
    egui::{
        Context, Event, FullOutput, Pos2, RawInput, Rect, Response,
        accesskit::Toggled,
        epaint::{RectShape, Shape},
        vec2,
    },
};

fn render(
    ctx: &Context,
    time: f64,
    shift: f32,
    pointer: Option<Pos2>,
    selected: &mut usize,
) -> (FullOutput, Vec<Response>) {
    let mut responses = Vec::new();
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(640., 320.))),
            time: Some(time),
            events: pointer.into_iter().map(Event::PointerMoved).collect(),
            ..Default::default()
        },
        |ui| {
            ui.horizontal(|ui| {
                ui.add_space(shift);
                responses =
                    ToggleButtonGroup::new("period", [Button::new("Day"), Button::new("Month")])
                        .show(ui, selected);
            });
        },
    );
    output.textures_delta.clear();
    (output, responses)
}

fn rectangles(output: &FullOutput) -> impl Iterator<Item = &RectShape> {
    output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            Shape::Rect(rect) => Some(rect),
            _ => None,
        })
}

#[test]
fn sliding_indicator_tracks_group_origin_without_delay() {
    let ctx = Context::default();
    let theme = Theme::default();
    theme.install(&ctx);
    let mut selected = 0;
    render(&ctx, 0., 12., None, &mut selected);
    selected = 1;
    render(&ctx, 1., 12., None, &mut selected);
    let (before, before_buttons) = render(&ctx, 1.08, 12., None, &mut selected);

    // Keep the animation time fixed so only the layout origin moves.
    let (after, after_buttons) = render(&ctx, 1.08, 132., None, &mut selected);
    let pill = |output: &FullOutput| {
        rectangles(output)
            .find_map(|shape| (shape.fill == theme.colors.accent.soft_hover).then_some(shape.rect))
    };
    let Some(before_pill) = pill(&before) else {
        panic!("missing selection indicator")
    };
    let Some(after_pill) = pill(&after) else {
        panic!("missing shifted selection indicator")
    };
    assert!(before_pill.left() > before_buttons[0].rect.left() + 2.);
    assert!(before_pill.left() < before_buttons[1].rect.left() + 2.);
    let offset = after_buttons[0].rect.left() - before_buttons[0].rect.left();
    assert!(offset > 100.);
    let expected = before_pill.translate(vec2(offset, 0.));
    assert!((after_pill.left() - expected.left()).abs() < 0.01);
    assert!((after_pill.right() - expected.right()).abs() < 0.01);
}

#[test]
fn clamped_selection_remains_visible_and_accessible_when_hovered() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let theme = Theme::default();
    theme.install(&ctx);
    let mut selected = usize::MAX;
    let (_, buttons) = render(&ctx, 0., 12., None, &mut selected);
    let pointer = buttons[1].rect.center();
    render(&ctx, 1., 12., Some(pointer), &mut selected);
    let (output, buttons) = render(&ctx, 2., 12., Some(pointer), &mut selected);
    assert!(buttons[1].hovered());

    let mut shapes = rectangles(&output);
    let Some(pill) = shapes.find(|shape| shape.fill == theme.colors.accent.soft_hover) else {
        panic!("missing selection indicator");
    };
    assert_eq!(pill.rect, buttons[1].rect.shrink(2.));
    assert!(shapes.all(|shape| !shape.rect.contains(pointer) || !shape.fill.is_opaque()));

    let Some(a11y) = &output.platform_output.accesskit_update else {
        panic!("missing accessibility update");
    };
    for (button, toggled) in buttons.iter().zip([Toggled::False, Toggled::True]) {
        let state = a11y.nodes.iter().find_map(|(id, node)| {
            if *id == button.id.accesskit_id() {
                node.toggled()
            } else {
                None
            }
        });
        assert_eq!(state, Some(toggled));
    }
}
