//! What:
//!  The plan for one `git worktree remove` that the wrapper may serve with a subvolume destroy:
//! the one directory it names,
//!  and whether the person forced it;
//!  and the directory one such command names in every other form,
//!  which is what the clone route's refusal reads.
//! Why:
//!  Only the plain form (one directory,
//! one optional `-f` or `--force`,
//! a non-empty name) is served;
//! every other option changes what Git checks,
//! so those stay with Git.
//!  A slot a whole clone is bound to is refused whatever the command's options,
//!  though,
//!  so the refusal reads the operand of every spelling Git accepts
//!  (`parse_removal_target`),
//!  and both readings take their operands from `worktree_operands`.
//! The name is read the way Git reads it:
//!  first as the last components of the registered worktree paths,
//! then as a path.
//! The force is part of the plan because it decides whether the removal runs Git's
//!  cleanliness proof at all (`remove_worktree` in Git's `builtin/worktree.c` calls
//! `check_clean_worktree` only when
//!  `force` is zero),
//! and a forced removal is the one an agent runs on a worktree it has worked in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parseRemovePlan(['worktree', 'remove', '../topic'], 0, cwd, commonDir); // undefined: Git removes it
//! ```

use crate::worktree_command_facts::RemovePlan;
use crate::worktree_operands::operands_after_options;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// Whether a token is one of the two spellings of Git's force option for this command.
/// Why:
///   Every other spelling (`-ff`,
/// `--forc`,
/// a cluster)
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
/// and because a forced removal is the form
///  the wrapper's own reports tell a person to run.
/// `common_dir` is the invoking repository's common Git directory,
/// where Git's list of worktree paths lives;
/// without it only the path form is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemovePlan(args: string[], subcommandIndex: number, start: string, commonDir?: string): RemovePlan | undefined;
/// ```
pub fn parse_remove_plan(
    arguments: &[OsString],
    subcommand_index: usize,
    start: &Path,
    common_dir: Option<&Path>,
) -> Option<RemovePlan> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree"
        || arguments.get(subcommand_index + 1)?.as_encoded_bytes() != b"remove"
    {
        return None;
    }
    let rest: &[OsString] = &arguments[subcommand_index + 2..];
    let (force, operand): (bool, &OsString) = match rest {
        [only] if !only.as_encoded_bytes().starts_with(b"-") => (false, only),
        [separator, only] if separator.as_encoded_bytes() == b"--" => (false, only),
        [flag, only] if is_force(flag) && !only.as_encoded_bytes().starts_with(b"-") => {
            (true, only)
        }
        [flag, separator, only] if is_force(flag) && separator.as_encoded_bytes() == b"--" => {
            (true, only)
        }
        _ => return None,
    };
    // Joined to the start directory an empty operand would name that directory, while Git answers that '' is not a working tree.
    if operand.is_empty() {
        return None;
    }
    // Git matches the typed name against the registered paths first, and a name that matches one of them names that worktree.
    if let Some(common) = common_dir
        && let Some(target) =
            crate::worktree_remove_registry::path_git_picks_for_operand(common, operand.as_os_str())
    {
        return Some(RemovePlan {
            target,
            operand: operand.clone(),
            force,
        });
    }
    let target: PathBuf = std::fs::canonicalize(start.join(operand)).ok()?;
    return target.is_dir().then_some(RemovePlan {
        target,
        operand: operand.clone(),
        force,
    });
}

/// What:
///  The worktree one `git worktree remove` names,
///  whatever options the command carries,
///  resolved to a directory that exists.
/// Why:
///  A slot a whole managed clone is bound to is refused for every spelling of
///  the command,
///  because `SPEC.md`'s sentence exempts none and Git's own hint steers a
///  person to the double force
///  (`SPEC.md`,
///  "Removal of a ZFS worktree clone";
///  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "refuse instead of falling through to Git's recursive removal").
///  The plan above serves only the plain form,
///  so the refusal needs the operand without the force decision,
///  and it reads the same operands the move refusal does.
///  A command naming two operands names no one worktree,
///  and Git answers it with its own usage;
///  the refusal leaves that form to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemovalTarget(args: string[], subcommandIndex: number, start: string, commonDir?: string): string | undefined;
/// ```
pub fn parse_removal_target(
    arguments: &[OsString],
    subcommand_index: usize,
    start: &Path,
    common_dir: Option<&Path>,
) -> Option<PathBuf> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree"
        || arguments.get(subcommand_index + 1)?.as_encoded_bytes() != b"remove"
    {
        return None;
    }
    let operands: Vec<&OsString> = operands_after_options(&arguments[subcommand_index + 2..]);
    // Git's removal takes exactly one worktree, and `remove_worktree` answers every other count with its usage.
    let [typed] = operands.as_slice() else {
        return None;
    };
    let operand: &OsString = typed;
    if operand.is_empty() {
        return None;
    }
    // Git matches the typed name against the registered paths first, and a name that matches one of them names that worktree.
    if let Some(common) = common_dir
        && let Some(target) =
            crate::worktree_remove_registry::path_git_picks_for_operand(common, operand.as_os_str())
    {
        return Some(target);
    }
    let target: PathBuf = std::fs::canonicalize(start.join(operand)).ok()?;
    return target.is_dir().then_some(target);
}

/// Removal plan controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_plan_tests.rs"]
mod tests;
