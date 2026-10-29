//! Application stylesheet,
//!  plus an opt-in debug tint.

/// What:
///  imports the default-display accessor type.
/// Why:
///  the stylesheet is attached to the display so every window inherits it.
use gtk4::gdk::Display;
/// What:
///  imports the CSS provider,
///  the application priority constant,
///  and the display-attach helper.
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
///  a pure-black background across the window and its views.
/// Why:
///  black is easier on the eyes than the theme's dark grey;
///  row selection and text keep the
///      theme's colors so highlighting and readability are unaffected.
const APP_CSS: &str =
    "window, scrolledwindow, viewport, listview, .view { background-color: #000000; }";

/// What:
///  the debug stylesheet:
///  distinct hues,
///  outlines,
///  and readable labels per structural layer.
/// Why:
///  debug tint should name every region in screenshots without implying that blank pane bodies
///      are scroll-travel areas.
const DEBUG_CSS: &str = "
.fm-column-root:nth-child(4n+1) { background-color: rgba(255,64,64,0.12); }
.fm-column-root:nth-child(4n+2) { background-color: rgba(64,200,64,0.12); }
.fm-column-root:nth-child(4n+3) { background-color: rgba(64,140,255,0.12); }
.fm-column-root:nth-child(4n+4) { background-color: rgba(230,190,64,0.12); }
.fm-canvas { background-color: rgba(255,120,0,0.10); }
.fm-debug-overlay { border: 1px solid rgba(255,255,255,0.20); }
.fm-debug-lane {
  background-color: rgba(40,255,120,0.08);
  border: 3px solid #28ff78;
  border-radius: 12px;
}
.fm-debug-badge {
  background-color: rgba(0,0,0,0.82);
  color: #ffffff;
  font-family: monospace;
  font-size: 11px;
  padding: 2px 4px;
}
";

/// What:
///  load the app stylesheet onto the default display,
///  plus the debug tint when its env is set.
/// Why:
///  installed once at startup (the display exists only after GTK init);
///  a missing display is
///      logged and skipped rather than panicking.
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
