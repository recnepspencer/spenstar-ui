use eframe::egui::{self, Align, Layout, Response, Ui, vec2};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    feedback::{Toast, ToastQueue, Tone},
    form::{FieldVariant, SearchField},
    icon::Icon,
    select::{Select, SelectOption, SelectSection, SelectStyle},
    tags::{Tag, TagGroup},
};

pub(super) fn width<R>(ui: &mut Ui, points: f32, content: impl FnOnce(&mut Ui) -> R) -> R {
    sized_width(ui, points, 18., content)
}

pub(super) fn sized_width<R>(
    ui: &mut Ui,
    points: f32,
    height: f32,
    content: impl FnOnce(&mut Ui) -> R,
) -> R {
    let points = points.min(ui.available_width()).max(0.);
    ui.allocate_ui_with_layout(vec2(points, height), Layout::top_down(Align::Min), |ui| {
        ui.set_width(points);
        content(ui)
    })
    .inner
}

pub(super) fn select(
    ui: &mut Ui,
    selected: &mut Option<String>,
    labels: &[&str],
    placeholder: &str,
) -> Response {
    select_styled(ui, selected, labels, placeholder, FieldVariant::default())
}

pub(super) fn select_styled(
    ui: &mut Ui,
    selected: &mut Option<String>,
    labels: &[&str],
    placeholder: &str,
    variant: FieldVariant,
) -> Response {
    let options: Vec<_> = labels
        .iter()
        .map(|label| SelectOption::new(label, label))
        .collect();
    Select {
        style: SelectStyle {
            placeholder,
            variant,
            ..SelectStyle::default()
        },
        ..Select::default()
    }
    .show(
        ui,
        selected,
        &[SelectSection {
            label: None,
            options: &options,
        }],
    )
    .response
}

pub(super) fn search(ui: &mut Ui, query: &mut String, placeholder: &str) -> Response {
    SearchField {
        placeholder,
        ..SearchField::default()
    }
    .show(ui, query)
    .response
}

pub(super) fn action(ui: &mut Ui, label: &str, icon: Icon) -> Response {
    Button::new(label)
        .icon_only(icon)
        .variant(Variant::Secondary)
        .show(ui)
}

pub(super) fn pill(ui: &mut Ui, label: &str, icon: Icon, active: bool) -> Response {
    Button::new(label)
        .leading(icon)
        .size(Size::Sm)
        .variant(if active {
            Variant::Soft
        } else {
            Variant::Secondary
        })
        .show(ui)
}

pub(super) fn tag(ui: &mut Ui, label: &str, tone: Tone) {
    TagGroup::default().show(
        ui,
        &[Tag {
            tone,
            ..Tag::new(label)
        }],
    );
}

pub(super) fn popover<R>(
    anchor: &Response,
    label: &str,
    open: &mut bool,
    width: f32,
    content: impl FnOnce(&mut Ui) -> R,
) -> Option<egui::InnerResponse<R>> {
    if anchor.clicked() {
        *open = !*open;
    }
    let theme = Theme::current(&anchor.ctx);
    let margin = 16;
    egui::Popup::from_response(anchor)
        .open_bool(open)
        .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
        .align(egui::RectAlign::BOTTOM_END)
        .gap(8.)
        .width(width + f32::from(margin) * 2. + 2.)
        .accessible_name(label)
        .frame(
            egui::Frame::new()
                .fill(theme.colors.overlay)
                .stroke(egui::Stroke::new(1., theme.colors.overlay_border))
                .corner_radius(16)
                .inner_margin(egui::Margin::same(margin))
                .shadow(theme.shadow()),
        )
        .show(|ui| {
            ui.set_width(width);
            ui.visuals_mut().extreme_bg_color = theme.colors.surface_secondary;
            content(ui)
        })
}

pub(super) fn toolbar(
    ui: &mut Ui,
    breakpoint: f32,
    leading: impl FnOnce(&mut Ui),
    trailing: impl FnOnce(&mut Ui),
) {
    let trailing = |ui: &mut Ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), trailing)
            .inner
    };
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 38.;
        if ui.available_width() >= breakpoint {
            ui.horizontal(|ui| {
                leading(ui);
                trailing(ui);
            });
        } else {
            ui.horizontal_wrapped(leading);
            ui.horizontal(trailing);
        }
    });
}

pub(super) fn notify(ctx: &egui::Context, label: &str, tone: Tone) {
    ToastQueue::notify(ctx, Toast::new(label).tone(tone));
}
