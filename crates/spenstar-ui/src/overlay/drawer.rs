use std::ops::RangeInclusive;

use egui::{
    Align, Context, CornerRadius, CursorIcon, Frame, Id, Layout, Rect, ScrollArea, Sense, Stroke,
    Ui, UiBuilder, Vec2, vec2,
};

use crate::{Theme, motion};

use super::{PanelResponse, key, panel::panel_header, set_input_surface};

/// Viewport edge from which a drawer enters.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub enum DrawerSide {
    /// Right edge.
    #[default]
    Right,
    /// Left edge.
    Left,
    /// Bottom edge.
    Bottom,
}

/// A resizable modal sheet docked to a viewport edge.
pub struct Drawer<'a> {
    id: Id,
    title: &'a str,
    subtitle: Option<&'a str>,
    side: DrawerSide,
    extent: f32,
    bounds: RangeInclusive<f32>,
    outside: bool,
}

impl<'a> Drawer<'a> {
    /// Create a drawer with a stable identity and default resize bounds.
    pub fn new(
        id: impl std::hash::Hash + std::fmt::Debug,
        title: &'a str,
        side: DrawerSide,
    ) -> Self {
        Self {
            id: Id::unique(id),
            title,
            subtitle: None,
            side,
            extent: 440.,
            bounds: if side == DrawerSide::Bottom {
                180. ..=600.
            } else {
                260. ..=720.
            },
            outside: true,
        }
    }

    /// Set the initial width or height. A dragged size takes precedence.
    pub fn extent(mut self, extent: f32) -> Self {
        self.extent = if extent.is_finite() { extent } else { 440. };
        self
    }

    /// Set minimum and maximum drag sizes in points. Equal bounds fix the size.
    /// A viewport smaller than the minimum takes precedence over either bound.
    pub fn resizable(mut self, bounds: RangeInclusive<f32>) -> Self {
        if bounds.start().is_finite() && bounds.end().is_finite() {
            let min = bounds.start().max(1.);
            self.bounds = min..=bounds.end().max(min);
        }
        self
    }

    /// Add supporting text beneath the title.
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Control whether the backdrop dismisses the drawer.
    pub fn dismiss_on_click_outside(mut self, dismiss: bool) -> Self {
        self.outside = dismiss;
        self
    }

