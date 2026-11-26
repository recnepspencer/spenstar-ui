use egui::{
    Align2, Color32, Frame, Key, Margin, Modifiers, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    pos2, vec2,
};

use crate::{
    Theme, display::Badge, icon::IconSource, input::lock_arrows, motion, theme::Mode,
    typography::Type,
};

/// A tab label with optional decoration.
#[derive(Clone, Debug)]
pub struct Tab<'a> {
    /// Visible title.
    pub label: &'a str,
    /// Leading icon.
    pub icon: Option<IconSource>,
    /// Trailing count.
    pub count: Option<usize>,
}

impl<'a> Tab<'a> {
    /// Create a plain tab.
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            count: None,
        }
    }
}

/// Tab bar treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    /// Raised button against a recessed track.
    #[default]
    Pill,
    /// Sliding accent underline.
    Underline,
    /// Color-filled active tab.
    Solid,
}

/// Tab bar density.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabSize {
    /// Compact tabs.
    #[default]
    Sm,
    /// Standard tabs.
    Md,
}

/// Controlled keyboard-navigable tab strip.
pub struct Tabs {
    /// Stable UI-local identity.
    pub id: egui::IdSalt,
    /// Tab bar treatment.
    pub variant: TabsVariant,
    /// Tab bar density.
    pub size: TabSize,
}

impl Tabs {
    /// Create a compact pill bar.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: egui::IdSalt::new(id),
            variant: TabsVariant::Pill,
            size: TabSize::Sm,
        }
    }

    /// Render the bar with a caller-owned active index.
    pub fn show(&self, ui: &mut Ui, tabs: &[Tab<'_>], active: &mut usize) -> Response {
        ui.push_id(self.id, |ui| self.draw(ui, tabs, active)).inner
    }

    fn draw(&self, ui: &mut Ui, tabs: &[Tab<'_>], active: &mut usize) -> Response {
        let theme = Theme::current(ui.ctx());
        let c = theme.colors;
        let pill = self.variant == TabsVariant::Pill;
        let old = *active;
        *active = (*active).min(tabs.len().saturating_sub(1));
        let mut out = Frame::new()
            .corner_radius(12)
            .inner_margin(Margin::same(if pill { 3 } else { 0 }))
            .fill(if pill {
                c.segmented_track
            } else {
                Color32::TRANSPARENT
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.;
                    let mut rows = Vec::with_capacity(tabs.len());
                    let mut destination = None;
                    for (index, tab) in tabs.iter().enumerate() {
                        let response = self.tab(ui, tab, *active == index, theme);
                        if response.clicked() {
                            *active = index;
                            response.request_focus();
                        }
                        lock_arrows(&response, true, false);
                        if response.has_focus() {
                            destination = keyboard(ui, index, tabs.len());
                        }
                        rows.push(response);
                    }
                    if let Some(i) = destination {
                        *active = i;
                        rows[i].request_focus();
                        lock_arrows(&rows[i], true, false);
                    }
                    rows.get(*active).map(|row| row.rect)
                })
                .inner
            });
        if self.variant == TabsVariant::Underline
            && let Some(tab) = out.inner
        {
            let origin = out.response.rect.left();
            let id = ui.make_persistent_id("tab-indicator");
            let x = ui.ctx().animate_value_with_time(
                id.with("left"),
                tab.left() - origin,
                motion::FAST,
            );
            let w = ui
                .ctx()
                .animate_value_with_time(id.with("width"), tab.width(), motion::FAST);
            let y = out.response.rect.bottom();
            ui.painter()
                .hline(out.response.rect.x_range(), y, Stroke::new(1., c.divider));
            ui.painter().rect_filled(
                Rect::from_min_max(pos2(origin + x, y - 1.), pos2(origin + x + w, y + 1.)),
                1,
                c.accent.base,
            );
        }
        if old != *active {
            out.response.mark_changed();
            ui.ctx().request_repaint();
        }
        out.response
    }

    fn tab(&self, ui: &mut Ui, tab: &Tab<'_>, selected: bool, theme: Theme) -> Response {
        let c = theme.colors;
        let (height, pad, font_size) = if self.size == TabSize::Sm {
            (31., 10., 12.)
        } else {
            (37., 13., 14.)
        };
        let solid = selected && self.variant == TabsVariant::Solid;
        let raised = selected && self.variant == TabsVariant::Pill;
        let ink = if solid {
            c.accent.foreground
        } else if selected {
            c.text_primary
        } else {
            c.text_secondary
        };
        let mut font = Type::Label.font();
        font.size = font_size;
        let painter = ui.painter();
        let label = painter.layout_no_wrap(tab.label.to_owned(), font, ink);
        let count = tab.count.map(|n| {
            let text = n.to_string();
            let (fill, ink) = Badge::new(&text).colors(theme, false);
            (fill, painter.layout_no_wrap(text, Type::Kbd.font(), ink))
        });
        let count_width = count.as_ref().map_or(0., |(_, label)| label.size().x + 17.);
        let icon_width = if tab.icon.is_some() { 20. } else { 0. };
        let width = pad * 2. + label.size().x + icon_width + count_width;
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        let painter = ui.painter();
        let (fill, border) = if solid {
            (c.accent.base, Stroke::NONE)
        } else if raised {
            let fill = if theme.mode == Mode::Light {
                Color32::WHITE
            } else {
                c.surface_elevated
            };
            (fill, Stroke::new(1., c.border_subtle))
        } else if response.hovered() {
            (c.field_hover, Stroke::NONE)
        } else {
            (Color32::TRANSPARENT, Stroke::NONE)
        };
        painter.rect(rect, 9, fill, border, StrokeKind::Inside);
        let mut left = rect.left() + pad;
        if let Some(icon) = &tab.icon {
            let icon_rect =
                Align2::LEFT_CENTER.anchor_size(pos2(left, rect.center().y), vec2(14., 14.));
            let _ = icon.paint(ui, icon_rect, ink);
            left += icon_width;
        }
        let center = rect.center().y;
        let label_size = label.size();
        painter.galley(pos2(left, center - label_size.y / 2.), label, ink);
        if let Some((fill, count)) = count {
            let size = count.size();
            let badge = Rect::from_center_size(
                pos2(left + label_size.x + 7. + (size.x + 10.) / 2., center),
                vec2(size.x + 10., 18.),
            );
            painter.rect_filled(badge, 9, fill);
            painter.galley(badge.center() - size / 2., count, c.text_primary);
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::Tab, response.enabled(), selected, tab.label)
        });
        motion::focus_ring(ui, &response, 9.into());
        response
    }
}

fn keyboard(ui: &Ui, current: usize, count: usize) -> Option<usize> {
    ui.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::ArrowLeft) {
            Some(current.checked_sub(1).unwrap_or(count - 1))
        } else if input.consume_key(Modifiers::NONE, Key::ArrowRight) {
            Some((current + 1) % count)
        } else if input.consume_key(Modifiers::NONE, Key::Home) {
            Some(0)
        } else if input.consume_key(Modifiers::NONE, Key::End) {
            Some(count - 1)
        } else {
            None
        }
    })
}
