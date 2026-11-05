//! What:
//!  The checks and writes on the unit files of the commit service: which files carry the wrapper's marker, whether any
//!  directory holds a file of a name, how a unit file is written atomically and how a settled one is removed.
//! Why:
//!  The provisioning (`service_provision.rs`) decides what to write and stop; these steps need no host or manager, so they sit
//!  in a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! isOwnUnitFile(path); carriesOwnMarker(text); anyNamedAnywhere(dirs, names); await writeAtomically(path, text); await removeSettled(path)
//! ```

use crate::service_messages::ClientFailure;
use std::path::{Path, PathBuf};
/// The words that mark a unit file as written by this wrapper: the marker its units carry now, and the descriptions the
///  units of earlier releases carried.
pub const OWN_MARKERS: [&str; 3] =
    ["(cli-git-unit)", "Description=Commit service socket for the git wrapper", "Description=Commit service for the git wrapper"];

/// Writes `text` to `path` through a temporary file and a rename, so a reader sees the old file or the new one whole.
pub(crate) fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    let number: u64 = TEMPORARY_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temporary: PathBuf = path.with_extension(format!("tmp-{}-{number}", std::process::id()));
    std::fs::write(&temporary, text.as_bytes())?;
    let renamed: std::io::Result<()> = std::fs::rename(&temporary, path);
    if renamed.is_err() {
        let _removed = std::fs::remove_file(&temporary);
    }
    return renamed;
}

pub(crate) static TEMPORARY_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Whether the unit file at `path` carries a marker of this wrapper's units.
pub(crate) fn is_own_unit_file(path: &Path) -> bool {
    return match std::fs::read_to_string(path) {
        Ok(text) => carries_own_marker(text.as_str()),
        Err(_unreadable) => false,
    };
}

/// What:
///  Whether the text of a unit file carries one of the markers of this wrapper's units.
/// Why:
///  Only a file with a marker is the wrapper's to stop or remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carriesOwnMarker = (text: string) => OWN_MARKERS.some((marker) => text.includes(marker));
/// ```
pub(crate) fn carries_own_marker(text: &str) -> bool {
    for marker in OWN_MARKERS {
        if text.contains(marker) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a unit file of this name that carries a marker of this wrapper's units sits in the runtime unit directory or the
///  user's own unit directory.
/// Why:
///  `in_place` counts a leftover file of an earlier name only when it is the wrapper's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [runtimeDirectory, userDirectory].some((directory) => directory && isOwnUnitFile(path.join(directory, name)))
/// ```
pub(crate) fn own_unit_file_in_runtime_or_user_directory(runtime_directory: &Path, user_directory: Option<&Path>, name: &str) -> bool {
    if is_own_unit_file(runtime_directory.join(name).as_path()) {
        return true;
    }
    return match user_directory {
        Some(found) => is_own_unit_file(found.join(name).as_path()),
        None => false,
    };
}

/// What:
///  Whether any of the unit names has a file in any of the directories.
/// Why:
///  A socket file with no unit file of either name anywhere is what a release left behind; one that exists is someone else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.some((name) => directories.some((directory) => existsSync(path.join(directory, name))))
/// ```
pub(crate) fn any_named_anywhere(directories: &[PathBuf], names: &[&str]) -> bool {
    for name in names {
        for found in directories {
            if found.join(name).exists() {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  Removes a unit file whose ownership was just checked; a file that is gone by now is a removal someone else finished.
/// Why:
///   Several commits provision at once without any lock, so two of them can read the same owned file and both remove it:
///  the second one's removal finds nothing, and the cleanup it wanted is done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { force: true }) // other failures throw
/// ```
pub(crate) fn remove_settled(path: &Path) -> Result<(), ClientFailure> {
    return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ClientFailure::CouldNotStart(format!("{} cannot be removed: {error}", path.display()))),
    };
}
