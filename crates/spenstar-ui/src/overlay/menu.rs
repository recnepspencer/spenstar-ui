use egui::{
    Context, Id, Key, PointerButton, Popup, PopupCloseBehavior, Pos2, Rect, RectAlign, Response,
    ScrollArea, Ui,
};

use super::{card, key, menu_row::row};

pub use super::menu_item::MenuItem;

/// Keyboard navigable menu placed below an anchor unless the viewport requires flipping.
pub struct DropdownMenu {
    id: Id,
    width: f32,
}

impl DropdownMenu {
    /// Create an anchored dropdown with a stable identity.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            id: Id::unique(id),
            width: 208.,
        }
    }

    /// Set the preferred inner width.
    pub fn width(mut self, points: f32) -> Self {
        self.width = points;
        self
    }

    /// Open programmatically, keeping the normal anchor behavior.
    pub fn open(&self, ctx: &Context) {
        self.open_at(ctx, None);
    }

    fn open_at(&self, ctx: &Context, point: Option<Pos2>) {
        ctx.data_mut(|data| {
            data.insert_temp(
                self.id,
                State {
                    point,
                    ..State::default()
                },
            )
        });
        ctx.memory_mut(|memory| {
            if let Some(focused) = memory.focused() {
                memory.surrender_focus(focused);
            }
        });
        ctx.request_repaint();
    }

    /// Close a menu opened with this identity.
    pub fn close(&self, ctx: &Context) {
        if ctx
            .data_mut(|data| data.remove_temp::<State>(self.id))
            .is_some()
        {
            ctx.request_repaint();
        }
    }

    /// Toggle on anchor activation, returning the activated command ID.
    pub fn show<'a>(self, anchor: &Response, items: &[MenuItem<'a>]) -> Option<&'a str> {
        let is_open = anchor
            .ctx
            .data(|data| data.get_temp::<State>(self.id).is_some());
        if is_open && anchor.clicked_by(PointerButton::Primary) {
            self.close(&anchor.ctx);
            return None;
        }
        if !is_open && anchor.clicked() {
            self.open(&anchor.ctx);
        }
        self.render(anchor, items)
    }

    fn render<'a>(self, anchor: &Response, items: &[MenuItem<'a>]) -> Option<&'a str> {
        let ctx = &anchor.ctx;
        let mut state = ctx.data(|data| data.get_temp::<State>(self.id))?;
        if !anchor.enabled() || !anchor.interact_rect.is_positive() {
            ctx.data_mut(|data| data.remove::<State>(self.id));
            ctx.request_repaint();
            return None;
        }
        let rect = ctx.content_rect();
        let width = self.width.min(rect.width() - 20.).max(1.);
        let mut frame = MenuFrame {
            width,
            height: (rect.height() - 20.).max(1.),
            ..Default::default()
        };
        let mut popup = Popup::from_response(anchor).id(self.id);
        if let Some(point) = state.point {
            popup = popup.at_position(point);
        }
        frame.level(popup, items, &mut state.level);
        let outside = ctx.input(|input| input.pointer.any_click())
            && ctx.pointer_interact_pos().is_some_and(|point| {
                let on_anchor = state.point.is_none() && anchor.interact_rect.contains(point);
                !on_anchor && !frame.rects.iter().any(|rect| rect.contains(point))
            })
            && !anchor.secondary_clicked();
        if frame.dismiss || frame.action.is_some() || outside {
            ctx.data_mut(|data| data.remove::<State>(self.id));
            if !outside {
                anchor.request_focus();
            }
            ctx.request_repaint();
        } else {
            ctx.data_mut(|data| data.insert_temp(self.id, state));
        }
        frame.action
    }
}

/// Menu opened by a secondary click at the pointer's position.
pub struct ContextMenu(DropdownMenu);

impl ContextMenu {
    /// Create a context menu with a stable identity.
    pub fn new(id: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self(DropdownMenu::new(id))
    }

    /// Attach to a response and return the activated command ID.
    pub fn show<'a>(self, target: &Response, items: &[MenuItem<'a>]) -> Option<&'a str> {
        if target.secondary_clicked() {
            let point = target
                .interact_pointer_pos()
                .or_else(|| target.ctx.pointer_interact_pos());
            self.0.open_at(&target.ctx, point);
        }
        self.0.render(target, items)
    }
}

#[derive(Clone, Default)]
struct State {
    point: Option<Pos2>,
    level: Selection,
}

#[derive(Clone, Default)]
struct Selection {
    index: Option<usize>,
    child: Option<(usize, Box<Selection>)>,
}

