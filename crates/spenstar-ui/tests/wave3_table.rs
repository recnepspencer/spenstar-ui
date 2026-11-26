//! Virtualization, sorting, filtering, resizing, and selection through real input.
#[path = "wave3/support.rs"]
mod support;

use std::cell::Cell;

use spenstar_ui::{
    Theme,
    egui::{
        Context, Event, Modifiers, MouseWheelSource, MouseWheelUnit, PointerButton, TouchPhase,
        pos2, vec2,
    },
    table::{Column, DataTable, Density, SortDirection, TableRows, TableState},
};

use support::{click, frame, location, texts};

#[test]
fn index_work_is_cached_and_scrolling_visits_only_visible_rows() {
    let ctx = Context::default();
    Theme::default().install(&ctx);
    let mut state = TableState::default();
    let scans = Cell::new(0);
    let comparisons = Cell::new(0);
    let mut visited = Vec::new();
    let mut draw = |time, events, revision, page_size, state: &mut TableState| {
        visited.clear();
        let out = frame(&ctx, time, events, |ui| {
            let columns = [Column::new("Project", 1.)];
            let table = DataTable {
                height: 140.,
                density: Density::Compact,
                selectable: true,
                page_size,
                ..DataTable::new("ten-thousand", &columns)
            };
            let source = TableRows::new(
                revision,
                10_000,
                |i| {
                    scans.set(scans.get() + 1);
                    revision == 0 || i % 2 == 0
                },
                |i| 1000 + i as u64,
                |_, a: usize, b| {
                    comparisons.set(comparisons.get() + 1);
                    b.cmp(&a)
                },
            );
            visited.clear();
            let result = table.show(ui, state, source, |ui, row, _| {
                visited.push(row);
                ui.label(format!("Project {row}"));
            });
            assert_eq!(result.rendered_rows, visited.len());
        });
        assert!((1..=7).contains(&visited.len()), "{visited:?}");
        (out, visited[0])
    };
    let (first, initial) = draw(0., vec![], 0, None, &mut state);
    assert_eq!(initial, 0);
    assert_eq!(scans.get(), 10_000);
    let header = location(&first, "Project");
    draw(0.1, click(header), 0, None, &mut state);
    let (_, sorted) = draw(0.2, vec![], 0, None, &mut state);
    assert_eq!(
        state.sort.map(|sort| sort.direction),
        Some(SortDirection::Asc)
    );
    assert_eq!(sorted, 9999);
    assert_eq!(scans.get(), 20_000);
    let comparison_count = comparisons.get();
    assert!(comparison_count >= 9999);
    let point = pos2(header.x, header.y + 70.);
    draw(0.3, vec![Event::PointerMoved(point)], 0, None, &mut state);
    let wheel = Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: vec2(0., -12000.),
        phase: TouchPhase::Move,
        source: MouseWheelSource::Unknown,
        modifiers: Modifiers::NONE,
    };
    draw(0.4, vec![wheel], 0, None, &mut state);
    let (scrolled, first_visible) = draw(0.5, vec![], 0, None, &mut state);
    assert!(first_visible < 9900, "{first_visible}");
    assert_eq!(location(&scrolled, "Project"), header);
    assert_eq!(scans.get(), 20_000);
    assert_eq!(comparisons.get(), comparison_count);
    state.toggle_row(0);
    assert!(state.selected().contains(&10999));
    let (_, filtered) = draw(0.6, vec![], 1, Some(250), &mut state);
    assert_eq!(state.filtered_count(), 5_000);
    assert_eq!(filtered, 9998);
    assert!(state.selected().contains(&10999));
    state.page = 19;
    let (_, last_page) = draw(0.7, vec![], 1, Some(250), &mut state);
    assert_eq!(last_page, 498);
    state.page = usize::MAX;
    draw(0.8, vec![], 1, Some(250), &mut state);
    assert_eq!(state.page, 19);
    assert_eq!(scans.get(), 30_000);
}

