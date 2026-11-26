use eframe::egui::{
    self, Align, Align2, Label, Layout, Rect, ScrollArea, Sense, Stroke, Ui, UiBuilder, pos2, vec2,
};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    feedback::{Spinner, Tone},
    icon::{Icon, IconSource, SvgIcon},
    overlay::{ContextMenu, DropdownMenu, MenuItem, Tooltip},
    typography::Type,
};

use super::threads::{Status, Thread};
use super::{Workbench, WorkbenchEvents, WorkbenchPalette, action, notify, pane, rgb};

pub(super) const PROJECTS: [(&str, &str); 4] = [
    ("WO", "worth-orchestration"),
    ("SU", "spenstar-ui"),
    ("WW", "work pc: worth"),
    ("WS", "workshop"),
];

impl Workbench {
    pub(super) fn rail(&mut self, ui: &mut Ui, p: WorkbenchPalette, events: &mut WorkbenchEvents) {
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0, p.rail);
        ui.painter().vline(
            rect.right() - 0.5,
            rect.y_range(),
            Stroke::new(1., p.border),
        );
        let content = rect.shrink2(vec2(8., 8.));
        let mut top = pane(
            ui,
            "top",
            Rect::from_min_max(content.min, pos2(content.right(), content.top() + 88.)),
        );
        rail_header(&mut top, self, p, events);
        let body = Rect::from_min_max(
            pos2(content.left(), content.top() + 96.),
            pos2(content.right(), content.bottom() - 40.),
        );
        ScrollArea::vertical()
            .id_salt("threads")
            .show(&mut pane(ui, "rows", body), |ui| {
                for (group, (mark, name)) in PROJECTS.into_iter().enumerate() {
                    if project(ui, p, mark, name, self.threads.expanded[group]).clicked() {
                        self.threads.expanded[group] = !self.threads.expanded[group];
                    }
                    if !self.threads.expanded[group] {
                        continue;
                    }
                    let mut rows: Vec<_> = self
                        .threads
                        .rows
                        .iter()
                        .filter(|row| {
                            row.group == group
                                && (!self.threads.only_working || row.status == Status::Working)
                        })
                        .collect();
                    if self.threads.reversed {
                        rows.reverse();
                    }
                    let mut event = None;
                    for row in rows {
                        let response = thread_row(ui, p, row, self.threads.selected == row.id);
                        let actions = [
                            MenuItem::action("rename", "Rename").leading(Icon::Pencil),
                            MenuItem::action("fork", "Fork thread").leading(Icon::GitBranch),
                            MenuItem::action("copy", "Copy link")
                                .leading(Icon::Link)
                                .shortcut("⌘ C"),
                            MenuItem::Separator,
                            MenuItem::action("archive", "Archive")
                                .leading(Icon::Inbox)
                                .destructive(),
                        ];
                        if let Some(command) =
                            ContextMenu::new(("workbench-thread", row.id)).show(&response, &actions)
                        {
                            event = Some((row.id, command));
                        } else if response.clicked() {
                            event = Some((row.id, "select"));
                        }
                    }
                    if let Some((id, command)) = event {
                        if command == "select" {
                            self.threads.select(id);
                        } else {
                            self.threads.context_action(ui.ctx(), id, command);
                        }
                    }
                    ui.add_space(5.);
                }
            });
        let bottom = Rect::from_min_max(pos2(content.left(), content.bottom() - 34.), content.max);
        pane(ui, "utilities", bottom).horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.;
            if action(ui, "Settings and gallery", Icon::Settings).clicked() {
                events.gallery = true;
            }
            if action(ui, "Toggle theme", Icon::Sun).clicked() {
                events.theme = true;
            }
            if action(ui, "Activity", Icon::Activity).clicked() {
                notify(ui.ctx(), "You’re all caught up", Tone::Neutral);
            }
            let upgrade = Button::new("Upgrade")
                .trailing(Icon::ArrowUpRight)
                .variant(Variant::Ghost)
                .size(Size::Sm)
                .show(ui);
            if upgrade.clicked() {
                notify(ui.ctx(), "You’re viewing the Workbench demo", Tone::Neutral);
            }
        });
    }
}

