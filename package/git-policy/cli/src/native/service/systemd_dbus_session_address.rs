//! What:
//!  Where the session bus of this user listens:
//! the `unix:path=` entry of the address variable,
//! and the `bus` of a runtime directory when the address names no usable path.
//! Why:
//!  The client connects to a socket (`systemd_dbus.rs`),
//! and the wrapper must find it without a program of its own,
//! so one function reads the address the session gives it.
//! A path with escapes in it is not guessed at:
//! the fallback to the runtime directory is the answer the caller can trust.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! sessionBusPath(address, runtimeDir) // '/run/user/1000/bus'
//! ```

use std::path::Path;

/// What:
///  The default session bus socket inside a runtime directory:
/// `bus` there.
/// Why:
///  `session_bus_path` falls back to it when the address variable names no usable path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const busInDirectory = (directory: string) => path.join(directory, 'bus');
/// ```
fn bus_in_directory(directory: &Path) -> std::path::PathBuf {
    return directory.join("bus");
}

/// the `unix:path=` entry of the address variable,
/// else `$XDG_RUNTIME_DIR/bus`.
pub fn session_bus_path(
    address: Option<&str>,
    runtime_dir: Option<&Path>,
) -> Option<std::path::PathBuf> {
    // The first `unix:` entry that names a usable `path=` option wins; an entry that does not is skipped.
    let mut from_address: Option<std::path::PathBuf> = None;
    if let Some(list) = address {
        for entry in list.split(';') {
            let Some(rest) = entry.strip_prefix("unix:") else {
                continue;
            };
            let mut named: Option<&str> = None;
            for option in rest.split(',') {
                if let Some(found) = option.strip_prefix("path=") {
                    named = Some(found);
                    break;
                }
            }
            let Some(path) = named else {
                continue;
            };
            // Escapes (`%xx`) in a path are not read; a path that uses them falls back to the default location.
            if path.contains('%') || path.is_empty() {
                continue;
            }
            from_address = Some(std::path::PathBuf::from(path));
            break;
        }
    }
    if from_address.is_some() {
        return from_address;
    }
    return runtime_dir.map(bus_in_directory);
}
