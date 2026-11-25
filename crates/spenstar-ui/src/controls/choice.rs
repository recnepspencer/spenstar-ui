use egui::{Rect, Response, Sense, Ui, Vec2, pos2, vec2};

use crate::{Theme, typography::Type};

pub(super) struct Choice {
    pub response: Response,
    pub indicator: Rect,
    background: egui::layers::ShapeIdx,
}

impl Choice {
    pub fn new(
        ui: &mut Ui,
        label: &str,
        description: Option<&str>,
        indicator: Vec2,
        card: bool,
    ) -> Self {
        let c = Theme::current(ui.ctx()).colors;
        let pad = if card { 12. } else { 4. };
        let gap = 8.;
        let width = (ui.available_width() - indicator.x - 2. * pad - gap).max(32.);
        let layout = |text: &str, typography: Type, color| {
            ui.painter()
                .layout(text.to_owned(), typography.font(), color, width)
        };
        let title = layout(label, Type::Body, c.text_primary);
        let description = description.map(|d| layout(d, Type::Small, c.text_secondary));
        let text_width = description
            .as_ref()
            .map_or(title.size().x, |d| d.size().x.max(title.size().x));
        let text_height = title.size().y + description.as_ref().map_or(0., |d| d.size().y + 2.);
        let (rect, response) = ui.allocate_exact_size(
            vec2(
                2. * pad + indicator.x + gap + text_width,
                2. * pad + text_height.max(indicator.y),
            )
            .max(vec2(0., 28.)),
            Sense::click(),
        );
        let background = ui.painter().add(egui::Shape::Noop);
        let indicator = Rect::from_center_size(
            pos2(rect.left() + pad + indicator.x * 0.5, rect.center().y),
            indicator,
        );
        let text = pos2(indicator.right() + gap, rect.center().y - text_height * 0.5);
        let title_height = title.size().y;
        ui.painter().galley(text, title, c.text_primary);
        if let Some(description) = description {
            ui.painter().galley(
                text + vec2(0., title_height + 2.),
                description,
                c.text_secondary,
            );
        }
        Self {
            response,
            indicator,
            background,
        }
    }

    pub fn paint(&self, ui: &Ui, selected: bool, card: bool) {
        let c = Theme::current(ui.ctx()).colors;
        let response = &self.response;
        let rect = response.rect;
        let radius = crate::theme::Radius::Md.for_size(rect.size());
        let fill = if selected && card {
            if response.hovered() {
                c.accent.soft_hover
            } else {
                c.accent.soft
            }
        } else if response.hovered() {
            c.surface_subtle
        } else {
            egui::Color32::TRANSPARENT
        };
        let stroke = if card {
            egui::Stroke::new(1., if selected { c.accent.ring } else { c.border })
        } else {
            egui::Stroke::NONE
        };
        ui.painter().set(
            self.background,
            egui::epaint::RectShape::new(rect, radius, fill, stroke, egui::StrokeKind::Inside),
        );
        crate::motion::focus_ring(ui, response, radius);
    }
}
