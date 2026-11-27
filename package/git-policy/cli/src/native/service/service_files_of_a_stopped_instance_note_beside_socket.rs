//! What:
//!  The note an instance's units leave beside their socket:
//! the file name it has,
//! the directory of notes the service manager reads no unit from,
//! and the two functions that write it.
//! Why:
//!  A unit instance is a hash of its worktree's own Git directory (`service_instance.rs`),
//! so once the worktree is gone the note is the one record of whose units these are:
//! provisioning writes it,
//! and a stop reads it back (`service_files_of_a_stopped_instance.rs`).
//! It lives in the socket's directory under the runtime directory,
//! which is removed with the units at logout or shutdown,
//! so no note outlives the units it names.
//! The real path of the Git directory is written,
//! because the instance is a hash of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await noteWorktree(runtimeDir, instance, gitDir); // writes <notes folder>/<instance>.git-dir
//! ```

use crate::diagnostic_log::warn;
use crate::path_bytes_in_records as path_bytes;
use crate::path_bytes_in_records::GITDIR_HEX_PREFIX;
use crate::private_storage::create_private_directory_all;
use crate::service_provision_host_and_manager::runtime_unit_dir;
use crate::service_provision_unit_file_ownership_and_writes::write_atomically;
use std::path::{
    Path,
    PathBuf,
};

/// Tag used for diagnostics from these files.
/// The removals in `service_files_of_a_stopped_instance.rs` name the same tag,
/// so a line about one instance reads as one line about it.
pub(crate) const TAG: &str = "serviceFilesOfAnInstance";

/// The ending of the note file that names the Git directory of an instance's worktree.
pub const NOTE_ENDING: &str = ".git-dir";

/// The folder of the notes,
/// inside the directory the manager reads the wrapper's unit files from,
/// where provisioning already writes and nothing else of the wrapper does.
/// The manager looks only at unit names in that directory,
/// so a folder of this name is no unit and starts nothing.
pub const NOTES_FOLDER: &str = "monochromatic-git-policy-cli-worktree-notes";

/// The note file of an instance.
pub fn note_path(
    runtime_dir: &Path,
    instance: &str,
) -> PathBuf {
    return runtime_unit_dir(runtime_dir)
        .join(NOTES_FOLDER)
        .join(format!("{instance}{NOTE_ENDING}"));
}

/// What:
/// Writes the note that names the worktree of an instance,
/// unless it already says so.
/// Why:
///   The note is the one record of whose units these are once the worktree is gone;
/// it is read before it is written so that a commit in a worktree whose units are in place writes nothing.
/// It lives in the socket's directory under the runtime directory,
/// which is removed with the units at logout or shutdown,
/// so no note outlives the units it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if ((await readText(notePath)) !== gitDir) await writeAtomically(notePath, gitDir);
/// ```
pub fn record_worktree(
    runtime_dir: &Path,
    instance: &str,
    git_dir: &Path,
) -> std::io::Result<()> {
    // The note holds the bytes of the directory: its text would name another place, or none, for a path that is not text.
    let text: String = format!("{GITDIR_HEX_PREFIX}{}\n", path_bytes::hex_of_path(git_dir));
    let path: PathBuf = note_path(runtime_dir, instance);
    // A note that is a named pipe is not the note this commit would write, and it answers at once rather than waiting for a
    // reader; the write below replaces the name.
    if matches!(crate::plain_file::read_plain_text(&path), Ok(current) if current == text) {
        return Ok(());
    }
    create_private_directory_all(runtime_unit_dir(runtime_dir).join(NOTES_FOLDER).as_path())?;
    return write_atomically(path.as_path(), text.as_str());
}

/// What:
/// Writes the note of an instance for the real Git directory of its worktree,
/// and logs a failure.
/// Why:
///   A note that cannot be written only means the units cannot be told apart later;
/// it never fails a commit.
/// The real path is written because the instance is a hash of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await noteWorktree(runtimeDir, instance, await realpath(gitDir)).catch(log);
/// ```
pub fn note_worktree(
    runtime_dir: &Path,
    instance: &str,
    git_dir: &Path,
) {
    let real: PathBuf = match std::fs::canonicalize(git_dir) {
        Ok(resolved) => resolved,
        Err(_unresolvable) => git_dir.to_path_buf(),
    };
    if let Err(error) = record_worktree(runtime_dir, instance, real.as_path()) {
        warn(TAG, format!("the note of the worktree of {instance} could not be written ({error}); its units cannot be told apart from a removed worktree's later").as_str());
    }
}
