use std::{cmp::Ordering, collections::BTreeSet, ops::Range};

/// Direction of a table sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    /// Low to high.
    Asc,
    /// High to low.
    Desc,
}

/// Active sort. An absent sort restores source order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sort {
    /// Index into the column definitions.
    pub column: usize,
    /// Sort direction.
    pub direction: SortDirection,
}

/// Random-access source callbacks for the table index.
///
/// Increment the revision when filtering, ordering, IDs, or comparison inputs change.
/// Row IDs must be unique and stable through reordering and filtering.
pub struct TableRows<F, I, C> {
    /// Source revision.
    pub revision: u64,
    /// Number of source rows, before filtering.
    pub len: usize,
    /// Whether a source index passes the current filter.
    pub include: F,
    /// Stable ID for a source index.
    pub id: I,
    /// Ascending comparison by column and two source indices.
    pub compare: C,
}

impl<F, I, C> TableRows<F, I, C>
where
    F: FnMut(usize) -> bool,
    I: FnMut(usize) -> u64,
    C: FnMut(usize, usize, usize) -> Ordering,
{
    /// Create a random-access source.
    pub fn new(revision: u64, len: usize, include: F, id: I, compare: C) -> Self {
        Self {
            revision,
            len,
            include,
            id,
            compare,
        }
    }
}

/// Persistent interaction state and the filtered source index.
#[derive(Default)]
pub struct TableState {
    /// Current zero-based page.
    pub page: usize,
    /// Active sort.
    pub sort: Option<Sort>,
    pub(super) widths: Vec<Option<f32>>,
    pub(super) drag: Option<(usize, f32)>,
    index: Vec<(usize, u64)>,
    key: Option<(u64, usize, Option<Sort>)>,
    selected: BTreeSet<u64>,
    matching_selected: usize,
    viewport_key: Option<(usize, usize, Option<usize>)>,
}

impl TableState {
    /// IDs currently selected, including selected rows hidden by filters.
    pub fn selected(&self) -> &BTreeSet<u64> {
        &self.selected
    }

    /// Number of rows after filtering.
    pub fn filtered_count(&self) -> usize {
        self.index.len()
    }

    /// Source index and stable ID at a filtered and sorted position.
    pub fn row_at(&self, position: usize) -> Option<(usize, u64)> {
        self.index.get(position).copied()
    }

    /// Toggle a row by its filtered position.
    pub fn toggle_row(&mut self, position: usize) {
        if let Some((_, id)) = self.row_at(position) {
            if self.selected.remove(&id) {
                self.matching_selected -= 1;
            } else {
                self.selected.insert(id);
                self.matching_selected += 1;
            }
        }
    }

    /// Select all filtered rows or deselect them if all are already selected.
    pub fn toggle_all(&mut self) {
        if self.matching_selected == self.index.len() {
            for &(_, id) in &self.index {
                self.selected.remove(&id);
            }
            self.matching_selected = 0;
        } else {
            self.selected.extend(self.index.iter().map(|&(_, id)| id));
            self.matching_selected = self.index.len();
        }
    }

    /// Clear the selection.
    pub fn clear_selection(&mut self) {
        self.selected.clear();
        self.matching_selected = 0;
    }

    /// Drop selection for rows removed from the caller's source.
    pub fn retain_selection(&mut self, keep: impl FnMut(&u64) -> bool) {
        self.selected.retain(keep);
        self.count_matching();
    }

    /// Advance a column through ascending, descending, and source order.
    pub fn cycle_sort(&mut self, column: usize) {
        self.sort = match self.sort {
            Some(Sort {
                column: current,
                direction: SortDirection::Asc,
            }) if current == column => Some(Sort {
                column,
                direction: SortDirection::Desc,
            }),
            Some(Sort {
                column: current,
                direction: SortDirection::Desc,
            }) if current == column => None,
            _ => Some(Sort {
                column,
                direction: SortDirection::Asc,
            }),
        };
        self.page = 0;
    }

    /// The window of filtered positions for a page size, clamping the current page.
    pub fn page_range(&mut self, per_page: Option<usize>) -> Range<usize> {
        let Some(size) = per_page.map(|size| size.max(1)) else {
            return 0..self.index.len();
        };
        self.page = self
            .page
            .min(self.index.len().div_ceil(size).saturating_sub(1));
        let start = self.page * size;
        start..start.saturating_add(size).min(self.index.len())
    }

    /// Refresh the index if the source revision or active sort changed.
    pub fn update<F, I, C>(&mut self, mut source: TableRows<F, I, C>) -> bool
    where
        F: FnMut(usize) -> bool,
        I: FnMut(usize) -> u64,
        C: FnMut(usize, usize, usize) -> Ordering,
    {
        let key = (source.revision, source.len, self.sort);
        if self.key == Some(key) {
            return false;
        }
        self.key = Some(key);
        self.index.clear();
        self.index.extend(
            (0..source.len)
                .filter(|&index| (source.include)(index))
                .map(|index| (index, (source.id)(index))),
        );
        if let Some(sort) = self.sort {
            self.index.sort_by(|&(a, _), &(b, _)| {
                let ascending = (source.compare)(sort.column, a, b);
                match sort.direction {
                    SortDirection::Asc => ascending,
                    SortDirection::Desc => ascending.reverse(),
                }
            });
        }
        self.count_matching();
        true
    }

    fn count_matching(&mut self) {
        self.matching_selected = self
            .index
            .iter()
            .filter(|(_, id)| self.selected.contains(id))
            .count();
    }

    pub(super) fn selection_status(&self) -> (bool, bool) {
        (
            self.matching_selected > 0 && self.matching_selected == self.index.len(),
            self.matching_selected > 0 && self.matching_selected < self.index.len(),
        )
    }

    pub(super) fn viewport_changed(&mut self, size: Option<usize>, density: usize) -> bool {
        let key = (self.page, density, size);
        self.viewport_key.replace(key) != Some(key)
    }
}
