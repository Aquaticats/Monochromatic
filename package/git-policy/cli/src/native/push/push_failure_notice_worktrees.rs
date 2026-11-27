//! What:
//!  Which worktree a record belongs to:
//!  the branch each worktree of the repository has checked out,
//!  the rule that says a record is the asking worktree's,
//!  and the way a commit is named as the person should read it.
//! Why:
//!  A record is shown once,
//!  to the worktree whose commit it is about,
//!  so another worktree's command must leave it where it is;
//!  and the commit name is Git's abbreviation in the repository as it is when the notice is shown,
//!  which is a question only the caller can ask.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Worktrees = { here?: string; all: string[] };
//! function checkedOutBranches(commonDir: string): string[];
//! interface CommitNamer { nameOf(commit: string): string }
//! ```

/// Import the borrowed path type the directories are read through.
use std::path::Path;

/// Where the worktrees of a repository stand:
/// the branch the asking worktree has checked out and every branch any
///  worktree has checked out.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Worktrees {
    /// The branch reference the command's worktree has checked out;
    /// nothing for a detached head or an unknown worktree.
    pub here: Option<String>,
    /// The branch references checked out in every worktree of the repository.
    pub all: Vec<String>,
}

/// What:
///  The branch reference a `HEAD` file names,
/// or nothing for a detached head or an unreadable file.
/// Why:
///   Only the plain `ref: refs/heads/<name>` form names a branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// branchOfHead(gitDir)
/// ```
pub fn branch_of_head(git_dir: &Path) -> Option<String> {
    // A HEAD that is a named pipe is no HEAD, and it answers at once rather than holding every command that asks.
    let text: String = crate::plain_file::read_plain_text(&git_dir.join("HEAD")).ok()?;
    let target: &str = text.lines().next()?.strip_prefix("ref: ")?.trim();
    if target.starts_with("refs/heads/") {
        return Some(target.to_string());
    }
    return None;
}

/// What:
///  The branches checked out in the repository's worktrees:
/// the main one and each linked one.
/// Why:
///   A linked worktree keeps its own `HEAD` under `worktrees/<name>`;
/// the main worktree's is the common directory's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkedOutBranches(commonDir)
/// ```
pub fn checked_out_branches(common_dir: &Path) -> Vec<String> {
    let mut branches: Vec<String> = branch_of_head(common_dir).into_iter().collect();
    if let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) {
        for entry in listing.flatten() {
            branches.extend(branch_of_head(entry.path().as_path()));
        }
    }
    return branches;
}

/// Whether a record of `branch_ref` is for the asking worktree:
/// it has the branch checked out,
/// or no worktree has.
pub(super) fn is_for(
    worktrees: &Worktrees,
    branch_ref: &str,
) -> bool {
    if worktrees.here.as_deref() == Some(branch_ref) {
        return true;
    }
    for held in &worktrees.all {
        if held == branch_ref {
            return false;
        }
    }
    return true;
}

/// What:
///  Where a failure notice asks for the name of a commit as the person should read it.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The name is Git's abbreviation in the repository as it is when the notice is shown,
/// asked of Git once per shown notice;
/// the
///  state the question needs (the Git context,
/// the directories) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CommitNamer { nameOf(commit: string): string }
/// ```
pub trait CommitNamer {
    /// The name to show for `commit`.
    fn name_of(
        &self,
        commit: &str,
    ) -> String;
}

/// A commit name as it is,
/// for the callers that show commits in full.
pub(super) struct FullCommitName;

impl CommitNamer for FullCommitName {
    fn name_of(
        &self,
        commit: &str,
    ) -> String {
        return commit.to_string();
    }
}
