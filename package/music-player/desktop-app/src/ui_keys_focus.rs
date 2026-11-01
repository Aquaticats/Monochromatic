//! Requests for keyboard focus from the Rust side (Alt+1, Alt+2, Escape out of a field).
//!
//! What: `request_focus` names the element that should take focus and bumps the tick the elements watch.
//! Why: the lists and pages sit inside conditional parts of the window, so Rust cannot call `focus()` on them; they watch
//! a target number and a tick instead, and a list created after the request takes focus when it appears.

use crate::AppWindow;

/// What: `FocusTarget` names the element a focus request is for.
/// Why: the window passes the target as a number, and the Rust side keeps the names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FocusTarget = "folders" | "tracks" | "settings-list";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusTarget {
    /// The folder picker list.
    Folders,
    /// The track list.
    Tracks,
    /// The Settings list.
    SettingsList,
}

/// Methods of `FocusTarget`.
impl FocusTarget {
    /// What: `to_int` gives the number the window's `focus-request` property uses.
    /// Why: Slint has no enum binding in this crate.
    pub(crate) fn to_int(self) -> i32 {
        return match self {
            Self::Folders => 1,
            Self::Tracks => 2,
            Self::SettingsList => 3,
        };
    }
}

/// What: `set_target` records which element a coming request is for without asking yet.
/// Why: a list that does not exist yet (the folded layout creates the picker when it opens) reads the target when it is
/// created, so the target must be set before the projection that creates it.
pub(crate) fn set_target(window: &AppWindow, target: FocusTarget) {
    window.set_focus_request(target.to_int());
}

/// What: `request_focus` asks the element to take keyboard focus now.
/// Why: the elements watch the tick, so a repeated request for the same target still acts.
pub(crate) fn request_focus(window: &AppWindow, target: FocusTarget) {
    tracing::debug!(?target, "keyboard focus requested");
    set_target(window, target);
    window.set_focus_tick(window.get_focus_tick() + 1);
}