impl Selection {
    fn step(&mut self, items: &[MenuItem<'_>], reverse: bool) {
        let mut candidates: Vec<_> = (0..items.len())
            .filter(|&i| items[i].selectable())
            .collect();
        if reverse {
            candidates.reverse();
        }
        let next = candidates
            .iter()
            .position(|&i| Some(i) == self.index)
            .map_or(0, |i| i + 1);
        self.index = candidates.get(next).or_else(|| candidates.first()).copied();
    }

    fn open(&mut self, index: usize, children: &[MenuItem<'_>], keyboard: bool) {
        if self
            .child
            .as_ref()
            .is_none_or(|(active, _)| *active != index)
        {
            let mut child = Self::default();
            if keyboard {
                child.step(children, false);
            }
            self.child = Some((index, Box::new(child)));
        }
    }
}

#[derive(Default)]
struct MenuFrame<'a> {
    width: f32,
    height: f32,
    rects: Vec<Rect>,
    dismiss: bool,
    action: Option<&'a str>,
}

impl<'a> MenuFrame<'a> {
    fn level(
        &mut self,
        popup: Popup<'_>,
        items: &[MenuItem<'a>],
        selection: &mut Selection,
    ) -> bool {
        let width = self.width;
        let height = self.height;
        let frame = card(popup.ctx(), 12, 6);
        let id = popup.get_id();
        let mut back = false;
        let content = popup
            .width(width + 14.)
            .frame(frame)
            .gap(3.)
            .close_behavior(PopupCloseBehavior::IgnoreClicks)
            .show(|ui| {
                if selection.child.is_none() {
                    back = self.keyboard(ui, items, selection);
                }
                ui.set_width(width);
                ScrollArea::vertical()
                    .max_height(height)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 1.;
                        let mut child_anchor = None;
                        for (index, item) in items.iter().enumerate() {
                            ui.push_id(index, |ui| {
                                if let Some(response) =
                                    row(ui, item, selection.index == Some(index), width)
                                {
                                    self.pointer(ui, item, index, &response, selection);
                                    if selection
                                        .child
                                        .as_ref()
                                        .is_some_and(|(active, _)| *active == index)
                                    {
                                        child_anchor = Some(response);
                                    }
                                }
                            });
                        }
                        child_anchor
                    })
                    .inner
            });
        let Some(content) = content else { return back };
        self.rects.push(content.response.rect);
        if let Some((index, child)) = &mut selection.child {
            if let (Some(anchor), Some(MenuItem::Action { children, .. })) =
                (content.inner, items.get(*index))
            {
                let popup = Popup::from_response(&anchor)
                    .id(id.with(*index))
                    .align(RectAlign::RIGHT_START)
                    .align_alternatives(&[
                        RectAlign::LEFT_START,
                        RectAlign::RIGHT_END,
                        RectAlign::LEFT_END,
                    ]);
                if self.level(popup, children, child) {
                    selection.child = None;
                    anchor.ctx.request_repaint();
                }
            } else {
                selection.child = None;
            }
        }
        back
    }

    fn keyboard(&mut self, ui: &Ui, items: &[MenuItem<'a>], selection: &mut Selection) -> bool {
        if key(ui, Key::Escape) {
            self.dismiss = true;
        }
        if key(ui, Key::ArrowDown) {
            selection.step(items, false);
        } else if key(ui, Key::ArrowUp) {
            selection.step(items, true);
        }
        if key(ui, Key::ArrowLeft) {
            return true;
        }
        let enter = key(ui, Key::Enter);
        let right = key(ui, Key::ArrowRight);
        if let Some((
            index,
            MenuItem::Action {
                id,
                children,
                enabled: true,
                ..
            },
        )) = selection
            .index
            .and_then(|i| items.get(i).map(|item| (i, item)))
        {
            if (enter || right) && !children.is_empty() {
                selection.open(index, children, true);
            } else if enter {
                self.action = Some(id);
            }
        }
        false
    }

    fn pointer(
        &mut self,
        ui: &Ui,
        item: &MenuItem<'a>,
        index: usize,
        response: &Response,
        selection: &mut Selection,
    ) {
        let MenuItem::Action {
            id,
            children,
            enabled: true,
            ..
        } = item
        else {
            return;
        };
        let moved = ui.input(|input| input.pointer.delta() != egui::Vec2::ZERO);
        if response.hovered() && (moved || selection.index.is_none()) {
            selection.index = Some(index);
            if children.is_empty() {
                selection.child = None;
            } else {
                selection.open(index, children, false);
            }
        }
        if response.clicked() {
            selection.index = Some(index);
            if children.is_empty() {
                self.action = Some(id);
            } else {
                selection.open(index, children, true);
            }
        }
    }
}
