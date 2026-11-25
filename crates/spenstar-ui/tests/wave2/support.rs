use spenstar_ui::{
    Theme,
    egui::{
        Context, Event, FullOutput, Id, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Ui,
        pos2, vec2,
    },
    theme::{Accent, Mode},
};

pub struct Harness {
    pub ctx: Context,
    time: f64,
    size: spenstar_ui::egui::Vec2,
}

impl Harness {
    pub fn new(mode: Mode) -> Self {
        let ctx = Context::default();
        Theme::new(mode, Accent::Blue).install(&ctx);
        ctx.enable_accesskit();
        Self {
            ctx,
            time: 0.,
            size: vec2(600., 480.),
        }
    }

    pub fn draw<T>(
        &mut self,
        mut events: Vec<Event>,
        mut render: impl FnMut(&mut Ui) -> T,
    ) -> (FullOutput, T) {
        self.time += 1. / 60.;
        let mut value = None;
        let modifiers = events
            .iter()
            .find_map(|event| match event {
                Event::Key { modifiers, .. } => Some(*modifiers),
                _ => None,
            })
            .unwrap_or_default();
        events.insert(0, Event::ModifiersChanged(modifiers));
        let mut output = self.ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
                time: Some(self.time),
                events,
                ..Default::default()
            },
            |ui| value = Some(render(ui)),
        );
        let Some(value) = value else {
            panic!("the root UI did not run");
        };
        output.textures_delta.clear();
        (output, value)
    }

    #[allow(dead_code)]
    pub fn click<T>(&mut self, at: Pos2, mut render: impl FnMut(&mut Ui) -> T) -> (FullOutput, T) {
        self.draw(vec![Event::PointerMoved(at)], &mut render);
        self.draw(pointer(at, true), &mut render);
        self.draw(pointer(at, false), &mut render)
    }

    #[allow(dead_code)]
    pub fn focus(&self, id: Id) {
        self.ctx.memory_mut(|memory| memory.request_focus(id));
    }
}

#[allow(dead_code)]
pub fn key(key: Key) -> Vec<Event> {
    key_with(key, Modifiers::NONE)
}

#[allow(dead_code)]
pub fn key_with(key: Key, modifiers: Modifiers) -> Vec<Event> {
    [true, false]
        .into_iter()
        .map(|pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        })
        .collect()
}

#[allow(dead_code)]
pub fn pointer(pos: Pos2, pressed: bool) -> Vec<Event> {
    vec![Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }]
}

#[allow(dead_code)]
pub fn node<'a>(output: &'a FullOutput, name: &str) -> &'a spenstar_ui::egui::accesskit::Node {
    let found = output
        .platform_output
        .accesskit_update
        .as_ref()
        .and_then(|tree| {
            tree.nodes
                .iter()
                .find_map(|(_, node)| (node.label() == Some(name)).then_some(node))
        });
    let Some(node) = found else {
        panic!("missing accessibility node {name}");
    };
    node
}

#[allow(dead_code)]
pub fn center(output: &FullOutput, name: &str) -> Pos2 {
    let Some(bounds) = node(output, name).bounds() else {
        panic!("missing bounds for {name}");
    };
    pos2(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    )
}
