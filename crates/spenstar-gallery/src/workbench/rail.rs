use eframe::egui::{
    self, Align, Align2, CursorIcon, Label, Layout, Rect, ScrollArea, Sense, Stroke, Ui, UiBuilder,
    pos2, vec2,
};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    feedback::{Spinner, Tone},
    icon::{Icon, IconSource, SvgIcon},
    overlay::{ContextMenu, DropdownMenu, MenuItem, Tooltip},
    typography::Type,
};

use super::{Workbench, WorkbenchEvents, WorkbenchPalette, action, notify, pane, rgb};
use super::{
    row,
    threads::{Status, Thread},
};

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
            .auto_shrink([false, false])
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
        let mut utilities = pane(ui, "utilities", bottom);
        row::split(
            &mut utilities,
            bottom,
            |ui| {
                let upgrade = Button::new("Upgrade")
                    .icon_only(Icon::ArrowUp)
                    .size(Size::Sm)
                    .variant(Variant::Outline)
                    .show(ui);
                Tooltip::new("Upgrade").show(&upgrade);
                if upgrade.clicked() {
                    notify(ui.ctx(), "You’re viewing the Workbench demo", Tone::Neutral);
                }
            },
            |ui| {
                if action(ui, "Settings and gallery", Icon::Settings).clicked() {
                    events.gallery = true;
                }
                if action(ui, "Toggle theme", Icon::Sun).clicked() {
                    events.theme = true;
                }
                if action(ui, "Activity", Icon::Activity).clicked() {
                    notify(ui.ctx(), "You’re all caught up", Tone::Neutral);
                }
            },
        );
    }

    pub(super) fn resize_rail(&mut self, ui: &Ui, rect: Rect, p: WorkbenchPalette) -> f32 {
        let id = ui.make_persistent_id("workbench-rail-divider");
        let max = (rect.width() - 300.).clamp(200., 340.);
        if ui.ctx().current_pass_index() == 0
            && let Some(response) = ui.ctx().read_response(id).filter(egui::Response::dragged)
        {
            self.rail_width = (self.rail_width.min(max) + response.drag_delta().x).clamp(200., max);
        }
        let width = self.rail_width.clamp(200., max);
        let x = rect.left() + width;
        let hit = Rect::from_min_max(pos2(x - 5., rect.top()), pos2(x + 5., rect.bottom()));
        let response = ui
            .interact(hit, id, Sense::drag())
            .on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);
        let color = if response.hovered() || response.dragged() {
            Theme::current(ui.ctx()).colors.accent.ring
        } else {
            p.border
        };
        // Center the stroke just outside the rail so its fill leaves the divider visible.
        let painter = ui.painter().with_clip_rect(rect);
        painter.vline(x + 0.5, rect.y_range(), Stroke::new(1., color));
        width
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
    let width = ui.available_width();
    let (top, _) = ui.allocate_exact_size(vec2(width, 32.), Sense::hover());
    row::split(
        ui,
        top,
        |ui| {
            if rail_icon(ui, p, "Next thread", Icon::ArrowRight).clicked() {
                workbench.threads.history(true);
            }
            if rail_icon(ui, p, "Previous thread", Icon::ArrowLeft).clicked() {
                workbench.threads.history(false);
            }
        },
        |ui| {
            if rail_icon(ui, p, "Hide project rail", Icon::PanelLeft).clicked() {
                workbench.rail = false;
                workbench.compact_rail = false;
            }
            ui.spacing_mut().item_spacing.x = 3.;
            IconSource::from(MARK).show(ui, 17., p.ink);
            ui.add(Label::new(Type::Brand.text("Spenstar Code").color(p.ink)).truncate());
        },
    );
    ui.add_space(8.);
    let (search_row, _) = ui.allocate_exact_size(vec2(width, 28.), Sense::hover());
    row::split(
        ui,
        search_row,
        |ui| {
            if rail_icon(ui, p, "New thread", Icon::Plus).clicked() {
                workbench
                    .threads
                    .add(workbench.threads.active().group, "New thread".into());
            }
            if rail_icon(ui, p, "Reverse thread order", Icon::ArrowUpDown).clicked() {
                workbench.threads.reversed = !workbench.threads.reversed;
            }
            let filter = rail_icon(ui, p, "Filter threads", Icon::Filter);
            let filters = [
                MenuItem::action("all", "All threads").checked(!workbench.threads.only_working),
                MenuItem::action("working", "Working only").checked(workbench.threads.only_working),
            ];
            if let Some(filter) = DropdownMenu::new("workbench-filter").show(&filter, &filters) {
                workbench.threads.only_working = filter == "working";
            }
        },
        |ui| {
            let search = row::trigger(ui, p, "Search", Some(Icon::Search), |_| {});
            let shortcut = if cfg!(target_os = "macos") {
                "⌘ K"
            } else {
                "Ctrl K"
            };
            Tooltip::new("Search gallery pages")
                .shortcut(shortcut)
                .show(&search);
            events.commands |= search.clicked();
        },
    );
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
    row::split(
        ui,
        rect.shrink2(vec2(8., 0.)),
        |ui| {
            let icon = if expanded {
                Icon::ChevronDown
            } else {
                Icon::ChevronRight
            };
            icon.show(ui, 12., p.muted);
        },
        |ui| {
            monogram(ui, mark, p);
            ui.add(Label::new(Type::Brand.text(name).color(p.ink)).truncate());
        },
    );
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
            gutter.min + vec2(12., 0.),
            pos2(
                (content.min_rect().left() - 8.).max(gutter.left() + 12.),
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

fn rail_icon(ui: &mut Ui, p: WorkbenchPalette, label: &str, icon: Icon) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(22., 26.), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    if response.hovered() || response.has_focus() {
        ui.painter().rect_filled(rect, 5, p.active);
    }
    let icon_rect = Rect::from_center_size(rect.center(), vec2(14., 14.));
    let _ = icon.svg().paint(ui, icon_rect, p.muted);
    Tooltip::new(label).show(&response);
    response
}
