//! What:
//!  Which branches a `git branch` region deletes.
//! Why:
//!  Deleting a branch takes its commits out of every reference's reach,
//!  and a forced deletion (`-D`) does
//!       that to a branch nothing else contains.
//!  The owner chose to let Git run as it always did and record what the deletion cost,
//!  so this file
//!       names the branches Git 2.56.0 would delete and no more.
//!  A form that creates or reads writes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // branchDeletions(['-D', 'gone']) === ['gone']
//! ```

use crate::command_branch_table::{BRANCH_TABLE, DELETE, DELETE_FORCE, REMOTES};
use crate::command_options::{DEFAULT_MODE, ParsedOptions, parse_options};
use crate::command_options_query::{is_enabled, is_stated, positional_tokens};
use std::ffi::OsString;

/// What:
///  The branches a `git branch` region deletes,
///  and which reference namespace they are in.
///  A `struct` is a record with named fields.
/// Why:
///   `--remotes` (or `-r`) makes the same spellings delete a remote-tracking reference,
///  and a record that named the local branch of the same name would carry the wrong commit to restore.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BranchDeletion = { names: string[]; remotes: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchDeletion {
    /// The names as written,
    ///  without the `refs/heads/` or `refs/remotes/` prefix.
    pub names: Vec<OsString>,
    /// Whether the deletion names remote-tracking references (`-r` or `--remotes`).
    pub remotes: bool,
}

/// What:
///  Which branches a `git branch` region deletes and from which namespace,
///  or nothing when it deletes none.
///  `Option<BranchDeletion>` is "the deletion or nothing".
/// Why:
///   The region is read with the same option table and tokenizer every other `git branch` fact uses,
///  so every spelling Git accepts is read and no list of spellings can fall behind it:
///  `--remote` is
///  the abbreviation Git accepts for `--remotes`,
///  `-rD` is one cluster,
///  and the value of `-u` is never
///  a name.
///  `--delete-merged` deletes only branches a merged branch already contains,
///  so it is not a
///  deletion this reader records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function branchDeletion(region: string[]): { names: string[]; remotes: boolean } | undefined;
/// ```
pub fn branch_deletion(region: &[OsString]) -> Option<BranchDeletion> {
    // A region Git refuses is one Git runs nothing for; the `?` is that refusal.
    let parsed: ParsedOptions = parse_options(region, BRANCH_TABLE, DEFAULT_MODE, &[]).ok()?;
    // `-d`/`--delete` is one bit that `--no-delete` clears; `-D` is another bit.
    if !is_enabled(&parsed, DELETE) && !is_stated(&parsed, DELETE_FORCE) {
        return None;
    }
    let mut names: Vec<OsString> = Vec::<OsString>::new();
    for index in positional_tokens(&parsed, region.len()) {
        names.push(region[index].clone());
    }
    return Some(BranchDeletion {
        names,
        remotes: is_stated(&parsed, REMOTES),
    });
}

/// What:
///  The names a `git branch` region deletes,
///  or nothing when the region deletes no branch.
///  `Option<Vec<OsString>>` is "the names,
/// or nothing".
/// Why:
///   The linked-worktree policy asks only whether the region deletes at all,
/// so it keeps the shape it was
///  written against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function branchDeletions(region: string[]): string[] | undefined;
/// ```
pub fn branch_deletions(region: &[OsString]) -> Option<Vec<OsString>> {
    let deletion: BranchDeletion = branch_deletion(region)?;
    return Some(deletion.names);
}

/// The spellings,
///  the clusters and the forms that delete nothing stay out of the release executable.
#[cfg(test)]
#[path = "command_branch_delete_tests.rs"]
mod tests;
