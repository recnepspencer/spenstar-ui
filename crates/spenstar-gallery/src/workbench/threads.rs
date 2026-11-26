use super::notify;
use eframe::egui;
use spenstar_ui::{
    button::{Button, Variant},
    feedback::Tone,
    overlay::Modal,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Status {
    Done,
    Working,
    Waiting,
    Time(&'static str),
}

pub(super) struct Thread {
    pub(super) id: u64,
    pub(super) group: usize,
    pub(super) title: String,
    pub(super) status: Status,
}

pub(super) struct Threads {
    pub(super) rows: Vec<Thread>,
    next: u64,
    pub(super) selected: u64,
    back: Vec<u64>,
    forward: Vec<u64>,
    pub(super) expanded: [bool; 4],
    pub(super) only_working: bool,
    pub(super) reversed: bool,
    rename: Option<(u64, String)>,
}

impl Default for Threads {
    fn default() -> Self {
        use Status::*;
        let samples = [
            (0, "orchestrator fork", Done),
            (0, "Device host handoff", Working),
            (0, "Review graph ownership", Waiting),
            (0, "Next milestone review", Time("4h")),
            (1, "Wave 4 · overlays & feedback", Working),
            (1, "Wave 3 · data & charts", Done),
            (1, "Gallery visual review", Time("1d")),
            (2, "Recover build cache", Waiting),
            (2, "Workspace sync", Time("1d")),
            (3, "Design system notes", Time("9d")),
        ];
        Self {
            rows: samples
                .into_iter()
                .enumerate()
                .map(|(id, (group, title, status))| Thread {
                    id: id as u64,
                    group,
                    title: title.into(),
                    status,
                })
                .collect(),
            next: samples.len() as u64,
            selected: 0,
            back: Vec::new(),
            forward: Vec::new(),
            expanded: [true; 4],
            only_working: false,
            reversed: false,
            rename: None,
        }
    }
}

impl Threads {
    pub(super) fn active(&self) -> &Thread {
        self.rows
            .iter()
            .find(|row| row.id == self.selected)
            .unwrap_or(&self.rows[0])
    }

    pub(super) fn select(&mut self, id: u64) {
        if self.selected != id {
            self.back.push(self.selected);
            self.forward.clear();
            self.selected = id;
        }
    }

    pub(super) fn history(&mut self, forward: bool) {
        let (from, to) = if forward {
            (&mut self.forward, &mut self.back)
        } else {
            (&mut self.back, &mut self.forward)
        };
        if let Some(id) = from.pop() {
            to.push(self.selected);
            self.selected = id;
        }
    }

    pub(super) fn add(&mut self, group: usize, title: String) {
        let id = self.next;
        self.next += 1;
        self.rows.push(Thread {
            id,
            group,
            title,
            status: Status::Time("now"),
        });
        self.expanded[group] = true;
        self.only_working = false;
        self.select(id);
    }

    pub(super) fn context_action(&mut self, ctx: &egui::Context, id: u64, action: &str) {
        let Some(row) = self.rows.iter().find(|row| row.id == id) else {
            return;
        };
        match action {
            "rename" => self.rename = Some((id, row.title.clone())),
            "fork" => {
                self.add(row.group, format!("{} fork", row.title));
                notify(ctx, "Thread forked", Tone::Success);
            }
            "copy" => {
                ctx.copy_text(format!("spenstar://workbench/thread/{id}"));
            }
            "archive" => {
                let group = row.group;
                self.rows.retain(|row| row.id != id);
                self.back.retain(|&item| item != id);
                self.forward.retain(|&item| item != id);
                if self.rows.is_empty() {
                    self.add(group, "New thread".into());
                    self.back.clear();
                } else if self.selected == id {
                    self.selected = self.rows[0].id;
                }
                notify(ctx, "Thread archived", Tone::Neutral);
            }
            _ => {}
        }
    }

    pub(super) fn fork_active(&mut self, ctx: &egui::Context) {
        self.context_action(ctx, self.active().id, "fork");
    }

    pub(super) fn rename(&mut self, ctx: &egui::Context) {
        let Some((id, draft)) = &mut self.rename else {
            return;
        };
        let mut open = true;
        let output = Modal::new("workbench-rename", "Rename thread").show(
            ctx,
            &mut open,
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(draft)
                        .hint_text("Thread title")
                        .desired_width(f32::INFINITY),
                );
            },
            |ui| {
                Button::new("Save name")
                    .variant(Variant::Soft)
                    .show(ui)
                    .clicked()
            },
        );
        if output.is_some_and(|output| output.footer) && !draft.trim().is_empty() {
            if let Some(row) = self.rows.iter_mut().find(|row| row.id == *id) {
                row.title = draft.trim().to_owned();
                ctx.request_repaint();
                notify(ctx, "Thread renamed", Tone::Success);
            }
            open = false;
        }
        if !open {
            self.rename = None;
        }
    }
}
