//! The `windows`,
//!  `focus`,
//!  and `close` answers:
//!  which toplevels the clients have mapped and which one has keyboard focus.
//!  A toplevel is mapped once its client has committed a buffer;
//!  before that it has no extent, no pointer hits it, and the answers do not list it.

/// Standard library items.
use std::sync::PoisonError;

/// Smithay compositor types.
use smithay::{
    desktop::Window,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

/// Sibling modules of this crate.
use crate::{
    protocol::Response,
    state::Compositor,
    wire_text::{encode_text, format_records},
};

/// What the answers say about one window.
///
/// What:
///      `pub struct WindowInfo { pub app_id: String, pub title: String, pub focused: bool }`.
/// Why:
///       Formatting is a pure function of this,
///  so it is tested without a display.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WindowInfo = { appId: string; title: string; focused: boolean };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    /// The app ID the client set,
    ///  empty when none.
    pub app_id: String,
    /// The title the client set,
    ///  empty when none.
    pub title: String,
    /// Whether the window has keyboard focus.
    pub focused: bool,
}

/// Format one window as a record.
///
/// What:
///      `fn format_window(index: usize, info: &WindowInfo) -> String`.
///  `index=N app-id=... title=... focused=true|false`,
///  values percent-encoded like the notification records.
/// Why:
///       A title with a space,
///  `;`,
///  or a line break must not split a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const formatWindow = (index: number, info: WindowInfo) => `index=${index} app-id=${enc(info.appId)} ...`;
/// ```
fn format_window(index: usize, info: &WindowInfo) -> String {
    return format!(
        "index={index} app-id={} title={} focused={}",
        encode_text(&info.app_id),
        encode_text(&info.title),
        info.focused,
    )
}

/// Format the `windows` answer.
///
/// What:
///      `pub fn format_windows(infos: &[WindowInfo]) -> String`.
///  `0` when there are none,
///  else the count and the records separated by `; `.
///  Index 0 is the first window mapped,
///  the one `title` and `app-id` describe.
/// Why:
///       Same list grammar as the notification and job commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const formatWindows = (infos: WindowInfo[]) => formatRecords(infos.map(formatWindow));
/// ```
#[must_use]
pub fn format_windows(infos: &[WindowInfo]) -> String {
    let records: Vec<String> = infos.iter().enumerate().map(|(index, info)| return format_window(index, info)).collect();
    return format_records(&records)
}

/// Format the `focus` answer.
///
/// What:
///      `pub fn format_focus(infos: &[WindowInfo]) -> String`.
///  `none` when no window has focus,
///  else the focused window's record.
/// Why:
///       A test asks one question:
///  who gets the keys now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const formatFocus = (infos: WindowInfo[]) => ...;
/// ```
#[must_use]
pub fn format_focus(infos: &[WindowInfo]) -> String {
    match infos.iter().position(|info| return info.focused) {
        Some(index) => return format_window(index, &infos[index]),
        None => return "none".to_string(),
    }
}

/// Read a window's title and app ID.
///
/// What:
///      `fn read_names(window: &Window) -> (String, String)`.
/// Why:
///       The client sets both on its xdg toplevel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readNames(window: Window): [string, string] { ... }
/// ```
fn read_names(window: &Window) -> (String, String) {
    let Some(toplevel) = window.toplevel() else {
        return (String::new(), String::new())
    };
    return with_states(toplevel.wl_surface(), |states| {
        let Some(data) = states.data_map.get::<XdgToplevelSurfaceData>() else {
            return (String::new(), String::new())
        };
        let attributes = data.lock().unwrap_or_else(PoisonError::into_inner);
        return (attributes.app_id.clone().unwrap_or_default(), attributes.title.clone().unwrap_or_default())
    })
}

/// The surface of the window that has keyboard focus,
///  if any.
///
/// What:
///      `fn focused_surface(state: &Compositor) -> Option<WlSurface>`.
/// Why:
///       Windows are matched to focus by their surface.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const focusedSurface = (state: Compositor) => state.seat.keyboard.focus;
/// ```
fn focused_surface(state: &Compositor) -> Option<WlSurface> {
    return state.seat.get_keyboard().and_then(|keyboard| return keyboard.current_focus())
}

