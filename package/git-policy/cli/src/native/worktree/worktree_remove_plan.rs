//! What:
//!  The plan for one `git worktree remove` that the wrapper may serve with a subvolume destroy:
//! the one directory it names.
//! Why:
//!  Only the plain form (one directory,
//! no option,
//! a non-empty name) is served;
//! `--force` and every other option change what Git checks,
//! so those stay with Git.
//! The name is read the way Git reads it:
//!  first as the last components of the registered worktree paths,
//! then as a path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parseRemovePlan(['worktree', 'remove', '../topic'], 0, cwd, commonDir); // undefined: Git removes it
//! ```

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The worktree a `git worktree remove <path>` names,
/// resolved to a directory that exists.
/// Why:
///   The removal needs the worktree's real path;
/// Git also accepts a unique last component,
/// which is
///  found among the registered paths before the name is read as a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovePlan = { target: string; operand: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovePlan {
    /// The real path of the worktree directory.
    pub target: PathBuf,
    /// The operand as the person typed it,
    /// which Git also matches against the last components of every registered path.
    pub operand: OsString,
}

/// What:
///  Plan one invocation of `git worktree remove` that names exactly one directory and no option.
/// Why:
///   `--force` and every other option change what Git checks,
/// so those forms stay with Git.
/// `common_dir` is the invoking repository's common Git directory,
/// where Git's list of worktree paths lives;
/// without it only the path form is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRemovePlan(args: string[], subcommandIndex: number, start: string, commonDir?: string): RemovePlan | undefined;
/// ```
pub fn parse_remove_plan(arguments: &[OsString], subcommand_index: usize, start: &Path, common_dir: Option<&Path>) -> Option<RemovePlan> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" || arguments.get(subcommand_index + 1)?.as_encoded_bytes() != b"remove" {
        return None;
    }
    let rest: &[OsString] = &arguments[subcommand_index + 2..];
    let operand: &OsString = match rest {
        [only] if !only.as_encoded_bytes().starts_with(b"-") => only,
        [separator, only] if separator.as_encoded_bytes() == b"--" => only,
        _ => return None,
    };
    // Joined to the start directory an empty operand would name that directory, while Git answers that '' is not a working tree.
    if operand.is_empty() {
        return None;
    }
    // Git matches the typed name against the registered paths first, and a name that matches one of them names that worktree.
    if let Some(common) = common_dir
        && let Some(target) = crate::worktree_remove_registry::path_git_picks_for_operand(common, operand.as_os_str())
    {
        return Some(RemovePlan { target, operand: operand.clone() });
    }
    let target: PathBuf = std::fs::canonicalize(start.join(operand)).ok()?;
    return target.is_dir().then_some(RemovePlan { target, operand: operand.clone() });
}

/// Removal plan controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_plan_tests.rs"]
mod tests;
