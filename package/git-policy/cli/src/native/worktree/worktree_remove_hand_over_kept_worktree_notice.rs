//! What:
//!  What a removal says and records when it keeps a worktree:
//!  the reason,
//!  the commits a reference lock held,
//!  and the notice the next command of the repository shows.
//! Why:
//!  A worktree this program kept is the person's until they say otherwise,
//!  and the command that would have said so is over:
//!  the record is the only thing left that speaks,
//!  and the notice is what reaches the person on their next command.
//!  The verdict is a failure,
//!  because a `done` record means the worktree is gone,
//!  which this one is not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! keepTheWorktree(item, target, registration, why, keptCommits): Verdict
//! ```

use crate::service_work::Verdict;
use crate::work_item::WorkItem;
use crate::worktree_kept_notices::record_kept_worktree;
use crate::worktree_landed_registrations::branch_of_registration;
use std::path::Path;

/// What:
///  Keep the worktree,
///  say why in the repository's kept-worktree notices,
///  and answer the failing verdict that keeps the record from being read as a removal that happened.
/// Why:
///  A worktree this program kept is the person's until they say otherwise,
///  and the command that would have said so is over:
///  the record is the only thing left that speaks,
///  and the notice is what reaches the person on their next command.
///  The verdict is a failure because a `done` record means the worktree is gone,
///  which this one is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepTheWorktree(item, target, registration, why): Verdict
/// ```
pub(crate) fn keep_the_worktree(
    item: &WorkItem,
    target: &Path,
    registration: &Path,
    why: &str,
    kept_commits: &[String],
) -> Verdict {
    let branch: String = branch_of_registration(registration)
        .unwrap_or_else(|| return String::from("a head with no branch"));
    // A commit a reference lock held reached a reference of this repository before anything was read here, and a person
    //  who reads why their worktree stayed should read that too.
    let said: String = match kept_commits.is_empty() {
        true => why.to_owned(),
        false => format!("{}. {why}", kept_commits.join(". ")),
    };
    if let Err(reason) = record_kept_worktree(
        item.git_dir.as_path(),
        target,
        branch.as_str(),
        said.as_str(),
    ) {
        crate::service_warn!(
            "the worktree {} was left in place and the notice about it could not be written: {reason}",
            target.display()
        );
    }
    return Verdict::Failed(format!(
        "the worktree was left in place, and the next command of this repository names it: {said}"
    ));
}
