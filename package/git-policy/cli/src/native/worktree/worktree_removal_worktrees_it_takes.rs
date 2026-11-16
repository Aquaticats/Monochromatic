//! What:
//!  The worktrees one removal takes:
//!  the one it names,
//!  and every worktree this repository registers inside it,
//!  each with its own Git directory.
//! Why:
//!  A removal deletes the files below the directory it names,
//!  and that directory holds the checkout and the own Git directory of every worktree registered inside it,
//!  so the removal takes those too.
//!  Every question a guard asks before a removal is a question about what that removal takes,
//!  and a question asked of the outer directory alone misses the Git work of every worktree inside it:
//!  a hidden edit in such a worktree went with the outer one while the same edit in the outer directory was refused
//!  (a review of `8858f843ee`, finding 2).
//!  So the list is made once, here, and every question of every route goes over it
//!  (`git_work_a_removal_would_strand`, `hidden_edits_beneath`).
//!  The registrations are read strictly: one `gitdir` that cannot be read may name a worktree inside the one being
//!  removed, so it is the reason nothing is removed rather than an entry quietly passed over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = theWorktreesItTakes({ commonDir, target, admin }); // [{ root, admin }], or throws the reason
//! ```

use crate::worktree_landed_registrations::registrations_a_removal_takes;
use std::path::{Path, PathBuf};

/// One worktree a removal takes,
/// with the Git directory that holds its own state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Taken = { root: string; admin: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Taken {
    /// The worktree's own directory,
    /// which the removal deletes.
    pub(crate) root: PathBuf,
    /// Its own Git directory,
    /// below the common directory,
    /// which the removal deletes with it.
    pub(crate) admin: PathBuf,
}

/// What:
///  The worktrees a removal of `target` takes,
///  `target` first,
///  or the reason one of them cannot be read.
/// Why:
///   One list,
///  made once,
///  is what keeps every question of a removal about the same set:
///  a question that walked its own list would keep or miss a worktree the others do not know about.
///  The outer worktree comes first, so a report about it reads first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theWorktreesItTakes({ commonDir, target, admin }): Taken[]; // throws the reason
/// ```
///
/// # Errors
///
/// The words for a registration that cannot be listed or read,
/// which may name a worktree inside `target`.
pub(crate) fn the_worktrees_it_takes(common_dir: &Path, target: &Path, admin: &Path) -> Result<Vec<Taken>, String> {
    let mut taken: Vec<Taken> = vec![Taken { root: target.to_path_buf(), admin: admin.to_path_buf() }];
    for one in registrations_a_removal_takes(common_dir, target)? {
        taken.push(Taken { root: one.worktree, admin: one.admin });
    }
    return Ok(taken);
}

/// The list of what a removal takes stays out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_worktrees_it_takes_tests.rs"]
mod tests;
