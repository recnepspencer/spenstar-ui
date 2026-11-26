use egui::{Response, Ui};

use crate::{
    Theme,
    button::{Button, Size, Variant},
    icon::{Icon, IconSource},
    typography::Type,
};

/// Breadcrumb separators.
#[derive(Clone, Copy, Debug, Default)]
pub enum BreadcrumbSeparator {
    /// Right chevrons.
    #[default]
    Chevron,
    /// Forward slashes.
    Slash,
}

/// A clickable path with optional collapse into a middle menu.
pub struct Breadcrumbs<'a> {
    /// Labels in order from the root.
    pub items: &'a [&'a str],
    /// Separator style.
    pub separator: BreadcrumbSeparator,
    /// Collapse the middle if more labels exist. A minimum of three slots is used.
    pub max_visible: Option<usize>,
}

impl<'a> Breadcrumbs<'a> {
    /// Make a chevron-separated trail.
    pub fn new(items: &'a [&'a str]) -> Self {
        Self {
            items,
            separator: BreadcrumbSeparator::Chevron,
            max_visible: None,
        }
    }

    /// Render the trail and return the activated path index.
    pub fn show(&self, ui: &mut Ui) -> Option<usize> {
        let mut selected = None;
        let c = Theme::current(ui.ctx()).colors;
        let count = self.items.len();
        let collapse = self.max_visible.map(|n| n.max(3)).filter(|&n| count > n);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.;
            for (index, label) in self.items.iter().enumerate() {
                if let Some(limit) = collapse {
                    let end = count - (limit - 2);
                    if index > 1 && index < end {
                        continue;
                    }
                    if index == 1 {
                        self.separate(ui);
                        let response = crumb(ui, "…").on_hover_text("Show parent pages");
                        egui::Popup::menu(&response).show(|ui| {
                            for i in 1..end {
                                if crumb(ui, self.items[i]).clicked() {
                                    selected = Some(i);
                                    ui.close();
                                }
                            }
                        });
                        continue;
                    }
                }
                if index > 0 {
                    self.separate(ui);
                }
                if index + 1 == count {
                    ui.label(Type::Label.text(*label).color(c.text_primary));
                } else if crumb(ui, label).clicked() {
                    selected = Some(index);
                }
            }
        });
        selected
    }

    fn separate(&self, ui: &mut Ui) {
        let ink = Theme::current(ui.ctx()).colors.text_muted;
        match self.separator {
            BreadcrumbSeparator::Chevron => {
                IconSource::from(Icon::ChevronRight).show(ui, 12., ink);
            }
            BreadcrumbSeparator::Slash => {
                ui.label(Type::Small.text("/").color(ink));
            }
        }
    }
}

fn crumb(ui: &mut Ui, label: &str) -> Response {
    Button::new(label)
        .size(Size::Sm)
        .variant(Variant::Ghost)
        .show(ui)
}
