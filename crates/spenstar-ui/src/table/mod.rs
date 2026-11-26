//! Virtualized tables with stable row identity and cached filtering and sorting.
mod header;
mod model;
mod view;

pub use model::{Sort, SortDirection, TableRows, TableState};
pub use view::{DataTable, TableResponse};

use egui::Align;

/// Column definition. The index in the supplied slice is the sort key.
#[derive(Clone, Copy, Debug)]
pub struct Column<'a> {
    /// Header text.
    pub title: &'a str,
    /// Relative share of remaining width before the user resizes it.
    pub weight: f32,
    /// Minimum width in points.
    pub min_width: f32,
    /// Cell alignment.
    pub align: Align,
    /// Enable the three-state sort cycle.
    pub sortable: bool,
}

impl<'a> Column<'a> {
    /// Create a sortable left-aligned column.
    pub const fn new(title: &'a str, weight: f32) -> Self {
        Self {
            title,
            weight,
            min_width: 72.,
            align: Align::Min,
            sortable: true,
        }
    }

    /// Set the horizontal cell alignment.
    pub const fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Set the column to display-only.
    pub const fn unsorted(mut self) -> Self {
        self.sortable = false;
        self
    }
}

/// Fixed row height for virtualized rendering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    /// 34-point rows.
    Compact,
    /// 44-point rows.
    #[default]
    Comfortable,
    /// 54-point rows.
    Spacious,
}

impl Density {
    /// Height in logical points, including the divider.
    pub const fn height(self) -> f32 {
        match self {
            Self::Compact => 34.,
            Self::Comfortable => 44.,
            Self::Spacious => 54.,
        }
    }
}
