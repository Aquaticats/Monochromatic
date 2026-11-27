//! What:
//!  The order that clears an incomplete registration Git's add left,
//!  and the one write that completes a cut-short managed lock.
//! Why:
//!  Every kill before the lock's removal must leave a locked row the finder recognises,
//!  and every kill after it a bare directory it recognises by name,
//!  so the files go before the lock and the directory goes last
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The recipe that clears rows 1 to 5,
//! 7 and 8").
//!  The slot's pointer goes first and only when its bytes name this registration,
//!  `logs/HEAD` goes before `logs/`,
//!  `ENOENT` at any step means the name is already gone and never a failure,
//!  and each removal runs through the pinned parent descriptor rather than a path.
//!  The lock is completed in place with one truncating write,
//!  because a staging rename would leave the registration unprotected.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const step of REMOVAL_STEPS) await remove(step); // ENOENT is done
//! ```

use super::registration_removals::{
    done_if_missing,
    remove_the_other_entries,
    remove_the_ref_store,
    remove_the_slot_pointer,
};

// What: `pub use` of the names a run works from and of the refusal a stopped run
//       answers, from the file that now holds them.
// Why: every caller names them at this path, and this file's own run needs the
//      same two items, so one line serves both.
//
// In TS you'd write (pseudocode):
// ```ts
// export { RecipeRefusal, ToClear } from './registration_removals.ts';
// ```
pub use super::registration_removals::{
    RecipeRefusal,
    ToClear,
};
use crate::fs_beneath::{
    open_directory_at,
    remove_at,
};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::ffi::OsStr;
use std::io::Write as _;
use std::path::Path;

/// One name group the recipe removes,
/// in its fixed order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemovalStep {
    /// The slot's `.git` pointer,
    /// when its bytes name this registration.
    SlotPointer,
    /// The registration's `gitdir`.
    Gitdir,
    /// Its `commondir`.
    Commondir,
    /// Its `HEAD`.
    Head,
    /// Every file inside `refs/` and `logs/`,
    /// `logs/HEAD` first,
    /// and then those two directories.
    RefsAndLogs,
    /// Any other entry the listing showed.
    OtherEntries,
    /// `locked` last,
    /// so every earlier kill leaves an attributable row.
    Lock,
    /// The registration's own directory,
    /// by `rmdir` from the pinned parent.
    Directory,
}

/// The recipe's order,
/// which is the design's and never the caller's.
pub const REMOVAL_STEPS: &[RemovalStep] = &[
    RemovalStep::SlotPointer,
    RemovalStep::Gitdir,
    RemovalStep::Commondir,
    RemovalStep::Head,
    RemovalStep::RefsAndLogs,
    RemovalStep::OtherEntries,
    RemovalStep::Lock,
    RemovalStep::Directory,
];

/// What:
///  The two moments a control may stand at,
///  which the design's re-reads require.
/// Why:
///  The entry is read once before the first removal and again before the lock's,
///  and a change to the lock under the recipe is a refusal:
///  a control must be able to build that second state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Seams { beforeFirstRemoval(): void; beforeTheLock(): void }
/// ```
pub trait RecipeSeams {
    /// Runs before the first removal.
    fn before_first_removal(&mut self) -> std::io::Result<()>;
    /// Runs immediately before the lock's own removal.
    fn before_the_lock(&mut self) -> std::io::Result<()>;
}

/// The seams of a run with no control standing at either moment.
pub struct NoSeams;

impl RecipeSeams for NoSeams {
    /// Nothing stands before the first removal.
    fn before_first_removal(&mut self) -> std::io::Result<()> {
        return Ok(());
    }

    /// Nothing stands before the lock.
    fn before_the_lock(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  Run the recipe's eight steps for one entry.
/// Why:
///  The rows differ only in which names stand,
///  and `ENOENT` is each step's own success,
///  so one run serves every row the table clears.
///
/// # Errors
/// A refusal naming what stands where the recipe expected nothing,
/// which stops the run and changes nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearRegistration(to, seams);
/// ```
pub fn clear_registration(
    to: &ToClear,
    seams: &mut dyn RecipeSeams,
) -> Result<(), RecipeRefusal> {
    seams.before_first_removal().map_err(RecipeRefusal::Io)?;
    for step in REMOVAL_STEPS {
        match step {
            RemovalStep::SlotPointer => remove_the_slot_pointer(to)?,
            RemovalStep::Gitdir | RemovalStep::Commondir | RemovalStep::Head => {
                let name: &str = match step {
                    RemovalStep::Gitdir => "gitdir",
                    RemovalStep::Commondir => "commondir",
                    _ => "HEAD",
                };
                let registration: std::fs::File =
                    match open_directory_at(to.worktrees_parent, to.registration_name) {
                        Ok(opened) => opened,
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                        Err(error) => return Err(RecipeRefusal::Io(error)),
                    };
                done_if_missing(remove_at(&registration, OsStr::new(name), false))
                    .map_err(RecipeRefusal::Io)?;
            }
            RemovalStep::RefsAndLogs | RemovalStep::OtherEntries | RemovalStep::Lock => {
                let registration: std::fs::File =
                    match open_directory_at(to.worktrees_parent, to.registration_name) {
                        Ok(opened) => opened,
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                        Err(error) => return Err(RecipeRefusal::Io(error)),
                    };
                match step {
                    RemovalStep::RefsAndLogs => {
                        remove_the_ref_store(&registration).map_err(RecipeRefusal::Io)?
                    }
                    RemovalStep::OtherEntries => {
                        remove_the_other_entries(&registration).map_err(RecipeRefusal::Io)?
                    }
                    _ => {
                        seams.before_the_lock().map_err(RecipeRefusal::Io)?;
                        done_if_missing(remove_at(&registration, OsStr::new("locked"), false))
                            .map_err(RecipeRefusal::Io)?;
                    }
                }
            }
            RemovalStep::Directory => {
                done_if_missing(remove_at(to.worktrees_parent, to.registration_name, true))
                    .map_err(RecipeRefusal::Io)?;
            }
        }
    }
    return Ok(());
}

/// What:
///  Complete a cut-short managed lock in place.
/// Why:
///  A staging rename would leave the registration unprotected for a moment,
///  and a lock that is short still protects,
///  so one truncating write of the recomposed bytes is the completion.
///
/// # Errors
/// The lock cannot be opened or written.
pub fn complete_a_cut_short_lock(
    registration: &Path,
    nonce: &RequestNonce,
) -> std::io::Result<()> {
    let path: std::path::PathBuf = registration.join("locked");
    let mut file: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(super::lock_words::lock_file_bytes(nonce).as_slice())?;
    return file.sync_all();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_recipe_tests.rs"]
mod tests;
