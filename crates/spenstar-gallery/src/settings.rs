use eframe::egui::{Align, Color32, Layout, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Size, ToggleButtonGroup, Variant},
    color::ColorSwatchPicker,
    controls::{Checkbox, RadioGroup, RadioItem, RadioVariant, Switch},
    form::{TextArea, TextField},
    icon::Icon,
    layout::{ResponsiveGrid, Separator},
    tags::TagInput,
    theme::Accent,
    typography::Type,
};

use crate::{forms::section, widgets};

#[derive(Clone, PartialEq)]
struct Profile {
    name: String,
    contact: String,
    public_email: bool,
    address: String,
    city: String,
    province: Option<String>,
    postal: String,
    currency: Option<String>,
    description: String,
    domains: Vec<String>,
    color: Color32,
    notifications: [bool; 3],
    plan: Option<usize>,
}

pub(super) struct Settings {
    saved: Profile,
    draft: Profile,
    domain_draft: String,
    saved_notice: bool,
    tab: usize,
}

impl Default for Settings {
    fn default() -> Self {
        let profile = Profile {
            name: "Spenstar Logistics".into(),
            contact: "alex@spenstar.example".into(),
            public_email: false,
            address: "4015 Meridian Parkway".into(),
            city: "Denver".into(),
            province: Some("Colorado".into()),
            postal: "80202".into(),
            currency: Some("USD · US Dollar".into()),
            description: "Regional distribution and last-mile delivery across the Mountain West."
                .into(),
            domains: vec!["spenstar.example".into(), "partners.example".into()],
            color: Accent::Blue.color(),
            notifications: [true, true, false],
            plan: Some(1),
        };
        Self {
            saved: profile.clone(),
            draft: profile,
            domain_draft: String::new(),
            saved_notice: false,
            tab: 0,
        }
    }
}