#[test]
fn interaction_selects_multiple_rows_and_dispatches_bulk_actions() {
    let ctx = Context::default();
    let theme = Theme::default();
    theme.install(&ctx);
    let mut state = TableState::default();
    let draw = |time, events, state: &mut TableState| {
        let mut action = None;
        let output = frame(&ctx, time, events, |ui| {
            let columns = [Column::new("Name", 1.)];
            let table = DataTable {
                selectable: true,
                bulk_actions: &["Export"],
                page_size: Some(10),
                ..DataTable::new("selection", &columns)
            };
            let source = TableRows::new(0, 30, |_| true, |i| i as u64, |_, a: usize, b| a.cmp(&b));
            action = table
                .show(ui, state, source, |ui, i, _| {
                    ui.label(format!("Name {i}"));
                })
                .bulk_action;
        });
        (output, action)
    };
    let (initial, _) = draw(0., vec![], &mut state);
    let first_row = location(&initial, "Name 0");
    draw(0.1, click(pos2(30., first_row.y)), &mut state);
    assert!(state.selected().contains(&0));
    let (partial, _) = draw(0.2, vec![], &mut state);
    let next_row = location(&partial, "Name 1");
    draw(0.3, click(pos2(30., next_row.y)), &mut state);
    assert_eq!(state.selected().iter().copied().collect::<Vec<_>>(), [0, 1]);
    let (with_bulk, _) = draw(0.4, vec![], &mut state);
    let (_, action) = draw(0.5, click(location(&with_bulk, "Export")), &mut state);
    assert_eq!(action, Some(0));
    let header = location(&with_bulk, "Name");
    draw(0.6, click(pos2(30., header.y)), &mut state);
    assert_eq!(state.selected().len(), 30);
    let (all, _) = draw(0.7, vec![], &mut state);
    draw(0.8, click(location(&all, "Clear")), &mut state);
    assert!(state.selected().is_empty());
    let (page_one, _) = draw(0.9, vec![], &mut state);
    // The pagination footer has a distinct page label even though the row index also contains a 3.
    draw(1., click(location(&page_one, "3")), &mut state);
    assert_eq!(state.page, 2);
    let (page_three, _) = draw(1.1, vec![], &mut state);
    assert!(
        texts(&page_three)
            .iter()
            .any(|(text, _)| *text == "Showing 21–30 of 30")
    );
}

#[test]
fn resizing_clamps_to_minimum_and_empty_and_loading_skip_callbacks() {
    let ctx = Context::default();
    Theme::default().install(&ctx);
    let mut state = TableState::default();
    let draw = |time, events, loading, len, state: &mut TableState| {
        frame(&ctx, time, events, |ui| {
            let columns = [
                Column {
                    min_width: 106.,
                    ..Column::new("Left column", 1.)
                },
                Column::new("Right column", 1.),
            ];
            let source = TableRows::new(0, len, |_| true, |i| i as u64, |_, a: usize, b| a.cmp(&b));
            let table = DataTable {
                loading,
                ..DataTable::new("resize", &columns)
            };
            let result = table.show(ui, state, source, |ui, _, column| {
                assert!(!loading && len > 0);
                ui.label(if column == 0 {
                    "First cell"
                } else {
                    "Second cell"
                });
            });
            assert_eq!(result.rendered_rows, if loading { 0 } else { len });
        })
    };
    let initial = draw(0., vec![], false, 1, &mut state);
    let left = location(&initial, "Left column");
    let right = location(&initial, "Right column");
    let right_ink = texts(&initial)
        .into_iter()
        .find_map(|(text, rect)| (text == "Right column").then_some(rect))
        .unwrap_or_else(|| panic!("missing right column ink"));
    let start = pos2(right_ink.left() - 15., left.y);
    let pressed = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    draw(
        0.1,
        vec![Event::PointerMoved(start), pressed(start, true)],
        false,
        1,
        &mut state,
    );
    let end = pos2(15., start.y);
    draw(0.2, vec![Event::PointerMoved(end)], false, 1, &mut state);
    draw(0.3, vec![pressed(end, false)], false, 1, &mut state);
    let resized = draw(0.4, vec![], false, 1, &mut state);
    let new_right = location(&resized, "Right column");
    assert!(
        new_right.x < 200. && new_right.x > 120.,
        "{new_right:?}, {right:?}"
    );
    draw(0.5, vec![], true, 1, &mut state);
    let empty = draw(0.6, vec![], false, 0, &mut state);
    location(&empty, "No rows to show");
}
