//! What:
//!  Whether a directory standing where the merge puts a file holds only files of the branch that the merge deletes,
//!  judged through every directory below it.
//! Why:
//!   The branch's own directory is the merge's to replace,
//!  and a directory holding a person's file cannot be replaced without losing it;
//!  the judgement of a merge's paths and the refusal a directory earns both ask this one question,
//!  so both read one answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const onlyTheBranchs = theDirectoryHoldsOnlyWhatTheMergeDeletes(change, changes, root);
//! ```

/// The paths the merge changes,
/// and the file system path of one of them.
use crate::merge_isolation_changes::{
    Change,
    worktree_path,
};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Whether a directory standing where the merge puts a file holds only files of the branch that the merge deletes,
///  and nothing else,
///  judged through every directory below it.
/// Why:
///   The branch's own directory is the merge's to replace,
/// but an untracked file in it is a person's,
/// and a directory
///  that keeps one cannot be replaced:
/// the merge would be half installed before the failure,
/// so it is refused
///  before anything is written.
/// The tree diff names the files the merge deletes and no entry for a directory of its own,
/// so a directory
///  below this one is judged by everything below it,
/// never by a change of its own name;
/// comparing the
///  immediate children with the diff read a directory of the branch's own nested files as someone else's,
/// and the
///  saved merge could never finish
///  (a cross-vendor review of the ninth round,
/// its eighth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aDirectoriesFilesAreAllDeletedByTheMerge({ change, changes, root }): boolean; // every entry below the directory is a deleted change or a directory of them
/// ```
pub fn the_directory_holds_only_what_the_merge_deletes(
    change: &Change,
    changes: &[Change],
    root: &Path,
) -> bool {
    let location: PathBuf = worktree_path(root, change.path.as_slice());
    return every_entry_below_is_one_the_merge_deletes(
        change.path.as_slice(),
        changes,
        location.as_path(),
    );
}

/// What:
///  Whether everything in one directory of the worktree is a path the merge deletes,
///  looked at in every directory
///  below it.
/// Why:
///   A directory cannot be named as a deleted path by a tree diff,
/// so a child that is a directory is judged by
///  its own entries.
/// An emptied directory below this one is accepted only where the merge deletes a path below it,
/// because
///  only then does the pruning of the merge's emptied directories reach it;
/// an emptied directory the pruning never
///  reaches keeps the checkout from holding the merge's file,
/// so it is refused before anything is written.
///  The directory this function is called on may itself be empty:
/// a recovery killed after finishing the last deleted
///  file and before removing the directory it emptied leaves exactly that,
/// and the merge's own deletions empty it.
///  A symbolic link is never followed:
/// its own path must be one the merge deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function everyEntryBelowIsOneTheMergeDeletes(prefix, changes, directory): boolean; // each child: a deleted change, or a directory whose every entry is
/// ```
fn every_entry_below_is_one_the_merge_deletes(
    prefix: &[u8],
    changes: &[Change],
    directory: &Path,
) -> bool {
    let listing: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(_) => return false,
    };
    for entry in listing.flatten() {
        let mut child: Vec<u8> = prefix.to_vec();
        child.push(b'/');
        child.extend_from_slice(entry.file_name().as_encoded_bytes());
        if one_change_deletes(changes, child.as_slice()) {
            continue;
        }
        let below: PathBuf = directory.join(entry.file_name());
        let is_a_directory: bool = match std::fs::symlink_metadata(below.as_path()) {
            Ok(found) => found.is_dir(),
            Err(_) => false,
        };
        if !is_a_directory {
            return false;
        }
        let mut inner: std::fs::ReadDir = match std::fs::read_dir(below.as_path()) {
            Ok(found) => found,
            Err(_) => return false,
        };
        if inner.next().is_none() {
            if !the_merge_deletes_a_path_below(child.as_slice(), changes) {
                return false;
            }
            continue;
        }
        if !every_entry_below_is_one_the_merge_deletes(child.as_slice(), changes, below.as_path()) {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether one of the changes deletes the path named here,
///  which is a path the merge had before it and does not have after it.
/// Why:
///  A tree diff names each deleted file,
/// so this is the only way a path below a directory can be the merge's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function oneChangeDeletes(changes, path): boolean; // changes.some((other) => other.path === path && other.existed() && !other.existsAfter())
/// ```
fn one_change_deletes(
    changes: &[Change],
    path: &[u8],
) -> bool {
    for other in changes {
        if other.path == path && other.existed() && !other.exists_after() {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether the merge deletes a path below one directory of the branch.
/// Why:
///   The pruning of the merge's emptied directories walks up from each deleted path,
/// so an emptied directory is the
///  branch's only when such a path lies below it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theMergeDeletesAPathBelow(prefix, changes): boolean; // changes.some((other) => other.path.startsWith(`${prefix}/`) && other.existed() && !other.existsAfter())
/// ```
fn the_merge_deletes_a_path_below(
    prefix: &[u8],
    changes: &[Change],
) -> bool {
    let mut slash: Vec<u8> = prefix.to_vec();
    slash.push(b'/');
    for other in changes {
        if other.path.starts_with(slash.as_slice()) && other.existed() && !other.exists_after() {
            return true;
        }
    }
    return false;
}
