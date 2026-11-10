//! What:
//!  The plan for one `git worktree remove` that the wrapper may serve with a subvolume destroy:
//! the one directory it names,
//!  and whether the person forced it.
//! Why:
//!  Only the plain form (one directory,
//! one optional `-f` or `--force`,
//! a non-empty name) is served;
//! every other option changes what Git checks,
//! and a
//!  name Git resolves by its last path component is not a directory here,
//! so those stay with Git.
//! The force is part of the plan because it decides whether the removal runs Git's
//!  cleanliness proof at all (`remove_worktree` in Git's `builtin/worktree.c` calls
//! `check_clean_worktree` only when
//!  `force` is zero),
//! and a forced removal is the one an agent runs on a worktree it has worked in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parseRemovePlan(['worktree', 'remove', '../topic'], 0, cwd); // undefined: Git removes it
//! ```

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The worktree a `git worktree remove <path>` names,
/// resolved to a directory that exists.
/// Why:
///   The removal needs the worktree's real path;
/// Git also accepts a unique last component,
/// which the wrapper leaves to
///  Git by finding no such directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovePlan = { target: string; operand: string; force: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovePlan {
    /// The real path of the worktree directory.
    pub target: PathBuf,
    /// The operand as the person typed it,
    /// which Git also matches against the last components of every registered path.
    pub operand: OsString,
    /// Whether the person passed `-f` or `--force`,
    /// which is what makes Git remove a worktree that holds changes.
    pub force: bool,
}

/// Whether a token is one of the two spellings of Git's force option for this command.
/// Why:
///   Every other spelling (`-ff`, `--forc`, a cluster)
/// counts the force twice or takes an abbreviation the wrapper does not
///  read,
/// so each of those is left to Git rather than read as one force.
fn is_force(token: &OsString) -> bool {
    return matches!(token.as_encoded_bytes(), b"-f" | b"--force");
}

/// What:
///  Plan one invocation of `git worktree remove` that names exactly one directory and at most one `-f` or `--force`.
/// Why:
///   Every other option changes what Git checks,
/// so those forms stay with Git.
/// The forced form is served because Git's own
///  `--force` removal deletes exactly the worktree the destroy deletes,
/// and because a worktree that holds changes is the
///  one carrying the largest ignored trees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemovePlan(args: string[], subcommandIndex: number, start: string): RemovePlan | undefined;
/// ```
pub fn parse_remove_plan(arguments: &[OsString], subcommand_index: usize, start: &Path) -> Option<RemovePlan> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" || arguments.get(subcommand_index + 1)?.as_encoded_bytes() != b"remove" {
        return None;
    }
    let rest: &[OsString] = &arguments[subcommand_index + 2..];
    let (force, operand): (bool, &OsString) = match rest {
        [only] if !only.as_encoded_bytes().starts_with(b"-") => (false, only),
        [separator, only] if separator.as_encoded_bytes() == b"--" => (false, only),
        [flag, only] if is_force(flag) && !only.as_encoded_bytes().starts_with(b"-") => (true, only),
        [flag, separator, only] if is_force(flag) && separator.as_encoded_bytes() == b"--" => (true, only),
        _ => return None,
    };
    // Joined to the start directory an empty operand would name that directory, while Git answers that '' is not a working tree.
    if operand.is_empty() {
        return None;
    }
    let target: PathBuf = std::fs::canonicalize(start.join(operand)).ok()?;
    return target.is_dir().then_some(RemovePlan { target, operand: operand.clone(), force });
}

/// Removal plan controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_plan_tests.rs"]
mod tests;
