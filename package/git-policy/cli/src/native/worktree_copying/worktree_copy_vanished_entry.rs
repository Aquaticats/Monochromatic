//! What:
//!  Whether an entry a walk listed is still there when the walk reaches it,
//! as the answer a walk of a live checkout needs:
//! the entry is gone,
//! so there is nothing to copy,
//! and that is no failure.
//! Why:
//!  A live checkout removes and rewrites its ignored files while a copy runs
//!  (a build deleting a stamp directory,
//! another command removing its stage container),
//! so a listing is a moment and not a promise.
//! The copy contract already says a source entry that is gone when its turn comes has nothing left to copy
//!  and leaves the manifest
//!  (`package/git-policy/cli/SPEC.md`,
//! "Linked-worktree ignored-state synchronization"),
//! so a walk asks this question instead of ending the command with the filesystem's error:
//! a person then holds a worktree Git already made and no ignored state,
//! for a file that was never theirs to keep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const metadata = await lstatOrAbsent(path); // undefined when the source no longer holds the entry
//! const names = await namesOrAbsent(directory); // undefined when the directory went while the walk ran
//! ```

use crate::path_bytes::path_bytes_of;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of};
use std::path::Path;

/// What:
///  Whether this failure says the entry went away.
/// Why:
///   Every walk of a live checkout asks the same question about the same kind of failure,
/// and the answer is that the entry has nothing left to copy;
/// naming it once keeps the walks from ending a command over a file its owner removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wentAway = (error: Error) => error.code === 'ENOENT';
/// ```
pub fn went_away(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound;
}

/// What:
///  The no-follow metadata of one entry,
///  or nothing when the source no longer holds it.
/// Why:
///   A missing entry is an answer for a walk of a live checkout,
/// not a failure;
/// every other kind of error is still the filesystem's to report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function lstatOrAbsent(path: string): Promise<Metadata | undefined>;
/// ```
pub fn lstat_or_absent(path: &Path) -> Result<Option<std::fs::Metadata>, WorktreeCopyError> {
    // `match` separates "no entry" (an answer) from every other failure.
    return match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if went_away(&error) => Ok(None),
        Err(error) => Err(raw_failure(&error)),
    };
}

/// What:
///  The names one directory holds,
/// as the bytes the filesystem listed,
///  or nothing when the source no longer holds the directory.
/// Why:
///   The walk records a directory's listing and then visits each name,
/// so a directory its owner removed in between is an answer and not a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function namesOrAbsent(directory: string): Promise<Uint8Array[] | undefined>;
/// ```
pub fn names_or_absent(directory: &Path) -> Result<Option<Vec<Vec<u8>>>, WorktreeCopyError> {
    let listing: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        Err(error) if went_away(&error) => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<Vec<u8>> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(raw_failure_of)?;
        // The name is the bytes the filesystem listed; a name that is not text is still this worktree's entry.
        names.push(path_bytes_of(Path::new(entry.file_name().as_os_str())).to_vec());
    }
    return Ok(Some(names));
}

/// Vanished-entry controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_vanished_entry_tests.rs"]
mod tests;
