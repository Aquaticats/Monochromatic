//! Whether a folder chooser can open at all, and what to tell the user when none can.
//!
//! What: `dialog_backend_missing` is true on Linux when neither the desktop portal nor `zenity` is available, the two
//! ways the file dialog library opens a chooser there; `NO_DIALOG_MESSAGE` says how to proceed.
//! Why: with neither, `Open folder` did nothing and said nothing (the polish review finding PB5), because the library
//! returns the same empty answer for "cancelled" and "no chooser exists".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const missing = !(portalAvailable() || executableOnPath("zenity", process.env.PATH));
//! ```

use std::ffi::OsStr;
use std::path::Path;

/// What:
///      `pub const NO_DIALOG_MESSAGE: &str`.
///  The message shown when no folder chooser is available.
/// Why:
///       Plain words, the cause, and both ways forward: install a chooser, or start the player with the folder (the
///  command line accepts a folder, D95's launch path).
pub const NO_DIALOG_MESSAGE: &str = "This computer has no folder chooser for the player to open. Install zenity, or start the player with a folder: music-player /path/to/music";

/// What:
///      `pub fn executable_on_path(program: &str, path_variable: &OsStr) -> bool`.
///  Whether a file called `program` exists in any folder listed in `path_variable`.
/// Why:
///       The same search the shell does, written out so it can be tested with a disposable folder list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const executableOnPath = (program: string, pathVariable: string) => pathVariable.split(delimiter).some((dir) => isFile(join(dir, program)));
/// ```
pub fn executable_on_path(program: &str, path_variable: &OsStr) -> bool {
    return std::env::split_paths(path_variable).any(|directory| return Path::new(&directory).join(program).is_file());
}

/// What:
///      `fn portal_is_available() -> bool` (Linux).
///  Whether the desktop portal service is running or can be started by the session bus.
/// Why:
///       The portal is the first way the dialog library tries. The bus is only read; no portal call is made.
#[cfg(target_os = "linux")]
fn portal_is_available() -> bool {
    const PORTAL: &str = "org.freedesktop.portal.Desktop";
    let connection = match zbus::blocking::Connection::session() {
        Ok(connection) => connection,
        Err(error) => {
            tracing::debug!(%error, "no session bus, so no desktop portal");
            return false;
        }
    };
    if crate::analysis_notification_wire::has_owner(&connection, PORTAL).unwrap_or(false) {
        return true;
    }
    let Ok(bus) = zbus::blocking::fdo::DBusProxy::new(&connection) else {
        return false;
    };
    return bus
        .list_activatable_names()
        .map(|names| return names.iter().any(|name| return name.as_str() == PORTAL))
        .unwrap_or(false);
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

/// What:
///      `pub fn dialog_backend_missing() -> bool`.
///  True when a folder chooser cannot open on this computer.
/// Why:
///       Only Linux can lack one: macOS and Windows have native choosers. On Linux the chooser needs the portal or `zenity`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dialogBackendMissing = () => process.platform === "linux" && !portalAvailable() && !executableOnPath("zenity", process.env.PATH ?? "");
/// ```
pub fn dialog_backend_missing() -> bool {
    #[cfg(target_os = "linux")]
    {
        let path_variable = std::env::var_os("PATH").unwrap_or_default();
        let zenity: bool = executable_on_path("zenity", &path_variable);
        let portal: bool = portal_is_available();
        tracing::info!(zenity, portal, "folder chooser backends");
        return no_chooser_can_open(zenity, portal);
    }
    #[cfg(not(target_os = "linux"))]
    {
        return false;
    }
}

#[cfg(test)]
#[path = "dialog_backend_tests.rs"]
mod tests;
