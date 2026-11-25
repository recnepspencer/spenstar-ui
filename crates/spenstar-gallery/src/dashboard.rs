use eframe::egui::{Rect, Sense, Stroke, Ui, pos2, vec2};
use spenstar_ui::{
    Theme,
    button::{Button, Size, Variant},
    icon::Icon,
    layout::{
        Card, CardVariant, EmptyState, ResponsiveGrid, ScrollShadow, SplitAxis, SplitPane, Surface,
        Toolbar,
    },
    typography::Type,
};

use crate::Page;

pub(crate) struct Dashboard {
    split: f32,
    draft: bool,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            split: 380.,
            draft: false,
        }
    }
}

const METRICS: [(&str, &str, &str, Icon); 4] = [
    ("Open shipments", "128", "+12 this week", Icon::Package),
    (
        "On-time delivery",
        "98.2%",
        "+2.4 points",
        Icon::CheckCircle2,
    ),
    ("Active carriers", "24", "Across 6 regions", Icon::Truck),
    ("Exceptions", "3", "1 awaits review", Icon::AlertTriangle),
];

const ACTIVITY: [(&str, &str, Icon); 9] = [
    (
        "Dispatch confirmed",
        "Shipment #SH-0482 · 4 min ago",
        Icon::Truck,
    ),
    (
        "Inventory reconciled",
        "West warehouse · 25 min ago",
        Icon::Boxes,
    ),
    (
        "Arrival confirmed",
        "Shipment #SH-0470 · 42 min ago",
        Icon::Check,
    ),
    (
        "Carrier assignment changed",
        "Shipment #SH-0468 · 1 hour ago",
        Icon::Users,
    ),
    (
        "Route exception flagged",
        "North region · 2 hours ago",
        Icon::AlertCircle,
    ),
    (
        "Manifest downloaded",
        "Shipment #SH-0440 · 3 hours ago",
        Icon::Download,
    ),
    (
        "Pick list generated",
        "East warehouse · 3 hours ago",
        Icon::ClipboardList,
    ),
    (
        "Delivery window updated",
        "Shipment #SH-0428 · 4 hours ago",
        Icon::Calendar,
    ),
    (
        "Inbound order received",
        "Central warehouse · 5 hours ago",
        Icon::Inbox,
    ),
];

impl Dashboard {
    pub(crate) fn show(&mut self, ui: &mut Ui) -> Option<Page> {
        let c = Theme::current(ui.ctx()).colors;
        ResponsiveGrid::new(164.)
            .max_columns(4)
            .show(ui, METRICS.len(), |ui, index| {
                let (label, value, note, icon) = METRICS[index];
                Card::new(CardVariant::Bordered).show(ui, |card| {
                    card.body(|ui| {
                        Toolbar::show(ui, |ui| {
                            icon.show(ui, 16., c.accent.ring);
                            ui.label(Type::Small.text(label).color(c.text_secondary));
                        });
                        ui.add_space(4.);
                        ui.label(Type::H2.text(value));
                        ui.label(Type::Caption.text(note).color(c.text_muted));
                    });
                });
            });
        if ui.available_width() >= 560. {
            SplitPane::new("operations", SplitAxis::Horizontal)
                .limits(248., 700., 244.)
                .show(ui, 270., &mut self.split, throughput, activity);
        } else {
            throughput(ui);
            activity(ui);
        }
        Surface::show(ui, |ui| {
            if self.draft {
                ui.label(Type::H3.text("Shipment draft"));
                ui.label(
                    Type::Body
                        .text("A new draft is ready for carrier and route details.")
                        .color(c.text_secondary),
                );
                if Button::new("Discard draft")
                    .variant(Variant::Outline)
                    .show(ui)
                    .clicked()
                {
                    self.draft = false;
                }
                return None;
            }
            let (primary, secondary) = EmptyState::new(
                Icon::Inbox,
                "No shipment drafts",
                "Start a draft to assign a carrier and schedule a dispatch.",
            )
            .primary(Button::new("New draft").leading(Icon::Plus).size(Size::Sm))
            .secondary(
                Button::new("Browse icons")
                    .variant(Variant::Outline)
                    .size(Size::Sm),
            )
            .show(ui);
            self.draft = primary.is_some_and(|response| response.clicked());
            secondary
                .filter(|response| response.clicked())
                .map(|_| Page::Icons)
        })
        .inner
    }
}

fn throughput(ui: &mut Ui) {
    let c = Theme::current(ui.ctx()).colors;
    Card::new(CardVariant::Elevated).show(ui, |card| {
        card.header("Shipment volume", Some("Daily shipments · last seven days"));
        card.body(|ui| {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 104.), Sense::hover());
            let graph = rect.shrink2(vec2(6., 8.));
            for fraction in [0., 0.33, 0.66, 1.] {
                let y = egui_y(graph, fraction);
                ui.painter()
                    .hline(graph.x_range(), y, Stroke::new(1., c.divider));
            }
            let values = [0.21, 0.44, 0.37, 0.67, 0.57, 0.78, 0.9];
            let mut previous = None;
            for (index, value) in values.into_iter().enumerate() {
                let x = graph.left() + graph.width() * index as f32 / 6.;
                let point = pos2(x, egui_y(graph, value));
                if let Some(previous) = previous {
                    ui.painter()
                        .line_segment([previous, point], Stroke::new(2., c.accent.ring));
                }
                ui.painter().circle_filled(point, 3., c.accent.ring);
                previous = Some(point);
            }
        });
        card.footer(|ui| {
            ui.horizontal(|ui| {
                ui.label(Type::Caption.text("Mon").color(c.text_muted));
                ui.with_layout(
                    eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                    |ui| {
                        ui.label(Type::Caption.text("Sun").color(c.text_muted));
                    },
                );
            });
        });
    });
}

fn egui_y(rect: Rect, fraction: f32) -> f32 {
    rect.bottom() - rect.height() * fraction
}

fn activity(ui: &mut Ui) {
    let c = Theme::current(ui.ctx()).colors;
    Card::new(CardVariant::Bordered).show(ui, |card| {
        card.header("Recent activity", Some("Dispatch and inventory events"));
        card.body(|ui| {
            ScrollShadow::new("activity", 158.).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 9.;
                for (label, note, icon) in ACTIVITY {
                    Toolbar::show(ui, |ui| {
                        icon.show(ui, 16., c.accent.ring);
                        ui.vertical(|ui| {
                            ui.label(Type::Small.text(label));
                            ui.label(Type::Caption.text(note).color(c.text_secondary));
                        });
                    });
                }
            });
        });
    });
}
