//! Whether a folder chooser can open at all,
//! and what to tell the user when none can.
//!
//! What:
//! `no_chooser_can_open` is true on Linux when neither the desktop portal nor `zenity` is available,
//! the two
//! ways the file dialog library opens a chooser there;
//! `NO_DIALOG_MESSAGE` says how to proceed,
//! and `resolve_on_path` finds the program to run.
//! Why:
//! with neither,
//! `Open folder` did nothing and said nothing (the polish review finding PB5),
//! because the library
//! returns the same empty answer for "cancelled" and "no chooser exists".
//! `src/folder_chooser.rs` runs the chooser with these facts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const missing = !(portalAvailable() || executableOnPath("zenity", process.env.PATH));
//! ```

/// Imports the operating-system string type that the program name search compares.
use std::ffi::OsStr;
/// Imports the path type that joins a program name onto each search directory.
use std::path::PathBuf;

/// What:
///      `pub const ZENITY: &str`.
///  The folder chooser program on Linux.
/// Why:
///       The program search,
/// the message and the chooser runner all name the same program,
/// so they cannot drift apart.
pub const ZENITY: &str = "zenity";

/// What:
///      `pub const NO_DIALOG_REMEDY: &str`.
///  The closing sentence of every message about a chooser that will not open.
/// Why:
///       Both the missing-chooser message and the failed-chooser message end with the two ways forward,
/// and a test holds
///       the longer message's ending to this text.
pub const NO_DIALOG_REMEDY: &str = "Install zenity, or start the player with a folder: music-player /path/to/music";

/// What:
///      `pub const NO_DIALOG_MESSAGE: &str`.
///  The message shown when no folder chooser is available.
/// Why:
///       Plain words,
/// the cause,
/// and both ways forward:
/// install a chooser,
/// or start the player with the folder (the
///  command line accepts a folder,
/// D95's launch path).
pub const NO_DIALOG_MESSAGE: &str = "This computer has no folder chooser for the player to open. Install zenity, or start the player with a folder: music-player /path/to/music";

/// What:
///      `pub fn executable_on_path(program: &str, path_variable: &OsStr) -> bool`.
///  Whether a file called `program` exists in any folder listed in `path_variable`.
/// Why:
///       The same search the shell does,
/// written out so it can be tested with a disposable folder list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const executableOnPath = (program: string, pathVariable: string) => pathVariable.split(delimiter).some((dir) => isFile(join(dir, program)));
/// ```
pub fn executable_on_path(program: &str, path_variable: &OsStr) -> bool {
    return resolve_on_path(program, path_variable).is_some();
}

/// What:
///      `pub fn resolve_on_path(program: &str, path_variable: &OsStr) -> Option<PathBuf>`.
///  The first file called `program` in the folders listed in `path_variable`,
///  as an absolute path;
///  nothing when no folder holds one.
/// Why:
///       Running the found path instead of the bare name starts exactly the program the search found,
///  even when the
///       process's own `PATH` names another one;
///  a test can then give a disposable folder list and be sure
///       only its stub runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolveOnPath = (program: string, pathVariable: string) => pathVariable.split(delimiter).map((dir) => join(dir, program)).find((file) => isFile(file));
/// ```
pub fn resolve_on_path(program: &str, path_variable: &OsStr) -> Option<PathBuf> {
    for directory in std::env::split_paths(path_variable) {
        let candidate: PathBuf = directory.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
///      `fn portal_is_available() -> bool` (Linux).
///  Whether the desktop portal service is running or can be started by the session bus.
/// Why:
///       The portal is the first way the dialog library tries.
/// The bus is only read;
/// no portal call is made.
#[cfg(target_os = "linux")]
pub(crate) fn portal_is_available() -> bool {
    /// Well-known bus name of the desktop portal,
    /// which a folder chooser needs on Linux.
    const PORTAL: &str = "org.freedesktop.portal.Desktop";
    let connection = match crate::dbus_step_bound::connect(
        zbus::connection::Builder::session(),
        crate::dbus_step_bound::DBUS_STEP_BOUND,
    ) {
        Ok(connection) => connection,
        Err(error) => {
            tracing::debug!(%error, "no session bus, so no desktop portal");
            return false;
        }
    };
    if crate::analysis_notification_wire::has_owner(&connection, PORTAL).unwrap_or(false) {
        return true;
    }
    let Ok(bus) = crate::dbus_step_bound::step(zbus::fdo::DBusProxy::new(&connection)) else {
        return false;
    };
    return match crate::dbus_step_bound::step(async { return Ok(bus.list_activatable_names().await?) }) {
        Ok(names) => {
            let mut found = false;
            for name in &names {
                if name.as_str() == PORTAL {
                    found = true;
                    break;
                }
            }
            found
        }
        Err(_error) => false,
    };
}

/// What:
///      `pub fn no_chooser_can_open(zenity: bool, portal: bool) -> bool`.
///  True when neither `zenity` nor the desktop portal can open a folder chooser.
/// Why:
///       The decision is separate from the probing of the computer so a test can give it every combination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noChooserCanOpen = (zenity: boolean, portal: boolean) => !zenity && !portal;
/// ```
pub fn no_chooser_can_open(zenity: bool, portal: bool) -> bool {
    return !zenity && !portal;
}

/// Unit tests of `dialog_backend`,
/// in the sidecar file `dialog_backend_tests.rs`.
/// Role of the module:
/// whether a folder chooser can open at all,
/// and what to tell the user when none can.
#[cfg(test)]
#[path = "dialog_backend_tests.rs"]
mod tests;
