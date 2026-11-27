//! What:
//!  The landing's index update without a Git process:
//! read the private index file,
//! reset the commit's paths to the commit
//!  as `index_reset.rs` does,
//! write the file back,
//! and say whether it did.
//! Why:
//!  The follow-up `git reset <commit> --pathspec-from-file=- --pathspec-file-nul` took a minute under the landing's index lock
//!  on a large checkout.
//! Everything it needed is in the repository files the wrapper already reads in process:
//! the index,
//! the
//!  commit's trees and the configuration.
//! Any index form,
//! configuration or hook the port does not reproduce returns `None`
//!  and the caller runs Git's own command (`doc/planning/cli-git-main-commit-within-native-plus-five.md`).
//!
//!  The update is split across siblings because this file is over the line
//!  limit:
//!  this one is the entry a landing calls,
//!  `index_reset_file_names_and_checksums.rs` holds the bounds,
//!  the checksums and the names it derives,
//!  `index_reset_file_surroundings.rs` reads the layout,
//!  the settings and the objects around the index file,
//!  `index_reset_file_reading.rs` reads the index file and the attribute
//!  directories it names,
//!  and `index_reset_file_reset.rs` is the update itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done: boolean = resetIndexFile({ git, repository, commit, paths, indexFile }); // false: run `git reset --no-refresh`
//! ```

/// The update this entry runs,
/// as the guard calls it.
use super::index_reset_file_reset::reset_file_guarded;
/// The line this update leaves for a person reading the wrapper's diagnostics.
use crate::diagnostic_log::debug;
/// The hook the update is guarded by,
/// and the copy and lock it works on.
use crate::repo_read_hook::guarded;
use crate::service_index_lock::IndexLock;
use crate::service_land_follow::{
    IndexPaths,
    PrivateCopy,
};
/// The Git context the guard is read in,
/// and the lock and copy only the landing's holder has.
use crate::transaction_git::GitContext;
/// The repository and the private copy's path.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  Reset the index entries of `paths` in the landing's private copy to the commit `new` without starting Git;
/// the names it reset,
/// or nothing when it declined and Git has to.
/// Why:
///   The private index is the copy the landing made while it holds the live index lock:
/// the function takes a borrow of that
///  lock,
/// so only the holder can call it,
/// and the copy,
/// a file the landing created exclusively,
/// so it works on no other file.
/// A `false` leaves the file as it was (the new file replaces it only by rename
///  after it is complete),
/// so the caller can run `git reset --no-refresh` on the same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resetIndexFile({ git, repository, commit, paths, indexFile }): names | undefined
/// ```
pub fn reset_index_file(
    git: &GitContext,
    repository: &Path,
    new: &str,
    paths: IndexPaths<'_>,
    lock: &IndexLock,
    copy: &PrivateCopy,
) -> Option<Vec<Vec<u8>>> {
    // The copy is installed by a rename onto the live index, so it must be beside it.
    if copy.directory().parent() != lock.index().parent() {
        return None;
    }
    let built: PathBuf = copy.path().to_path_buf();
    let done: Option<Vec<Vec<u8>>> = guarded(
        (
            git,
            repository,
            new,
            paths,
            built.as_path(),
            copy.live_times(),
        ),
        reset_file_guarded,
    );
    // `publish` flushes the new bytes before it renames them onto the copy, and nothing writes to the copy after it returns.
    copy.set_flushed(done.is_some());
    debug(
        "indexResetFile",
        format!(
            "in-process reset of {}: {}",
            built.display(),
            if done.is_some() {
                "done"
            } else {
                "declined, Git answers"
            }
        )
        .as_str(),
    );
    return done;
}

/// Fixtures for the differential controls;
/// never part of the release executable.
#[cfg(test)]
#[path = "index_reset_support.rs"]
pub(crate) mod support;

/// Differential controls against `git reset`;
/// never part of the release executable.
#[cfg(test)]
#[path = "index_reset_file_tests.rs"]
mod tests;

/// Controls for the forms the update leaves to Git;
/// never part of the release executable.
#[cfg(test)]
#[path = "index_reset_fallback_tests.rs"]
mod fallback_tests;

/// Controls for the stat data the update records on the entries it writes;
/// never part of the release executable.
#[cfg(test)]
#[path = "index_reset_recorded_stat_tests.rs"]
mod recorded_stat_tests;
