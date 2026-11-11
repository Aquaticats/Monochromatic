//! What:
//!  The one record that says the wrapper made a linked worktree,
//!  and the reading of it.
//! Why:
//!  The wrapper removes a worktree by itself only when it can prove it made that worktree.
//!  `git worktree remove` deletes every file Git ignores,
//!  and a person's own worktree can hold ignored work of theirs
//!  (a `.env`,
//!  a database dump,
//!  a folder of notes),
//!  so guessing "the wrapper made it" from a subvolume root or from the path would discard their data.
//!  Nothing else on disk proves it today:
//!  a registration writes its copy owner only when Git refused an existing destination
//!  (`worktree_add_binding.rs`),
//!  and the ignored-state copy removes its journal when the copy finishes.
//!  So the add writes this record,
//!  one small file beside the registration,
//!  and every automatic removal asks for it first.
//!  The registration's directory is deleted with the worktree,
//!  so the record needs no removal of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! recordAddedByTheWrapper(commonDir, worktree);          // after a `git worktree add` that git accepted
//! addedByTheWrapper(admin, worktree)                     // false when no record, or one naming another path
//! ```

use crate::hooks_path_values::{without_the_terminator, without_trailing_newlines};
use crate::private_storage::write_private_file_unsynced;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeAddedByTheWrapper";

/// The name of the record inside a worktree's administrative directory.
/// Why the name carries the program:
///  the directory belongs to Git,
///  and a person reading it should know which program put the file there.
pub const ADDED_RECORD_NAME: &str = "monochromatic-git-policy-cli-worktree-added";

/// What:
///  The administrative directory of the worktree at `worktree`,
///  proved to be one of this repository's registrations.
/// Why:
///   The record must land beside the registration of the worktree at this path,
///  and a `.git` file that names a directory outside `<common>/worktrees`
///  is another repository's worktree or not a worktree at all.
///  Only the trailing line breaks Git's own reader takes are taken,
///  so a directory whose name ends in a space is the registration the pointer names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const admin = registrationAdminOf(commonDir, worktree); // throws the reason
/// ```
fn registration_admin(common_dir: &Path, worktree: &Path) -> Result<PathBuf, String> {
    let pointer_path: PathBuf = worktree.join(".git");
    let pointer: Vec<u8> = match std::fs::read(pointer_path.as_path()) {
        Ok(bytes) => bytes,
        Err(error) => return Err(format!("{} cannot be read as a file ({error}), so it is not a linked worktree", pointer_path.display())),
    };
    let line: &[u8] = without_trailing_newlines(pointer.as_slice());
    let Some(named) = line.strip_prefix(b"gitdir: ") else {
        return Err(format!("{} does not start with `gitdir: `", pointer_path.display()));
    };
    let named_path: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(named));
    let admin: PathBuf = match std::fs::canonicalize(named_path.as_path()) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("{} cannot be resolved ({error})", named_path.display())),
    };
    let worktrees: PathBuf = match std::fs::canonicalize(common_dir.join("worktrees")) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("{} cannot be resolved ({error})", common_dir.join("worktrees").display())),
    };
    if admin.parent() != Some(worktrees.as_path()) {
        return Err(format!("{} is not one of the registrations in {}", admin.display(), worktrees.display()));
    }
    return Ok(admin);
}

/// What:
///  Write the record that this repository's wrapper made the worktree at `worktree`.
/// Why:
///   The add that made the worktree calls this once real Git accepted it;
///  a record of the path is what a later automatic removal compares against,
///  so a registration that a later command replaced at another path is not taken for this one.
///  The write is not flushed:
///  a record a power loss takes back only keeps a worktree that nobody then removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordAddedByTheWrapper(commonDir, worktree); // throws the reason
/// ```
pub fn record(common_dir: &Path, worktree: &Path) -> Result<(), String> {
    let admin: PathBuf = registration_admin(common_dir, worktree)?;
    let record_path: PathBuf = admin.join(ADDED_RECORD_NAME);
    let mut bytes: Vec<u8> = worktree.as_os_str().as_bytes().to_vec();
    bytes.push(b'\n');
    return match write_private_file_unsynced(record_path.as_path(), bytes.as_slice()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("the record {} cannot be written ({error})", record_path.display())),
    };
}

/// What:
///  The same record written for `worktree`,
/// with a failure only logged,
/// for a caller whose command has already succeeded.
/// Why:
///   A worktree that Git accepted is the person's worktree whatever the record says;
///  a record that cannot be written costs the automatic cleanup of that one worktree and nothing else,
///  and must not make a successful `git worktree add` fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordOrLog(commonDir, worktree) // never throws
/// ```
pub fn record_or_log(common_dir: &Path, worktree: &Path) {
    let Err(reason) = record(common_dir, worktree) else {
        return;
    };
    crate::diagnostic_log::warn(TAG, format!("this worktree is not recorded as one the wrapper made, so no later command removes it by itself: {reason}").as_str());
}

/// What:
///  Whether the record beside the registration `admin` says this wrapper made the worktree at `worktree`.
/// Why:
///   The only proof an automatic removal accepts.
///  A record that cannot be read,
///  is empty,
///  or names another path is not this worktree's,
///  and is a "no" like an absent one.
///  The record ends with the one line feed `record` writes, and the path before it is compared as bytes,
///  so a worktree whose path ends in a space is the worktree the record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// addedByTheWrapper(admin, worktree) // boolean
/// ```
pub fn added_by_the_wrapper(admin: &Path, worktree: &Path) -> bool {
    let Ok(bytes) = std::fs::read(admin.join(ADDED_RECORD_NAME)) else {
        return false;
    };
    return without_the_terminator(bytes.as_slice()) == worktree.as_os_str().as_bytes();
}

/// The records of the worktrees a wrapper made stay out of the release executable's tests.
#[cfg(test)]
#[path = "worktree_added_by_the_wrapper_tests.rs"]
mod tests;
