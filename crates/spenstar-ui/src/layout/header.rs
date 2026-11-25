use egui::{Align, Frame, InnerResponse, Layout, Margin, Response, Ui, vec2};

use crate::{
    Theme,
    button::{Button, Variant},
    icon::Icon,
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
                        .icon_only(Icon::Search)
                        .variant(Variant::Secondary)
                        .show(ui)
                })
            })
            .inner
        };
        let width = ui.available_width();
        let layout = Layout::left_to_right(Align::Center);
        if width < 440. {
            ui.allocate_ui_with_layout(vec2(width, 46.), layout, |ui| {
                ui.set_min_height(46.);
                breadcrumbs(ui);
            });
            ui.allocate_ui_with_layout(vec2(width, 36.), layout, actions)
                .inner
        } else {
            ui.allocate_ui_with_layout(vec2(width, 64.), layout, |ui| {
                ui.set_min_height(64.);
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
            .inner_margin(Margin::symmetric(4, 0))
            .show(ui, |ui| {
                Navbar::show(
                    ui,
                    self.search,
                    |ui| {
                        breadcrumbs(ui);
                        if let Some(subtitle) = self.subtitle {
                            let title = Type::H1.text(self.title);
                            let subtitle_text = Type::Small.text(subtitle).color(c.text_muted);
                            let width = ui
                                .fonts_mut(|fonts| {
                                    [(self.title, Type::H1), (subtitle, Type::Small)]
                                        .into_iter()
                                        .map(|(text, role)| {
                                            fonts
                                                .layout_no_wrap(
                                                    text.to_owned(),
                                                    role.font(),
                                                    c.text_primary,
                                                )
                                                .size()
                                                .x
                                        })
                                        .fold(0_f32, f32::max)
                                })
                                .min(ui.available_width());
                            ui.allocate_ui_with_layout(
                                vec2(width, 41.),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    ui.spacing_mut().item_spacing.y = 2.;
                                    ui.label(title);
                                    ui.label(subtitle_text);
                                },
                            );
                        } else {
                            ui.label(Type::H1.text(self.title));
                        }
                    },
                    actions,
                )
            })
    }
}
