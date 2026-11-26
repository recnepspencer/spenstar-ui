use std::time::Duration;

use eframe::egui::{self, Context, Frame, Margin, Stroke, Ui, vec2};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    feedback::{
        Alert, Meter, ProgressBar, ProgressCircle, Skeleton, Spinner, Toast, ToastQueue, Tone,
    },
    icon::Icon,
    overlay::{
        AlertDialog, AlertDialogResponse, ContextMenu, Drawer, DrawerSide, DropdownMenu, MenuItem,
        Modal, Popover, Tooltip,
    },
    typography::Type,
};

use crate::widgets::notify;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum OverlayScene {
    #[default]
    None,
    Toasts,
    Modal,
    Alert,
    Drawer,
    Dropdown,
    WorkbenchDrawer,
}

#[derive(Default)]
pub(super) struct Showcase {
    modal: bool,
    alert: bool,
    drawer: bool,
    side: DrawerSide,
    popover: bool,
    dismiss_banner: bool,
    checked: bool,
    note: String,
}

impl Showcase {
    pub(super) fn capture(&mut self, ctx: &Context, scene: OverlayScene) {
        self.modal = scene == OverlayScene::Modal;
        self.alert = scene == OverlayScene::Alert;
        self.drawer = scene == OverlayScene::Drawer;
        self.side = DrawerSide::Left;
        let menu = DropdownMenu::new("showcase-menu");
        if scene == OverlayScene::Dropdown {
            menu.open(ctx);
        } else {
            menu.close(ctx);
        }
        if scene == OverlayScene::Toasts {
            for (title, description, tone) in [
                (
                    "Deploy complete",
                    "Production is up to date.",
                    Tone::Success,
                ),
                (
                    "Syncing workspace",
                    "Updating project metadata.",
                    Tone::Accent,
                ),
                (
                    "Storage nearing capacity",
                    "Review your workspace usage.",
                    Tone::Warning,
                ),
            ] {
                ToastQueue::notify(
                    ctx,
                    Toast::new(title)
                        .description(description)
                        .tone(tone)
                        .duration(Duration::from_secs(2)),
                );
            }
        }
    }

    pub(super) fn show(&mut self, ui: &mut Ui) {
        let c = Theme::current(ui.ctx()).colors;
        ui.spacing_mut().item_spacing.y = 9.;
        ui.label(
            Type::Description
                .text("Dialogs, contextual actions, and live status for workspace workflows.")
                .color(c.text_secondary),
        );
        section(ui, "Overlays", |ui| self.triggers(ui));
        section(ui, "Notifications", |ui| {
            ui.horizontal_wrapped(|ui| {
                for (name, message, tone) in [
                    ("Neutral", "Workspace updated", Tone::Neutral),
                    ("Accent", "Syncing workspace", Tone::Accent),
                    ("Success", "Changes saved", Tone::Success),
                    ("Warning", "Storage nearing capacity", Tone::Warning),
                    ("Danger", "Upload failed", Tone::Danger),
                ] {
                    if Button::new(name)
                        .variant(Variant::Outline)
                        .size(Size::Sm)
                        .show(ui)
                        .clicked()
                    {
                        let mut toast = Toast::new(message).tone(tone);
                        if tone == Tone::Danger {
                            toast = toast
                                .description("Check the network connection.")
                                .action("Retry");
                        }
                        ToastQueue::notify(ui.ctx(), toast);
                    }
                }
                if Button::new("Stack toasts")
                    .variant(Variant::Soft)
                    .size(Size::Sm)
                    .show(ui)
                    .clicked()
                {
                    for (label, tone) in [
                        ("Queued", Tone::Neutral),
                        ("Building", Tone::Accent),
                        ("Build complete", Tone::Success),
                    ] {
                        notify(ui.ctx(), label, tone);
                    }
                }
            });
        });
        ui.add_space(1.);
        if !self.dismiss_banner {
            let banner = Alert::new("Workspace is in sync")
                .description("All queued changes were pushed to origin/main.")
                .tone(Tone::Success)
                .action("Details")
                .dismissible(true)
                .show(ui);
            self.dismiss_banner |= banner.dismissed;
            if banner.action_clicked {
                notify(ui.ctx(), "No pending changes", Tone::Success);
            }
        }
        section(ui, "Progress & loading", |ui| {
            if ui.available_width() >= 580. {
                ui.columns(2, |columns| {
                    tracks(&mut columns[0]);
                    indicators(&mut columns[1]);
                });
            } else {
                tracks(ui);
                indicators(ui);
            }
        });
        self.panels(ui.ctx());
    }

