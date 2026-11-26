use egui::{Align, Area, Context, Id, Layout, Response, ScrollArea, Ui, Vec2, vec2};

use crate::{Theme, button::CloseButton, typography::Type};

use super::{card, key, set_input_surface};

/// Output produced by an open modal or drawer.
pub struct PanelResponse<R, F> {
    /// Response of the dialog or sheet surface.
    pub response: Response,
    /// Value returned by the scrollable body.
    pub body: R,
    /// Value returned by the pinned footer.
    pub footer: F,
}

struct Panel<'a> {
    id: Id,
    title: &'a str,
    subtitle: Option<&'a str>,
    outside: bool,
}

impl Panel<'_> {
    fn show<R, F>(
        &self,
        ctx: &Context,
        area: Area,
        size: Vec2,
        open: &mut bool,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui) -> F,
    ) -> PanelResponse<R, F> {
        let c = Theme::current(ctx).colors;
        let modal = egui::Modal::new(self.id)
            .area(area)
            .frame(card(ctx, 16, 18))
            .backdrop_color(c.scrim);
        let output = modal.show(ctx, |ui| {
            if !*open {
                ui.disable();
            }
            set_input_surface(ui);
            ui.set_width(size.x);
            let close = panel_header(ui, self.title, self.subtitle);
            ui.add_space(8.);
            let gap = ui.spacing().item_spacing.y;
            let remaining = (size.y - ui.min_size().y - gap - 4.).max(1.);
            let height = (remaining - 44. - gap).max(1.);
            let body = scroll(self.id, ui, height, body);
            ui.add_space(4.);
            let footer = ui
                .allocate_ui_with_layout(
                    vec2(size.x, 44.),
                    Layout::right_to_left(Align::Center),
                    footer,
                )
                .inner;
            (body, footer, close || key(ui, egui::Key::Escape))
        });
        let (body, footer, close) = output.inner;
        if *open
            && (close
                || output.response.should_close()
                || (self.outside && output.backdrop_response.clicked()))
        {
            *open = false;
            ctx.request_repaint();
        }
        PanelResponse {
            response: output.response,
            body,
            footer,
        }
    }
}

fn scroll<R>(id: Id, ui: &mut Ui, height: f32, body: impl FnOnce(&mut Ui) -> R) -> R {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), height),
        Layout::top_down(Align::Min),
        |ui| {
            ScrollArea::vertical()
                .id_salt(id.with("body"))
                .auto_shrink([false, true])
                .max_height(height)
                .show(ui, body)
                .inner
        },
    )
    .inner
}

/// Centered dialog with a scrollable body and caller-owned open state.
pub struct Modal<'a> {
    panel: Panel<'a>,
    width: f32,
}

impl<'a> Modal<'a> {
    /// Create a dialog with a stable identity and title.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug, title: &'a str) -> Self {
        Self {
            panel: Panel {
                id: Id::unique(id),
                title,
                subtitle: None,
                outside: true,
            },
            width: 432.,
        }
    }

    /// Add supporting text beneath the title.
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.panel.subtitle = Some(subtitle);
        self
    }

    /// Set the desired inner width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Control whether the backdrop dismisses the dialog.
    pub fn dismiss_on_click_outside(mut self, dismiss: bool) -> Self {
        self.panel.outside = dismiss;
        self
    }

    /// Render the dialog when open. The footer lays out actions from right to left.
    pub fn show<R, F>(
        self,
        ctx: &Context,
        open: &mut bool,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui) -> F,
    ) -> Option<PanelResponse<R, F>> {
        if !*open {
            return None;
        }
        let rect = ctx.content_rect();
        let area = egui::Modal::default_area(self.panel.id).fade_in(false);
        let size = vec2(
            dimension(self.width, rect.width() - 60.),
            (rect.height() - 60.).max(1.),
        );
        Some(self.panel.show(ctx, area, size, open, body, footer))
    }
}

fn dimension(wanted: f32, maximum: f32) -> f32 {
    if wanted.is_finite() { wanted } else { 432. }.clamp(1., maximum.max(1.))
}

pub(super) fn panel_header(ui: &mut Ui, title: &str, subtitle: Option<&str>) -> bool {
    let c = Theme::current(ui.ctx()).colors;
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 42.).max(1.);
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.label(Type::H1.text(title));
            if let Some(subtitle) = subtitle {
                ui.label(Type::Description.text(subtitle).color(c.text_secondary));
            }
        });
        CloseButton::show(ui).clicked()
    })
    .inner
}
