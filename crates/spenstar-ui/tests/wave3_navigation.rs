//! Headless input and layout tests for command and navigation widgets.
#[path = "wave3/support.rs"]
mod support;

use std::collections::BTreeSet;

use spenstar_ui::{
    Theme,
    command::{CommandGroup, CommandItem, CommandPalette},
    egui::{Context, Event, FullOutput, Key, Modifiers, Pos2, Rect, TextEdit, epaint::Shape},
    navigation::{
        Accordion, AccordionMode, Disclosure, PageSlot, Tab, Tabs, TabsVariant, page_slots,
    },
};

use support::{click, frame, key, location, texts};

#[test]
fn tabs_move_by_arrows_home_and_end_and_animate_relative_to_origin() {
    let ctx = Context::default();
    let theme = Theme::default();
    theme.install(&ctx);
    let mut active = 0;
    let draw = |time, shift, events, active: &mut usize| {
        frame(&ctx, time, events, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(shift);
                let tabs = ["First", "Second", "Last"].map(Tab::new);
                Tabs {
                    variant: TabsVariant::Underline,
                    ..Tabs::new("directory-tabs")
                }
                .show(ui, &tabs, active);
            });
        })
    };
    let output = draw(0., 12., vec![], &mut active);
    let first = location(&output, "First");
    draw(0.1, 12., click(first), &mut active);
    draw(
        0.2,
        12.,
        vec![key(Key::ArrowRight, Modifiers::NONE)],
        &mut active,
    );
    assert_eq!(active, 1);
    let before = draw(0.24, 12., vec![], &mut active);
    let after = draw(0.24, 120., vec![], &mut active);
    let indicator = |output: &FullOutput| -> Rect {
        output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Rect(rect)
                    if rect.fill == theme.colors.accent.base && rect.rect.height() == 2. =>
                {
                    Some(rect.rect)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing accent underline"))
    };
    let x = indicator(&before).left();
    assert!(x < location(&before, "Second").x && x > first.x - 10.);
    assert!((indicator(&after).left() - x - 108.).abs() < 0.01);
    for (time, keycode, wanted) in [
        (0.4, Key::End, 2),
        (0.5, Key::Home, 0),
        (0.6, Key::ArrowLeft, 2),
    ] {
        draw(time, 120., vec![key(keycode, Modifiers::NONE)], &mut active);
        assert_eq!(active, wanted);
    }
}

#[test]
fn disclosures_animate_and_accordion_modes_constrain_open_sections() {
    let ctx = Context::default();
    Theme::default().install(&ctx);
    let mut open = false;
    let mut height = 0.;
    let mut draw = |time, events, open: &mut bool| {
        let output = frame(&ctx, time, events, |ui| {
            Disclosure::new("release-notes", "Release notes").show(ui, open, |ui| {
                ui.label("Release notes body");
                ui.add_space(35.);
            });
            height = ui.cursor().top();
        });
        (output, height)
    };
    let (initial, collapsed) = draw(0., vec![], &mut open);
    draw(0.1, click(location(&initial, "Release notes")), &mut open);
    assert!(open);
    let (_, halfway) = draw(0.18, vec![], &mut open);
    let (expanded, end) = draw(0.6, vec![], &mut open);
    assert!(
        halfway > collapsed && halfway < end,
        "{collapsed} {halfway} {end}"
    );
    location(&expanded, "Release notes body");
    draw(0.7, click(location(&expanded, "Release notes")), &mut open);
    assert!(!open);
    let (closed, _) = draw(1., vec![], &mut open);
    assert!(
        !texts(&closed)
            .iter()
            .any(|(text, _)| *text == "Release notes body")
    );

    let mut open = BTreeSet::from([0, 1, 99]);
    for (time, mode, expected) in [
        (1.1, AccordionMode::Multiple, BTreeSet::from([0, 1])),
        (1.2, AccordionMode::Single, BTreeSet::from([0])),
    ] {
        frame(&ctx, time, vec![], |ui| {
            Accordion {
                mode,
                ..Accordion::new("accordion")
            }
            .show(
                ui,
                &["First panel", "Second panel"],
                &mut open,
                |ui, index| {
                    ui.label(format!("Content {index}"));
                },
            );
        });
        assert_eq!(open, expected);
    }
}

#[test]
fn page_ellipsis_keeps_endpoints_and_current_page_in_bounds() {
    use PageSlot::{Gap, Page};
    assert_eq!(
        page_slots(4, 24),
        [Page(0), Gap, Page(3), Page(4), Page(5), Gap, Page(23)]
    );
    assert!(page_slots(usize::MAX, 0).is_empty());
    for count in [1, 3, 8, 24, usize::MAX] {
        for current in [0, 1, 4, count / 2, count - 1, usize::MAX] {
            let slots = page_slots(current, count);
            assert_eq!(slots.first(), Some(&Page(0)));
            assert_eq!(slots.last(), Some(&Page(count - 1)));
            assert!(slots.contains(&Page(current.min(count - 1))));
            let pages: Vec<_> = slots
                .iter()
                .filter_map(|slot| match slot {
                    Page(i) => Some(*i),
                    Gap => None,
                })
                .collect();
            assert!(pages.windows(2).all(|pair| pair[0] < pair[1]));
            assert!(slots.len() <= 7);
        }
    }
}

