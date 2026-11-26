use eframe::egui::{self, Align, Frame, Layout, Margin, Rect, ScrollArea, Stroke, Ui, pos2, vec2};
use spenstar_ui::{
    button::{Button, Size, Variant},
    feedback::Tone,
    icon::{Icon, SvgIcon},
    navigation::{Disclosure, DisclosureVariant},
    overlay::{DropdownMenu, MenuItem, Tooltip},
    typography::Type,
};

use super::{
    Workbench, WorkbenchEvents, WorkbenchPalette, action, notify, pane,
    rail::{PROJECTS, monogram},
};

const INITIAL_REQUEST: &str =
    "Pick up the orchestrator fork and verify the Wave 3 handoff before starting Wave 4.";
const ASSISTANT_SUMMARY: &str = "The orchestration fork is ready. The Wave 3 handoff passed the workspace checks, and the gallery screenshots are on their preview branch.";

impl Workbench {
    pub(super) fn conversation(
        &mut self,
        ui: &mut Ui,
        p: WorkbenchPalette,
        rail_visible: bool,
        events: &mut WorkbenchEvents,
    ) {
        let rect = ui.max_rect();
        let header = Rect::from_min_max(
            rect.min + vec2(14., 8.),
            pos2(rect.right() - 14., rect.top() + 49.),
        );
        self.breadcrumb(
            &mut pane(ui, "breadcrumbs", header),
            p,
            rail_visible,
            events,
        );

        let width = (rect.width() - 52.).clamp(160., 704.);
        let left = rect.center().x - width * 0.5;
        let stream = Rect::from_min_max(
            pos2(left, rect.top() + 64.),
            pos2(left + width, rect.bottom() - 146.),
        );
        let compose = Rect::from_min_max(
            pos2(left, rect.bottom() - 132.),
            pos2(left + width, rect.bottom() - 14.),
        );
        let mut stream_ui = pane(ui, "stream", stream);
        let selected = self.threads.active().id;
        let title = self.threads.active().title.clone();
        ScrollArea::vertical()
            .id_salt(("turns", selected))
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(&mut stream_ui, |ui| {
                ui.spacing_mut().item_spacing.y = 7.;
                if selected == 0 {
                    self.seed_conversation(ui, p);
                } else {
                    ui.label(Type::H2.text(title).color(p.ink));
                    ui.label(
                        Type::Description
                            .text("Continue the conversation in this thread.")
                            .color(p.muted),
                    );
                }
                for (_, message) in self.messages.iter().filter(|(id, _)| *id == selected) {
                    ui.add_space(8.);
                    user_bubble(ui, p, message);
                }
            });
        self.minimap(ui, stream, selected, p);
        self.composer(&mut pane(ui, "composer", compose), p);
    }

