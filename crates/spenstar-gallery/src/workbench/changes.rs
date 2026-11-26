use eframe::egui::{
    self, Align, CursorIcon, Id, Layout, Rect, ScrollArea, Sense, Stroke, Ui, UiBuilder, pos2, vec2,
};
use spenstar_ui::{
    Theme,
    display::{DiffKind, DiffLine, DiffView},
    icon::Icon,
    navigation::{Tab, Tabs, TabsVariant},
    typography::Type,
};

use super::{WorkbenchPalette, action, row};

const MIN_WIDTH: f32 = 300.;
const MAX_WIDTH: f32 = 560.;
const DEFAULT_WIDTH: f32 = 348.;
const CHAT_MIN_WIDTH: f32 = 350.;
const WIDTH_KEY: &str = "workbench-changes-width";

#[derive(Default)]
pub(super) struct Changes {
    pub(super) open: bool,
    tab: usize,
    reveal_file: Option<usize>,
}

impl Changes {
    pub(super) fn capture(&mut self, open: bool) {
        self.open = open;
        self.tab = 0;
        self.reveal_file = None;
    }

    pub(super) fn split(
        &self,
        ui: &Ui,
        bounds: Rect,
        git_width: f32,
        p: WorkbenchPalette,
    ) -> (Rect, Option<Rect>) {
        if !self.open {
            return (bounds, None);
        }
        let max = (bounds.width() - git_width - CHAT_MIN_WIDTH).min(MAX_WIDTH);
        if max < MIN_WIDTH {
            let dock = Rect::from_min_max(
                pos2(bounds.left(), bounds.bottom() - bounds.height() * 0.34),
                pos2(bounds.right() - git_width, bounds.bottom()),
            );
            return (bounds, Some(dock));
        }
        let divider = ui.make_persistent_id("workbench-changes-divider");
        let width = Self::width(ui, divider, max);
        let edge = bounds.right() - width;
        let mut work = bounds;
        work.max.x = edge;
        let hit = Rect::from_min_max(
            pos2(edge - 5., bounds.top()),
            pos2(edge + 5., bounds.bottom()),
        );
        let response = ui
            .interact(hit, divider, Sense::drag())
            .on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);
        let divider_color = if response.hovered() || response.dragged() {
            Theme::current(ui.ctx()).colors.accent.ring
        } else {
            p.border
        };
        ui.painter()
            .vline(edge - 0.5, bounds.y_range(), Stroke::new(1., divider_color));
        (
            work,
            Some(Rect::from_min_max(pos2(edge, bounds.top()), bounds.max)),
        )
    }

    pub(super) fn saved_width(ctx: &egui::Context) -> f32 {
        ctx.memory_mut(|memory| memory.data.get_persisted::<f32>(Id::unique(WIDTH_KEY)))
            .unwrap_or(DEFAULT_WIDTH)
    }

    fn width(ui: &Ui, divider: Id, max: f32) -> f32 {
        let ctx = ui.ctx();
        let width = Self::saved_width(ctx).clamp(MIN_WIDTH, max);
        if ctx.current_pass_index() == 0
            && let Some(drag) = ctx.read_response(divider).filter(egui::Response::dragged)
        {
            let width = (width - drag.drag_delta().x).clamp(MIN_WIDTH, max);
            ctx.memory_mut(|memory| memory.data.insert_persisted(Id::unique(WIDTH_KEY), width));
            return width;
        }
        width
    }

    pub(super) fn show(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        let bounds = ui.max_rect();
        ui.painter().rect_filled(bounds, 0, p.card);
        ui.painter().hline(
            bounds.x_range(),
            bounds.top() + 0.5,
            Stroke::new(1., p.border),
        );
        let inner = bounds.shrink2(vec2(12., 8.));
        let tabs_rect = Rect::from_min_size(inner.min, vec2(inner.width(), 34.));
        row::split(
            ui,
            tabs_rect,
            |ui| {
                if action(ui, "Close changes panel", Icon::X).clicked() {
                    self.open = false;
                    ui.ctx().request_repaint();
                }
            },
            |ui| {
                let mut tabs = Tabs::new("workbench-changes-tabs");
                tabs.variant = TabsVariant::Underline;
                let mut strip = ui.new_child(UiBuilder::new().layout(Layout::top_down(Align::Min)));
                tabs.show(
                    &mut strip,
                    &[Tab::new("Diff"), Tab::new("Files")],
                    &mut self.tab,
                );
            },
        );
        let line = tabs_rect.bottom() + 4.;
        ui.painter()
            .hline(bounds.x_range(), line, Stroke::new(1., p.border));
        let body = Rect::from_min_max(pos2(inner.left(), line + 12.), inner.max);
        let mut body = ui.new_child(
            UiBuilder::new()
                .id_salt("changes-body")
                .max_rect(body)
                .layout(Layout::top_down(Align::Min)),
        );
        body.shrink_clip_rect(body.max_rect());
        ScrollArea::vertical()
            .id_salt(("changes-scroll", self.tab))
            .auto_shrink([false, false])
            .show(&mut body, |ui| {
                if self.tab == 0 {
                    self.diff(ui, p);
                } else {
                    self.files(ui, p);
                }
            });
    }

    fn diff(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        let reveal = self.reveal_file.take();
        for (index, file) in FILES.iter().enumerate() {
            let heading = ui
                .horizontal(|ui| {
                    Icon::FileText.show(ui, 14., p.muted);
                    ui.label(Type::Label.text(format!("src/{}", file.name)).color(p.ink));
                })
                .response;
            if reveal == Some(index) {
                heading.scroll_to_me(Some(Align::Min));
            }
            DiffView {
                language: Some("Rust"),
                copy: true,
                line_numbers: true,
                lines: file.lines,
            }
            .show(ui);
            ui.add_space(12.);
        }
    }

    fn files(&mut self, ui: &mut Ui, p: WorkbenchPalette) {
        let folder =
            egui::CollapsingHeader::new(Type::Label.text("src").color(p.ink)).default_open(true);
        folder.show(ui, |ui| {
            for (index, file) in FILES.iter().enumerate() {
                let label = row::trigger(ui, p, file.name, Some(Icon::FileText), |ui| {
                    ui.label(Type::Caption.text("M").color(p.muted));
                });
                if label.clicked() {
                    self.reveal_file = Some(index);
                    self.tab = 0;
                    ui.ctx().request_repaint();
                }
            }
        });
    }
}

struct ChangedFile {
    name: &'static str,
    lines: &'static [DiffLine<'static>],
}

const FILES: &[ChangedFile] = &[
    ChangedFile {
        name: "orchestrator.rs",
        lines: ORCHESTRATOR,
    },
    ChangedFile {
        name: "workspace.rs",
        lines: WORKSPACE,
    },
];

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
        text: "    let next = queue",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "        .claim_ready(workspace);",
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
        text: "pub fn restore(",
    },
    DiffLine {
        kind: DiffKind::Context,
        text: "    snapshot: Snapshot,",
    },
    DiffLine {
        kind: DiffKind::Context,
        text: ") -> Workspace {",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "    let lineage =",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "        snapshot.lineage;",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "    Workspace::with_lineage(",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "        snapshot.root, lineage,",
    },
    DiffLine {
        kind: DiffKind::Added,
        text: "    )",
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
