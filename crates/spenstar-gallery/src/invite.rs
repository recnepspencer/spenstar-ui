use eframe::egui::{self, Align, Id, Layout, Ui};
use spenstar_ui::{
    Theme,
    button::{Button, CloseButton},
    form::{Field, TextField},
    icon::Icon,
    typography::Type,
};

#[derive(Default)]
pub(super) struct Invitations {
    email: String,
    pending: Vec<String>,
    open: bool,
    notice: Option<String>,
}

impl Invitations {
    pub(super) fn show(&mut self, ui: &mut Ui, open: bool) {
        if open {
            self.open = true;
            self.email.clear();
            self.notice = None;
        }
        let c = Theme::current(ui.ctx()).colors;
        if !self.open {
            return;
        }
        let email = self.email.trim();
        let valid = email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && domain.contains('.') && !email.contains(char::is_whitespace)
        });
        let duplicate = self
            .pending
            .iter()
            .any(|invite| invite.eq_ignore_ascii_case(email));
        let modal = egui::Modal::new(Id::unique("spenstar-gallery/invite"))
            .backdrop_color(c.scrim)
            .show(ui.ctx(), |ui| {
                ui.set_width(360.);
                let close = ui
                    .horizontal(|ui| {
                        ui.label(Type::H2.text("Invite a team member"));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            CloseButton::show(ui).clicked()
                        })
                        .inner
                    })
                    .inner;
                if let Some(notice) = &self.notice {
                    ui.label(Type::Body.text(notice));
                    return (close || Button::new("Done").show(ui).clicked(), false);
                }
                Field {
                    description: Some("The invitation is recorded in this demo session."),
                    error: duplicate
                        .then_some("An invitation is already pending for this address."),
                    ..Field::new("Email address")
                }
                .show(ui, |ui, invalid| {
                    TextField {
                        invalid,
                        ..TextField::new("name@example.com")
                    }
                    .show(ui, &mut self.email);
                });
                let submit = Button::new("Add invitation")
                    .leading(Icon::Plus)
                    .enabled(valid && !duplicate)
                    .show(ui)
                    .clicked();
                (close, submit)
            });
        let (close, submit) = modal.inner;
        if submit {
            let email = self.email.trim().to_owned();
            self.notice = Some(format!("Demo invitation pending for {email}"));
            self.pending.push(email);
            crate::widgets::notify(
                ui.ctx(),
                "Invitation added",
                spenstar_ui::feedback::Tone::Success,
            );
        }
        if close || modal.should_close() {
            self.open = false;
        }
    }
}
