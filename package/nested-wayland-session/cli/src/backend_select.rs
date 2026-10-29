//! Which render backend a session uses:
//!  the nested winit window (default) or the surfaceless headless backend (opt-in).
//!
//! The choice is a pure function of the `--headless` flag so a test can pin it down without a display.
//! The parent display environment only explains the choice (a log line) and the failure message;
//! it never changes which backend is chosen.
//! The nested window stays the default everywhere,
//!  including scripts and machines with no display,
//!  because a person can restore that window and inspect what the hosted app is doing.

/// Environment values are raw OS strings,
/// not necessarily UTF-8.
use std::ffi::OsStr;

/// Record the selected backend in the log.
use tracing::info;

/// Which backend renders and paces the nested screen.
///
/// What:
///      `pub enum BackendKind { NestedWinit, Headless }`.
///  A plain two-variant tag,
/// `Copy` so it passes by value.
/// Why:
///       `run` matches on it once to build the winit window or the surfaceless renderer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BackendKind = 'nested-winit' | 'headless';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKind {
    /// A winit window inside the parent Wayland session (or X11 session),
    /// minimized on launch.
    NestedWinit,
    /// Surfaceless EGL on the DRM render node;
    /// no window,
    /// no parent display.
    Headless,
}

/// Which parent display variables the process was started with.
///
/// What:
///      `pub struct ParentEnvironment { ... }`.
///  Three booleans,
/// true only for a variable that is set and not empty.
/// Why:
///       Selection and the failure message take it as input,
///  so tests cover "parent present" and "parent absent" without touching the real environment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParentEnvironment {
    /// `WAYLAND_DISPLAY` names a parent Wayland socket.
    pub wayland_display: bool,
    /// `WAYLAND_SOCKET` hands over an already connected parent socket.
    pub wayland_socket: bool,
    /// `DISPLAY` names a parent X11 server,
    /// which winit falls back to when it is compiled in.
    pub x11_display: bool,
}

/// Reading and querying the parent display variables.
impl ParentEnvironment {
    /// Read the three variables from this process's environment.
    ///
    /// What:
    ///      `pub fn from_process() -> Self`.
    ///  `var_os` returns the raw value without requiring UTF-8.
    /// Why:
    ///       The only place that touches the real environment,
    /// so everything else stays testable.
    pub fn from_process() -> Self {
        return Self::from_values(
            std::env::var_os("WAYLAND_DISPLAY").as_deref(),
            std::env::var_os("WAYLAND_SOCKET").as_deref(),
            std::env::var_os("DISPLAY").as_deref(),
        )
    }

    /// Build the record from the three raw values.
    ///
    /// What:
    ///      `pub fn from_values(wayland_display: Option<&OsStr>, ...) -> Self`.
    ///  An empty value counts as absent,
    /// as it does for libwayland and Xlib.
    /// Why:
    ///       Lets tests pass literals instead of mutating the process environment.
    pub fn from_values(
        wayland_display: Option<&OsStr>,
        wayland_socket: Option<&OsStr>,
        x11_display: Option<&OsStr>,
    ) -> Self {
        let present = |value: Option<&OsStr>| return value.is_some_and(|text| return !text.is_empty());
        return Self {
            wayland_display: present(wayland_display),
            wayland_socket: present(wayland_socket),
            x11_display: present(x11_display),
        }
    }

    /// Whether any parent display variable is present.
    ///
    /// What:
    ///      `pub fn any_display(&self) -> bool`.
    /// Why:
    ///       The nested window needs one of them;
    /// the failure message names `--headless` only without any.
    pub fn any_display(&self) -> bool {
        return self.wayland_display || self.wayland_socket || self.x11_display
    }
}

/// The selected backend and whether a present parent display is deliberately left unused.
///
/// What:
///      `pub struct BackendChoice { ... }`.
/// Why:
///       `run` logs `ignored_parent_display` so a person who passed `--headless` inside a desktop session
///  sees that no window will appear there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendChoice {
    /// The backend to build.
    pub kind: BackendKind,
    /// True when `--headless` was given although a parent display variable is set.
    pub ignored_parent_display: bool,
}

/// Choose the backend:
/// the nested window unless `--headless` was given.
///
/// What:
///      `pub fn select_backend(headless: bool, parent: ParentEnvironment) -> BackendChoice`.
///  Only the flag picks the kind.
/// Why:
///       Headless is opt-in.
///  A missing parent display never selects it silently:
///  the nested window then fails with a message that names the flag,
///  because the nested window is what a person restores to inspect the running app.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kind = headless ? 'headless' : 'nested-winit';
/// ```
///
/// @example
/// ```ts
/// selectBackend(false, { waylandDisplay: false }); // => nested-winit, then a clear failure
/// selectBackend(true, { waylandDisplay: true });   // => headless, parent ignored
/// ```
pub fn select_backend(headless: bool, parent: ParentEnvironment) -> BackendChoice {
    let kind = if headless { BackendKind::Headless } else { BackendKind::NestedWinit };
    let choice = BackendChoice {
        kind,
        ignored_parent_display: kind == BackendKind::Headless && parent.any_display(),
    };
    info!(?choice, ?parent, "selected the render backend");
    return choice
}

/// Text for a failed nested window start.
///
/// What:
///      `pub fn nested_failure_message(cause: &str, parent: ParentEnvironment) -> String`.
///  Returns the original cause alone when a parent display variable is present,
///  and the cause plus the `--headless` option when none is.
/// Why:
///       A machine with no display (CI,
/// SSH) otherwise sees only a winit connection error and no way forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return parent.anyDisplay() ? cause : `${cause}; no parent display ... pass --headless`;
/// ```
pub fn nested_failure_message(cause: &str, parent: ParentEnvironment) -> String {
    if parent.any_display() {
        return format!("winit backend init failed: {cause}")
    }
    return format!(
        "winit backend init failed: {cause}. \
         No parent display is set (WAYLAND_DISPLAY, WAYLAND_SOCKET and DISPLAY are all empty or unset), \
         so the nested window has nothing to open on. \
         Either start the session from a desktop or display session, \
         or pass --headless to render without any display, \
         which hosts the app on the GPU through EGL without opening a window \
         (see \"Headless mode\" in the README)"
    )
}

/// Verifies the default choice for every parent display combination and the failure message.
#[cfg(test)]
#[path = "backend_select_tests.rs"]
mod tests;
