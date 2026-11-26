use egui::{Align, Area, Context, Id, Layout, Rect, Response, ScrollArea, Sense, Ui, pos2, vec2};

use crate::{Theme, button::CloseButton, motion, typography::Type};

use super::{card, key, set_input_surface};

/// Output produced by an open modal or drawer.
pub struct PanelResponse<R, F> {
    /// Response of the dialog's card.
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
        geometry: Geometry,
        open: &mut bool,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui) -> F,
    ) -> PanelResponse<R, F> {
        let c = Theme::current(ctx).colors;
        let modal = egui::Modal::new(self.id)
            .area(area)
            .frame(card(ctx, 16, 18))
            .backdrop_color(c.scrim.gamma_multiply(geometry.scrim_opacity));
        let output = modal.show(ctx, |ui| {
            if !*open {
                ui.disable();
            }
            set_input_surface(ui);
            ui.set_width(geometry.width);
            let close = ui
                .horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width((geometry.width - 42.).max(1.));
                        ui.label(Type::H1.text(self.title));
                        if let Some(subtitle) = self.subtitle {
                            ui.label(Type::Description.text(subtitle).color(c.text_secondary));
                        }
                    });
                    CloseButton::show(ui).clicked()
                })
                .inner;
            ui.add_space(8.);
            let gap = ui.spacing().item_spacing.y;
            let remaining = (geometry.height - ui.min_size().y - gap - 4.).max(1.);
            let footer_height = if geometry.footer { 44. } else { 0. };
            let height = (remaining - footer_height - gap).max(1.);
            let body = scroll(self.id, ui, height, !geometry.fixed_height, body);
            if geometry.footer {
                ui.add_space(4.);
            }
            let footer = ui
                .allocate_ui_with_layout(
                    vec2(geometry.width, footer_height),
                    Layout::right_to_left(Align::Center),
                    footer,
                )
                .inner;
            if geometry.fixed_height {
                ui.expand_to_include_y(ui.min_rect().top() + geometry.height);
            }
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

fn scroll<R>(id: Id, ui: &mut Ui, height: f32, shrink: bool, body: impl FnOnce(&mut Ui) -> R) -> R {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), height),
        Layout::top_down(Align::Min),
        |ui| {
            ScrollArea::vertical()
                .id_salt(id.with("body"))
                .auto_shrink([false, shrink])
                .max_height(height)
                .show(ui, body)
                .inner
        },
    )
    .inner
}

#[derive(Clone, Copy)]
struct Geometry {
    width: f32,
    height: f32,
    fixed_height: bool,
    footer: bool,
    scrim_opacity: f32,
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
        let geometry = Geometry {
            width: dimension(self.width, rect.width() - 60.),
            height: (rect.height() - 60.).max(1.),
            fixed_height: false,
            footer: true,
            scrim_opacity: 1.,
        };
        Some(self.panel.show(ctx, area, geometry, open, body, footer))
    }
}

/// Viewport edge from which a drawer enters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerSide {
    /// Right edge.
    #[default]
    Right,
    /// Left edge.
    Left,
    /// Bottom edge.
    Bottom,
}

/// Slide-over dialog with an optional pinned footer.
pub struct Drawer<'a> {
    panel: Panel<'a>,
    side: DrawerSide,
    extent: f32,
}

impl<'a> Drawer<'a> {
    /// Create a drawer identified across frames.
    pub fn new(
        id: impl std::hash::Hash + std::fmt::Debug,
        title: &'a str,
        side: DrawerSide,
    ) -> Self {
        Self {
            panel: Panel {
                id: Id::unique(id),
                title,
                subtitle: None,
                outside: true,
            },
            side,
            extent: 440.,
        }
    }

    /// Set the desired outer width (left or right) or height (bottom).
    pub fn extent(mut self, extent: f32) -> Self {
        self.extent = extent;
        self
    }

    /// Add supporting text beneath the title.
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.panel.subtitle = Some(subtitle);
        self
    }

    /// Control whether the backdrop dismisses the drawer.
    pub fn dismiss_on_click_outside(mut self, dismiss: bool) -> Self {
        self.panel.outside = dismiss;
        self
    }

    /// Render a drawer without footer actions, including during its exit animation.
    pub fn show<R>(
        self,
        ctx: &Context,
        open: &mut bool,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<PanelResponse<R, ()>> {
        self.render(ctx, open, body, |_| {}, false)
    }

    /// Render the drawer with pinned footer actions, including during its exit animation.
    pub fn show_with_footer<R, F>(
        self,
        ctx: &Context,
        open: &mut bool,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui) -> F,
    ) -> Option<PanelResponse<R, F>> {
        self.render(ctx, open, body, footer, true)
    }

    fn render<R, F>(
        self,
        ctx: &Context,
        open: &mut bool,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui) -> F,
        has_footer: bool,
    ) -> Option<PanelResponse<R, F>> {
        let progress = self.progress(ctx, *open);
        if !*open && progress <= 0. {
            return None;
        }
        let (position, mut geometry) = self.placement(ctx.content_rect().shrink(10.), progress);
        geometry.footer = has_footer;
        let area = egui::Modal::default_area(self.panel.id)
            .kind(egui::UiKind::Modal)
            .anchor(
                egui::Align2::LEFT_TOP,
                position - ctx.content_rect().left_top(),
            )
            .sense(Sense::hover())
            .constrain(false)
            .fade_in(false);
        Some(self.panel.show(ctx, area, geometry, open, body, footer))
    }

    fn progress(&self, ctx: &Context, open: bool) -> f32 {
        let id = self.panel.id.with("slide");
        if ctx.data_mut(|data| {
            let known = data.get_temp::<bool>(id).is_some();
            data.insert_temp(id, true);
            !known
        }) {
            motion::eased(ctx, id, false, motion::NORMAL);
        }
        motion::eased(ctx, id, open, motion::NORMAL)
    }

    fn placement(&self, rect: Rect, progress: f32) -> (egui::Pos2, Geometry) {
        let (x, y, width, height) = match self.side {
            DrawerSide::Right => {
                let width = dimension(self.extent, rect.width());
                (
                    rect.right() - width * progress,
                    rect.top(),
                    width,
                    rect.height(),
                )
            }
            DrawerSide::Left => {
                let width = dimension(self.extent, rect.width());
                (
                    rect.left() - width * (1. - progress),
                    rect.top(),
                    width,
                    rect.height(),
                )
            }
            DrawerSide::Bottom => {
                let height = dimension(self.extent, rect.height());
                (
                    rect.left(),
                    rect.bottom() - height * progress,
                    rect.width(),
                    height,
                )
            }
        };
        (
            pos2(x, y),
            Geometry {
                width: (width - 38.).max(1.),
                height: (height - 38.).max(1.),
                fixed_height: true,
                footer: false,
                scrim_opacity: progress,
            },
        )
    }
}

fn dimension(wanted: f32, maximum: f32) -> f32 {
    if wanted.is_finite() { wanted } else { 432. }.clamp(1., maximum.max(1.))
}
