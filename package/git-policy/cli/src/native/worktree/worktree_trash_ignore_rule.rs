//! What:
//!  Whether Git ignores the place a snapshot's trash would be made at in one worktree.
//! Why:
//!  A trash directory Git does not ignore would list as untracked until it is emptied,
//! so the leave-out removes in the
//!  foreground instead when the answer is no or unknown.
//! The question is one `git check-ignore` run,
//! bounded by the step terms its class gives it,
//! and it has two
//!  faces because two callers ask:
//! the synchronous check of a snapshot's source and the task of the runtime that reads
//!  it beside that source's other two checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ignored = await trashLocationIsIgnoredAwaiting(context, root);
//! ```

use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::worktree_trash::TRASH_PARENT;
use std::path::Path;

/// What:
///  The run that asks Git whether it ignores the place the trash would be made at.
/// Why:
///   Both faces of the question start the same child,
/// so they cannot ask different things.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // `git check-ignore --no-index -- .claude/cli-git-trash/probe/x`
/// ```
fn trash_ignore_request(root: &Path) -> GitRequest {
    return GitRequest {
        cwd: root.to_path_buf(),
        arguments: ["check-ignore", "--no-index", "--"].iter().map(std::ffi::OsString::from).chain(std::iter::once(std::ffi::OsString::from(format!("{TRASH_PARENT}/probe/x")))).collect(),
        without_prefix: true,
        ..GitRequest::default()
    };
}

/// What:
///  Whether Git ignores the place the trash would be made at in this worktree,
/// for a caller that is not a task of the runtime.
/// Why:
///   The source check's callers are synchronous,
/// and a task of the runtime awaits a child rather than entering the runtime to block on
///  one;
/// `trash_location_is_ignored_awaiting` is the face such a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGit(['check-ignore', '--no-index', '--', `${TRASH_PARENT}/probe/x`]).exitCode === 0
/// ```
pub fn trash_location_is_ignored(context: &GitContext, root: &Path) -> bool {
    return match run_git(context, &trash_ignore_request(root)) {
        Ok(output) => output.code == Some(0),
        Err(_not_started) => false,
    };
}

/// What:
///  `trash_location_is_ignored` with the Git child awaited on the runtime.
/// Why:
///   The source check asks this beside the configuration read and the cleanliness check,
/// on a task of the runtime,
/// which
///  cannot enter that runtime to run a child;
/// the run's bound and quiet window are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function trashLocationIsIgnoredAwaiting(context: GitContext, root: string): Promise<boolean>;
/// ```
pub async fn trash_location_is_ignored_awaiting(context: &GitContext, root: &Path) -> bool {
    return match run_git_awaiting(context, &trash_ignore_request(root)).await {
        Ok(output) => output.code == Some(0),
        Err(_not_started) => false,
    };
}
