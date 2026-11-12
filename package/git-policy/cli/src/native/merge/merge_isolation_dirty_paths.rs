//! What:
//!  Every path of the worktree whose own work a merge would write over:
//! the paths Git's status reports,
//! untracked files included,
//! and the ignored untracked files of the person's that stand at the paths the merge touches.
//! Why:
//!   A merge that writes such a path would discard what it holds,
//! and an untracked file at a path the merge adds is why
//!  native Git refuses such a merge itself;
//!  `git status` never lists a file an ignore rule covers,
//! so those paths are read from
//!  Git's own list of the untracked files the rules cover (`merge_isolation_ignored_paths.rs`),
//!  because the
//!  merge writes over the only copy of their bytes as it would over any other untracked file's.
//!  The files that decide a merge live here rather than with the merge,
//!  which is at its code-line limit,
//!  and both a merge that
//!  creates a commit and one that fast-forwards read them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const dirty = statusPaths(await git(['--no-optional-locks', 'status', '--porcelain=v1', '-z', '--untracked-files=all'])).concat(ignoredUntrackedAmong(context, root, touched));
//! ```

/// Names and sets of unsaved work.
use crate::auto_push_dirty_paths::status_paths;
/// The ignored untracked files of the person's that stand at the paths a merge writes,
/// which `git status` never lists.
use crate::merge_isolation_ignored_paths::ignored_untracked_among;
/// The person's own work in the paths the merge changes,
/// captured as objects and kept on a ref of its own.
use crate::merge_isolation_carry::{capture_work, keep_capture};
/// The person's work as the capture holds it.
use crate::merge_isolation_carry_parts::Capture;
/// The session a merge asks its questions in,
/// and the way it gives up.
use crate::merge_isolation_git_reads::Session;
use crate::merge_isolation_run_types::Bail;
/// Real Git.
use crate::transaction_git::{GitContext, GitOutput};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Every path whose worktree or index differs from the branch,
/// untracked files included,
/// and the ignored untracked files of the person's
///  that stand at the paths the merge touches.
/// Why:
///   These are exactly the paths a merge that writes them would discard,
/// and the untracked ones are why native
///  Git refuses a merge that adds a path a file already stands at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dirty = statusPaths(status).concat(ignoredUntrackedAmong(context, root, touched));
/// ```
pub(crate) fn dirty_paths(session: &Session, root: &Path, touched: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, Bail> {
    let status: GitOutput = session.run(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
    ])?;
    if !status.succeeded() {
        return Err(Bail::from(format!("git status failed: {}", status.error_text())));
    }
    let mut paths: Vec<Vec<u8>> = status_paths(status.stdout.as_slice());
    for path in ignored_untracked_among(&session.context, root, touched).map_err(Bail::from)? {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    return Ok(paths);
}

/// What:
///  The person's own work in `paths`,
/// captured as objects and kept on a ref of its own.
/// Why:
///   A ref written before the record and before the checkout changes is what makes the bytes survive a kill at any
///  instant (`SPEC.md`,
/// "Merges and commits never fail").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureFor(context, root, paths): Promise<Capture | null>;
/// ```
pub(crate) fn capture_for(context: &GitContext, root: &Path, paths: &[Vec<u8>]) -> Result<Option<Capture>, String> {
    if paths.is_empty() {
        return Ok(None);
    }
    let capture: Capture = capture_work(context, root, paths)?;
    return Ok(Some(keep_capture(context, root, &capture)?));
}
