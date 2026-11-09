//! What:
//!  What the word-level second pass will not replace:
//! the index stages and the worktree file of every merged path are read again just before it installs,
//! and a path another command moved is left exactly as it is now.
//! Why:
//!  The pass computes a merge from the stages Git left and then stages a resolved entry and writes the worktree file.
//! Another command can resolve the same conflict and stage it in that window,
//! and a person can edit the file,
//! so the merged bytes can be computed from something that no longer exists.
//! The owner's rule is that user data is never discarded,
//! so the pass replaces a path only while both places still hold what the merge read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = stillTheSame({ worktreeRoot, entries, held }, merged, oids); // { files, oids, moved }
//! ```

use super::merge_second_pass_worktree::conflicts_of_index;
use crate::merge_second_pass::{Conflict, Merged};
use crate::path_bytes::os_from_path_bytes;
use crate::repo_read_index::IndexEntry;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};

/// The largest worktree file the pass records:
/// a conflicted file holds both sides' bytes and Git's markers,
/// and the pass merges no stage larger than `LARGEST_STAGE_BYTES`,
/// so three of them bound it,
/// with room for the markers.
pub(super) const LARGEST_RECORDED_BYTES: u64 = 3 * crate::merge_second_pass::LARGEST_STAGE_BYTES + 64 * 1024;

/// What a worktree path held when the conflicts were read:
/// its bytes,
/// or nothing when they cannot be compared (the file cannot be read,
/// or it is larger than a conflicted file can be).
/// Why:
///   The pass computes a merge from the index stages and then replaces the worktree file.
/// A file that moved in between holds bytes the merge was never computed from,
/// and writing over it discards them.
/// The bytes are what it compares,
/// not their length and time:
/// a writer can write different bytes of the same length and put the saved modification time back,
/// and `SPEC.md` records that such a file is one the pass has already read.
pub(super) type Held = Option<Vec<u8>>;

/// What the worktree holds at `path` now,
/// or nothing when it cannot be compared.
pub(super) fn worktree_state(path: &Path) -> Held {
    let file: std::fs::File = std::fs::File::open(path).ok()?;
    let mut bytes: Vec<u8> = Vec::new();
    let kept: std::io::Result<usize> = file.take(LARGEST_RECORDED_BYTES + 1).read_to_end(&mut bytes);
    let Ok(read) = kept else { return None };
    return if read as u64 > LARGEST_RECORDED_BYTES { None } else { Some(bytes) };
}

/// The path a stage names,
/// below the top of the worktree.
fn path_under(root: &Path, path: &[u8]) -> PathBuf {
    return root.join(os_from_path_bytes(path));
}

/// What the worktree holds at each of `paths`.
pub(super) fn held_paths(root: &Path, paths: &BTreeSet<Vec<u8>>) -> BTreeMap<Vec<u8>, Held> {
    let mut held: BTreeMap<Vec<u8>, Held> = BTreeMap::new();
    for path in paths {
        held.insert(path.clone(), worktree_state(path_under(root, path).as_path()));
    }
    return held;
}

/// What the install must find before it replaces anything.
pub(super) struct Before<'a> {
    /// The top of the worktree,
    /// whose paths the stages name.
    pub(super) worktree_root: &'a Path,
    /// The index as it is now,
    /// read again after the merge was computed.
    pub(super) entries: &'a [IndexEntry],
    /// What the worktree held at each path when the conflicts were read.
    pub(super) held: &'a BTreeMap<Vec<u8>, Held>,
}

/// What the install may replace,
/// and the paths it may not.
pub(super) struct Kept {
    /// The merged files whose index stages and worktree file are still what the merge was computed from.
    pub(super) files: Vec<Merged>,
    /// Their blobs,
    /// in the same order.
    pub(super) oids: Vec<String>,
    /// The paths that moved,
    /// which the install leaves as it finds them.
    pub(super) moved: Vec<Vec<u8>>,
}

/// What:
///  Whether the index stages and the worktree file of `file` are still what the merge was computed from.
/// Why:
///   Another command that resolved the conflict and staged it leaves a stage 0 entry where the merge read three stages,
/// and a file a person
///  edited is not the file Git left;
/// either way the merged bytes were computed from something else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = (file) => stagesOf(index, file.path) === file.stages && stat(file.path) === recorded;
/// ```
fn still_holds(before: &Before<'_>, file: &Merged) -> bool {
    let only: BTreeSet<Vec<u8>> = [file.path.clone()].into_iter().collect();
    let now: Vec<Conflict> = conflicts_of_index(before.entries, &only);
    if now.as_slice() != [file.stages.clone()] {
        return false;
    }
    let Some(recorded) = before.held.get(&file.path) else { return false };
    let current: Held = worktree_state(path_under(before.worktree_root, file.path.as_slice()).as_path());
    return *recorded == current;
}

/// What:
///  Keep only the merged files whose path still holds what the merge was computed from,
/// and name the paths that moved.
/// Why:
///   The person's own resolution,
/// a commit another session made,
/// or any other command may touch the conflict between the read and the
///  install;
/// the pass then leaves that path exactly as it is and says which one it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stillTheSame(before, merged, oids) { ... } // { files, oids, moved }
/// ```
pub(super) fn still_the_same(before: &Before<'_>, merged: Vec<Merged>, oids: Vec<String>) -> Kept {
    let mut kept: Kept = Kept { files: Vec::new(), oids: Vec::new(), moved: Vec::new() };
    for (file, oid) in merged.into_iter().zip(oids) {
        if still_holds(before, &file) {
            kept.files.push(file);
            kept.oids.push(oid);
        } else {
            kept.moved.push(file.path.clone());
        }
    }
    return kept;
}
