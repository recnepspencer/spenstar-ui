use std::collections::BTreeSet;

use egui::{
    Align, Color32, Frame, Label, Layout, Margin, Response, Sense, Stroke, Ui, UiBuilder, pos2,
    vec2,
};

use crate::{Theme, motion, typography::Type};

/// Disclosure surface treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisclosureVariant {
    /// Bordered surface with rounded corners.
    #[default]
    Bordered,
    /// Individual tinted sections.
    Splitted,
    /// Divider and no fill.
    Light,
}

/// A controlled expandable section.
pub struct Disclosure<'a> {
    /// Stable identity shared between frames.
    pub id: egui::IdSalt,
    /// Clickable title.
    pub title: &'a str,
    /// Surface treatment.
    pub variant: DisclosureVariant,
}

impl<'a> Disclosure<'a> {
    /// Create a bordered disclosure.
    pub fn new(id: impl egui::AsIdSalt, title: &'a str) -> Self {
        Self {
            id: egui::IdSalt::new(id),
            title,
            variant: DisclosureVariant::Bordered,
        }
    }

    /// Animate and render a section, updating its open flag.
    pub fn show(&self, ui: &mut Ui, open: &mut bool, body: impl FnOnce(&mut Ui)) -> Response {
        ui.push_id(self.id, |ui| self.draw(ui, open, body)).inner
    }

    fn draw(&self, ui: &mut Ui, open: &mut bool, body: impl FnOnce(&mut Ui)) -> Response {
        let id = ui.make_persistent_id("openness");
        let c = Theme::current(ui.ctx()).colors;
        let (fill, border) = match self.variant {
            DisclosureVariant::Bordered => (c.surface, Stroke::new(1., c.border)),
            DisclosureVariant::Splitted => (c.surface_secondary, Stroke::NONE),
            DisclosureVariant::Light => (Color32::TRANSPARENT, Stroke::NONE),
        };
        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, *open);
        state.set_open(*open);
        Frame::new()
            .fill(fill)
            .stroke(border)
            .corner_radius(10)
            .inner_margin(Margin::symmetric(10, 5))
            .show(ui, |ui| {
                let width = ui.available_width();
                let (rect, response) = ui.allocate_exact_size(vec2(width, 30.), Sense::click());
                if response.clicked() {
                    *open = !*open;
                    state.set_open(*open);
                }
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::Role::Button,
                        response.enabled(),
                        *open,
                        self.title,
                    )
                });
                if response.hovered() {
                    ui.painter().rect_filled(rect, 6, c.field_hover);
                }
                let label_rect = rect.shrink2(vec2(8., 0.));
                let mut label_rect = label_rect;
                label_rect.max.x = (label_rect.right() - 20.).max(label_rect.left());
                let mut header = ui.new_child(
                    UiBuilder::new()
                        .max_rect(label_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                header.set_clip_rect(ui.clip_rect().intersect(label_rect));
                header.add(
                    Label::new(Type::Label.text(self.title))
                        .selectable(false)
                        .truncate(),
                );
                let angle = state.openness(ui.ctx()) * std::f32::consts::FRAC_PI_2;
                let rotation = egui::emath::Rot2::from_angle(angle);
                let center = pos2(rect.right() - 12., rect.center().y);
                let points =
                    [vec2(-2., -4.), vec2(2., 0.), vec2(-2., 4.)].map(|p| center + rotation * p);
                for points in points.windows(2) {
                    ui.painter()
                        .line_segment([points[0], points[1]], Stroke::new(1.5, c.text_secondary));
                }
                motion::focus_ring(ui, &response, 8.into());
                state.show_body_unindented(ui, |ui| {
                    if !*open {
                        ui.disable();
                    }
                    ui.add_space(4.);
                    body(ui);
                    ui.add_space(5.);
                });
                if self.variant == DisclosureVariant::Light {
                    crate::layout::Separator::Horizontal.show(ui);
                }
                response
            })
            .inner
    }
}

/// Whether expanded accordion sections are exclusive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccordionMode {
    /// At most one open section.
    #[default]
    Single,
    /// Independent section expansion.
    Multiple,
}

/// A set of animated disclosures controlled by the caller.
pub struct Accordion {
    /// Stable UI-local identity.
    pub id: egui::IdSalt,
    /// Expanded section policy.
    pub mode: AccordionMode,
    /// Surface treatment used by all sections.
    pub variant: DisclosureVariant,
}

impl Accordion {
    /// Create a single-open bordered accordion.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: egui::IdSalt::new(id),
            mode: AccordionMode::Single,
            variant: DisclosureVariant::Bordered,
        }
    }

    /// Render indexed sections and normalize the caller-owned open set.
    pub fn show(
        &self,
        ui: &mut Ui,
        titles: &[&str],
        open: &mut BTreeSet<usize>,
        mut body: impl FnMut(&mut Ui, usize),
    ) {
        open.retain(|&i| i < titles.len());
        if self.mode == AccordionMode::Single
            && let Some(&first) = open.first()
        {
            open.retain(|&i| i == first);
        }
        for (index, title) in titles.iter().enumerate() {
            let was_open = open.contains(&index);
            let mut expanded = was_open;
            Disclosure {
                id: egui::IdSalt::new((self.id, index)),
                title,
                variant: self.variant,
            }
            .show(ui, &mut expanded, |ui| body(ui, index));
            if expanded != was_open {
                if expanded {
                    if self.mode == AccordionMode::Single {
                        open.clear();
                    }
                    open.insert(index);
                } else {
                    open.remove(&index);
                }
                ui.ctx().request_repaint();
            }
        }
    }
}
