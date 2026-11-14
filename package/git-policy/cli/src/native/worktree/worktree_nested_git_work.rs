//! What:
//!  The work of a worktree registered inside one being removed that no other copy holds:
//!  an edit to a tracked file,
//!  a staged change,
//!  and a commit nothing else names.
//! Why:
//!  The owner's decision of 2026-11-12T13:04:14
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "How far the protection of a worktree being removed reaches") says
//!  "Git work is never destroyed by a removal:
//!  a commit that no reference holds,
//!  and an edit to a tracked file,
//!  however it is hidden".
//!  A worktree nested inside another is usually under a name the outer worktree ignores,
//!  so real Git deletes the whole nested checkout without asking the nested worktree's own rules,
//!  and an uncommitted edit there has no other copy anywhere.
//!  The automatic routes already leave a worktree that holds a registered worktree,
//!  which is the stronger answer;
//!  this question is asked where a person types the removal,
//!  so the refusal can name the files instead of leaving the worktree alone with no word.
//!  A branch names its own commits and the branch survives a removal,
//!  so a nested worktree on a branch holds nothing here that goes:
//!  only a `HEAD` that names a commit directly is a commit with no other name,
//!  and only while no reference contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holding = nestedWorktreesHoldingGitWork(git, commonDir, target); // [{ worktree, changed, holdsACommitNothingElseNames }]
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_landed_registrations::{RegisteredWorktree, registered_worktrees};
use crate::worktree_ladder_target::checkout_environment;
use crate::worktree_removal_deletions::DeletedEntries;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// How many changed paths a refusal names before it counts the rest.
/// Why:
///  A person decides about their own work,
///  so the first names are shown,
///  and a list longer than a few would bury the sentence it sits in.
const SHOWN_PATHS: usize = 3;

/// What:
///  One nested worktree,
///  and the work it holds that a removal would take with no other copy.
/// Why:
///  The refusal names the same facts the record does,
///  so the value carries both the worktree and what was found there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NestedHoldingGitWork = { worktree: string; changed: DeletedEntries; holdsACommitNothingElseNames: boolean };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NestedHoldingGitWork {
    /// The nested worktree's directory.
    pub worktree: PathBuf,
    /// The tracked files it has changed,
    /// staged changes among them.
    pub changed: DeletedEntries,
    /// Whether its `HEAD` names a commit no reference contains,
    /// which a removal deletes the only name of.
    pub holds_a_commit_nothing_else_names: bool,
}

impl NestedHoldingGitWork {
    /// Whether there is any Git work here at all.
    pub fn anything(&self) -> bool {
        return self.holds_a_commit_nothing_else_names || self.changed.total > 0;
    }

    /// What:
    ///  The words that say what this worktree holds,
    ///  for the sentence a refusal prints.
    /// Why:
    ///  A person has to know what would go and where it is,
    ///  so the files are named,
    ///  a longer list is counted,
    ///  and a commit with no other name is said in its own words.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// holding.why() // '`tracked.txt` and 1 more, and a commit no reference names'
    /// ```
    pub fn why(&self) -> String {
        let mut parts: Vec<String> = Vec::with_capacity(SHOWN_PATHS + 2);
        if self.changed.total > 0 {
            parts.push(format!("changed tracked files ({})", names_of(&self.changed)));
        }
        if self.holds_a_commit_nothing_else_names {
            parts.push(String::from("a commit that no reference names, which only its own HEAD holds"));
        }
        return parts.join(", and ");
    }
}

/// What:
///  The nested worktrees of `target` that hold work a removal would take with no other copy.
/// Why:
///   The question is asked before anything is removed,
///  and a worktree whose own state cannot be read is an error rather than an empty answer:
///  nothing then proves the removal would take nothing of the person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nestedWorktreesHoldingGitWork(git, commonDir, target) // throws the reason
/// ```
pub fn nested_worktrees_holding_git_work(context: &GitContext, common_dir: &Path, target: &Path) -> Result<Vec<NestedHoldingGitWork>, String> {
    // The registrations are read with their `HEAD`s,
    //  the detached ones included:
    //  a detached worktree is exactly the one whose commit has no other name,
    //  and the landed sweep's own reader leaves it out because nothing can land on a branch it does not have.
    let mut holding: Vec<NestedHoldingGitWork> = Vec::new();
    for nested in registered_worktrees(common_dir)? {
        if nested.worktree == target || !nested.worktree.starts_with(target) {
            continue;
        }
        let found: NestedHoldingGitWork = one_nested_worktree(context, &nested)?;
        if found.anything() {
            holding.push(found);
        }
    }
    return Ok(holding);
}