impl Settings {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        ToggleButtonGroup::new(
            "settings-section",
            ["Organization", "Notifications & billing"]
                .map(|label| Button::new(label).size(Size::Sm)),
        )
        .show(ui, &mut self.tab);
        if self.tab == 0 {
            self.profile(ui);
        } else {
            ResponsiveGrid::new(320.)
                .max_columns(2)
                .show(ui, 2, |ui, index| {
                    if index == 0 {
                        self.notifications(ui);
                    } else {
                        self.plan(ui);
                    }
                });
        }
        Separator::Horizontal.show(ui);
        let dirty = self.draft != self.saved;
        let valid = self.draft.contact.contains('@') && !self.draft.name.trim().is_empty();
        let note = if dirty {
            "You have unsaved changes"
        } else if self.saved_notice {
            "Workspace settings saved"
        } else {
            "All changes saved"
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(
                Type::Small
                    .text(note)
                    .color(Theme::current(ui.ctx()).colors.text_muted),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new("Save changes")
                    .leading(Icon::Check)
                    .enabled(valid)
                    .show(ui)
                    .clicked()
                {
                    self.saved = self.draft.clone();
                    self.saved_notice = true;
                    crate::widgets::notify(
                        ui.ctx(),
                        "Workspace settings saved",
                        spenstar_ui::feedback::Tone::Success,
                    );
                }
                if Button::new("Reset")
                    .variant(Variant::Ghost)
                    .enabled(dirty)
                    .show(ui)
                    .clicked()
                {
                    self.draft = self.saved.clone();
                    self.domain_draft.clear();
                    self.saved_notice = false;
                }
            });
        });
    }

    fn profile(&mut self, ui: &mut Ui) {
        let profile = &mut self.draft;
        setting(
            ui,
            "Organization name",
            "Displayed on your public profile and shipment records.",
            |ui| {
                let invalid = profile.name.trim().is_empty();
                TextField {
                    invalid,
                    ..TextField::new("Your organization")
                }
                .show(ui, &mut profile.name);
            },
        );
        setting(
            ui,
            "Organization bio",
            "Tell customers what your organization does.",
            |ui| {
                TextArea {
                    min_rows: 2,
                    max_rows: 2,
                    placeholder: "Describe your organization",
                    ..TextArea::default()
                }
                .show(ui, &mut profile.description);
            },
        );
        setting(
            ui,
            "Organization email",
            "A contact address customers can use for support.",
            |ui| {
                let invalid = !profile.contact.contains('@');
                TextField {
                    invalid,
                    ..TextField::new("info@example.com")
                }
                .show(ui, &mut profile.contact);
                Checkbox::new("Show email on public profile").show(ui, &mut profile.public_email);
            },
        );
        setting(
            ui,
            "Address",
            "This is where your organization is registered.",
            |ui| {
                TextField::new("Street address").show(ui, &mut profile.address);
                TextField::new("City").show(ui, &mut profile.city);
                ResponsiveGrid::new(125.)
                    .max_columns(2)
                    .show(ui, 2, |ui, index| {
                        if index == 0 {
                            widgets::select(
                                ui,
                                &mut profile.province,
                                &["Colorado", "Utah", "Wyoming", "New Mexico"],
                                "Province / State",
                            );
                        } else {
                            TextField::new("Postal code").show(ui, &mut profile.postal);
                        }
                    });
            },
        );
        setting(
            ui,
            "Currency",
            "Used in reports, orders, and invoices.",
            |ui| {
                widgets::select(
                    ui,
                    &mut profile.currency,
                    &["USD · US Dollar", "CAD · Canadian Dollar", "EUR · Euro"],
                    "Select currency",
                );
            },
        );
        setting(
            ui,
            "Workspace appearance",
            "Choose the brand accent for this workspace.",
            |ui| {
                ColorSwatchPicker {
                    circular: true,
                    disabled: false,
                }
                .show(ui, &mut profile.color, &Accent::PRESETS.map(Accent::color));
            },
        );
        setting(
            ui,
            "Allowed domains",
            "Press Enter to add a domain for workspace access.",
            |ui| {
                TagInput {
                    placeholder: "Add a domain",
                    ..TagInput::default()
                }
                .show(ui, &mut profile.domains, &mut self.domain_draft);
            },
        );
    }

    fn notifications(&mut self, ui: &mut Ui) {
        section(ui, "Notifications", |ui| {
            for (index, (label, description)) in [
                (
                    "Dispatch updates",
                    "Assignment changes and delivery confirmations",
                ),
                (
                    "Operational alerts",
                    "Delays, exceptions, and failed deliveries",
                ),
                ("Weekly digest", "A summary of activity every Monday"),
            ]
            .into_iter()
            .enumerate()
            {
                if index > 0 {
                    Separator::Horizontal.show(ui);
                }
                Switch {
                    description: Some(description),
                    ..Switch::new(label)
                }
                .show(ui, &mut self.draft.notifications[index]);
            }
        });
    }

    fn plan(&mut self, ui: &mut Ui) {
        section(ui, "Billing plan", |ui| {
            RadioGroup {
                variant: RadioVariant::Card,
                ..RadioGroup::default()
            }
            .show(
                ui,
                &mut self.draft.plan,
                &[
                    RadioItem {
                        description: Some("Core dispatch tools for small teams"),
                        icon: Some(Icon::Package.into()),
                        ..RadioItem::new("Starter")
                    },
                    RadioItem {
                        description: Some("Automation and reporting across locations"),
                        icon: Some(Icon::Building2.into()),
                        ..RadioItem::new("Operations")
                    },
                    RadioItem {
                        description: Some("Custom controls and dedicated support"),
                        icon: Some(Icon::Shield.into()),
                        ..RadioItem::new("Enterprise")
                    },
                ],
            );
        });
    }
}

fn setting(ui: &mut Ui, title: &str, description: &str, content: impl FnOnce(&mut Ui)) {
    let c = Theme::current(ui.ctx()).colors;
    let label = |ui: &mut Ui| {
        ui.spacing_mut().item_spacing.y = 4.;
        ui.label(Type::Label.text(title));
        ui.label(Type::Description.text(description).color(c.text_muted));
    };
    Separator::Horizontal.show(ui);
    let width = ui.available_width();
    if width >= 600. {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = width * 0.06;
            widgets::width(ui, width * 0.38, label);
            widgets::width(ui, width * 0.56, |ui| {
                ui.spacing_mut().item_spacing.y = 8.;
                content(ui);
            });
        });
    } else {
        ui.vertical(label);
        content(ui);
    }
}
