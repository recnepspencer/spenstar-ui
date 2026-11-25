use spenstar_ui::controls::Tone;

pub(super) const STATES: &[&str] = &["In transit", "Ready", "Needs review", "Delivered"];
pub(super) const HUBS: &[&str] = &["North depot", "West warehouse", "Central hub"];

#[derive(Clone)]
pub(super) struct Shipment {
    pub id: String,
    pub name: String,
    pub hub: String,
    pub carrier: String,
    pub status: String,
    pub notes: String,
    pub day: u8,
}

impl Shipment {
    pub fn tone(&self) -> Tone {
        match self.status.as_str() {
            "In transit" => Tone::Accent,
            "Needs review" => Tone::Warning,
            "Delivered" => Tone::Success,
            _ => Tone::Neutral,
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        [&self.id, &self.name, &self.hub, &self.carrier, &self.status]
            .iter()
            .any(|value| value.to_lowercase().contains(query))
    }
}

pub(super) fn samples() -> Vec<Shipment> {
    [
        (
            "0482",
            "Pacific replenishment",
            0,
            "Northline Freight",
            0,
            25,
        ),
        ("0481", "Retail restock", 1, "Summit Logistics", 1, 24),
        (
            "0480",
            "Temperature-controlled freight",
            2,
            "Juniper Express",
            2,
            24,
        ),
        ("0479", "Cross-dock transfer", 0, "Northline Freight", 3, 23),
        ("0478", "Store fixtures", 2, "Summit Logistics", 3, 21),
        ("0477", "Packaging supplies", 1, "Juniper Express", 3, 20),
    ]
    .into_iter()
    .map(|(id, name, hub, carrier, status, day)| Shipment {
        id: format!("SHP-{id}"),
        name: name.into(),
        hub: HUBS[hub].into(),
        carrier: carrier.into(),
        status: STATES[status].into(),
        notes: if id == "0482" {
            "Check in at dock 4. Verify the seal before departure.".into()
        } else {
            String::new()
        },
        day,
    })
    .collect()
}
