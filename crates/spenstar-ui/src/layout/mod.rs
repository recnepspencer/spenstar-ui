//! Dashboard shell and responsive layout primitives.
mod card;
mod grid;
mod header;
mod scroll;
mod shell;
mod sidebar;
mod sidebar_row;
mod split;

pub use card::{Card, CardBody, CardVariant, Surface};
pub use grid::ResponsiveGrid;
pub use header::{Navbar, PageHeader, SearchTrigger};
pub use scroll::ScrollShadow;
pub use shell::{AppShell, Inspector, MainContent};
pub use sidebar::{
    CollapseMode, MenuItem, MenuSection, Sidebar, SidebarEvent, SidebarPage, SidebarVariant,
};
pub use split::{SplitAxis, SplitPane};

use egui::{Align, InnerResponse, Layout, Sense, Stroke, Ui, pos2, vec2};

use crate::{Theme, button::Button, icon::IconSource, typography::Type};

/// An aligned horizontal collection of actions.
pub struct Toolbar;

impl Toolbar {
    /// Render a row with standard action spacing.
    pub fn show<R>(ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.;
            contents(ui)
        })
    }
}

/// A section divider.
pub enum Separator<'a> {
    /// Horizontal rule.
    Horizontal,
    /// Vertical rule of the given height in points.
    Vertical(f32),
    /// Horizontal rule with an inline label.
    Labeled(&'a str),
}

impl Separator<'_> {
    /// Render the divider in the current UI.
    pub fn show(self, ui: &mut Ui) {
        let c = Theme::current(ui.ctx()).colors;
        let stroke = Stroke::new(1., c.divider);
        match self {
            Self::Horizontal => {
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 9.), Sense::hover());
                ui.painter().hline(rect.x_range(), rect.center().y, stroke);
            }
            Self::Vertical(height) => {
                let (rect, _) = ui.allocate_exact_size(vec2(9., height.max(0.)), Sense::hover());
                ui.painter().vline(rect.center().x, rect.y_range(), stroke);
            }
            Self::Labeled(label) => {
                ui.horizontal(|ui| {
                    ui.label(Type::Caption.text(label).color(c.text_secondary));
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 9.), Sense::hover());
                    ui.painter().line_segment(
                        [
                            pos2(rect.left(), rect.center().y),
                            pos2(rect.right(), rect.center().y),
                        ],
                        stroke,
                    );
                });
            }
        }
    }
}

/// A compact message and actions for an empty view.
pub struct EmptyState<'a> {
    icon: IconSource,
    title: &'a str,
    description: &'a str,
    primary: Option<Button>,
    secondary: Option<Button>,
}

impl<'a> EmptyState<'a> {
    /// Create an empty view with an icon, heading, and descriptive text.
    pub fn new(icon: impl Into<IconSource>, title: &'a str, description: &'a str) -> Self {
        Self {
            icon: icon.into(),
            title,
            description,
            primary: None,
            secondary: None,
        }
    }

    /// Add a primary action.
    pub fn primary(mut self, button: Button) -> Self {
        self.primary = Some(button);
        self
    }

    /// Add a secondary action.
    pub fn secondary(mut self, button: Button) -> Self {
        self.secondary = Some(button);
        self
    }

    /// Render and return the optional primary and secondary responses.
    pub fn show(self, ui: &mut Ui) -> (Option<egui::Response>, Option<egui::Response>) {
        let c = Theme::current(ui.ctx()).colors;
        ui.vertical_centered(|ui| {
            ui.add_space(12.);
            self.icon.show(ui, 28., c.accent.ring);
            ui.add_space(4.);
            ui.label(Type::H3.text(self.title));
            ui.label(Type::Small.text(self.description).color(c.text_secondary));
            ui.add_space(8.);
            let actions = [&self.primary, &self.secondary];
            let count = actions.iter().filter(|button| button.is_some()).count();
            let responses = if count == 0 {
                (None, None)
            } else {
                let width = actions
                    .into_iter()
                    .flatten()
                    .map(|button| button.natural_width(ui))
                    .sum::<f32>();
                let width = width + count.saturating_sub(1) as f32 * ui.spacing().item_spacing.x;
                let size = vec2(
                    width.min(ui.available_width()),
                    ui.spacing().interact_size.y,
                );
                let layout = Layout::left_to_right(Align::Center).with_main_wrap(true);
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    (
                        self.primary.map(|button| button.show(ui)),
                        self.secondary.map(|button| button.show(ui)),
                    )
                })
                .inner
            };
            ui.add_space(12.);
            responses
        })
        .inner
    }
}
