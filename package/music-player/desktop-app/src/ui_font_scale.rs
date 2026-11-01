//! Scales every UI font to 0.9x the OS UI font while still tracking the OS setting.
//!
//! Every font in the window is a rem multiple of the window `default-font-size`:
//!  the
//! hand-drawn Text (`Typography.body-rem * 1rem`) and the native std-widgets
//! (Button / CheckBox / Slider),
//!  whose only font-size lever is that base.
//!  So one
//! knob,
//!  `default-font-size`,
//!  scales all of them at once.
//!  To shrink them 10% while
//! still honouring the OS UI-font preference we set the base to `0.9 * the OS font`.
//!
//! A relative `default-font-size: 0.9rem` in `.slint` is rejected as a binding loop
//! (rem resolves against this very property),
//!  so the 0.9 is applied from here.
//!  Slint
//! reads the OS font from the desktop portal ASYNCHRONOUSLY
//! (i-slint-backend-winit `xdg_desktop_settings.rs`),
//!  so it is the fallback at window
//! creation and only becomes real once the portal answers,
//!  which flips the `.slint`
//! `os-font-size` (= 1rem) and fires the `probe-os-font` callback wired below.
//! See `doc/troubleshooting/slint-rem-binding-loop.md`.

/// What:
///      `use crate::AppWindow;`.
///  The generated Slint window type,
///  which only
///           exists in this binary crate (same as the sibling `ui_page` module).
/// Why:
///       The scale handler reads and writes this window's generated properties
///           and callback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AppWindow } from "./main";
/// ```
use crate::AppWindow;

/// What:
///      `use slint::ComponentHandle;`.
///  The trait that provides `.as_weak()` on a
///           generated component handle.
/// Why:
///       We capture a WEAK handle in the callback so it does not keep the window
///           alive,
///  then `upgrade()` it back on the UI thread when the callback fires.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ComponentHandle } from "slint";
/// ```
use slint::ComponentHandle;

/// What:
///      `pub(crate) fn apply_os_font_scale(app: &AppWindow)`.
///  Registers the
///           `probe-os-font` callback that the `.slint` `changed os-font-size` fires,
///           and on the first real value sets `base-font-size` to `0.9 * os_px`.
///           `Cell::new(false)` is a single-threaded flag (all of this runs on the UI
///           thread,
///  so no `Mutex`);
///  the `move` closure owns it plus a weak handle.
/// Why:
///       Slint delivers the OS font asynchronously (portal),
///  so we cannot read it
///           at window creation;
///  the callback fires once the real value lands.
///  The
///           `applied` guard makes it a one-shot:
///  setting `base-font-size` feeds back
///           into `os-font-size` and would re-fire the callback,
///  so the guard stops a
///           runaway (the same self-reference that makes a `.slint` `0.9rem` a binding
///           loop).
///  `changed` fires on change,
///  not on the initial value,
///  so a portal
///           that already answered before first evaluation (rare,
///  it is async) leaves
///           the OS size unscaled rather than looping.
///  `0.9` is the 10% reduction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyOsFontScale(app: AppWindow): void {
///   let applied = false;
///   app.onProbeOsFont((osPx) => {
///     if (applied || osPx <= 0) return;
///     applied = true;
///     app.baseFontSize = osPx * 0.9;
///   });
/// }
/// ```
pub(crate) fn apply_os_font_scale(app: &AppWindow) {
    let weak_font = app.as_weak();
    let applied = std::cell::Cell::new(false);
    app.on_probe_os_font(move |os_px| {
        if applied.get() || os_px <= 0.0 {
            return;
        }
        applied.set(true);
        // The upgraded window binds as `window` so it never shadows the borrowed
        // `app` parameter this weak handle came from.
        if let Some(window) = weak_font.upgrade() {
            let scaled = os_px * 0.9;
            tracing::info!(os_font_px = os_px, base_font_px = scaled, "scaling every UI font to 0.9x the OS font");
            window.set_base_font_size(scaled);
        }
    });
}