    fn triggers(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            self.modal |= trigger(ui, "Modal", Icon::Layers).clicked();
            self.alert |= trigger(ui, "Delete confirmation", Icon::Trash2).clicked();
            for (side, label) in [
                (DrawerSide::Right, "Right drawer"),
                (DrawerSide::Left, "Left drawer"),
                (DrawerSide::Bottom, "Bottom sheet"),
            ] {
                if trigger(ui, label, Icon::PanelLeft).clicked() {
                    self.side = side;
                    self.drawer = true;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            let anchor = trigger(ui, "Popover", Icon::Info);
            Popover::new("showcase-popover").show(&anchor, &mut self.popover, |ui| {
                ui.label(Type::Label.text("Workspace details"));
                ui.label(Type::Description.text("The primary workspace has 3 active projects."));
                if Button::new("Copy workspace name")
                    .variant(Variant::Soft)
                    .size(Size::Sm)
                    .show(ui)
                    .clicked()
                {
                    ui.ctx().copy_text("Primary workspace".into());
                }
            });
            let dropdown = trigger(ui, "Dropdown menu", Icon::ChevronDown);
            let children = [
                MenuItem::Heading("EXPORT FORMATS"),
                MenuItem::action("markdown", "Markdown")
                    .leading(Icon::FileText)
                    .shortcut("⌘ ⇧ M"),
                MenuItem::action("csv", "CSV")
                    .leading(Icon::Table)
                    .shortcut("⌘ ⇧ E"),
            ];
            let items = [
                MenuItem::Heading("WORKSPACE ACTIONS"),
                MenuItem::submenu("Export as", &children).leading(Icon::Download),
                MenuItem::action("copy", "Copy link")
                    .leading(Icon::Link)
                    .shortcut("⌘ C"),
                MenuItem::action("pin", "Pin workspace").checked(self.checked),
                MenuItem::action("share", "Share workspace")
                    .leading(Icon::Share2)
                    .enabled(false),
                MenuItem::Separator,
                MenuItem::action("delete", "Delete workspace")
                    .leading(Icon::Trash2)
                    .destructive(),
            ];
            if let Some(action) = DropdownMenu::new("showcase-menu").show(&dropdown, &items) {
                match action {
                    "copy" => ui.ctx().copy_text("spenstar://workspace/primary".into()),
                    "pin" => self.checked = !self.checked,
                    "delete" => self.alert = true,
                    _ => notify(ui.ctx(), "Export ready", Tone::Success),
                }
            }

            let target = Button::new("Right-click this card")
                .leading(Icon::MoreHorizontal)
                .radius(spenstar_ui::theme::Radius::Md)
                .variant(Variant::Outline)
                .size(Size::Sm)
                .show(ui);
            let actions = [
                MenuItem::action("open", "Open workspace").leading(Icon::FolderOpen),
                MenuItem::action("copy", "Copy workspace").leading(Icon::Copy),
            ];
            if let Some(action) = ContextMenu::new("showcase-context").show(&target, &actions) {
                if action == "copy" {
                    ui.ctx().copy_text("Primary workspace".into());
                } else {
                    notify(ui.ctx(), "Workspace opened", Tone::Accent);
                }
            }
            let help = trigger(ui, "Hover for help", Icon::HelpCircle);
            Tooltip::new("Show workspace commands")
                .shortcut("⌘ K")
                .show(&help);
        });
    }

