//! What:
//!  The walks of a worktree's tracked files against the index's recorded stat data,
//! by metadata alone or by content for the
//!  files whose metadata cannot vouch for them.
//! Why:
//!  A file's name,
//! size and times are cheap to compare;
//! only a file with a racy or stale time is read,
//! and only then is
//!  its content judged against the committed bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reason = treeDiffersFromIndex(root, entries, indexSeconds); // undefined when every file agrees
//! ```

//! This file is the module's index:
//!  the walks that judge a tracked file by its metadata alone,
//!  by a path and below a held directory.
//!  The walks that judge by content are in `worktree_clean_tree_walk_by_content.rs`,
//!  the directory chain and the stat read are in `worktree_clean_tree_walk_held_paths.rs`,
//!  and both were split out because this module was over the line limit,
//!  with every name they defined re-exported here so no caller changes.

use crate::fs_beneath::{
    entry_metadata_at,
    open_root,
    plain_components,
};
use crate::fs_beneath_directory_chain::DirectoryChain;
use crate::worktree_clean_stat_verdicts::{
    CleanRefusal,
    StatEntry,
    changed_file,
    stat_finding,
};
use crate::worktree_clean_tree_walk_held_paths::directory_of;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// The walks that judge a tracked file by its content,
/// and what the content decision needs besides the entries.
pub use crate::worktree_clean_tree_walk_by_content::{
    ContentTarget,
    paths_whose_content_differs,
    tree_differs_from_index_by_content,
    tree_fields_by_content,
};
/// The directory chain a walk keeps,
/// the cursor that reads one file's stat fields,
/// and the single read of one path's fields.
pub use crate::worktree_clean_tree_walk_held_paths::{
    StatCursor,
    stat_fields_beneath,
};

/// What:
///  The first reason a tracked file of `root` differs from its index entry,
/// or nothing when every file agrees.
/// Why:
///   After a snapshot the same comparison runs against the snapshot's own files,
/// so a source that changed while the
///  snapshot was taken is caught on the copy.
/// The files are reached below a held root with no link followed,
/// so a directory
///  replaced by a link is not read through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndex(root: string, entries: StatEntry[], indexSeconds: number): string | undefined;
/// ```
pub fn tree_differs_from_index(
    root: &Path,
    entries: &[StatEntry],
    index_seconds: u32,
) -> Option<String> {
    return match open_root(root) {
        Ok(held) => tree_differs_from_index_beneath(&held, entries, index_seconds),
        Err(error) => Some(format!(
            "{} cannot be opened without following a link ({error})",
            root.display()
        )),
    };
}

/// The first refusal of a tracked file of `root` against its index entry,
/// with its kind;
/// the files are reached below a held
///  root with no link followed.
pub fn tree_refusal(
    root: &Path,
    entries: &[StatEntry],
    index_seconds: u32,
) -> Option<CleanRefusal> {
    return match open_root(root) {
        Ok(held) => tree_refusal_beneath(&held, entries, index_seconds),
        Err(error) => Some(CleanRefusal::unreadable(format!(
            "{} cannot be opened without following a link ({error})",
            root.display()
        ))),
    };
}

/// What:
///  The first reason a tracked file below the held directory `root` differs from its index entry,
/// or nothing when every
///  file agrees;
/// each file is looked at below handles opened without following a link.
/// Why:
///   A snapshot is checked against the index it will carry;
/// a path walk through a link inside the snapshot would compare
///  a file outside it,
/// so every directory on the way must be a real one on the snapshot's device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndexBeneath(root: Dir, entries: StatEntry[], indexSeconds: number): string | undefined;
/// ```
pub fn tree_differs_from_index_beneath(
    root: &std::fs::File,
    entries: &[StatEntry],
    index_seconds: u32,
) -> Option<String> {
    return match tree_refusal_beneath(root, entries, index_seconds) {
        Some(refusal) => Some(refusal.reason),
        None => None,
    };
}

/// The first refusal of a tracked file below the held directory `root` against its index entry,
/// with its kind.
fn tree_refusal_beneath(
    root: &std::fs::File,
    entries: &[StatEntry],
    index_seconds: u32,
) -> Option<CleanRefusal> {
    let Ok(mut chain) = DirectoryChain::new(root) else {
        return Some(CleanRefusal::unreadable(String::from(
            "the worktree root cannot be examined",
        )));
    };
    for entry in entries {
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice())))
        else {
            return Some(CleanRefusal::unreadable(format!(
                "{shown} is not a plain path"
            )));
        };
        let Some((name, parents)) = names.split_last() else {
            return Some(CleanRefusal::unreadable(format!(
                "{shown} is not a plain path"
            )));
        };
        let Some(directory) = directory_of(&mut chain, parents) else {
            return changed_file(format!(
                "{shown} is missing from the worktree or reached through a link"
            ));
        };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else {
            return changed_file(format!("{shown} is missing from the worktree"));
        };
        if let Some(refusal) = stat_finding(entry, &metadata, index_seconds) {
            return Some(refusal);
        }
    }
    return None;
}
