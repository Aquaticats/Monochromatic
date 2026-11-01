// What:     Headless tests for `ui_theme.rs`, pulled in by `#[cfg(test)] #[path = "ui_theme_tests.rs"] mod tests;`.
// Why:      The window must show the roles Rust derives, follow an accent change while running, and keep the baseline
//           literals in `ui/theme.slint` equal to the derivation, so a viewer without Rust still renders the baseline.

use music_player::oklch::Srgb8;
use music_player::theme_roles::{self, ThemePalette};
use slint::{Color, ComponentHandle};

use super::*;

/// What: `init_platform` installs the testing backend once per test thread.
/// Why:  `init_no_event_loop` installs a thread-local platform, and the test harness runs each test on its own
///       thread, so every thread installs its own once.
fn init_platform() {
    thread_local! {
        static INSTALLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    if !INSTALLED.get() {
        i_slint_backend_testing::init_no_event_loop();
        INSTALLED.set(true);
    }
}

/// What: `window_with_scheme(scheme)` builds the window forced to a scheme (1 light, 2 dark).
fn window_with_scheme(scheme: i32) -> AppWindow {
    init_platform();
    let app = AppWindow::new().expect("the window should build on the testing backend");
    app.global::<Theme>().set_scheme_override(scheme);
    return app;
}

/// What: `color_of(hex)` builds an opaque Slint color from `0xRRGGBB`.
fn color_of(hex: u32) -> Color {
    return Color::from_rgb_u8(((hex >> 16) & 0xFF) as u8, ((hex >> 8) & 0xFF) as u8, (hex & 0xFF) as u8);
}

/// What: `settle()` lets Slint run its pending `changed` handlers, as the event loop does before each frame.
fn settle() {
    slint::platform::update_timers_and_animations();
}

#[test]
fn the_literals_in_theme_slint_equal_the_baseline_derivation() {
    // Before any Rust writes, the Theme global holds the literals of `ui/theme.slint`.
    init_platform();
    let app = AppWindow::new().expect("the window should build on the testing backend");
    let palette: ThemePalette = theme_roles::derive_roles(None);
    let theme = app.global::<Theme>();
    assert_eq!(
        theme.get_dark_roles(),
        slint_roles(&palette.dark),
        "dark literals in ui/theme.slint differ from `derive_roles(None)`: copy the derived values into dark-roles"
    );
    assert_eq!(
        theme.get_light_roles(),
        slint_roles(&palette.light),
        "light literals in ui/theme.slint differ from `derive_roles(None)`: copy the derived values into light-roles"
    );
}

#[test]
fn a_transparent_accent_is_no_root_and_an_opaque_one_is_the_root() {
    assert_eq!(root_of(Color::from_argb_u8(0, 10, 20, 30)), None);
    assert_eq!(root_of(color_of(0x6060C0)), Some(Srgb8 { red: 0x60, green: 0x60, blue: 0xC0 }));
}

#[test]
fn without_an_override_and_without_a_system_accent_the_window_shows_the_baseline() {
    let app = window_with_scheme(2);
    install_accent_tracking(&app);
    let palette: ThemePalette = theme_roles::derive_roles(None);
    let theme = app.global::<Theme>();
    assert_eq!(theme.get_accent_source().alpha(), 0, "the testing backend reports no system accent");
    assert_eq!(theme.get_primary(), slint_color(palette.dark.primary));
}

#[test]
fn the_override_drives_the_roles_in_both_schemes() {
    let accent: Color = color_of(0x6060C0);
    let palette: ThemePalette = theme_roles::derive_roles(root_of(accent));
    let baseline: ThemePalette = theme_roles::derive_roles(None);
    assert_ne!(palette.dark.primary, baseline.dark.primary, "positive control: the accent moves the primary");

    let dark_app = window_with_scheme(2);
    dark_app.global::<Theme>().set_accent_override(accent);
    install_accent_tracking(&dark_app);
    assert_eq!(dark_app.global::<Theme>().get_primary(), slint_color(palette.dark.primary));
    assert_eq!(dark_app.global::<Theme>().get_canvas(), slint_color(palette.dark.canvas));
    assert_eq!(dark_app.global::<Theme>().get_match_fill(), slint_color(palette.dark.match_fill));

    let light_app = window_with_scheme(1);
    light_app.global::<Theme>().set_accent_override(accent);
    install_accent_tracking(&light_app);
    assert_eq!(light_app.global::<Theme>().get_primary(), slint_color(palette.light.primary));
    assert_eq!(
        light_app.global::<Theme>().get_secondary_container_hovered(),
        slint_color(palette.light.secondary_container_hovered)
    );
}

#[test]
fn a_running_window_follows_an_accent_change_and_a_return_to_the_system() {
    let app = window_with_scheme(2);
    install_accent_tracking(&app);
    let theme = app.global::<Theme>();
    let baseline_primary: Color = theme.get_primary();

    let red: Color = color_of(0xFF0000);
    theme.set_accent_override(red);
    settle();
    let red_primary: Color = theme.get_primary();
    assert_ne!(red_primary, baseline_primary, "the accent change did not reach the roles");
    assert_eq!(red_primary, slint_color(theme_roles::derive_roles(root_of(red)).dark.primary));

    let yellow: Color = color_of(0xFFD000);
    theme.set_accent_override(yellow);
    settle();
    assert_eq!(theme.get_primary(), slint_color(theme_roles::derive_roles(root_of(yellow)).dark.primary));
    assert_ne!(theme.get_primary(), red_primary);

    // Transparent follows the system again, which reports no accent here, so the baseline returns.
    theme.set_accent_override(Color::from_argb_u8(0, 0, 0, 0));
    settle();
    assert_eq!(theme.get_primary(), baseline_primary);
}

#[test]
fn the_scheme_choice_stays_in_slint_and_needs_no_rust_round_trip() {
    let app = window_with_scheme(2);
    app.global::<Theme>().set_accent_override(color_of(0x6060C0));
    install_accent_tracking(&app);
    let theme = app.global::<Theme>();
    let dark_primary: Color = theme.get_primary();
    theme.set_scheme_override(1);
    assert_ne!(theme.get_primary(), dark_primary);
    assert_eq!(theme.get_canvas(), color_of(0xFFFFFF));
    theme.set_scheme_override(2);
    assert_eq!(theme.get_primary(), dark_primary);
    assert_eq!(theme.get_canvas(), color_of(0x000000));
}

/// What: `forbidden_color_calls(source)` lists the sRGB color operations D108 bans in a `.slint` source.
/// Why:  `mix`, `brighter`, `darker` and `transparentize` change lightness, chroma or hue in sRGB; `with-alpha` is
///       alpha-only layering, which D108 allows.
fn forbidden_color_calls(source: &str) -> Vec<&'static str> {
    let mut found: Vec<&'static str> = Vec::new();
    for call in [".mix(", ".brighter(", ".darker(", ".transparentize("] {
        if source.contains(call) {
            found.push(call);
        }
    }
    return found;
}

#[test]
fn the_ui_sources_hold_no_srgb_color_manipulation() {
    // Positive control: the scanner does find each banned call and ignores the allowed one.
    assert_eq!(forbidden_color_calls("a.mix(b, 0.5) c.brighter(1) d.darker(1) e.transparentize(0.5)").len(), 4);
    assert!(forbidden_color_calls("tint.with-alpha(0.08)").is_empty());

    let ui_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let mut scanned: usize = 0;
    for directory in [ui_dir.clone(), ui_dir.join("preview")] {
        let entries = std::fs::read_dir(&directory).expect("the ui directory should be readable");
        for entry in entries {
            let path = entry.expect("a directory entry should be readable").path();
            if path.extension() != Some(std::ffi::OsStr::new("slint")) {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("a .slint file should be readable");
            assert!(forbidden_color_calls(&source).is_empty(), "{} manipulates colors in sRGB", path.display());
            scanned += 1;
        }
    }
    assert!(scanned > 15, "the scan reached only {scanned} files");
}