#[test]
fn command_search_navigation_escape_and_guard_handle_platform_shortcuts() {
    let ctx = Context::default();
    Theme::default().install(&ctx);
    let mut palette = CommandPalette::new("command-test");
    let mut draft = String::new();
    let mut input_pos = Pos2::ZERO;
    let mut popup = false;
    let items = [
        CommandItem::new("analytics", "Analytics"),
        CommandItem::new("palette", "Accent palette"),
        CommandItem {
            keywords: "report",
            ..CommandItem::new("pipeline", "CRM Pipeline")
        },
    ];
    let groups = [CommandGroup {
        label: "Pages",
        items: &items,
    }];
    let mut draw = |time, events, editor, popup_open, palette: &mut CommandPalette| {
        let mut result = None;
        let output = frame(&ctx, time, events, |ui| {
            if editor {
                let response = ui.add(TextEdit::singleline(&mut draft));
                input_pos = response.rect.center();
            }
            if popup_open {
                let anchor = ui.button("Popup");
                egui_popup(&anchor, &mut popup);
            }
            result = palette.show(ui, &groups);
        });
        (output, result, input_pos)
    };
    let mac = Modifiers {
        mac_cmd: true,
        command: true,
        ..Modifiers::NONE
    };
    let ctrl = Modifiers {
        ctrl: true,
        command: true,
        ..Modifiers::NONE
    };
    draw(0., vec![key(Key::K, mac)], false, false, &mut palette);
    assert!(palette.is_open());
    draw(
        0.1,
        vec![Event::Text("ANLT".to_owned())],
        false,
        false,
        &mut palette,
    );
    assert_eq!(palette.query(), "ANLT");
    let (filtered, _, _) = draw(0.2, vec![], false, false, &mut palette);
    location(&filtered, "Analytics");
    assert!(
        !texts(&filtered)
            .iter()
            .any(|(text, _)| *text == "CRM Pipeline")
    );
    let (_, picked, _) = draw(
        0.3,
        vec![key(Key::Enter, Modifiers::NONE)],
        false,
        false,
        &mut palette,
    );
    assert_eq!(picked, Some("analytics"));
    assert!(!palette.is_open());
    draw(0.35, vec![], false, false, &mut palette);
    draw(0.4, vec![key(Key::K, ctrl)], false, false, &mut palette);
    assert!(palette.is_open());
    draw(
        0.5,
        vec![key(Key::ArrowUp, Modifiers::NONE)],
        false,
        false,
        &mut palette,
    );
    draw(
        0.6,
        vec![key(Key::ArrowDown, Modifiers::NONE)],
        false,
        false,
        &mut palette,
    );
    draw(
        0.7,
        vec![key(Key::ArrowDown, Modifiers::NONE)],
        false,
        false,
        &mut palette,
    );
    let (_, picked, _) = draw(
        0.8,
        vec![key(Key::Enter, Modifiers::NONE)],
        false,
        false,
        &mut palette,
    );
    assert_eq!(picked, Some("palette"));
    palette.open();
    draw(0.9, vec![], false, false, &mut palette);
    let (empty, _, _) = draw(
        1.,
        vec![Event::Text("zzzz".to_owned())],
        false,
        false,
        &mut palette,
    );
    location(&empty, "No matching commands");
    draw(
        1.1,
        vec![key(Key::Escape, Modifiers::NONE)],
        false,
        false,
        &mut palette,
    );
    assert!(!palette.is_open());
    let (_, _, editor) = draw(1.2, vec![], true, false, &mut palette);
    draw(1.3, click(editor), true, false, &mut palette);
    draw(1.4, vec![key(Key::K, ctrl)], true, false, &mut palette);
    assert!(!palette.is_open(), "focused editor must keep the shortcut");
    ctx.memory_mut(|memory| {
        if let Some(id) = memory.focused() {
            memory.surrender_focus(id);
        }
    });
    draw(1.5, vec![], false, true, &mut palette);
    draw(1.6, vec![key(Key::K, ctrl)], false, true, &mut palette);
    assert!(!palette.is_open(), "an open popup must keep the shortcut");
}

fn egui_popup(anchor: &spenstar_ui::egui::Response, open: &mut bool) {
    *open = true;
    spenstar_ui::egui::Popup::from_response(anchor)
        .open_bool(open)
        .show(|ui| {
            ui.label("Other overlay");
        });
}
