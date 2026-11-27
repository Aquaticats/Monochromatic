//! What:
//!  What the word-level second pass watches:
//!  the paths an operation had already left unmerged when it started,
//!  and the conflicts the index now holds for the paths that operation created.
//! Why:
//!  Only a conflict its own command created is the pass's to resolve,
//!  so the pass reads the index before the command runs and again after it,
//!  and an index the readers cannot take in process leaves the pass off for that run:
//!  it never guesses.
//!  The index is a plain file read to a bound, never a path checked and then opened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = watchBefore(forward); // { unmergedBefore: Set<string> } | undefined
//! const created = conflictedPaths(entriesAfter).filter((path) => !watch.unmergedBefore.has(path));
//! const conflicts = conflictsOfIndex(entriesAfter, created); // [{ path, base, ours, theirs }]
//! ```

use crate::child_environment::environment_value;
use crate::conflict_side_state::conflicted_paths;
use crate::index_writer_forward::CoordinatedForward;
use crate::merge_second_pass::{
    Conflict,
    Stage,
};
use crate::private_storage::read_regular_file;
use crate::repo_read_index::{
    IndexEntry,
    parse_index,
};
use crate::repo_read_layout::{
    RepoLayout,
    read_layout_after_success_for,
};
use crate::repo_read_object::to_hex;
use std::collections::BTreeSet;

/// The largest index file this reads.
const INDEX_LIMIT_BYTES: u64 = 1 << 30;

/// The commands whose conflicts get the pass;
/// a pull's own merge or rebase stops in the same state the command would.
const OPERATIONS: [&[u8]; 5] = [b"merge", b"rebase", b"cherry-pick", b"revert", b"pull"];

/// What an operation had left unmerged before it ran.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Watch {
    /// Every path that had conflict stages in the index.
    pub(super) unmerged_before: BTreeSet<Vec<u8>>,
}

/// The layout of the repository the command runs in,
/// when the in-process readers can describe it.
pub(super) fn layout_of(forward: &CoordinatedForward) -> Option<RepoLayout> {
    if environment_value(forward.environment.as_slice(), "GIT_INDEX_FILE").is_some() {
        return None;
    }
    return read_layout_after_success_for(
        &forward.command.arguments[..forward.command.subcommand_index],
        forward.overlay.as_slice(),
    );
}

/// The index entries of the layout's worktree,
/// or nothing when the in-process reader declines the file.
pub(super) fn index_entries(layout: &RepoLayout) -> Option<Vec<IndexEntry>> {
    let bytes: Vec<u8> =
        read_regular_file(layout.git_dir.join("index").as_path(), INDEX_LIMIT_BYTES).ok()?;
    return parse_index(bytes.as_slice(), layout.object_format);
}

/// What:
///  The conflicts of `entries` for the paths in `only`,
/// with their stages as object names.
/// Why:
///   The index lists the three stages of a path one after the other;
/// a missing stage stays `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conflictsOfIndex(entries: IndexEntry[], only: Set<string>): Conflict[];
/// ```
pub fn conflicts_of_index(
    entries: &[IndexEntry],
    only: &BTreeSet<Vec<u8>>,
) -> Vec<Conflict> {
    let mut conflicts: Vec<Conflict> = Vec::new();
    for entry in entries {
        if !(entry.stage != 0 && only.contains(&entry.path)) {
            continue;
        }
        let starts_new: bool = match conflicts.last() {
            Some(last) => last.path != entry.path,
            None => true,
        };
        if starts_new {
            conflicts.push(Conflict {
                path: entry.path.clone(),
                base: None,
                ours: None,
                theirs: None,
            });
        }
        let Some(conflict) = conflicts.last_mut() else {
            continue;
        };
        let side: Option<Stage> = Some(Stage {
            mode: entry.mode,
            oid: to_hex(entry.id.as_slice()),
        });
        match entry.stage {
            1 => conflict.base = side,
            2 => conflict.ours = side,
            3 => conflict.theirs = side,
            _ => {}
        }
    }
    return conflicts;
}

/// What:
///  Note which paths are unmerged before the command runs,
/// when the command is one the pass serves.
/// Why:
///   Only a conflict this command creates is the pass's to resolve.
/// When the index cannot be read in process the pass is off
///  for this run:
/// it never guesses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchBefore(forward: CoordinatedForward): Watch | undefined;
/// ```
pub fn watch_before(forward: &CoordinatedForward) -> Option<Watch> {
    if !forward.command.is_builtin() || !OPERATIONS.contains(&forward.command.word()) {
        return None;
    }
    let layout: RepoLayout = layout_of(forward)?;
    let entries: Vec<IndexEntry> = index_entries(&layout)?;
    return Some(Watch {
        unmerged_before: conflicted_paths(entries.as_slice()),
    });
}
