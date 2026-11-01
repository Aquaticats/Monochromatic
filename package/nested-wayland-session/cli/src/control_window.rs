//! Answers `title` and `app-id` from the hosted client's toplevel as it stands right now.
//!
//! The values are read on every request rather than cached,
//!  so a title the app changes at runtime shows on the next answer.

/// Imports the lock recovery that keeps a poisoned attribute lock from panicking.
use std::sync::PoisonError;

/// Imports the surface-state accessor and the toplevel types smithay stores per window.
use smithay::{
    desktop::Window,
    wayland::{
        compositor::with_states,
        shell::xdg::{ToplevelSurface, XdgToplevelSurfaceData},
    },
};

/// Imports the compositor state and the wire response.
use crate::{protocol::Response, state::Compositor};

/// Returns the toplevel of the one hosted window, if it has mapped one.
///
/// What:
///      `fn hosted_toplevel(state: &Compositor) -> Option<&ToplevelSurface>`.
///  Takes the first window in the space; the compositor hosts exactly one client.
/// Why:
///       The title and app ID live on the toplevel surface,
///  not on the space element that wraps it.
fn hosted_toplevel(state: &Compositor) -> Option<&ToplevelSurface> {
    let window: &Window = state.space.elements().next()?;
    return window.toplevel()
}

/// Escapes text so it fits on one response line and stays reversible.
///
/// What:
///      `pub fn escape_text(text: &str) -> String`.
///  A backslash becomes `\\`, a line feed `\n`, and a carriage return `\r`.
///  Every other character passes through unchanged.
/// Why:
///       A client may put a line break in its title.
///  An unescaped break would end the response line early and corrupt the next answer,
///  and escaping the backslash keeps the mapping one-to-one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const escapeText = (text: string) => text.replace(/\\/g, "\\\\").replace(/\n/g, "\\n").replace(/\r/g, "\\r");
/// ```
pub fn escape_text(text: &str) -> String {
    // What:     `for character in text.chars() { match character { ... } }`. Walks the
    //           characters once, writing each one's escape or the character itself.
    // Why:      A single pass keeps the escaping linear in the title length.
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            other => escaped.push(other),
        }
    }
    return escaped
}

/// Reads the title the hosted client set, or nothing when it set none.
///
/// What:
///      `pub fn title(state: &Compositor) -> Option<String>`.
///  `None` means no toplevel exists yet or the client never set a title.
/// Why:
///       The attribute is updated the moment the client sends `set_title`,
///  so reading it here always reflects the latest request.
pub fn title(state: &Compositor) -> Option<String> {
    let toplevel = hosted_toplevel(state)?;
    return with_states(toplevel.wl_surface(), |states| {
        let data = states.data_map.get::<XdgToplevelSurfaceData>()?;
        let attributes = data.lock().unwrap_or_else(PoisonError::into_inner);
        return attributes.title.clone()
    })
}

/// Reads the app ID the hosted client set, or nothing when it set none.
///
/// What:
///      `pub fn app_id(state: &Compositor) -> Option<String>`.
/// Why:
///       Same rule as `title`: the attribute reflects the client's latest request.
pub fn app_id(state: &Compositor) -> Option<String> {
    let toplevel = hosted_toplevel(state)?;
    return with_states(toplevel.wl_surface(), |states| {
        let data = states.data_map.get::<XdgToplevelSurfaceData>()?;
        let attributes = data.lock().unwrap_or_else(PoisonError::into_inner);
        return attributes.app_id.clone()
    })
}

/// Builds the `title` answer: the escaped title, or an empty payload when there is none.
///
/// What:
///      `pub fn title_response(state: &Compositor) -> Response`.
/// Why:
///       An empty payload gives the documented `ok ` answer for a client without a title.
pub fn title_response(state: &Compositor) -> Response {
    return Response::OkWith(escape_text(&title(state).unwrap_or_default()))
}

/// Builds the `app-id` answer: the escaped app ID, or an empty payload when there is none.
///
/// What:
///      `pub fn app_id_response(state: &Compositor) -> Response`.
pub fn app_id_response(state: &Compositor) -> Response {
    return Response::OkWith(escape_text(&app_id(state).unwrap_or_default()))
}

/// Unit tests for the escaping rule.
#[cfg(test)]
#[path = "control_window_tests.rs"]
mod tests;
