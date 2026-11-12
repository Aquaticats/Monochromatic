//! What:
//!  The one path the commit service's unit starts its program from,
//! and the link the wrapper points at the executable it is running:
//! a file of the wrapper's own inside the runtime directory,
//! replaced by a rename,
//! so its text never depends on which release,
//! or which build of a release,
//! a command was started from.
//! Why:
//!   The unit's text is the manager's copy of a file,
//! and the manager reads it again only when it is asked to reload every unit it
//!  holds.
//! A template whose `ExecStart` named the executable of the release that wrote it
//! changed with every release, at its full path,
//! and with every build of the wrapper that started from another directory:
//! each such change made the next commit write the template and ask for a reload,
//! which on this host read every unit for 1 to 13 seconds,
//! and reading a slow manager's answer past the exchange's bound
//!  failed the commit whose release had just been installed
//! (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`).
//! Naming a link the wrapper replaces costs one small read and,
//! when the program moved,
//! one rename:
//! no unit text changes,
//! so the manager is told nothing,
//! and the commit pays neither a reload nor a manager round trip for a release change.
//! The link is also the reason a unit cannot name a program an upgrade removed:
//! every provisioning points it at the executable that is running before anything is started,
//! and a start that follows reads what the link holds then.
//! The link sits beside the record of the unit texts the manager was told about,
//! in the folder of the runtime unit directory that holds the wrapper's own files,
//! which the manager never reads as a unit.
//! It is a link rather than a copy:
//! a copy of the executable would have to be written for every build that provisions,
//! and the release an upgrade removes is still held by the process that runs it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await pointAt(runtimeDir, ownExecutable); const current = pointsAt(runtimeDir, ownExecutable); // link in the notes folder
//! ```

use crate::service_provision_host_and_manager::runtime_unit_dir;
use crate::service_units_of_removed_worktrees::NOTES_FOLDER;
use std::path::{Path, PathBuf};

/// The name of the link inside the notes folder,
/// which the sweep of removed worktrees ignores because it does not end like a note.
pub const PROGRAM_POINTER: &str = "service-program";

/// What:
///  The path the commit service's unit names as its program.
/// Why:
///   It is a file the wrapper owns under the runtime directory,
/// so two releases,
/// and two builds of one release,
/// write the very same unit text,
/// and a unit file that is written once stays the one the manager holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// programPointerPath('/run/user/1000') // '/run/user/1000/systemd/user/<notes>/service-program'
/// ```
pub fn program_pointer_path(runtime_dir: &Path) -> PathBuf {
    return runtime_unit_dir(runtime_dir).join(NOTES_FOLDER).join(PROGRAM_POINTER);
}

/// What:
///  The path a unit of this host names as the service's program:
/// the link on Unix,
/// where the manager starts a service by a path,
/// and the executable itself where there is no such link.
/// Why:
///   A platform without a link of this shape has no unit to write either,
/// and naming the executable keeps the text of a unit that is never written truthful there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// programOf('/run/user/1000', '/opt/wrapper/git') // the link on Unix, the executable elsewhere
/// ```
pub fn program_of(runtime_dir: &Path, program: &Path) -> PathBuf {
    if cfg!(unix) {
        return program_pointer_path(runtime_dir);
    }
    return program.to_path_buf();
}

/// What:
///  Whether the link names `program`.
/// Why:
///   Provisioning asks this before it reads anything else:
/// a link that names the executable this command runs,
/// and units the manager was told about,
/// mean a commit needs neither the files nor the manager.
/// An absent link is a link that names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointsAt('/run/user/1000', '/opt/wrapper/git')
/// ```
pub fn points_at(runtime_dir: &Path, program: &Path) -> bool {
    #[cfg(unix)]
    {
        return matches!(std::fs::read_link(program_pointer_path(runtime_dir)), Ok(named) if named == program);
    }
    #[cfg(not(unix))]
    {
        let _unused = (runtime_dir, program);
        return true;
    }
}

/// What:
///  Points the link the unit names at `program`,
/// replacing whatever it held,
/// so a reader sees the old program or the new one and never a half-written link.
/// Why:
///   `symlink` followed by `rename` replaces the link in one step,
/// and a command killed between them leaves the old link,
/// which still names a program that exists:
/// the next provisioning points it again.
/// The rename is what the manager's next start follows,
/// so it is made before any unit is started or any text is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // symlink to a temporary name in the same folder, then rename over the link
/// await pointAt('/run/user/1000', '/opt/wrapper/git')
/// ```
pub fn point_at(runtime_dir: &Path, program: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let pointer: PathBuf = program_pointer_path(runtime_dir);
        let Some(folder) = pointer.parent() else {
            return Err(std::io::Error::other("the program link has no folder"));
        };
        crate::private_storage::create_private_directory_all(folder)?;
        let number: u64 = crate::service_provision_unit_file_ownership_and_writes::TEMPORARY_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temporary: PathBuf = folder.join(format!("{PROGRAM_POINTER}.tmp-{}-{number}", std::process::id()));
        // A temporary name another command left behind names nothing a unit reads, so replacing it costs nothing.
        let _replaced = std::fs::remove_file(&temporary);
        symlink(program, &temporary)?;
        let renamed: std::io::Result<()> = std::fs::rename(&temporary, &pointer);
        if renamed.is_err() {
            let _removed = std::fs::remove_file(&temporary);
        }
        return renamed;
    }
    #[cfg(not(unix))]
    {
        let _unused = (runtime_dir, program);
        return Ok(());
    }
}

/// What a command writes into the link,
/// and what it reads back from it,
/// stays out of the release executable.
#[cfg(test)]
#[path = "service_provision_program_pointer_tests.rs"]
mod tests;
