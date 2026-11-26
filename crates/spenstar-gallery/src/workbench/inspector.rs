use eframe::egui::{self, Align, Context, Frame, Layout, Margin, Response, Stroke, Ui, vec2};
use spenstar_ui::{
    button::{Button, Size, Variant},
    display::{Badge, BadgeSize},
    feedback::Tone,
    icon::Icon,
    overlay::{DropdownMenu, MenuItem, Modal, Popover, Tooltip},
    typography::Type,
};

use super::{WorkbenchPalette, notify, row};

pub(super) struct Git {
    branch: &'static str,
    editor: Option<&'static str>,
    pull_request: bool,
    commit: bool,
    commit_message: String,
    script: bool,
    script_name: String,
    scripts: Vec<String>,
}

impl Default for Git {
    fn default() -> Self {
        Self {
            branch: "main",
            editor: None,
            pull_request: false,
            commit: false,
            commit_message: String::new(),
            script: false,
            script_name: String::new(),
            scripts: Vec::new(),
        }
    }
}

impl Git {
    pub(super) fn inspector(&mut self, ui: &mut Ui, p: WorkbenchPalette, changes_open: &mut bool) {
        let mut rect = ui.max_rect();
        rect.min += vec2(3., 60.);
        rect.max.x -= 11.;
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            Frame::new()
                .fill(p.card)
                .stroke(Stroke::new(1., p.border))
                .corner_radius(14)
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width((rect.width() - 26.).max(1.));
                    ui.spacing_mut().item_spacing.y = 3.;
                    card_row(ui, p, "MAC-STUDIO-M5", Icon::Cpu, false, |ui| {
                        Icon::Folder.show(ui, 14., p.muted);
                    });
                    self.editor_menu(ui, p);
                    if card_row(ui, p, "Add project script", Icon::Plus, true, |_| {}).clicked() {
                        self.script = true;
                    }
                    for script in &self.scripts {
                        card_row(ui, p, script, Icon::Terminal, false, |_| {});
                    }
                    divider(ui, p);
                    self.branch_menu(ui, p);
                    self.pull_request(ui, p);
                    self.commit |=
                        card_row(ui, p, "Commit", Icon::GitCommit, true, |_| {}).clicked();
                    let c = spenstar_ui::Theme::current(ui.ctx()).colors;
                    let changes = card_row(
                        ui,
                        p,
                        "Changes vs origin/main",
                        Icon::FileText,
                        true,
                        |ui| {
                            ui.label(Type::Kbd.text("−8").color(c.danger.ring));
                            ui.label(Type::Kbd.text("+42").color(c.success.ring));
                        },
                    );
                    if changes.clicked() {
                        *changes_open = !*changes_open;
                        ui.ctx().request_repaint();
                    }
                    divider(ui, p);
                    card_row(ui, p, "Lineage", Icon::GitMerge, false, |ui| {
                        Icon::MoreHorizontal.show(ui, 14., p.muted);
                    });
                    card_row(ui, p, "orchestrator", Icon::GitBranch, false, |ui| {
                        ui.label(Type::Caption.text("Done").color(c.success.ring));
                        Icon::Check.show(ui, 11., c.success.ring);
                    });
                });
        });
    }

    fn editor_menu(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        let editor = card_row(ui, p, "Open in Editor", Icon::Code2, true, |ui| {
            Icon::ChevronDown.show(ui, 12., p.muted);
        });
        if let Some(editor_name) = self.editor {
            Tooltip::new(editor_name).show(&editor);
        }
        let editors = ["Cursor", "VS Code", "Zed", "Terminal"].map(|name| {
            MenuItem::action(name, name)
                .leading(Icon::Terminal)
                .checked(self.editor == Some(name))
        });
        if let Some(editor) = DropdownMenu::new("workbench-editors").show(&editor, &editors) {
            self.editor = Some(editor);
            notify(
                ui.ctx(),
                &format!("Selected {editor} for this workspace"),
                Tone::Accent,
            );
        }
    }

    fn branch_menu(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        let branch = card_row(ui, p, self.branch, Icon::GitBranch, true, |ui| {
            Icon::ChevronDown.show(ui, 12., p.muted);
        });
        let branches = ["main", "orchestrator-fork", "sol/w4-overlays-feedback"]
            .map(|name| MenuItem::action(name, name).checked(self.branch == name));
        if let Some(branch) = DropdownMenu::new("workbench-branches")
            .width(232.)
            .show(&branch, &branches)
        {
            self.branch = branch;
            notify(ui.ctx(), "Branch selected", Tone::Accent);
        }
    }

    fn pull_request(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        let pr = card_row(ui, p, "PR #407", Icon::GitPullRequest, true, |ui| {
            let mut status = Badge::new("Ready");
            status.tone = Tone::Success;
            status.size = BadgeSize::Sm;
            let width = ui
                .painter()
                .layout_no_wrap("Ready".into(), Type::Badge.font(), p.ink)
                .size()
                .x
                + 14.;
            ui.allocate_ui_with_layout(
                vec2(width, 22.),
                Layout::left_to_right(Align::Center),
                |ui| {
                    status.show(ui);
                },
            );
        });
        Popover::new("workbench-pull-request").show(&pr, &mut self.pull_request, |ui| {
            ui.label(Type::Label.text("Orchestrator workspace handoff"));
            ui.label(Type::Description.text("Ready for review · targets main"));
            if Button::new("Copy reference")
                .leading(Icon::Copy)
                .size(Size::Sm)
                .variant(Variant::Ghost)
                .show(ui)
                .clicked()
            {
                ui.ctx().copy_text("PR #407".into());
            }
        });
    }

    pub(super) fn overlays(&mut self, ctx: &Context) {
        let commit = Modal::new("workbench-commit", "Commit changes")
            .subtitle(self.branch)
            .show(
                ctx,
                &mut self.commit,
                |ui| {
                    ui.label(Type::Label.text("Commit message"));
                    ui.add(
                        egui::TextEdit::multiline(&mut self.commit_message)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .hint_text("Describe the change"),
                    );
                },
                |ui| {
                    Button::new("Commit changes")
                        .variant(Variant::Soft)
                        .show(ui)
                        .clicked()
                },
            );
        if commit.is_some_and(|output| output.footer) && !self.commit_message.trim().is_empty() {
            self.commit = false;
            self.commit_message.clear();
            notify(ctx, "Demo commit recorded", Tone::Success);
        }
        let script = Modal::new("workbench-script", "Add project script").show(
            ctx,
            &mut self.script,
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.script_name)
                        .desired_width(f32::INFINITY)
                        .hint_text("cargo test --workspace"),
                );
            },
            |ui| {
                Button::new("Save script")
                    .variant(Variant::Soft)
                    .show(ui)
                    .clicked()
            },
        );
        if script.is_some_and(|output| output.footer) && !self.script_name.trim().is_empty() {
            self.scripts.push(self.script_name.trim().into());
            self.script_name.clear();
            self.script = false;
            notify(ctx, "Project script saved", Tone::Success);
        }
    }
}

fn card_row(
    ui: &mut Ui,
    p: WorkbenchPalette,
    label: &str,
    icon: Icon,
    interactive: bool,
    trailing: impl FnOnce(&mut Ui),
) -> Response {
    let (rect, response) = row::surface(ui, p, label, interactive);
    let icon_center = egui::pos2(rect.left() + 10., rect.center().y);
    let _ = icon.svg().paint(
        ui,
        egui::Rect::from_center_size(icon_center, vec2(14., 14.)),
        p.muted,
    );
    let mut columns = rect;
    columns.min.x += 28.;
    row::split(ui, columns, trailing, |ui| {
        ui.add(
            egui::Label::new(Type::Description.text(label).color(p.ink))
                .truncate()
                .selectable(false),
        );
    });
    response
}

fn divider(ui: &mut Ui, p: WorkbenchPalette) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 8.), egui::Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.center().y, Stroke::new(1., p.border));
}
