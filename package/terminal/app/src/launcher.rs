//! Desktop-shell integration for the terminal prototype.

/// What:
///      `use i_slint_backend_winit::winit::platform::wayland::WindowAttributesExtWayland;`
///           imports the trait that adds `with_name` to winit window attributes.
/// Why:
///       Slint's default backend selector has no hook for Wayland app id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { withWaylandName } from "winit-wayland";
/// ```
use i_slint_backend_winit::winit::platform::wayland::WindowAttributesExtWayland;

/// What:
///      `use i_slint_backend_winit::winit::window::WindowAttributes;` imports
///           the creation-time window settings record.
/// Why:
///       The app-id hook receives and returns this exact type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { WindowAttributes } from "winit";
/// ```
use i_slint_backend_winit::winit::window::WindowAttributes;

/// What:
///      `pub const APP_ID: &str = "monochromatic.terminal";` declares a public
///           borrowed string constant.
///  Sibling `String` would allocate at runtime.
/// Why:
///       Wayland app id and `.desktop` StartupWMClass must stay identical.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const APP_ID = "monochromatic.terminal";
/// ```
pub const APP_ID: &str = "monochromatic.terminal";

/// What:
///      `pub fn set_window_app_id(...) -> WindowAttributes` declares a public
///           hook function used by Slint's winit backend builder.
/// Why:
///       KDE and other shells associate windows with desktop files by app id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setWindowAppId(attributes) {
///   return attributes.withName(APP_ID, APP_ID);
/// }
/// ```
pub fn set_window_app_id(attributes: WindowAttributes) -> WindowAttributes {
    // What:     `attributes.with_name(APP_ID, APP_ID)` sets Wayland app id and X11
    //           WM_CLASS name pair, then returns the modified builder value.
    // Why:      The id must be present before the native window is created.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return attributes.withName(APP_ID, APP_ID);
    // ```
    return attributes.with_name(APP_ID, APP_ID)
}
