//! What:
//!  The checks and writes on the unit files of the commit service:
//! which files carry the wrapper's marker,
//! whether any
//!  directory holds a file of a name,
//! how a unit file is written atomically and how a settled one is removed.
//! Why:
//!  The provisioning (`service_provision.rs`) decides what to write and stop;
//! these steps need no host or manager,
//! so they sit
//!  in a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! isOwnUnitFile(path); carriesOwnMarker(text); anyNamedAnywhere(dirs, names); await writeAtomically(path, text); await removeSettled(path)
//! ```

use crate::service_messages::ClientFailure;
use std::path::{Path, PathBuf};
/// The first bytes of every name this package gives a unit, the link beside them and the notes of a worktree.
/// Why:
///   The manager's own directory holds another program's files too,
///  so a sweep of the temporaries there needs a name it can call its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NAME_PREFIX = 'monochromatic-git-policy-cli'
/// ```
const NAME_PREFIX: &str = "monochromatic-git-policy-cli";

/// The words that mark a unit file as written by this wrapper:
/// the marker its units carry now,
/// and the descriptions the
///  units of earlier releases carried.
pub const OWN_MARKERS: [&str; 4] =
    ["(monochromatic-git-policy-cli-unit)", "(cli-git-unit)", "Description=Commit service socket for the git wrapper", "Description=Commit service for the git wrapper"];

/// Writes `text` to `path` through a temporary file and a rename,
/// so a reader sees the old file or the new one whole.
pub(crate) fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    sweep_beside(path);
    let temporary: PathBuf = crate::temporary_file_of_a_writer::name_of(path);
    std::fs::write(&temporary, text.as_bytes())?;
    crate::service_fault::reach(crate::service_fault::BEFORE_A_UNIT_RENAME);
    let renamed: std::io::Result<()> = std::fs::rename(&temporary, path);
    if renamed.is_err() {
        let _removed = std::fs::remove_file(&temporary);
    }
    return renamed;
}

/// What:
///  Removes the temporaries a killed write left in the folder of `path`,
///  before the write that follows puts its own there.
/// Why:
///   A unit file, the link beside it and the note that names them are written through a temporary and a rename,
///  so a process killed between the two leaves a file in the manager's own directory that nothing else names;
///  the folder is the manager's, so only a temporary whose writer the kernel proves gone is removed,
///  and a live writer's own file is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepBeside(path) // the folder of path loses what a dead writer left
/// ```
fn sweep_beside(path: &Path) {
    let Some(folder) = path.parent() else {
        return;
    };
    // The manager's own directory holds another program's files too, so only a temporary whose name begins with the one this
    //  package gives its units, its link and its notes, or that stands in a folder of this package, is removed.
    let swept: crate::temporary_file_of_a_writer::Swept = crate::temporary_file_of_a_writer::sweep_temporaries(folder, crate::service_journal::TEMPORARY_GRACE, &|name| {
        return name.starts_with(NAME_PREFIX) || folder.file_name().is_some_and(|part| { return part.to_string_lossy().starts_with(NAME_PREFIX); });
    });
    if swept.removed > 0 {
        crate::service_log!("removed {} temporary file(s) a killed write of a unit left in {}", swept.removed, folder.display());
    }
}

/// What:
///  Makes `path` a link to `target` through a temporary link and a rename,
/// so a reader sees the old link or the new one whole.
/// Why:
///  The service unit runs whatever this link names at the moment the manager starts it,
/// so a half-written link
///  would make a start run no program at all.
/// A rename replaces the entry atomically,
/// which a link to a link would not
///  (`symlink` refuses a name that exists).
pub(crate) fn link_atomically(path: &Path, target: &Path) -> std::io::Result<()> {
    sweep_beside(path);
    let temporary: PathBuf = crate::temporary_file_of_a_writer::name_of(path);
    let _stale = std::fs::remove_file(&temporary);
    std::os::unix::fs::symlink(target, &temporary)?;
    crate::service_fault::reach(crate::service_fault::BEFORE_A_UNIT_RENAME);
    let renamed: std::io::Result<()> = std::fs::rename(&temporary, path);
    if renamed.is_err() {
        let _removed = std::fs::remove_file(&temporary);
    }
    return renamed;
}

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
///  A socket file with no unit file of either name anywhere is what a release left behind;
/// one that exists is someone else's.
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
///  Removes a unit file whose ownership was just checked;
/// a file that is gone by now is a removal someone else finished.
/// Why:
///   Several commits provision at once without any lock,
/// so two of them can read the same owned file and both remove it:
///  the second one's removal finds nothing,
/// and the cleanup it wanted is done.
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