/// Whether a toplevel has content the pointer can hit.
///
/// What:
///      `pub fn is_mapped(window: &Window) -> bool`.
///  True once the client has committed a buffer,
///  which gives the window an extent.
/// Why:
///       A client creates its toplevel,
///  then spends tens to hundreds of milliseconds starting its renderer before the first buffer.
///  In that time the toplevel exists but no point of the screen belongs to it,
///  so `wheel` and `scroll` are refused and `click` and `move` reach nobody.
///  A driver that waits for `windows` to list a window must find input working when it does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isMapped = (window: Window) => window.bbox.width > 0 && window.bbox.height > 0;
/// ```
#[must_use]
pub fn is_mapped(window: &Window) -> bool {
    return !window.bbox().is_empty()
}

/// The mapped toplevels,
///  first mapped first.
///
/// What:
///      `fn mapped_windows(state: &Compositor) -> impl Iterator<Item = &Window>`.
/// Why:
///       `windows`,
///  `focus`,
///  and `close` number the same list,
///  so an index from one names the same window in another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mappedWindows = (state: Compositor) => state.space.elements().filter(isMapped);
/// ```
fn mapped_windows(state: &Compositor) -> impl Iterator<Item = &Window> {
    return state.space.elements().filter(|window| return is_mapped(window))
}

/// Describe every mapped toplevel,
///  first mapped first.
///
/// What:
///      `pub fn describe_windows(state: &Compositor) -> Vec<WindowInfo>`.
/// Why:
///       Source of the `windows` and `focus` answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeWindows(state: Compositor): WindowInfo[] { ... }
/// ```
#[must_use]
pub fn describe_windows(state: &Compositor) -> Vec<WindowInfo> {
    let focused = focused_surface(state);
    return mapped_windows(state)
        .map(|window| {
            let (app_id, title) = read_names(window);
            let is_focused = match (window.toplevel(), focused.as_ref()) {
                (Some(toplevel), Some(surface)) => toplevel.wl_surface() == surface,
                _ => false,
            };
            return WindowInfo { app_id, title, focused: is_focused }
        })
        .collect()
}

/// Answer `windows`.
///
/// What:
///      `pub fn windows_response(state: &Compositor) -> Response`.
/// Why:
///       Lists every window with its focus flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const windowsResponse = (state: Compositor) => ok(formatWindows(describeWindows(state)));
/// ```
#[must_use]
pub fn windows_response(state: &Compositor) -> Response {
    return Response::OkWith(format_windows(&describe_windows(state)))
}

/// Answer `focus`.
///
/// What:
///      `pub fn focus_response(state: &Compositor) -> Response`.
/// Why:
///       Names the window that gets the keys.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const focusResponse = (state: Compositor) => ok(formatFocus(describeWindows(state)));
/// ```
#[must_use]
pub fn focus_response(state: &Compositor) -> Response {
    return Response::OkWith(format_focus(&describe_windows(state)))
}

/// Answer `close [INDEX]` by sending the window `xdg_toplevel.close`.
///
/// What:
///      `pub fn close_response(state: &Compositor, index: Option<usize>) -> Response`.
///  The window with keyboard focus when no index is given.
///  The client decides what to do with the request,
///  as it does when its close button is pressed.
/// Why:
///       Closing a window is a user action a test must be able to take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function closeResponse(state: Compositor, index?: number): Response { ... }
/// ```
#[must_use]
pub fn close_response(state: &Compositor, index: Option<usize>) -> Response {
    let infos = describe_windows(state);
    let target = match index {
        Some(chosen) if chosen < infos.len() => chosen,
        Some(chosen) => return Response::Err(format!("no window with index {chosen}; there are {}", infos.len())),
        None => match infos.iter().position(|info| return info.focused) {
            Some(focused) => focused,
            None => return Response::Err("no window has keyboard focus".to_string()),
        },
    };
    let Some(window) = mapped_windows(state).nth(target) else {
        return Response::Err(format!("window {target} disappeared"))
    };
    let Some(toplevel) = window.toplevel() else {
        return Response::Err(format!("window {target} is not a toplevel"))
    };
    toplevel.send_close();
    return Response::Ok
}

/// What:
///      `#[cfg(test)] #[path = "window_list_tests.rs"] mod tests;`.
/// Why:
///       Keep the formatting tests beside the formatter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./window_list_tests";
/// ```
#[cfg(test)]
#[path = "window_list_tests.rs"]
mod tests;
