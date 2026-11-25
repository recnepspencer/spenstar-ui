//! Color samples and selectable palettes.
use egui::{Color32, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2};

use crate::{
    Theme,
    icon::Icon,
    motion,
    theme::{luminance, mix},
};

/// An individual color sample.
#[derive(Clone, Copy, Debug)]
pub struct ColorSwatch {
    /// Paint color, including alpha.
    pub color: Color32,
    /// Sample diameter or square edge length.
    pub size: f32,
    /// Draw a circular chip.
    pub circular: bool,
    /// Draw the selected ring and check.
    pub selected: bool,
}

impl ColorSwatch {
    /// Make a rounded 28-point color sample.
    pub fn new(color: Color32) -> Self {
        Self {
            color,
            size: 28.,
            circular: false,
            selected: false,
        }
    }

    /// Render an informational sample as a button when selectable.
    pub fn show(self, ui: &mut Ui, selectable: bool) -> Response {
        let size = if self.size.is_finite() {
            self.size.clamp(12., 120.)
        } else {
            28.
        };
        let (rect, response) = ui.allocate_exact_size(
            vec2(size, size),
            if selectable {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let radius = if self.circular { (size / 2.) as u8 } else { 6 };
        let c = Theme::current(ui.ctx()).colors;
        if self.color.a() < 255 {
            paint_checker(ui, rect, f32::from(radius));
        }
        ui.painter().rect(
            rect,
            radius,
            self.color,
            Stroke::new(1., c.border),
            StrokeKind::Inside,
        );
        if self.selected {
            ui.painter().rect_stroke(
                rect.expand(2.),
                radius.saturating_add(2),
                Stroke::new(2., c.accent.ring),
                StrokeKind::Outside,
            );
            let [r, g, b, alpha] = self.color.to_srgba_unmultiplied();
            let background = mix(
                Color32::from_gray(190),
                Color32::from_rgb(r, g, b),
                f32::from(alpha) / 255.,
            );
            let ink = if luminance(background) > 0.179 {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            let _ = Icon::Check.svg().paint(
                ui,
                Rect::from_center_size(rect.center(), vec2(size * 0.58, size * 0.58)),
                ink,
            );
        } else if response.hovered() && selectable {
            ui.painter().rect_stroke(
                rect.expand(1.),
                radius,
                Stroke::new(1., c.accent.ring),
                StrokeKind::Outside,
            );
        }
        motion::focus_ring(ui, &response, radius.saturating_add(2).into());
        let [r, g, b, a] = self.color.to_srgba_unmultiplied();
        let label = if a == 255 {
            format!("#{r:02X}{g:02X}{b:02X}")
        } else {
            format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
        };
        response.widget_info(|| {
            if selectable {
                WidgetInfo::selected(Role::Button, response.enabled(), self.selected, &label)
            } else {
                WidgetInfo::labeled(Role::Image, response.enabled(), &label)
            }
        });
        response.on_hover_text(label)
    }
}

fn paint_checker(ui: &Ui, rect: Rect, radius: f32) {
    let light = Color32::from_gray(236);
    let dark = Color32::from_gray(172);
    let size = rect.width() / 6.;
    ui.painter().rect_filled(rect, radius as u8, light);
    // Clip the alternating tiles by horizontal chords of the rounded sample.
    for row in 0..6 {
        for column in 0..6 {
            if (row + column) % 2 == 0 {
                continue;
            }
            let top = rect.top() + row as f32 * size;
            let left = rect.left() + column as f32 * size;
            let strips = size.ceil() as usize;
            let height = size / strips as f32;
            for strip in 0..strips {
                let y = top + strip as f32 * height;
                let distance = (radius - (y - rect.top()).min(rect.bottom() - y - height)).max(0.);
                let inset = radius - (radius * radius - distance * distance).max(0.).sqrt();
                let x0 = left.max(rect.left() + inset);
                let x1 = (left + size).min(rect.right() - inset);
                if x1 > x0 {
                    ui.painter().rect_filled(
                        Rect::from_min_max(pos2(x0, y), pos2(x1, y + height)),
                        0,
                        dark,
                    );
                }
            }
        }
    }
}

/// A wrapping palette of selectable swatches.
#[derive(Clone, Copy, Debug, Default)]
pub struct ColorSwatchPicker {
    /// Use circular samples.
    pub circular: bool,
    /// Disable selection.
    pub disabled: bool,
}

impl ColorSwatchPicker {
    /// Render the palette and update the selected color. The returned response reports changes.
    pub fn show(self, ui: &mut Ui, selected: &mut Color32, colors: &[Color32]) -> Response {
        let mut out = ui.scope(|ui| {
            if self.disabled {
                ui.disable();
            }
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(10., 10.);
                let mut changed = false;
                for &color in colors {
                    let swatch = ColorSwatch {
                        circular: self.circular,
                        selected: color == *selected,
                        ..ColorSwatch::new(color)
                    };
                    if swatch.show(ui, true).clicked() && color != *selected {
                        *selected = color;
                        changed = true;
                    }
                }
                changed
            })
            .inner
        });
        if out.inner {
            out.response.mark_changed();
        }
        out.response
    }
}
