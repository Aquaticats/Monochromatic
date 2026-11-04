//! What:
//!  Which operation is in progress in a worktree's Git directory,
//! and which paths the index marks as conflicted.
//! Why:
//!  Taking a side of a conflicted file is allowed only while a merge,
//! a rebase,
//! a cherry-pick or a revert waits,
//! and only for
//!  paths with an unmerged index entry.
//! Both are read in process,
//! from the state files and the index file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const operation = operationInProgress(gitDir); // 'merge' | 'rebase' | 'cherry-pick' | 'revert' | undefined
//! const conflicted = new Set(entries.filter((entry) => entry.stage !== 0).map((entry) => entry.path));
//! ```

use crate::repo_read_index::IndexEntry;
use std::collections::BTreeSet;
use std::path::Path;


/// An operation that stops with conflicts and waits for them to be settled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// `git merge`.
    Merge,
    /// `git rebase`,
    /// either backend.
    Rebase,
    /// `git cherry-pick`.
    CherryPick,
    /// `git revert`.
    Revert,
}

/// What:
///  The operation in progress in a worktree's Git directory,
/// if any.
/// Why:
///   Git leaves `MERGE_HEAD`,
/// `CHERRY_PICK_HEAD` or `REVERT_HEAD` while a merge,
/// cherry-pick or revert waits,
/// and a
///  `rebase-merge` directory or a `rebase-apply` directory that holds `rebasing` for a rebase (the same directory with
///  `applying` is `git am`,
/// whose conflicts are not covered).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function operationInProgress(gitDir: string): Operation | undefined;
/// ```
pub fn operation_in_progress(git_dir: &Path) -> Option<Operation> {
    if git_dir.join("MERGE_HEAD").exists() {
        return Some(Operation::Merge);
    }
    if git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply/rebasing").exists() {
        return Some(Operation::Rebase);
    }
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        return Some(Operation::CherryPick);
    }
    if git_dir.join("REVERT_HEAD").exists() {
        return Some(Operation::Revert);
    }
    return None;
}

/// What:
///  The paths with an unmerged entry,
/// once each.
/// Why:
///   A conflicted path has entries at stage 1,
/// 2 or 3 in place of the one at stage 0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const conflicted = new Set(entries.filter((entry) => entry.stage !== 0).map((entry) => entry.path));
/// ```
pub fn conflicted_paths(entries: &[IndexEntry]) -> BTreeSet<Vec<u8>> {
    let mut paths: BTreeSet<Vec<u8>> = BTreeSet::new();
    for entry in entries {
        if entry.stage != 0 {
            paths.insert(entry.path.clone());
        }
    }
    return paths;
}
