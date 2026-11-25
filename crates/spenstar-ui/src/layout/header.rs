use egui::{Align, Frame, InnerResponse, Layout, Margin, Response, Stroke, Ui};

use crate::{
    Theme,
    button::{Button, Size, Variant},
    icon::Icon,
    theme::Radius,
    typography::Type,
};

/// A command or search trigger and its displayed keyboard hint.
pub struct SearchTrigger<'a> {
    /// Trigger title.
    pub label: &'a str,
    /// Optional, platform-appropriate shortcut hint.
    pub shortcut: Option<&'a str>,
}

/// The upper navigation row with breadcrumbs, optional search, and actions.
pub struct Navbar;

impl Navbar {
    /// Render the navigation row and return the search button response when present.
    pub fn show(
        ui: &mut Ui,
        search: Option<SearchTrigger<'_>>,
        breadcrumbs: impl FnOnce(&mut Ui),
        actions: impl FnOnce(&mut Ui),
    ) -> Option<Response> {
        let actions = |ui: &mut Ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                actions(ui);
                search.map(|search| {
                    let label = search.shortcut.map_or_else(
                        || search.label.to_owned(),
                        |hint| format!("{}     {hint}", search.label),
                    );
                    Button::new(label)
                        .radius(Radius::Full)
                        .leading(Icon::Search)
                        .size(Size::Sm)
                        .variant(Variant::Outline)
                        .show(ui)
                })
            })
            .inner
        };
        if ui.available_width() < 420. {
            ui.horizontal(breadcrumbs);
            ui.horizontal(actions).inner
        } else {
            ui.horizontal(|ui| {
                breadcrumbs(ui);
                actions(ui)
            })
            .inner
        }
    }
}

/// A dashboard page title, subtitle, and navigation row.
pub struct PageHeader<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    search: Option<SearchTrigger<'a>>,
}

impl<'a> PageHeader<'a> {
    /// Create a header with the given title.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            search: None,
        }
    }

    /// Add descriptive text beneath the title.
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Add a search or command trigger to the navigation row.
    pub fn search(mut self, search: SearchTrigger<'a>) -> Self {
        self.search = Some(search);
        self
    }

    /// Render a full-width header and expose the optional search response.
    pub fn show(
        self,
        ui: &mut Ui,
        breadcrumbs: impl FnOnce(&mut Ui),
        actions: impl FnOnce(&mut Ui),
    ) -> InnerResponse<Option<Response>> {
        let c = Theme::current(ui.ctx()).colors;
        Frame::new()
            .fill(c.surface)
            .stroke(Stroke::new(1., c.border_subtle))
            .inner_margin(Margin::symmetric(20, 12))
            .corner_radius(10)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                let search = Navbar::show(ui, self.search, breadcrumbs, actions);
                ui.add_space(8.);
                ui.label(Type::H1.text(self.title));
                if let Some(subtitle) = self.subtitle {
                    ui.label(Type::Body.text(subtitle).color(c.text_secondary));
                }
                search
            })
    }
}
