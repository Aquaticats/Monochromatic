//! Application stylesheet,
//!  plus an opt-in debug tint for the sticky rails.

/// What:
///  imports the default-display accessor type.
/// Why:
///  the stylesheet is attached to the display so every window inherits it.
use gtk4::gdk::Display;
/// What:
///  imports the CSS provider,
///  the application priority constant,
///  and the display-attach
///       helper.
/// Why:
///  a provider at application priority overrides the theme's background across the app.
use gtk4::{
    CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, style_context_add_provider_for_display,
};

/// What:
///  imports the debug-tint env-var name.
/// Why:
///  the debug stylesheet is loaded only when that variable is set.
use crate::constants::DEBUG_TINT_ENV;

/// What:
///  the app stylesheet:
///  a pure-black background,
///  matching the original.
/// Why:
///  identical base styling keeps screenshots of the two apps comparable.
const APP_CSS: &str =
    "window, scrolledwindow, viewport, listview, .view { background-color: #000000; }";

/// What:
///  the debug stylesheet:
///  green sticky-rail rectangles.
/// Why:
///  the rails are this variant's counterpart of the original's green `Y6L` lane overlays and
///      the Electron prototype's `.debug-tint .rail` outlines,
///  so screenshots align one to one.
const DEBUG_CSS: &str = "
.fm-sticky-rail {
  background-color: rgba(40,255,120,0.08);
  border: 3px solid #28ff78;
  border-radius: 12px;
}
.fm-canvas { background-color: rgba(255,120,0,0.10); }
";

/// What:
///  load the app stylesheet onto the default display,
///  plus the debug tint when its env is
///       set.
/// Why:
///  installed once at startup;
///  a missing display is logged and skipped rather than panicking.
pub(crate) fn install() {
    let Some(display) = Display::default() else {
        tracing::warn!("no default display; skipping stylesheet");
        return;
    };
    let provider = CssProvider::new();
    provider.load_from_string(APP_CSS);
    style_context_add_provider_for_display(&display, &provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
    if std::env::var_os(DEBUG_TINT_ENV).is_some() {
        let debug = CssProvider::new();
        debug.load_from_string(DEBUG_CSS);
        style_context_add_provider_for_display(
            &display,
            &debug,
            STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
        tracing::info!("debug tint enabled");
    }
}
