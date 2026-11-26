use std::collections::BTreeSet;

use eframe::egui::{Align, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    display::{
        Avatar, AvatarGroup, AvatarStatus, Badge, Chip, CodeBlock, DescriptionItem,
        DescriptionList, DiffKind, DiffLine, DiffView, Tone,
    },
    icon::Icon,
    layout::{Card, ResponsiveGrid},
    navigation::{Accordion, Breadcrumbs, DisclosureVariant, Tab, Tabs, TabsVariant},
    table::{Column, DataTable, Density, TableRows, TableState},
    typography::Type,
};

use crate::widgets;

const TEAMS: [&str; 5] = ["Design", "Engineering", "Operations", "Finance", "Support"];
const PROJECTS: [&str; 8] = [
    "Beacon",
    "Atlas",
    "Northwind",
    "Cedar",
    "Harbor",
    "Mosaic",
    "Relay",
    "Summit",
];
const OWNERS: [&str; 6] = [
    "Avery Chen",
    "Mina Ford",
    "Jordan Reed",
    "Leila Diaz",
    "Sam Patel",
    "Kai Morgan",
];
const STATUS: [(&str, Tone); 4] = [
    ("Active", Tone::Success),
    ("In review", Tone::Accent),
    ("Paused", Tone::Warning),
    ("Archived", Tone::Neutral),
];
const COUNT: usize = 10_000;

pub(super) struct DataDemo {
    table: TableState,
    query: String,
    revision: u64,
    density: Option<String>,
    paginated: bool,
    loading: bool,
    tabs: [usize; 3],
    accordion: BTreeSet<usize>,
    notice: Option<String>,
    chips: [bool; 2],
}

impl Default for DataDemo {
    fn default() -> Self {
        Self {
            table: TableState::default(),
            query: String::new(),
            revision: 0,
            density: Some("Compact".to_owned()),
            paginated: true,
            loading: false,
            tabs: [0, 1, 2],
            accordion: BTreeSet::from([0]),
            notice: None,
            chips: [false; 2],
        }
    }
}

