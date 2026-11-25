# spenstar-ui

Dashboard layouts, controls, typography, and HiDPI SVG icons for [egui 0.37](https://github.com/emilk/egui). The core crate works with any egui integration. The native gallery uses eframe.

## Run the gallery

Use Rust 1.95 or later.

```sh
cargo run -p spenstar-gallery
```

Explore the Dashboard, Orders, Tracker, Inbox, Settings, and Dispatch Workspace templates, or browse buttons, 109 Lucide icons, design tokens, forms, and pickers. The **Forms & Controls** page lets you change the gallery’s accent with a swatch. Open **Appearance** in the page header to try the sidebar styles, Light and Dark modes, and accent presets. Use **Ctrl+B** (**⌘B** on macOS) to cycle through expanded, rail, and hidden navigation.


Use the gallery’s capture mode to write 2× PNGs of the templates, the optional inspector, and the component pages. The gallery closes when the capture is complete.

```sh
cargo run -p spenstar-gallery -- --capture-dir /tmp/spenstar-shots
```

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

## License

Code is available under the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE).

- [Inter](https://github.com/rsms/inter) and [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) are bundled under the SIL Open Font License 1.1. [Inter notice](crates/spenstar-ui/assets/fonts/LICENSE-INTER) · [JetBrains Mono notice](crates/spenstar-ui/assets/fonts/LICENSE-JETBRAINS-MONO)
- [Lucide](https://lucide.dev) icons are bundled with their upstream ISC and MIT notices. [Lucide notice](crates/spenstar-ui/assets/icons/LICENSE-LUCIDE)