    fn panels(&mut self, ctx: &Context) {
        let saved = Modal::new("showcase-modal", "Workspace preferences")
            .subtitle("Adjust how this workspace appears in your project list.")
            .show(
                ctx,
                &mut self.modal,
                |ui| {
                    ui.label(Type::Label.text("Display name"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.note)
                            .hint_text("Primary workspace")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(5.);
                    Alert::new("Workspace members can see this name")
                        .tone(Tone::Neutral)
                        .show(ui);
                },
                |ui| {
                    Button::new("Save preferences")
                        .variant(Variant::Soft)
                        .show(ui)
                        .clicked()
                },
            );
        if saved.is_some_and(|result| result.footer) {
            self.modal = false;
            notify(ctx, "Preferences saved", Tone::Success);
        }
        if AlertDialog::new(
            "showcase-alert",
            "Delete workspace?",
            "Deployment history and saved settings for this workspace will be permanently removed.",
        )
        .tone(Tone::Danger)
        .actions("Cancel", "Delete workspace")
        .show(ctx, &mut self.alert)
            == AlertDialogResponse::Confirmed
        {
            notify(ctx, "Workspace deleted", Tone::Danger);
        }
        let saved = Drawer::new("showcase-drawer", "Workspace activity", self.side)
            .extent(410.)
            .subtitle("Recent changes across this workspace")
            .show_with_footer(
                ctx,
                &mut self.drawer,
                |ui| {
                    for (title, note, tone) in [
                        (
                            "Build completed",
                            "Production deployment · Just now",
                            Tone::Success,
                        ),
                        (
                            "Review requested",
                            "Workspace settings · 18 minutes ago",
                            Tone::Accent,
                        ),
                        (
                            "Storage threshold reached",
                            "Workspace quota · 1 hour ago",
                            Tone::Warning,
                        ),
                    ] {
                        Alert::new(title).description(note).tone(tone).show(ui);
                        ui.add_space(6.);
                    }
                },
                |ui| {
                    Button::new("Done")
                        .variant(Variant::Soft)
                        .show(ui)
                        .clicked()
                },
            );
        if saved.is_some_and(|result| result.footer) {
            self.drawer = false;
            ctx.request_repaint();
        }
    }
}

fn section(ui: &mut Ui, title: &str, content: impl FnOnce(&mut Ui)) {
    let c = Theme::current(ui.ctx()).colors;
    let width = ui.available_width();
    Frame::new()
        .fill(c.surface)
        .stroke(Stroke::new(1., c.border_subtle))
        .corner_radius(12)
        .inner_margin(Margin::same(13))
        .show(ui, |ui| {
            ui.set_width((width - 28.).max(1.));
            ui.label(Type::H3.text(title));
            ui.add_space(3.);
            content(ui);
        });
}

fn trigger(ui: &mut Ui, name: &str, icon: Icon) -> egui::Response {
    Button::new(name)
        .leading(icon)
        .size(Size::Sm)
        .variant(Variant::Secondary)
        .show(ui)
}

fn tracks(ui: &mut Ui) {
    ProgressBar::new(0.72)
        .label("Deploying release")
        .percentage(true)
        .show(ui);
    ui.add_space(8.);
    ProgressBar::indeterminate()
        .label("Indexing changes")
        .size(Size::Sm)
        .show(ui);
    ui.add_space(8.);
    Meter::new(0.78)
        .thresholds(0.70, 0.97)
        .labels("Workspace storage", Some("78 GB / 100 GB"))
        .show(ui);
    ui.add_space(12.);
    ui.horizontal(|ui| {
        ProgressCircle::new(0.64)
            .percentage(true)
            .tone(Tone::Success)
            .diameter(46.)
            .show(ui);
        ui.label(Type::Description.text("Checks complete"));
    });
}

fn indicators(ui: &mut Ui) {
    ui.horizontal(|ui| {
        for (size, name) in [
            (Size::Sm, "Small"),
            (Size::Md, "Medium"),
            (Size::Lg, "Large"),
        ] {
            ui.horizontal(|ui| {
                Spinner::new(size).show(ui);
                ui.label(Type::Small.text(name));
            });
            ui.add_space(7.);
        }
    });
    ui.add_space(6.);
    Skeleton::card(ui);
    Skeleton::rect(vec2(ui.available_width() * 0.48, 8.)).show(ui);
}
