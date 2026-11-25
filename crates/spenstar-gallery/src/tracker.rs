use crate::{orders::initials, widgets};
use eframe::egui::{Align, Frame, Layout, Margin, Rect, Sense, Stroke, Ui, vec2};
use spenstar_ui::theme::Mode;
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    controls::Tone,
    icon::Icon,
    layout::{Card, ResponsiveGrid},
    typography::Type,
};

struct Task {
    category: &'static str,
    title: &'static str,
    description: &'static str,
    date: &'static str,
    progress: (u8, u8),
    people: &'static [&'static str],
    stage: usize,
}

pub(super) struct Tracker {
    tasks: [Task; 5],
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            tasks: [
                Task {
                    category: "Engineering",
                    title: "Audit carrier webhooks",
                    description: "Check retries and missing delivery events.",
                    date: "Dec 12",
                    progress: (0, 5),
                    people: &["Avery Chen"],
                    stage: 0,
                },
                Task {
                    category: "Design",
                    title: "Update exception states",
                    description: "Review empty and error states across dispatch.",
                    date: "Dec 10",
                    progress: (1, 4),
                    people: &["Maya Ford", "Leila Diaz"],
                    stage: 0,
                },
                Task {
                    category: "Frontend",
                    title: "Improve form validation",
                    description: "Add inline feedback to the order form.",
                    date: "Dec 8",
                    progress: (3, 7),
                    people: &["Noah Reed"],
                    stage: 1,
                },
                Task {
                    category: "Product",
                    title: "Draft onboarding guide",
                    description: "Write the first-run checklist for coordinators.",
                    date: "Dec 14",
                    progress: (2, 5),
                    people: &["Sam Patel", "Maya Ford"],
                    stage: 1,
                },
                Task {
                    category: "Design",
                    title: "Publish navigation patterns",
                    description: "Document sidebar, toolbar, and split-pane layouts.",
                    date: "Dec 5",
                    progress: (4, 4),
                    people: &["Leila Diaz"],
                    stage: 2,
                },
            ],
        }
    }
}

const STAGES: [&str; 3] = ["To do", "In progress", "Done"];

impl Tracker {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let c = theme.colors;
        let well = if theme.mode == Mode::Light {
            c.surface_secondary
        } else {
            c.surface_subtle
        };
        ui.label(
            Type::Body
                .text("Track work across your team.")
                .color(c.text_muted),
        );
        let stages = [
            (c.accent, Icon::List),
            (c.warning, Icon::Clock),
            (c.success, Icon::CheckCircle2),
        ];
        let counts = STAGES.map(|stage| {
            self.tasks
                .iter()
                .filter(|task| STAGES[task.stage] == stage)
                .count()
        });
        Card::default().show(ui, |card| {
            card.body(|ui| {
                let width = ui.available_width();
                let grid = ResponsiveGrid::new(125.).max_columns(3);
                let columns = grid.columns(width);
                let bounds = ui.available_rect_before_wrap();
                if columns == 3 {
                    for split in [1., 2.] {
                        let x = bounds.left() + (width + 12.) * split / 3. - 6.;
                        ui.painter().vline(
                            x,
                            bounds.top()..=bounds.top() + 66.,
                            Stroke::new(1., c.divider),
                        );
                    }
                }
                grid.show(ui, 3, |ui, index| {
                    let (palette, icon) = stages[index];
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(30., 30.), Sense::hover());
                        ui.painter().rect_filled(rect, 9, palette.soft);
                        let _ = icon.svg().paint(
                            ui,
                            Rect::from_center_size(rect.center(), vec2(15., 15.)),
                            palette.ring,
                        );
                        ui.label(
                            Type::Small
                                .text(if index == 2 {
                                    "Completed"
                                } else {
                                    STAGES[index]
                                })
                                .color(c.text_muted),
                        );
                    });
                    ui.label(Type::Metric.text(counts[index].to_string()));
                });
            })
        });
        let mut move_task = None;
        ResponsiveGrid::new(220.)
            .max_columns(3)
            .show(ui, 3, |ui, stage| {
                ui.horizontal(|ui| {
                    let (dot, _) = ui.allocate_exact_size(vec2(12., 12.), Sense::hover());
                    ui.painter()
                        .circle_filled(dot.center(), 4., stages[stage].0.base);
                    ui.label(Type::Label.text(STAGES[stage]));
                    ui.label(
                        Type::Small
                            .text(counts[stage].to_string())
                            .color(c.text_muted),
                    );
                });
                Frame::new()
                    .inner_margin(Margin::same(8))
                    .corner_radius(16)
                    .fill(well)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.set_min_height(408.);
                        for (index, item) in self.tasks.iter().enumerate() {
                            if item.stage == stage && ui.push_id(index, |ui| task(ui, item)).inner {
                                move_task = Some(index);
                            }
                        }
                    });
            });
        if let Some(index) = move_task {
            let task = &mut self.tasks[index];
            task.stage += 1;
            if task.stage == 2 {
                task.progress.0 = task.progress.1;
            }
            ui.ctx().request_repaint();
        }
    }
}

fn task(ui: &mut Ui, task: &Task) -> bool {
    let c = Theme::current(ui.ctx()).colors;
    Card::default()
        .show(ui, |card| {
            card.body(|ui| {
                ui.spacing_mut().item_spacing.y = 8.;
                ui.horizontal(|ui| {
                    let tone = match task.category {
                        "Design" => Tone::Danger,
                        "Frontend" => Tone::Warning,
                        _ => Tone::Accent,
                    };
                    widgets::tag(ui, task.category, tone);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(Type::Caption.text(task.date).color(c.text_muted));
                        Icon::Clock.show(ui, 12., c.text_muted);
                    });
                });
                let title = Type::Label.text(task.title);
                ui.label(if task.stage == 2 {
                    title.strikethrough().color(c.text_muted)
                } else {
                    title
                });
                ui.label(Type::Description.text(task.description).color(c.text_muted));
                let (done, total) = task.progress;
                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 32.).max(1.);
                    let (rect, _) = ui.allocate_exact_size(vec2(width, 4.), Sense::hover());
                    ui.painter().rect_filled(rect, 2, c.surface_secondary);
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            rect.min,
                            vec2(width * f32::from(done) / f32::from(total), 4.),
                        ),
                        2,
                        c.accent.base,
                    );
                    ui.label(
                        Type::Caption
                            .text(format!("{done}/{total}"))
                            .color(c.text_muted),
                    );
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.;
                    for person in task.people {
                        let (rect, resp) = ui.allocate_exact_size(vec2(23., 23.), Sense::hover());
                        initials(ui, rect, person, c.default.base, c.text_secondary);
                        resp.on_hover_text(*person);
                    }
                    task.stage < 2
                        && ui
                            .with_layout(Layout::right_to_left(Align::Center), |ui| {
                                Button::new(format!("Move to {}", STAGES[task.stage + 1]))
                                    .icon_only(Icon::ArrowRight)
                                    .size(Size::Sm)
                                    .variant(Variant::Ghost)
                                    .show(ui)
                                    .clicked()
                            })
                            .inner
                })
                .inner
            })
        })
        .inner
}
