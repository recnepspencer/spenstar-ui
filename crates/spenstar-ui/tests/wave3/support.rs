use spenstar_ui::egui::{
    Context, Event, FullOutput, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Ui,
    epaint::Shape, vec2,
};

pub(super) fn frame(
    ctx: &Context,
    time: f64,
    events: Vec<Event>,
    mut draw: impl FnMut(&mut Ui),
) -> FullOutput {
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800., 780.))),
            time: Some(time),
            events: events
                .into_iter()
                .flat_map(|event| {
                    let release = match event {
                        Event::Key {
                            key,
                            physical_key,
                            pressed: true,
                            modifiers,
                            ..
                        } => Some(Event::Key {
                            key,
                            physical_key,
                            pressed: false,
                            modifiers,
                            repeat: false,
                        }),
                        _ => None,
                    };
                    std::iter::once(event).chain(release)
                })
                .collect(),
            ..RawInput::default()
        },
        |ui| draw(ui),
    );
    output.textures_delta.clear();
    output
}

#[allow(dead_code)]
pub(super) fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}

#[allow(dead_code)]
pub(super) fn click(pos: Pos2) -> Vec<Event> {
    std::iter::once(Event::PointerMoved(pos))
        .chain(
            [true, false]
                .into_iter()
                .map(|pressed| Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                }),
        )
        .collect()
}

pub(super) fn texts(output: &FullOutput) -> Vec<(&str, Rect)> {
    fn visit<'a>(shape: &'a Shape, clip: Rect, out: &mut Vec<(&'a str, Rect)>) {
        match shape {
            Shape::Text(text) => {
                let visible = text.visual_bounding_rect().intersect(clip);
                if visible.is_positive() {
                    out.push((text.galley.text(), visible));
                }
            }
            Shape::Vec(children) => {
                for shape in children {
                    visit(shape, clip, out);
                }
            }
            _ => {}
        }
    }

    let mut result = vec![];
    for shape in &output.shapes {
        visit(&shape.shape, shape.clip_rect, &mut result);
    }
    result
}

pub(super) fn location(output: &FullOutput, target: &str) -> Pos2 {
    texts(output)
        .into_iter()
        .find_map(|(text, rect)| (text == target).then_some(rect.center()))
        .unwrap_or_else(|| panic!("missing {target}"))
}