    fn breadcrumb(
        &mut self,
        ui: &mut Ui,
        p: WorkbenchPalette,
        rail_visible: bool,
        events: &mut WorkbenchEvents,
    ) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if action(ui, "Toggle Git inspector", Icon::Sidebar).clicked() {
                self.inspector = !self.inspector;
            }
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                if !rail_visible {
                    if action(ui, "Show project rail", Icon::PanelLeft).clicked() {
                        self.rail = true;
                        self.compact_rail = true;
                    }
                    if action(ui, "Return to gallery", Icon::LayoutDashboard).clicked() {
                        events.gallery = true;
                    }
                }
                let active = self.threads.active();
                let (mark, name) = PROJECTS[active.group];
                monogram(ui, mark, p);
                ui.label(Type::Description.text(name).color(p.muted));
                ui.label(Type::Description.text("/").color(p.muted));
                ui.add(egui::Label::new(Type::Label.text(&active.title).color(p.ink)).truncate());
            });
        });
    }

    fn minimap(&self, ui: &Ui, stream: Rect, id: u64, p: WorkbenchPalette) {
        let count = self
            .messages
            .iter()
            .filter(|(thread, _)| *thread == id)
            .count()
            + usize::from(id == 0) * 2;
        let max_ticks = (stream.height() / 9.).max(0.) as usize;
        let visible = count.min(max_ticks);
        for turn in 0..visible {
            let y = stream.top() + 14. + turn as f32 * 9.;
            let left = stream.left() - 17.;
            let (len, color) = if turn + 1 == visible {
                (9., p.muted)
            } else {
                (6., p.border)
            };
            ui.painter()
                .hline(left..=left + len, y, Stroke::new(1., color));
        }
    }

    fn seed_conversation(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        user_bubble(ui, p, INITIAL_REQUEST);
        ui.add_space(14.);
        let mut disclosure = Disclosure::new("workbench-trace", "Worked for 1m 24s");
        disclosure.variant = DisclosureVariant::Light;
        disclosure.show(ui, &mut self.trace, |ui| {
            for (icon, step) in [
                (
                    Icon::Terminal,
                    "Checked the working tree and upstream branch",
                ),
                (
                    Icon::CheckCircle2,
                    "Ran workspace formatting, lint, and tests",
                ),
                (Icon::GitPullRequest, "Recorded the review handoff"),
            ] {
                ui.horizontal(|ui| {
                    icon.show(ui, 14., p.muted);
                    ui.label(Type::Description.text(step).color(p.muted));
                });
            }
        });
        ui.add_space(13.);
        ui.label(Type::Body.text(ASSISTANT_SUMMARY).color(p.ink));
        ui.add_space(4.);
        ui.horizontal_wrapped(|ui| {
            ui.label(Type::Body.text("•  Base branch").color(p.ink));
            code(ui, p, "main");
            ui.label(Type::Body.text("is pinned at").color(p.ink));
            code(ui, p, "6de708b9b362");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(Type::Body.text("•").color(p.ink));
            code(ui, p, "DataTable");
            ui.label(
                Type::Body
                    .text("retains column sizing and selection across refreshes.")
                    .color(p.ink),
            );
        });
        ui.label(
            Type::Body
                .text("•  The review thread includes the capture set and release checklist.")
                .color(p.ink),
        );
        ui.add_space(10.);
        ui.horizontal_wrapped(|ui| {
            let pr = Button::new("PR #4")
                .leading(Icon::GitPullRequest)
                .size(Size::Sm)
                .variant(Variant::Outline)
                .show(ui);
            if pr.clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(
                    "https://github.com/recnepspencer/spenstar-ui/pull/4",
                ));
            }
            let thread = Frame::new()
                .fill(p.card)
                .stroke(Stroke::new(1., p.border))
                .corner_radius(6)
                .inner_margin(Margin::symmetric(7, 5))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        monogram(ui, "SU", p);
                        ui.link(Type::Description.text("spenstar-ui Wave 4").color(p.ink))
                    })
                    .inner
                })
                .inner;
            if thread.clicked() {
                self.threads.select(4);
            }
        });
        ui.add_space(7.);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 3.;
            if action(ui, "Copy reply", Icon::Copy).clicked() {
                ui.ctx().copy_text(ASSISTANT_SUMMARY.into());
            }
            if action(ui, "Fork thread", Icon::GitBranch).clicked() {
                self.threads.fork_active(ui.ctx());
            }
            let reply = action(ui, "Reply", Icon::ArrowUpRight);
            if reply.clicked() {
                ui.ctx()
                    .memory_mut(|mem| mem.request_focus(composer_input()));
            }
            let timestamp = ui.label(Type::Caption.text("5:23 PM").color(p.muted));
            Tooltip::new("Today at 5:23 PM").show(&timestamp);
        });
    }

    fn composer(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        const MODEL_OPTIONS: [&str; 3] = ["4 Neon High", "6.2 Sol Ultrafast", "Opus 5.5 High"];
        const PAPERCLIP: SvgIcon = SvgIcon::from_static(
            "workbench-paperclip",
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"><path d="M8 13l6-6a2.5 2.5 0 0 1 3.5 3.5l-7 7a4 4 0 0 1-5.7-5.7l8-8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
        );
        const MIC: SvgIcon = SvgIcon::from_static(
            "workbench-mic",
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"><rect x="9" y="3" width="6" height="12" rx="3" stroke="currentColor" stroke-width="1.8"/><path d="M5.5 11v1a6.5 6.5 0 0 0 13 0v-1M12 18.5V22" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
        );
        let width = ui.available_width();
        Frame::new()
            .fill(p.card)
            .stroke(Stroke::new(1., p.border))
            .corner_radius(16)
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                ui.set_width((width - 22.).max(1.));
                ScrollArea::vertical()
                    .id_salt("draft-scroll")
                    .max_height(56.)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.draft)
                                .id(composer_input())
                                .hint_text("Ask anything. @ files, $ skills, / commands")
                                .font(Type::Body.font())
                                .desired_width(f32::INFINITY)
                                .desired_rows(3)
                                .frame(Frame::NONE),
                        );
                    });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let send = Button::new("Send message")
                        .icon_only(Icon::ArrowUp)
                        .variant(Variant::Soft)
                        .size(Size::Sm)
                        .enabled(!self.draft.trim().is_empty())
                        .show(ui);
                    Tooltip::new("Send message").show(&send);
                    if send.clicked() {
                        self.send(ui.ctx());
                        ui.ctx()
                            .memory_mut(|memory| memory.request_focus(composer_input()));
                    }
                    if action(ui, "Voice input", MIC).clicked() {
                        notify(
                            ui.ctx(),
                            "Voice input is unavailable in this demo",
                            Tone::Neutral,
                        );
                    }
                    let model = Button::new(self.model)
                        .trailing(Icon::ChevronDown)
                        .variant(Variant::Secondary)
                        .size(Size::Sm)
                        .show(ui);
                    let items = MODEL_OPTIONS
                        .map(|name| MenuItem::action(name, name).checked(self.model == name));
                    if let Some(selection) =
                        DropdownMenu::new("workbench-model").show(&model, &items)
                    {
                        self.model = selection;
                    }
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        let attach = action(ui, "Attach context", PAPERCLIP);
                        let attach_items = [
                            MenuItem::action("README.md", "README.md").leading(Icon::FileText),
                            MenuItem::action("Cargo.toml", "Cargo.toml").leading(Icon::FileText),
                        ];
                        if let Some(path) =
                            DropdownMenu::new("workbench-attachments").show(&attach, &attach_items)
                        {
                            if !self.draft.is_empty() {
                                self.draft.push(' ');
                            }
                            self.draft.push('@');
                            self.draft.push_str(path);
                            ui.ctx()
                                .memory_mut(|mem| mem.request_focus(composer_input()));
                        }
                    });
                });
            });
    }
}

fn composer_input() -> egui::Id {
    egui::Id::unique("workbench-draft")
}

fn user_bubble(ui: &mut Ui, p: WorkbenchPalette, text: &str) {
    let font = Type::Body.font();
    let available = ui.available_width();
    let max = (available * 0.72 - 26.).max(1.);
    let text_width =
        ui.fonts_mut(|fonts| fonts.layout_no_wrap(text.to_owned(), font, p.ink).size().x);
    let width = text_width.min(max) + 27.;
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.allocate_ui_with_layout(vec2(width, 0.), Layout::top_down(Align::Min), |ui| {
            Frame::new()
                .fill(p.bubble)
                .corner_radius(14)
                .inner_margin(Margin::symmetric(13, 10))
                .show(ui, |ui| {
                    ui.set_width(width - 26.);
                    ui.label(Type::Body.text(text).color(p.ink));
                });
        });
    });
}

fn code(ui: &mut Ui, p: WorkbenchPalette, text: &str) {
    Frame::new()
        .fill(p.active)
        .corner_radius(5)
        .inner_margin(Margin::symmetric(5, 2))
        .show(ui, |ui| ui.label(Type::Mono.text(text).color(p.ink)));
}
