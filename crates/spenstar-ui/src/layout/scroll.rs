use egui::{Color32, Rect, ScrollArea, Ui, epaint::Mesh, pos2};

use crate::Theme;

/// Vertical scrolling with fade masks indicating hidden content above and below.
pub struct ScrollShadow {
    area: ScrollArea,
    fill: Option<Color32>,
}

impl ScrollShadow {
    /// Create a scroll area capped to the supplied height in points.
    pub fn new(id: impl egui::AsIdSalt, max_height: f32) -> Self {
        Self {
            area: ScrollArea::vertical()
                .id_salt(id)
                .max_height(max_height)
                .auto_shrink([false, false]),
            fill: None,
        }
    }

    /// Match the masks to the surrounding surface.
    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = Some(color);
        self
    }

    /// Show the scroll area and return its contents' result.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let color = self
            .fill
            .unwrap_or_else(|| Theme::current(ui.ctx()).colors.surface);
        let out = self.area.show(ui, content);
        let rect = out.inner_rect;
        let top = out.state.clamped_offset().y;
        let bottom = out.content_size.y - rect.height() - top;
        let depth = (rect.height() * 0.5).clamp(0., 16.);
        for (edge, remaining, direction) in [(rect.top(), top, 1.), (rect.bottom(), bottom, -1.)] {
            if remaining > 0.5 && depth > 0. {
                fade(
                    ui,
                    rect,
                    edge,
                    depth * direction,
                    color.gamma_multiply((remaining / 8.).min(1.)),
                );
            }
        }
        out.inner
    }
}

fn fade(ui: &Ui, rect: Rect, edge: f32, delta: f32, color: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(pos2(rect.left(), edge), color);
    mesh.colored_vertex(pos2(rect.right(), edge), color);
    mesh.colored_vertex(pos2(rect.left(), edge + delta), Color32::TRANSPARENT);
    mesh.colored_vertex(pos2(rect.right(), edge + delta), Color32::TRANSPARENT);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(2, 1, 3);
    ui.painter().add(mesh);
}
