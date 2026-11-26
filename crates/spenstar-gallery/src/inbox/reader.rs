use eframe::egui::{self, Align, Id, Layout, ScrollArea, Sense, Ui, vec2};
use spenstar_ui::{
    Theme,
    button::{Button, CloseButton, Size, Variant},
    form::{Field, TextArea, TextField},
    icon::Icon,
    layout::{Card, Separator},
    typography::Type,
};

use super::{Folder, Message, Thread};
use crate::orders::initials;

pub(super) enum ReaderAction {
    Close,
    Previous,
    Next,
    Reply(String),
    EditDraft,
    Move(Folder),
}

pub(super) fn show(
    ui: &mut Ui,
    thread: &mut Thread,
    previous: bool,
    next: bool,
) -> Option<ReaderAction> {
    let height = (ui.available_height() - 38.).max(144.);
    let c = Theme::current(ui.ctx()).colors;
    Card::default()
        .show(ui, |card| {
            card.body(|ui| {
                ui.set_min_height(height);
                let mut action = None;
                ui.horizontal_wrapped(|ui| {
                    if CloseButton::show(ui).clicked() {
                        action = Some(ReaderAction::Close);
                    }
                    if icon(
                        ui,
                        "Move to trash",
                        Icon::Trash2,
                        thread.folder != Folder::Trash,
                    )
                    .clicked()
                    {
                        action = Some(ReaderAction::Move(Folder::Trash));
                    }
                    let archived = thread.folder == Folder::Archive;
                    if icon(
                        ui,
                        if archived {
                            "Move to inbox"
                        } else {
                            "Archive conversation"
                        },
                        Icon::Boxes,
                        true,
                    )
                    .clicked()
                    {
                        action = Some(ReaderAction::Move(if archived {
                            Folder::Inbox
                        } else {
                            Folder::Archive
                        }));
                    }
                    if star(ui, thread.starred) {
                        thread.starred = !thread.starred;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if icon(ui, "Next conversation", Icon::ChevronRight, next).clicked() {
                            action = Some(ReaderAction::Next);
                        }
                        if icon(ui, "Previous conversation", Icon::ChevronLeft, previous).clicked()
                        {
                            action = Some(ReaderAction::Previous);
                        }
                        if ui.available_width() >= 65. {
                            let count = thread.messages.len();
                            ui.label(
                                Type::Small
                                    .text(if count == 1 {
                                        "1 message".into()
                                    } else {
                                        format!("{count} messages")
                                    })
                                    .color(c.text_muted),
                            );
                        }
                    });
                });
                Separator::Horizontal.show(ui);
                let mut font = Type::H1.font();
                font.size = 18.;
                ui.label(
                    egui::RichText::new(&thread.subject)
                        .font(font)
                        .color(c.text_primary),
                );
                if thread.folder == Folder::Drafts
                    && Button::new("Edit draft")
                        .leading(Icon::Pencil)
                        .size(Size::Sm)
                        .show(ui)
                        .clicked()
                {
                    return Some(ReaderAction::EditDraft);
                }
                ScrollArea::vertical()
                    .id_salt("message-scroll")
                    .max_height((height - 138.).max(90.))
                    .show(ui, |ui| {
                        for (index, message) in thread.messages.iter().enumerate() {
                            if index > 0 {
                                Separator::Horizontal.show(ui);
                            }
                            ui.push_id(index, |ui| {
                                if show_message(ui, message) {
                                    action = Some(ReaderAction::Reply(message.email.clone()));
                                }
                            });
                        }
                    });
                action
            })
        })
        .inner
}

fn show_message(ui: &mut Ui, message: &Message) -> bool {
    let c = Theme::current(ui.ctx()).colors;
    let mut reply = false;
    ui.horizontal_wrapped(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(36., 36.), Sense::hover());
        initials(ui, rect, &message.sender, c.accent.soft, c.text_primary);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.;
            ui.label(Type::Brand.text(&message.sender));
            ui.label(Type::Small.text("to me").color(c.text_muted))
                .on_hover_text(&message.email);
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let more = icon(ui, "Copy sender address", Icon::MoreHorizontal, true);
            if more.clicked() {
                ui.ctx().copy_text(message.email.clone());
            }
            let id = ui.make_persistent_id("star-message");
            let mut starred = ui
                .data(|data| data.get_temp::<bool>(id))
                .unwrap_or_default();
            if star(ui, starred) {
                starred = !starred;
                ui.data_mut(|data| data.insert_temp(id, starred));
            }
            reply = icon(ui, "Reply", Icon::ArrowLeft, true).clicked();
            if ui.available_width() >= 102. {
                ui.label(Type::Small.text(&message.time).color(c.text_muted));
            }
        });
    });
    ui.label(Type::Body.text(&message.body).color(c.text_secondary));
    reply
}