impl DataDemo {
    pub(super) fn show(&mut self, ui: &mut Ui) -> bool {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 8.;
            self.content(ui)
        })
        .inner
    }

    fn content(&mut self, ui: &mut Ui) -> bool {
        let c = Theme::current(ui.ctx()).colors;
        let mut open = false;
        widgets::toolbar(
            ui,
            595.,
            |ui| {
                let crumb = Breadcrumbs {
                    max_visible: Some(4),
                    ..Breadcrumbs::new(&[
                        "Library",
                        "Components",
                        "Patterns",
                        "Data display",
                        "Records",
                    ])
                };
                if let Some(index) = crumb.show(ui) {
                    self.notice = Some(format!("Selected {}.", crumb.items[index]));
                }
            },
            |ui| {
                open = Button::new("Open command palette")
                    .leading(Icon::Command)
                    .variant(Variant::Secondary)
                    .size(Size::Sm)
                    .show(ui)
                    .clicked();
            },
        );
        ui.horizontal(|ui| {
            ui.label(Type::H2.text("Project records"));
            Badge {
                tone: Tone::Accent,
                ..Badge::new("10,000 rows")
            }
            .show(ui);
            if let Some(note) = &self.notice {
                ui.label(Type::Small.text(note).color(c.text_secondary));
            }
        });
        self.show_table(ui);
        ResponsiveGrid::new(320.)
            .max_columns(2)
            .show(ui, 2, |ui, index| {
                if index == 0 {
                    self.navigation(ui);
                } else {
                    source(ui);
                }
            });
        open
    }

    fn show_table(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            widgets::sized_width(ui, 245., 34., |ui| {
                if widgets::search(ui, &mut self.query, "Search project records…").changed() {
                    self.revision = self.revision.wrapping_add(1);
                    self.table.page = 0;
                }
            });
            widgets::sized_width(ui, 154., 34., |ui| {
                widgets::select(
                    ui,
                    &mut self.density,
                    &["Compact", "Comfortable", "Spacious"],
                    "Density",
                );
            });
            ui.checkbox(&mut self.paginated, "Pagination");
            ui.checkbox(&mut self.loading, "Loading");
        });
        let columns = [
            Column::new("Project", 1.3),
            Column::new("Owner", 1.0),
            Column::new("Team", 0.85),
            Column::new("Status", 0.85),
            Column::new("Budget", 0.7).align(Align::Max),
        ];
        let query = self.query.trim().to_lowercase();
        let rows = TableRows::new(
            self.revision,
            COUNT,
            |i| {
                query.is_empty()
                    || [&project(i), owner(i), team(i), status(i).0]
                        .iter()
                        .any(|text| text.to_lowercase().contains(&query))
            },
            |i| 1001 + i as u64,
            |column, a, b| match column {
                0 => project(a).cmp(&project(b)),
                1 => owner(a).cmp(owner(b)),
                2 => team(a).cmp(team(b)),
                3 => status(a).0.cmp(status(b).0),
                4 => budget(a).cmp(&budget(b)),
                _ => a.cmp(&b),
            },
        );
        let table = DataTable {
            density: match self.density.as_deref() {
                Some("Spacious") => Density::Spacious,
                Some("Comfortable") => Density::Comfortable,
                _ => Density::Compact,
            },
            height: 170.,
            selectable: true,
            loading: self.loading,
            page_size: self.paginated.then_some(250),
            bulk_actions: &["Copy IDs"],
            ..DataTable::new("project-records", &columns)
        };
        let response = table.show(ui, &mut self.table, rows, |ui, i, column| match column {
            0 => {
                ui.label(Type::Label.text(project(i)));
            }
            1 => {
                Avatar {
                    size: 21,
                    tone: [Tone::Accent, Tone::Success, Tone::Warning, Tone::Neutral][i % 4],
                    ..Avatar::new(owner(i))
                }
                .show(ui);
                ui.add(eframe::egui::Label::new(Type::Small.text(owner(i))).truncate());
            }
            2 => {
                ui.label(Type::Small.text(team(i)));
            }
            3 => {
                let (label, tone) = status(i);
                Badge {
                    tone,
                    ..Badge::new(label)
                }
                .show(ui);
            }
            _ => {
                ui.label(Type::Mono.text(format!("${},000", budget(i))));
            }
        });
        if response.bulk_action == Some(0) {
            let ids = self
                .table
                .selected()
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            ui.ctx().copy_text(ids);
            self.notice = Some(format!(
                "Copied {} project IDs.",
                self.table.selected().len()
            ));
        }
    }

    fn navigation(&mut self, ui: &mut Ui) {
        Card::default().padding(14).show(ui, |card| {
            card.body(|ui| {
                ui.spacing_mut().item_spacing.y = 6.;
                ui.horizontal_wrapped(|ui| {
                    ui.label(Type::H3.text("Navigation"));
                    members(ui);
                });
                let tabs = [
                    Tab {
                        icon: Some(Icon::List.into()),
                        ..Tab::new("Overview")
                    },
                    Tab {
                        count: Some(8),
                        ..Tab::new("Activity")
                    },
                    Tab::new("Settings"),
                ];
                for (i, variant) in [
                    TabsVariant::Pill,
                    TabsVariant::Underline,
                    TabsVariant::Solid,
                ]
                .into_iter()
                .enumerate()
                {
                    Tabs {
                        variant,
                        ..Tabs::new(("tab-style", i))
                    }
                    .show(ui, &tabs, &mut self.tabs[i]);
                }
                let accordion = Accordion {
                    variant: DisclosureVariant::Splitted,
                    ..Accordion::new("record-details")
                };
                accordion.show(
                    ui,
                    &["Project details", "Collaborators"],
                    &mut self.accordion,
                    |ui, index| {
                        if index == 0 {
                            DescriptionList {
                                items: &[
                                    DescriptionItem {
                                        label: "Reference",
                                        value: "PRJ-1001",
                                        mono: true,
                                        copy: true,
                                    },
                                    DescriptionItem {
                                        label: "Team",
                                        value: "Design",
                                        mono: false,
                                        copy: false,
                                    },
                                ],
                            }
                            .show(ui);
                        } else {
                            members(ui);
                        }
                    },
                );
                ui.horizontal_wrapped(|ui| {
                    for (i, label) in ["Client work", "Internal"].into_iter().enumerate() {
                        let mut chip = Chip::new(label);
                        chip.badge.tone = Tone::Accent;
                        chip.selectable = true;
                        chip.selected = self.chips[i];
                        if chip.show(ui).toggled {
                            self.chips[i] = !self.chips[i];
                        }
                    }
                });
            });
        });
    }
}

fn source(ui: &mut Ui) {
    Card::default().padding(14).show(ui, |card| {
        card.header("Source & changes", None);
        card.body(|ui| {
            CodeBlock { language: Some("Rust"), ..CodeBlock::new("let rows = projects.len();\nlet table = DataTable::new(\"projects\", &columns);") }.show(ui);
            DiffView { language: Some("Diff"), copy: true, line_numbers: true, lines: &[
                DiffLine { kind: DiffKind::Context, text: "table.selectable = true;" },
                DiffLine { kind: DiffKind::Removed, text: "table.page_size = Some(50);" },
                DiffLine { kind: DiffKind::Added, text: "table.page_size = Some(250);" },
            ] }.show(ui);
        });
    });
}

fn members(ui: &mut Ui) {
    let avatars = OWNERS.map(Avatar::new);
    let mut avatars = avatars;
    for (i, avatar) in avatars.iter_mut().enumerate() {
        avatar.tone = [Tone::Accent, Tone::Success, Tone::Warning, Tone::Danger][i % 4];
        avatar.status = (i == 1).then_some(AvatarStatus::Online);
    }
    AvatarGroup {
        size: 26,
        limit: 3,
        ..AvatarGroup::new(&avatars)
    }
    .show(ui);
}

fn project(i: usize) -> String {
    format!("{} #{:04}", PROJECTS[i % PROJECTS.len()], 1001 + i)
}
fn owner(i: usize) -> &'static str {
    OWNERS[i % OWNERS.len()]
}
fn team(i: usize) -> &'static str {
    TEAMS[i % TEAMS.len()]
}
fn status(i: usize) -> (&'static str, Tone) {
    STATUS[i % STATUS.len()]
}
fn budget(i: usize) -> usize {
    14 + (i * 17) % 110
}
