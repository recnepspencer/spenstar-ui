# spenstar-ui

Dashboard layouts, controls, typography, and HiDPI SVG icons for [egui 0.37](https://github.com/emilk/egui). The core crate works with any egui integration. The native gallery uses eframe.

## Run the gallery

Use Rust 1.95 or later.

```sh
cargo run -p spenstar-gallery
```

Open **Agent Workbench** for a three-pane coding workspace with project threads, a conversation stream, a live composer, and a Git and lineage inspector. **Overlays & Feedback** demonstrates dialogs, menus, notifications, and loading states. Use **Ctrl+K** (**⌘K** on macOS) to jump to any gallery page and **Ctrl+B** (**⌘B** on macOS) to cycle gallery navigation.

Capture the 12 Wave 4 scenes. The gallery renders at 2× and exports 1280 × 820 PNGs, then closes. Use `--wave3` for the Wave 3 set or `--legacy` for the earlier scenes and Wave 3 at 2560 × 1640.

```sh
cargo run -p spenstar-gallery -- --capture-dir /tmp/spenstar-shots
```

## Component inventory

| Wave | Components |
| --- | --- |
| 1 · Foundation | Light and dark themes, accent palettes, Inter and JetBrains Mono typography, Lucide and custom SVG icons, buttons, button groups, toggle buttons and segmented controls, close and link buttons, surfaces, cards, responsive grids, split panes, app shells, sidebars, inspectors, headers and search triggers, toolbars, scroll shadows, separators, and empty states |
| 2 · Forms | Fields, input groups, text and search fields, text areas, number fields, checkboxes, radio groups, switches, sliders and range sliders, selects, multi-selects, comboboxes, tags and tag input, color swatches and swatch pickers, calendars, and date, date-range, and time pickers |
| 3 · Data and navigation | Badges and chips, avatars and avatar groups, KPI cards, description lists, code and diff views, data tables, bar, line, area, donut, funnel, and sparkline charts, breadcrumbs, tabs, pagination, disclosure, accordion, and command palette |
| 4 · Overlays and feedback | Modal, alert dialog, drawer and bottom sheet, popover, dropdown and context menus with nested submenus, tooltip, toast queue, alert and banner, progress bar and ring, meter, spinner, and skeleton |

The gallery templates include **Agent Workbench**, **Dashboard**, **Orders**, **CRM Pipeline**, **Portfolio**, **Analytics**, **Tracker**, **Inbox**, **Settings**, and **Dispatch Workspace**. Component pages cover Buttons & Triggers, SVG Icon Browser, Tokens & Typography, Forms & Controls, Pickers & Dates, Data & Charts, and Overlays & Feedback.

## Use the library

Call `spenstar_ui::install` once during initialization. Then render controls in any `egui::Ui`.

```rust
use spenstar_ui::{
    button::Button, egui, icon::Icon,
    theme::{Accent, Mode, Theme},
};

fn initialize(ctx: &egui::Context) {
    Theme::new(Mode::Dark, Accent::Indigo).install(ctx);
}

fn toolbar(ui: &mut egui::Ui) {
    if Button::new("New shipment").leading(Icon::Plus).show(ui).clicked() {
        // Create the draft.
    }
}
```

Use `Theme::apply` to change colors after installation. Use `AppShell` to compose a sidebar, page header, main view, and optional inspector. `ResponsiveGrid` and `SplitPane` work inside any UI.

`SvgIcon::from_bytes` validates self-contained UTF-8 SVG documents up to 64 KiB. `SvgIcon::from_static` accepts embedded assets and parses them on the first render. SVG paint values using `currentColor` take the supplied tint, while explicit colors are preserved. Rasterization uses physical viewport pixels, with a maximum edge of 512 pixels. Each egui context caches up to 192 texture handles and 8 MiB of RGBA texture data. Convert text to vector outlines and use vector paths for embedded imagery.

## Forms and pickers

Use `Field` to pair a control with a label, helper text, and validation. Text, search, and tag editors update caller-owned drafts. `NumberField`, `DateField`, and `TimeField` commit typed values on Enter or focus loss.

```rust
use spenstar_ui::{egui, form::{Field, TextField}};

fn contact(ui: &mut egui::Ui, email: &mut String) {
    let mut field = Field::new("Contact email");
    field.required = true;
    field.error = (!email.is_empty() && !email.contains('@')).then_some("Include an @ sign.");
    field.show_with(ui, |_| {}, |ui, invalid, label| {
        TextField {
            invalid,
            clearable: true,
            ..TextField::new("name@example.com")
        }.show(ui, email).labelled_by(label);
    });
}
```

`Select` and `MultiSelect` store option values, while `ComboBox` edits a display string and reports the chosen value or custom text. Run application keyboard shortcuts through `ShortcutGuard` after rendering the controls so focused editors and open popovers receive the keys first.

`CalendarDate` uses the proleptic Gregorian calendar and Monday-based weekdays. Pickers default to today in UTC. Set `CalendarOptions::today` to the application’s local date when the current day depends on a time zone.

## Data display

`display` includes semantic badges and chips, avatars, KPI cards, description lists, and code and diff views. Native charts live in `charts`. Cartesian series skip non-finite samples, leaving gaps in lines and areas. Donuts use positive contributions and funnels treat invalid counts as zero.

`table::DataTable` renders only the rows in its scroll viewport. Retain a `TableState` per table, supply stable unique row IDs, and increment the `TableRows` revision when rows or filter inputs change. Selection applies across the current filtered result, including pages outside the viewport. The table reports bulk actions for the app to apply to the selected IDs.

`CommandPalette::show` handles the global shortcut through `ShortcutGuard` and returns the activated command ID. Call it after page controls. A button can open it explicitly with `CommandPalette::open`.

## Overlays and feedback

Use stable identities for each overlay. Dialogs, drawers, and popovers accept caller-owned open flags. Dropdown and context menus return the action ID so the caller can update application state. A context menu opens on a secondary click. Render application shortcuts after overlays so `ShortcutGuard` can respect their layers.

```rust
use spenstar_ui::{
    button::{Button, Variant},
    egui,
    feedback::{Toast, ToastQueue, Tone},
    overlay::Modal,
};

fn preferences(ui: &mut egui::Ui, open: &mut bool) {
    *open |= Button::new("Preferences").show(ui).clicked();
    let result = Modal::new("workspace-preferences", "Preferences").show(
        ui.ctx(),
        open,
        |ui| ui.label("Apply preferences to this workspace."),
        |ui| Button::new("Save").variant(Variant::Soft).show(ui).clicked(),
    );
    if result.is_some_and(|output| output.footer) {
        *open = false;
        ToastQueue::notify(ui.ctx(), Toast::new("Preferences saved").tone(Tone::Success));
    }
    ToastQueue::show_context(ui.ctx());
}
```

Call `ToastQueue::show_context` once per frame after the rest of the UI. Use a retained `ToastQueue` when you need independent notification stacks or custom placement.

## License

Code is available under the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE).

- [Inter](https://github.com/rsms/inter) and [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) are bundled under the SIL Open Font License 1.1. [Inter notice](crates/spenstar-ui/assets/fonts/LICENSE-INTER) · [JetBrains Mono notice](crates/spenstar-ui/assets/fonts/LICENSE-JETBRAINS-MONO)
- [Lucide](https://lucide.dev) icons are bundled with their upstream ISC and MIT notices. [Lucide notice](crates/spenstar-ui/assets/icons/LICENSE-LUCIDE)