fn icon(ui: &mut Ui, label: &str, icon: Icon, enabled: bool) -> egui::Response {
    Button::new(label)
        .icon_only(icon)
        .size(Size::Sm)
        .variant(Variant::Ghost)
        .enabled(enabled)
        .show(ui)
}

fn star(ui: &mut Ui, starred: bool) -> bool {
    Button::new(if starred { "Unstar" } else { "Star" })
        .icon_only(Icon::Star)
        .size(Size::Sm)
        .variant(if starred {
            Variant::Soft
        } else {
            Variant::Ghost
        })
        .show(ui)
        .clicked()
}

#[derive(Default)]
pub(super) struct Composer {
    to: String,
    subject: String,
    body: String,
}

impl Composer {
    pub(super) fn reply(to: String, subject: &str) -> Self {
        let subject = if subject.starts_with("Re: ") {
            subject.to_owned()
        } else {
            format!("Re: {subject}")
        };
        Self {
            to,
            subject,
            body: String::new(),
        }
    }

    pub(super) fn draft(thread: &Thread) -> Option<Self> {
        let last = thread.messages.last()?;
        Some(Self {
            to: last.email.clone(),
            subject: thread.subject.clone(),
            body: last.body.clone(),
        })
    }

    pub(super) fn show(&mut self, ctx: &egui::Context) -> Option<Option<Thread>> {
        let c = Theme::current(ctx).colors;
        let modal = egui::Modal::new(Id::unique("spenstar-gallery/compose"))
            .backdrop_color(c.scrim)
            .show(ctx, |ui| {
                ui.set_width(ctx.content_rect().width().clamp(240., 450.) - 32.);
                let close = ui
                    .horizontal(|ui| {
                        ui.label(Type::H2.text("New message"));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            CloseButton::show(ui).clicked()
                        })
                        .inner
                    })
                    .inner;
                Field::new("To").show(ui, |ui, _| {
                    TextField::new("name@example.com").show(ui, &mut self.to)
                });
                Field::new("Subject").show(ui, |ui, _| {
                    TextField::new("Message subject").show(ui, &mut self.subject)
                });
                TextArea {
                    min_rows: 5,
                    placeholder: "Write a message…",
                    ..TextArea::default()
                }
                .show(ui, &mut self.body);
                ui.label(
                    Type::Small
                        .text("Messages are stored in this demo session.")
                        .color(c.text_muted),
                );
                let complete = self.to.contains('@')
                    && !self.subject.trim().is_empty()
                    && !self.body.trim().is_empty();
                let changed =
                    !self.to.is_empty() || !self.subject.is_empty() || !self.body.is_empty();
                let (send, save) = ui
                    .horizontal(|ui| {
                        let send = Button::new("Send")
                            .leading(Icon::Send)
                            .enabled(complete)
                            .show(ui)
                            .clicked();
                        let save = Button::new("Save draft")
                            .variant(Variant::Secondary)
                            .enabled(changed)
                            .show(ui)
                            .clicked();
                        (send, save)
                    })
                    .inner;
                (close, send, save)
            });
        let (close, send, save) = modal.inner;
        if send || save {
            crate::widgets::notify(
                ctx,
                if send { "Message sent" } else { "Draft saved" },
                spenstar_ui::feedback::Tone::Success,
            );
            let thread = Thread {
                subject: if self.subject.is_empty() {
                    "(No subject)".into()
                } else {
                    std::mem::take(&mut self.subject)
                },
                folder: if send { Folder::Sent } else { Folder::Drafts },
                label: "Work",
                starred: false,
                unread: false,
                messages: vec![Message {
                    sender: "Alex Morgan".into(),
                    email: std::mem::take(&mut self.to),
                    time: "Just now".into(),
                    body: std::mem::take(&mut self.body),
                }],
            };
            Some(Some(thread))
        } else if close || modal.should_close() {
            Some(None)
        } else {
            None
        }
    }
}
