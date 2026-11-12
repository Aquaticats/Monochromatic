//! What:
//!  The replay's reads as faces a task of the runtime may await:
//!  a commit's tree,
//!  a look at the private index an attempt captured,
//!  and the write-tree of that index.
//! Why:
//!   A task of the runtime may neither wait on a filesystem nor enter the runtime to wait for a child,
//! and a replay that is a task reads the change its preparation captured through these faces.
//! The questions,
//! the `indexed_request` line and the words of every stop are the synchronous faces',
//! which `service_replay.rs` still holds,
//! so both faces ask Git the same thing and answer one stop the same way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await treeOfCommitAwaiting(context, cwd, commit); await capturedTreeAwaiting(context, cwd, commitIndex, objectDirectory);
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::commit_transaction::{TransactionStop, failed};
use crate::private_index_request::indexed_request;
use crate::transaction_git::GitContext;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use crate::wait_ceilings::COMMIT_OWN_STATE_CEILING;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{NOTHING_CHANGED, tree_of, tree_of_commit_request};

/// `tree_of_commit` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await treeOfCommitAwaiting(context, cwd, commit)
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` when the commit cannot be read.
pub async fn tree_of_commit_awaiting(context: &GitContext, cwd: &Path, commit: &str) -> Result<String, TransactionStop> {
    let read = run_git_checked_awaiting(context, &tree_of_commit_request(cwd, commit)).await?;
    return Ok(tree_of(read.stdout.as_slice()));
}

/// The private index one look names,
/// owned so that a job of the blocking pool can take it.
struct CapturedIndex {
    /// The private index file of the attempt that captured the change.
    path: PathBuf,
}

/// What one look at a private index answered.
enum CapturedIndexLook {
    /// The file is there,
    /// so the change it holds can be written out.
    Present,
    /// Nothing is there.
    Absent,
    /// The look gave no answer in time.
    NoAnswer,
}

/// The look one job of the pool does:
/// whether the private index of the attempt is a file.
fn captured_index_is_there(work: CapturedIndex) -> bool {
    return matches!(std::fs::symlink_metadata(work.path.as_path()), Ok(metadata) if metadata.is_file());
}

/// What:
///  Whether the private index of the attempt that captured the change is there,
/// as a job of the blocking pool awaited no
///  longer than `ceiling`.
/// Why:
///   A task of the runtime may not wait on a filesystem,
/// and this look waits in the kernel on a mount that stopped answering.
///  The three answers are kept apart:
/// a file that is there,
/// one that is not,
/// and a look that gave nothing in time,
/// so
///  no answer is never read as "the index is gone".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function capturedIndexLookWithin(ceilingMs, index): Promise<'present' | 'absent' | 'no answer'>;
/// ```
async fn captured_index_look_within(ceiling: Duration, index: &Path) -> CapturedIndexLook {
    let work: CapturedIndex = CapturedIndex { path: index.to_path_buf() };
    return match run_a_job_awaited_no_longer_than(ceiling, work, captured_index_is_there).await {
        TheAnswerOfAJobAwaited::Ran(true) => CapturedIndexLook::Present,
        TheAnswerOfAJobAwaited::Ran(false) => CapturedIndexLook::Absent,
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => CapturedIndexLook::NoAnswer,
    };
}

/// What:
///  `captured_tree` with its look at the private index a job of the blocking pool awaited no longer than
///  `COMMIT_OWN_STATE_CEILING`,
///  and its Git child awaited on the runtime.
/// Why:
///   A task of the runtime may neither wait on a filesystem nor enter the runtime to wait for a child,
/// and a replay
///  that is a task reads the change its preparation captured through this face.
/// A look that gave nothing in time is told
///  apart from an index that is gone,
/// and the commit stops either way,
/// because a write-tree over an index nothing saw would
///  rebuild the commit as a deletion of every file and report a conflict naming paths it never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await capturedTreeAwaiting(context, cwd, commitIndex, objectDirectory)
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` when the private index is not there or cannot be looked at,
/// and when the write-tree
///  fails.
pub async fn captured_tree_awaiting(context: &GitContext, cwd: &Path, commit_index: &Path, object_directory: &Path) -> Result<String, TransactionStop> {
    match captured_index_look_within(COMMIT_OWN_STATE_CEILING, commit_index).await {
        CapturedIndexLook::Present => {}
        CapturedIndexLook::Absent => {
            return Err(failed(format!(
                "The change this commit captured cannot be read back: the private index of the attempt that captured it, {}, is not there, \
                 so this commit was not made. Run the commit again. {NOTHING_CHANGED}",
                commit_index.display()
            )));
        }
        CapturedIndexLook::NoAnswer => {
            return Err(failed(format!(
                "The change this commit captured cannot be read back: the private index of the attempt that captured it, {}, could not be \
                 looked at in time, so this commit was not made. Run the commit again. {NOTHING_CHANGED}",
                commit_index.display()
            )));
        }
    }
    let written = run_git_checked_awaiting(context, &indexed_request(context, cwd, &["write-tree"], Some(commit_index), Some(object_directory))).await?;
    return Ok(String::from_utf8_lossy(written.stdout.as_slice()).trim().to_string());
}

