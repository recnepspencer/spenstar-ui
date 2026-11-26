use eframe::egui::{self, Align, Context, Frame, Layout, Margin, Stroke, Ui};
use spenstar_ui::{
    button::{Button, Size, Variant},
    display::{Badge, BadgeSize, DiffKind, DiffLine, DiffView},
    feedback::Tone,
    icon::Icon,
    overlay::{Drawer, DrawerSide, DropdownMenu, MenuItem, Modal, Popover, Tooltip},
    typography::Type,
};

use super::{WorkbenchPalette, notify};

pub(super) struct Git {
    branch: &'static str,
    editor: Option<&'static str>,
    pull_request: bool,
    pub(super) changes: bool,
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
            changes: false,
            commit: false,
            commit_message: String::new(),
            script: false,
            script_name: String::new(),
            scripts: Vec::new(),
        }
    }
}

impl Git {
    pub(super) fn inspector(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        let mut rect = ui.max_rect();
        rect.min += egui::vec2(3., 60.);
        rect.max.x -= 11.;
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            Frame::new()
                .fill(p.card)
                .stroke(Stroke::new(1., p.border))
                .corner_radius(14)
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width((rect.width() - 26.).max(1.));
                    ui.spacing_mut().item_spacing.y = 7.;
                    ui.horizontal(|ui| {
                        Icon::Cpu.show(ui, 14., p.muted);
                        ui.label(Type::SectionLabel.text("MAC-STUDIO-M5").color(p.ink));
                    });
                    let editor = Button::new("Open in Editor")
                        .leading(Icon::Code2)
                        .trailing(Icon::ChevronDown)
                        .variant(Variant::Secondary)
                        .size(Size::Sm)
                        .show(ui);
                    if let Some(editor_name) = self.editor {
                        Tooltip::new(editor_name).show(&editor);
                    }
                    let editors = ["Cursor", "VS Code", "Zed", "Terminal"].map(|name| {
                        MenuItem::action(name, name)
                            .leading(Icon::Terminal)
                            .checked(self.editor == Some(name))
                    });
                    if let Some(editor) =
                        DropdownMenu::new("workbench-editors").show(&editor, &editors)
                    {
                        self.editor = Some(editor);
                        notify(
                            ui.ctx(),
                            &format!("Selected {editor} for this workspace"),
                            Tone::Accent,
                        );
                    }
                    if control(ui, "Add project script", Icon::Plus) {
                        self.script = true;
                    }
                    for script in &self.scripts {
                        ui.label(Type::Mono.text(script).color(p.muted));
                    }
                    divider(ui, p);
                    let branch = Button::new(self.branch)
                        .leading(Icon::GitBranch)
                        .trailing(Icon::ChevronDown)
                        .variant(Variant::Secondary)
                        .size(Size::Sm)
                        .show(ui);
                    let branches = ["main", "orchestrator-fork", "sol/w4-overlays-feedback"]
                        .map(|name| MenuItem::action(name, name).checked(self.branch == name));
                    if let Some(branch) = DropdownMenu::new("workbench-branches")
                        .width(232.)
                        .show(&branch, &branches)
                    {
                        self.branch = branch;
                        notify(ui.ctx(), "Branch selected", Tone::Accent);
                    }
                    ui.horizontal(|ui| {
                        Icon::GitPullRequest.show(ui, 14., p.muted);
                        let pr = ui.link(Type::Description.text("PR #407").color(p.ink));
                        Popover::new("workbench-pull-request").show(
                            &pr,
                            &mut self.pull_request,
                            |ui| {
                                ui.label(Type::Label.text("Orchestrator workspace handoff"));
                                ui.label(Type::Description.text("Ready for review · targets main"));
                                if control(ui, "Copy reference", Icon::Copy) {
                                    ui.ctx().copy_text("PR #407".into());
                                }
                            },
                        );
                        let mut status = Badge::new("Ready");
                        status.tone = Tone::Success;
                        status.size = BadgeSize::Sm;
                        status.show(ui);
                    });
                    if control(ui, "Commit", Icon::GitCommit) {
                        self.commit = true;
                    }
                    let changes = Button::new("Changes vs origin/main")
                        .leading(Icon::FileText)
                        .size(Size::Sm)
                        .variant(Variant::Ghost)
                        .show(ui);
                    if changes.clicked() {
                        self.changes = true;
                    }
                    ui.horizontal(|ui| {
                        ui.add_space(8.);
                        let c = spenstar_ui::Theme::current(ui.ctx()).colors;
                        ui.label(Type::Kbd.text("+42").color(c.success.ring));
                        ui.label(Type::Kbd.text("−8").color(c.danger.ring));
                    });
                    divider(ui, p);
                    ui.label(Type::Label.text("Lineage").color(p.ink));
                    ui.horizontal(|ui| {
                        ui.label(Type::Description.text("↳ orchestrator").color(p.muted));
                        let c = spenstar_ui::Theme::current(ui.ctx()).colors;
                        Icon::Check.show(ui, 11., c.success.ring);
                        ui.label(Type::Caption.text("Done").color(c.success.ring));
                    });
                });
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

        Drawer::new(
            "workbench-changes",
            "Changes vs origin/main",
            DrawerSide::Right,
        )
        .extent(536.)
        .subtitle("worth-orchestration  ·  2 changed files")
        .show(ctx, &mut self.changes, |ui| {
            let c = spenstar_ui::Theme::current(ctx).colors;
            ui.horizontal(|ui| {
                ui.label(Type::Kbd.text("+42").color(c.success.ring));
                ui.label(Type::Kbd.text("−8").color(c.danger.ring));
            });
            ui.add_space(12.);
            diff(ui, "src/orchestrator.rs", ORCHESTRATOR);
            ui.add_space(18.);
            diff(ui, "src/workspace.rs", WORKSPACE);
        });
    }
}

