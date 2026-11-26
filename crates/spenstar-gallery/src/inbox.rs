mod reader;
mod rows;
mod samples;

use eframe::egui::{Color32, Frame, Margin, Panel, Sense, Ui, vec2};
use spenstar_ui::{
    Theme,
    button::Button,
    icon::Icon,
    layout::{EmptyState, ScrollShadow, Separator},
    typography::Type,
};

use crate::{orders::initials, widgets};
use reader::{Composer, ReaderAction};
use rows::{folder_row, thread_row};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Folder {
    Inbox,
    Starred,
    Sent,
    Drafts,
    Snoozed,
    Archive,
    Spam,
    Trash,
    Label(&'static str),
}

const FOLDERS: [(Folder, &str, Icon); 8] = [
    (Folder::Inbox, "Inbox", Icon::Inbox),
    (Folder::Starred, "Starred", Icon::Star),
    (Folder::Sent, "Sent", Icon::Send),
    (Folder::Drafts, "Drafts", Icon::FileText),
    (Folder::Snoozed, "Snoozed", Icon::Clock),
    (Folder::Archive, "Archive", Icon::Boxes),
    (Folder::Spam, "Spam", Icon::ShieldAlert),
    (Folder::Trash, "Trash", Icon::Trash2),
];

const LABELS: [(&str, Color32); 5] = [
    ("Work", Color32::from_rgb(4, 133, 247)),
    ("Personal", Color32::from_rgb(129, 129, 142)),
    ("Billing", Color32::from_rgb(245, 165, 36)),
    ("Travel", Color32::from_rgb(23, 191, 100)),
    ("Urgent", Color32::from_rgb(255, 56, 60)),
];

#[derive(Clone)]
struct Message {
    sender: String,
    email: String,
    time: String,
    body: String,
}

struct Thread {
    subject: String,
    label: &'static str,
    messages: Vec<Message>,
    folder: Folder,
    starred: bool,
    unread: bool,
}

pub(super) struct Inbox {
    mailbox: Vec<Thread>,
    folder: Folder,
    selected: Option<usize>,
    query: String,
    composer: Option<Composer>,
    source: Option<usize>,
}

impl Default for Inbox {
    fn default() -> Self {
        Self {
            mailbox: samples::mailbox(),
            folder: Folder::Inbox,
            selected: Some(0),
            query: String::new(),
            composer: None,
            source: None,
        }
    }
}

impl Inbox {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let wide = ui.available_width() >= 785.;
        if wide {
            let sidebar_width = (ui.available_width() * 0.17).clamp(154., 210.);
            Panel::left("mail-folders")
                .exact_size(sidebar_width)
                .resizable(false)
                .frame(Frame::new().inner_margin(Margin::symmetric(8, 10)))
                .show(ui, |ui| self.folders(ui));
            let thread_width = (ui.available_width() * 0.42).clamp(260., 344.);
            Panel::left("mail-threads")
                .exact_size(thread_width)
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::new().inner_margin(Margin::symmetric(12, 10)))
                .show(ui, |ui| self.threads(ui));
            Frame::new()
                .inner_margin(Margin::symmetric(6, 10))
                .show(ui, |ui| self.read(ui));
        } else {
            ui.horizontal_wrapped(|ui| {
                let labels: Vec<_> = FOLDERS.iter().map(|(_, name, _)| *name).collect();
                let mut folder = FOLDERS
                    .iter()
                    .find(|(folder, _, _)| *folder == self.folder)
                    .map(|(_, name, _)| (*name).to_owned());
                widgets::sized_width(ui, 126., 38., |ui| {
                    if widgets::select(ui, &mut folder, &labels, "Folders").changed()
                        && let Some((folder, _, _)) = FOLDERS
                            .iter()
                            .find(|(_, title, _)| Some(*title) == folder.as_deref())
                    {
                        self.selected = None;
                        self.folder = *folder;
                    }
                });
                if Button::new("New message")
                    .leading(Icon::Pencil)
                    .show(ui)
                    .clicked()
                {
                    self.composer = Some(Composer::default());
                }
            });
            if self.selected.is_some() {
                self.read(ui);
            } else {
                self.threads(ui);
            }
        }
        if let Some(composer) = &mut self.composer
            && let Some(completed) = composer.show(ui.ctx())
        {
            if let Some(thread) = completed {
                if let Some(index) = self.source.take() {
                    let existing = &mut self.mailbox[index];
                    if existing.folder == Folder::Drafts {
                        *existing = thread;
                        self.selected = None;
                    } else {
                        if thread.folder == Folder::Sent {
                            existing.messages.extend(thread.messages.iter().cloned());
                        }
                        self.mailbox.insert(0, thread);
                        self.selected = self.selected.map(|index| index + 1);
                    }
                } else {
                    self.mailbox.insert(0, thread);
                    self.selected = self.selected.map(|index| index + 1);
                }
            }
            self.source = None;
            self.composer = None;
        }
    }

    fn folders(&mut self, ui: &mut Ui) {
        let c = Theme::current(ui.ctx()).colors;
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(36., 36.), Sense::hover());
            initials(ui, rect, "Alex Morgan", c.default.base, c.text_primary);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.;
                ui.label(Type::Brand.text("Alex Morgan"));
                ui.label(
                    Type::Small
                        .text("alex@spenstar.example")
                        .color(c.text_muted),
                );
            });
        });
        ui.add_space(4.);
        let compose = Panel::bottom("compose-button")
            .resizable(false)
            .show_separator_line(false)
            .frame(Frame::new().inner_margin(Margin::symmetric(0, 6)))
            .show(ui, |ui| {
                Button::new("New message")
                    .leading(Icon::Pencil)
                    .min_width(ui.available_width())
                    .show(ui)
                    .clicked()
            })
            .inner;
        if compose {
            self.composer = Some(Composer::default());
        }
        let unread = self
            .mailbox
            .iter()
            .filter(|thread| thread.folder == Folder::Inbox && thread.unread)
            .count();
        let starred = self
            .mailbox
            .iter()
            .filter(|thread| thread.starred && thread.folder != Folder::Trash)
            .count();
        ScrollShadow::new("mail-folder-scroll", ui.available_height())
            .fill(c.background)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 3.;
                for (folder, name, icon) in FOLDERS {
                    let count = match folder {
                        Folder::Inbox => Some(unread),
                        Folder::Starred => Some(starred),
                        _ => None,
                    };
                    if folder_row(
                        ui,
                        name,
                        Some(icon),
                        c.text_muted,
                        count,
                        self.folder == folder,
                    ) {
                        self.folder = folder;
                        self.selected = None;
                    }
                }
                Separator::Horizontal.show(ui);
                ui.label(Type::SectionLabel.text("Labels").color(c.text_muted));
                for (name, color) in LABELS {
                    let folder = Folder::Label(name);
                    if folder_row(ui, name, None, color, None, self.folder == folder) {
                        self.folder = folder;
                        self.selected = None;
                    }
                }
            });
    }

    fn visible(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        self.mailbox
            .iter()
            .enumerate()
            .filter_map(|(index, thread)| {
                let in_folder = match self.folder {
                    Folder::Starred => thread.starred && thread.folder != Folder::Trash,
                    Folder::Label(label) => thread.label == label && thread.folder != Folder::Trash,
                    folder => thread.folder == folder,
                };
                (in_folder
                    && (thread.subject.to_lowercase().contains(&query)
                        || thread.messages.iter().any(|message| {
                            message.sender.to_lowercase().contains(&query)
                                || message.body.to_lowercase().contains(&query)
                        })))
                .then_some(index)
            })
            .collect()
    }

    fn threads(&mut self, ui: &mut Ui) {
        let c = Theme::current(ui.ctx()).colors;
        widgets::search(ui, &mut self.query, "Search messages…");
        let visible = self.visible();
        ScrollShadow::new("mail-thread-scroll", ui.available_height())
            .fill(c.background)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 6.;
                if visible.is_empty() {
                    EmptyState::new(
                        Icon::Inbox,
                        "No messages",
                        "Try a different folder or search.",
                    )
                    .show(ui);
                }
                for index in visible {
                    let thread = &mut self.mailbox[index];
                    ui.push_id(index, |ui| {
                        if thread_row(ui, thread, self.selected == Some(index)) {
                            self.selected = Some(index);
                            thread.unread = false;
                        }
                    });
                }
            });
    }

    fn read(&mut self, ui: &mut Ui) {
        let visible = self.visible();
        let Some((index, thread)) = self
            .selected
            .and_then(|index| self.mailbox.get_mut(index).map(|thread| (index, thread)))
        else {
            EmptyState::new(
                Icon::Mail,
                "Select a conversation",
                "Read and reply to messages here.",
            )
            .show(ui);
            return;
        };
        let current = visible.iter().position(|&row| row == index);
        let previous = current
            .and_then(|position| position.checked_sub(1))
            .and_then(|position| visible.get(position))
            .copied();
        let next = current
            .and_then(|position| visible.get(position + 1))
            .copied();
        match reader::show(ui, thread, previous.is_some(), next.is_some()) {
            Some(ReaderAction::Close) => self.selected = None,
            Some(ReaderAction::Previous) => self.selected = previous,
            Some(ReaderAction::Next) => self.selected = next,
            Some(ReaderAction::Reply(email)) => {
                self.source = Some(index);
                self.composer = Some(Composer::reply(email, &thread.subject));
            }
            Some(ReaderAction::EditDraft) => {
                self.source = Some(index);
                self.composer = Composer::draft(thread);
            }
            Some(ReaderAction::Move(folder)) => {
                thread.folder = folder;
                self.selected = None;
            }
            None => return,
        }
        if let Some(thread) = self.selected.and_then(|index| self.mailbox.get_mut(index)) {
            thread.unread = false;
        }
        ui.ctx().request_repaint();
    }
}