/// What:
///  What one nested worktree holds.
/// Why:
///  The two questions are asked of the same worktree and answered together,
///  and each of them is Git's own answer:
///  a status that leaves out the untracked files,
///  which go with the removal and are recorded,
///  and a `for-each-ref` that says whether any reference contains the commit `HEAD` names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// oneNestedWorktree(git, nested) // { worktree, changed, holdsACommitNothingElseNames }
/// ```
fn one_nested_worktree(context: &GitContext, nested: &RegisteredWorktree) -> Result<NestedHoldingGitWork, String> {
    let changed: DeletedEntries = DeletedEntries::changed_tracked_of(context, nested.worktree.as_path(), Some(nested.admin.as_path()));
    if let Some(reason) = &changed.unreadable {
        return Err(reason.clone());
    }
    let holds: bool = detached_commit_nothing_else_names(context, nested)?;
    return Ok(NestedHoldingGitWork { worktree: nested.worktree.clone(), changed, holds_a_commit_nothing_else_names: holds });
}

/// What:
///  Whether the commit this worktree's `HEAD` names is held by no reference at all.
/// Why:
///  A `HEAD` that is a branch name points at a reference that lives in the common Git directory and survives the removal,
///  so the commit goes on having a name.
///  A `HEAD` that is an object id names a commit directly and only as long as the worktree's own Git directory exists,
///  and `for-each-ref --contains` is Git's own answer to whether another reference reaches it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// detachedCommitNothingElseNames(git, nested) // boolean, or throws the reason
/// ```
fn detached_commit_nothing_else_names(context: &GitContext, nested: &RegisteredWorktree) -> Result<bool, String> {
    // A `HEAD` that names a branch points at a reference of the common Git directory, which survives the removal,
    //  so only a detached one is a commit whose only name goes.
    if nested.branch.is_some() {
        return Ok(false);
    }
    let arguments: Vec<OsString> = ["for-each-ref", "--contains", "HEAD", "--format=%(refname)"].iter().map(OsString::from).collect();
    let request: GitRequest = GitRequest {
        cwd: nested.worktree.clone(),
        arguments,
        without_prefix: true,
        environment: checkout_environment(nested.admin.as_path(), nested.worktree.as_path()),
        ..GitRequest::default()
    };
    let output: GitOutput = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("Git could not be started to ask whether a reference names the commit of {} ({error})", nested.worktree.display())),
    };
    if output.code != Some(0) {
        let said: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned();
        return Err(format!("Git could not say whether a reference names the commit of {} ({said})", nested.worktree.display()));
    }
    // `HEAD` that names no commit at all, as an unborn branch does, answers nothing and holds nothing.
    return Ok(output.code == Some(0) && String::from_utf8_lossy(output.stdout.as_slice()).trim().is_empty());
}

/// What:
///  The names of the changed paths a refusal shows,
///  a few named and the rest counted.
/// Why:
///  A person decides about their own work,
///  so the names come first,
///  and a list longer than a few would bury the sentence it sits in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesOf(changed) // '`a.txt`, `b.txt` and 1 more'
/// ```
fn names_of(changed: &DeletedEntries) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(SHOWN_PATHS + 1);
    for name in changed.shown.iter().take(SHOWN_PATHS) {
        parts.push(format!("`{name}`"));
    }
    let rest: usize = changed.total.saturating_sub(parts.len());
    if rest > 0 {
        parts.push(format!("and {rest} more"));
    }
    return parts.join(", ");
}

/// Controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_nested_git_work_tests.rs"]
mod tests;