fn diff(ui: &mut Ui, name: &str, lines: &[DiffLine<'_>]) {
    ui.horizontal(|ui| {
        Icon::FileText.show(
            ui,
            14.,
            spenstar_ui::Theme::current(ui.ctx()).colors.text_secondary,
        );
        ui.label(Type::Label.text(name));
    });
    DiffView {
        language: Some("Rust"),
        copy: true,
        line_numbers: true,
        lines,
    }
    .show(ui);
}

fn control(ui: &mut Ui, label: &str, icon: Icon) -> bool {
    Button::new(label)
        .leading(icon)
        .size(Size::Sm)
        .variant(Variant::Ghost)
        .show(ui)
        .clicked()
}

fn divider(ui: &mut Ui, p: WorkbenchPalette) {
    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 8.), egui::Sense::hover());
        ui.painter()
            .hline(rect.x_range(), rect.center().y, Stroke::new(1., p.border));
    });
}

const ORCHESTRATOR: &[DiffLine<'static>] = &[
    DiffLine {
        kind: DiffKind::Context,
        text: "impl Orchestrator {",
    },
    DiffLine {
        kind: DiffKind::Removed,
        text: "    let next = queue.pop_front();",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "    let next = queue.claim_ready(workspace);",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "    journal.record_claim(&next)?;",
    },
    DiffLine {
        kind: DiffKind::Context,
        text: "    self.dispatch(next).await",
    },
    DiffLine {
        kind: DiffKind::Context,
        text: "}",
    },
];

const WORKSPACE: &[DiffLine<'static>] = &[
    DiffLine {
        kind: DiffKind::Context,
        text: "pub fn restore(snapshot: Snapshot) -> Workspace {",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "    let lineage = snapshot.lineage;",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "    Workspace::with_lineage(snapshot.root, lineage)",
    },
    DiffLine {
        kind: DiffKind::Removed,
        text: "    Workspace::new(snapshot.root)",
    },
    DiffLine {
        kind: DiffKind::Context,
        text: "}",
    },
];