fn rail_header(
    ui: &mut Ui,
    workbench: &mut Workbench,
    p: WorkbenchPalette,
    events: &mut WorkbenchEvents,
) {
    const MARK: SvgIcon = SvgIcon::from_static(
        "spenstar-workbench-mark",
        include_str!("../../assets/spenstar-mark.svg"),
    );
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 1.;
        if action(ui, "Hide project rail", Icon::PanelLeft).clicked() {
            workbench.rail = false;
            workbench.compact_rail = false;
        }
        IconSource::from(MARK).show(ui, 20., p.ink);
        ui.label(Type::Brand.text("Spenstar Code").color(p.ink));
        if action(ui, "Previous thread", Icon::ArrowLeft).clicked() {
            workbench.threads.history(false);
        }
        if action(ui, "Next thread", Icon::ArrowRight).clicked() {
            workbench.threads.history(true);
        }
    });
    ui.add_space(8.);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.;
        let search = Button::new("Search")
            .leading(Icon::Search)
            .variant(Variant::Secondary)
            .size(Size::Sm)
            .min_width(108.)
            .show(ui);
        Tooltip::new("Search gallery pages")
            .shortcut("⌘ K")
            .show(&search);
        events.commands |= search.clicked();
        let filter = action(ui, "Filter threads", Icon::Filter);
        let filters = [
            MenuItem::action("all", "All threads").checked(!workbench.threads.only_working),
            MenuItem::action("working", "Working only").checked(workbench.threads.only_working),
        ];
        if let Some(filter) = DropdownMenu::new("workbench-filter").show(&filter, &filters) {
            workbench.threads.only_working = filter == "working";
        }
        if action(ui, "Reverse thread order", Icon::ArrowUpDown).clicked() {
            workbench.threads.reversed = !workbench.threads.reversed;
        }
        if action(ui, "New thread", Icon::Plus).clicked() {
            workbench
                .threads
                .add(workbench.threads.active().group, "New thread".into());
        }
    });
}

pub(super) fn monogram(ui: &mut Ui, mark: &str, p: WorkbenchPalette) {
    let (rect, _) = ui.allocate_exact_size(vec2(18., 18.), Sense::hover());
    ui.painter().rect_filled(rect, 4, p.active);
    let mut font = Type::Brand.font();
    font.size = 9.;
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, mark, font, p.ink);
}

fn project(
    ui: &mut Ui,
    p: WorkbenchPalette,
    mark: &str,
    name: &str,
    expanded: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 32.), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 6, p.active);
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink2(vec2(6., 0.)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    monogram(&mut child, mark, p);
    child.label(Type::Brand.text(name).color(p.ink));
    child.with_layout(Layout::right_to_left(Align::Center), |ui| {
        let icon = if expanded {
            Icon::ChevronDown
        } else {
            Icon::ChevronRight
        };
        icon.show(ui, 12., p.muted);
    });
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, expanded, name));
    response
}

fn thread_row(ui: &mut Ui, p: WorkbenchPalette, row: &Thread, selected: bool) -> egui::Response {
    ui.push_id(row.id, |ui| {
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 29.), Sense::click());
        if selected || response.hovered() {
            ui.painter().rect_filled(rect, 6, p.active);
        }
        let gutter = rect.shrink2(vec2(8., 0.));
        let mut content = ui.new_child(
            UiBuilder::new()
                .max_rect(gutter)
                .layout(Layout::right_to_left(Align::Center)),
        );
        let theme = Theme::current(ui.ctx());
        let dark = theme.mode == spenstar_ui::theme::Mode::Dark;
        let green = rgb(if dark { 0x75DB9F } else { 0x237346 });
        let blue = rgb(if dark { 0x80B8FF } else { 0x145CA5 });
        content.spacing_mut().item_spacing.x = 4.;
        match row.status {
            Status::Done => {
                content.label(Type::Caption.text("Done").color(green));
                Icon::Check.show(&mut content, 12., green);
            }
            Status::Working => {
                content.label(Type::Caption.text("Working").color(blue));
                Spinner::new(Size::Sm).show(&mut content);
            }
            Status::Waiting => {
                content.label(Type::Caption.text("Waiting").color(p.muted));
                Icon::MoreHorizontal.show(&mut content, 12., p.muted);
            }
            Status::Time(time) => {
                content.label(Type::Caption.text(time).color(p.muted));
            }
        }
        let title_rect = Rect::from_min_max(
            gutter.min + vec2(16., 0.),
            pos2(
                content.available_rect_before_wrap().right() - 3.,
                gutter.bottom(),
            ),
        );
        let mut title = ui.new_child(
            UiBuilder::new()
                .max_rect(title_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        title.set_clip_rect(ui.clip_rect().intersect(title_rect));
        title.add(
            Label::new(Type::Description.text(&row.title).color(if selected {
                p.ink
            } else {
                p.muted
            }))
            .selectable(false)
            .truncate(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::Button, true, selected, &row.title)
        });
        response
    })
    .inner
}