    /// Render the drawer and its scrolling body, including during exit.
    pub fn show<R>(
        self,
        ctx: &Context,
        open: &mut bool,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<PanelResponse<R, ()>> {
        self.render(ctx, open, body, |_| {}, false)
    }

    /// Render the drawer with a sticky action footer, including during exit.
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
        let viewport = ctx.content_rect();
        let handle = self.id.with(("resize", self.side));
        let axis = usize::from(self.side == DrawerSide::Bottom);
        let extent = self.resolved_extent(ctx, handle, viewport.size()[axis], *open);
        let bounds = self.placement(viewport, extent, progress);
        let area = egui::Area::new(self.id)
            .kind(egui::UiKind::Modal)
            .order(egui::Order::Foreground)
            .fixed_pos(bounds.min)
            .default_size(bounds.size())
            .constrain(false)
            .sense(Sense::hover())
            .fade_in(false);
        let c = Theme::current(ctx).colors;
        let output = egui::Modal::new(self.id)
            .area(area)
            .frame(Frame::NONE)
            .backdrop_color(c.scrim.gamma_multiply(progress))
            .show(ctx, |ui| {
                if !*open {
                    ui.disable();
                }
                let (rect, _) = ui.allocate_exact_size(bounds.size(), Sense::hover());
                ui.painter().rect_filled(rect, self.corners(), c.overlay);
                self.draw_edge(ui, rect, handle);
                let mut content = ui.new_child(
                    UiBuilder::new()
                        .max_rect(rect.shrink2(vec2(
                            18.0_f32.min(rect.width() / 4.),
                            16.0_f32.min(rect.height() / 4.),
                        )))
                        .layout(Layout::top_down(Align::Min)),
                );
                content.shrink_clip_rect(rect);
                set_input_surface(&mut content);
                content.set_min_width(content.available_width());
                let close = panel_header(&mut content, self.title, self.subtitle);
                content.add_space(10.);
                let (body, footer) = Self::contents(&mut content, body, footer, has_footer);
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
        Some(PanelResponse {
            response: output.response,
            body,
            footer,
        })
    }

    fn contents<R, F>(
        ui: &mut Ui,
        body: impl FnOnce(&mut Ui) -> R,
        footer: impl FnOnce(&mut Ui) -> F,
        has_footer: bool,
    ) -> (R, F) {
        let width = ui.available_width();
        let gap = ui.spacing().item_spacing.y;
        let footer_height = if has_footer { 44. } else { 0. };
        let available = ui.available_height().max(0.);
        let body_height = (available - footer_height - gap).max(0.);
        let body = ScrollArea::vertical()
            .id_salt("body")
            .auto_shrink([false, false])
            .max_height(body_height)
            .show(ui, body)
            .inner;
        let footer = ui
            .allocate_ui_with_layout(
                vec2(width, footer_height),
                Layout::right_to_left(Align::Center),
                footer,
            )
            .inner;
        (body, footer)
    }

    fn resolved_extent(&self, ctx: &Context, handle: Id, space: f32, open: bool) -> f32 {
        let size_id = handle.with("extent");
        let max = self.bounds.end().min(space.max(1.));
        let min = self.bounds.start().min(max);
        let saved = ctx.memory_mut(|memory| memory.data.get_persisted::<f32>(size_id));
        let extent = saved.unwrap_or(self.extent).clamp(min, max);
        let dragging = open && ctx.current_pass_index() == 0;
        if dragging
            && let Some(response) = ctx.read_response(handle).filter(egui::Response::dragged)
        {
            let delta = response.drag_delta();
            let change = match self.side {
                DrawerSide::Left => delta.x,
                DrawerSide::Right => -delta.x,
                DrawerSide::Bottom => -delta.y,
            };
            let extent = (extent + change).clamp(min, max);
            ctx.memory_mut(|memory| memory.data.insert_persisted(size_id, extent));
            return extent;
        }
        extent
    }

    fn progress(&self, ctx: &Context, open: bool) -> f32 {
        let id = self.id.with("slide");
        if ctx.data_mut(|data| {
            let known = data.get_temp::<bool>(id).is_some();
            data.insert_temp(id, true);
            !known
        }) {
            motion::eased(ctx, id, false, motion::NORMAL);
        }
        motion::eased(ctx, id, open, motion::NORMAL)
    }

    fn placement(&self, viewport: Rect, extent: f32, progress: f32) -> Rect {
        let mut top_left = viewport.min;
        let size = match self.side {
            DrawerSide::Left => {
                top_left.x -= extent * (1. - progress);
                vec2(extent, viewport.height())
            }
            DrawerSide::Right => {
                top_left.x = viewport.right() - extent * progress;
                vec2(extent, viewport.height())
            }
            DrawerSide::Bottom => {
                top_left.y = viewport.bottom() - extent * progress;
                vec2(viewport.width(), extent)
            }
        };
        Rect::from_min_size(top_left, size.max(Vec2::ZERO))
    }

    fn corners(&self) -> CornerRadius {
        if self.side == DrawerSide::Bottom {
            CornerRadius {
                nw: 12,
                ne: 12,
                ..CornerRadius::ZERO
            }
        } else {
            CornerRadius::ZERO
        }
    }

    fn draw_edge(&self, ui: &Ui, rect: Rect, handle: Id) {
        let c = Theme::current(ui.ctx()).colors;
        let mut strip = rect;
        let (cursor, coordinate) = match self.side {
            DrawerSide::Left => {
                strip.min.x = (rect.right() - 10.).max(rect.left());
                (CursorIcon::ResizeHorizontal, rect.right() - 0.5)
            }
            DrawerSide::Right => {
                strip.max.x = (rect.left() + 10.).min(rect.right());
                (CursorIcon::ResizeHorizontal, rect.left() + 0.5)
            }
            DrawerSide::Bottom => {
                strip.max.y = (rect.top() + 10.).min(rect.bottom());
                (CursorIcon::ResizeVertical, rect.top() + 0.5)
            }
        };
        let active = if self.bounds.start() < self.bounds.end() {
            let response = ui
                .interact(strip, handle, Sense::drag())
                .on_hover_and_drag_cursor(cursor);
            response.hovered() || response.dragged()
        } else {
            false
        };
        let stroke = Stroke::new(
            1.,
            if active {
                c.accent.ring
            } else {
                c.overlay_border
            },
        );
        if self.side == DrawerSide::Bottom {
            let inset = (rect.width() * 0.5).min(12.);
            ui.painter().hline(
                rect.left() + inset..=rect.right() - inset,
                coordinate,
                stroke,
            );
        } else {
            ui.painter().vline(coordinate, rect.y_range(), stroke);
        }
    }
}
