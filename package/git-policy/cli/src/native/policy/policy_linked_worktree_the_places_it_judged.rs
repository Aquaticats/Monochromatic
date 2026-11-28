//! What:
//!  The two places a dirty refusal of the linked-worktree-only policy names,
//!  read from the identity the facts answered with.
//! Why:
//!  A `!` alias's shell hands the nested Git one worktree's Git directory while the caller's own `-C` moves
//!  the files to another worktree,
//!  so the state a form is refused in can be two worktrees' and a text that said "this worktree" named
//!  neither of them (the alpha.52 read's L2).
//!  The reading sits in a file of its own,
//!  because the check that uses it would otherwise pass the linter's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { WorktreeIdentity, gitDirectory, worktreeRoot } from './worktree_identity.ts';
//! ```

use crate::repository_location::RepositoryLocation;
/// The form a dirty refusal names,
/// and whether the Git directory and the worktree are one place.
use crate::rule_linked_worktree_dirty_refusal::DirtyWorktree;
/// The two places an identity names.
use crate::worktree_identity::{
    git_directory,
    worktree_root,
};
use std::path::Path;

/// What:
///  The two places a dirty refusal names,
///  read from the identity the facts answered with:
///  the Git directory whose index and `HEAD` a guarded form rewrites,
///  the worktree whose files it writes,
///  and whether that directory is the one that worktree uses.
/// Why:
///  The two are one place everywhere but the shape a `!` alias makes,
///  and the text says which of the two shapes it read.
///  A worktree whose `.git` entry names something other than that Git directory is the shape that reads as two,
///  and an entry this run cannot read is not proven to be the same one,
///  so it reads as two as well.
///  Only the two worktree identities reach a dirty question,
///  and any other shape answers nothing to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dirtyWorktree = (command, location) => ({
///   command,
///   gitDirectory: gitDirectory(location.identity),
///   worktreeRoot: worktreeRoot(location.identity),
///   oneWorktree: dotGitTarget(worktreeRoot(location.identity))?.[0] === gitDirectory(location.identity),
/// });
/// ```
pub(crate) fn dirty_worktree<'a>(
    command: crate::the_guarded_commands::GuardedCommand,
    location: &'a RepositoryLocation,
) -> Option<DirtyWorktree<'a>> {
    let git_directory: &Path = git_directory(&location.identity)?;
    let worktree_root: &Path = worktree_root(&location.identity)?;
    return Some(DirtyWorktree {
        command,
        git_directory,
        worktree_root,
        one_worktree: the_worktree_uses_this_git_directory(worktree_root, git_directory),
    });
}

/// What:
///  Whether the worktree at a root uses that Git directory:
///  the entry its `.git` names is the directory the facts were asked with.
/// Why:
///  The `.git` entry is the reading Git itself makes:
///  a directory in a main checkout,
///  and a file naming the Git directory in a linked worktree
///  (`setup.c`'s `read_gitfile_gently` in Git 2.56.1).
///  Both answers are resolved before they are compared,
///  because the worktree's own path and the entry's target can hold links and `..`,
///  and a path that does not resolve is compared as it stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theWorktreeUsesThisGitDirectory = (root, gitDirectory) => {
///   const named = dotGitTarget(root);
///   return named !== undefined && realpath(named[0]) === realpath(gitDirectory);
/// };
/// ```
fn the_worktree_uses_this_git_directory(
    worktree_root: &Path,
    git_directory: &Path,
) -> bool {
    let Some(Some((named, _is_a_file))) = crate::repo_read_layout::dot_git_target(worktree_root)
    else {
        return false;
    };
    let resolved_named: std::path::PathBuf =
        std::fs::canonicalize(named.as_path()).unwrap_or(named);
    let resolved_git_directory: std::path::PathBuf = std::fs::canonicalize(git_directory)
        .unwrap_or_else(|_error| return git_directory.to_path_buf());
    return resolved_named == resolved_git_directory;
}
