//! What:
//!  The one question the index of a worktree cannot answer:
//! whether an untracked file is one Git ignores.
//! Why:
//!  Only Git's ignore rules can tell an untracked file that matters from one it ignores,
//! so the answer is one `git ls-files` run
//!  and never a walk of the directory tree by this process.
//! The run is bounded by the step terms its class gives it,
//! and it has two faces
//!  because two callers ask:
//! the synchronous cleanliness check and its controls,
//! and the task of the runtime that
//!  reads it beside the proof of the tracked files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const untracked = await untrackedFileAwaiting(context, root); // the first name no ignore rule covers
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::worktree_clean_stat_verdicts::CleanRefusal;
use std::path::Path;

/// What:
///  Whether a byte is NUL,
/// the end of each name in Git's `-z` output.
/// Why:
///   The untracked names are split on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The run that lists the untracked files no ignore rule covers.
/// Why:
///   Both faces of the scan start the same child,
/// so the words of a failure and the answer are the same for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // `git ls-files --others --exclude-standard --directory --no-empty-directory -z`
/// ```
fn untracked_listing(root: &Path) -> GitRequest {
    return GitRequest {
        cwd: root.to_path_buf(),
        arguments: ["ls-files", "--others", "--exclude-standard", "--directory", "--no-empty-directory", "-z"].map(Into::into).to_vec(),
        without_prefix: true,
        ..GitRequest::default()
    };
}

/// What:
///  The first untracked name Git listed,
/// or the refusal the run is.
/// Why:
///   `--directory` stops at the first untracked directory and `--no-empty-directory` leaves out directories without files,
/// as
///  `git status` does.
/// The two faces of the scan make their answer here,
/// so neither can read a run the other refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstUntrackedName(run: GitOutput | Error): string | undefined; // throws the reason
/// ```
fn first_untracked_name(run: std::io::Result<GitOutput>) -> Result<Option<String>, CleanRefusal> {
    let output = match run {
        Ok(finished) => finished,
        Err(error) => return Err(CleanRefusal::unreadable(format!("Git could not list untracked files ({error})"))),
    };
    if output.code != Some(0) {
        return Err(CleanRefusal::unreadable(format!("Git could not list untracked files: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim())));
    }
    // The first name that is not empty, as `find` chose it.
    for name in output.stdout.split(is_nul_byte) {
        if !name.is_empty() {
            return Ok(Some(String::from_utf8_lossy(name).into_owned()));
        }
    }
    return Ok(None);
}

/// What:
///  Whether Git lists an untracked file that no ignore rule covers,
/// for a caller that is not a task of the runtime.
/// Why:
///   The scan's callers are synchronous (the cleanliness check and its controls),
/// and a task of the runtime awaits a child rather
///  than entering the runtime to block on one;
/// `untracked_file_awaiting` is the face such a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function untrackedFile(context: GitContext, root: string): string | undefined; // throws the reason
/// ```
fn untracked_file(context: &GitContext, root: &Path) -> Result<Option<String>, CleanRefusal> {
    return first_untracked_name(run_git(context, &untracked_listing(root)));
}

/// What:
///  `untracked_file` with the Git child awaited on the runtime.
/// Why:
///   The scan runs beside the tracked proof on the runtime,
/// so its child is awaited there;
/// the bound and the quiet window
///  of the run are the synchronous face's (`transaction_git_awaiting.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function untrackedFileAwaiting(context: GitContext, root: string): Promise<string | undefined>;
/// ```
async fn untracked_file_awaiting(context: &GitContext, root: &Path) -> Result<Option<String>, CleanRefusal> {
    return first_untracked_name(run_git_awaiting(context, &untracked_listing(root)).await);
}

/// What:
///  `untracked_file`,
/// timed as the work `untrackedScan` because it is a wait of its own.
/// Why:
///   The add timing record sums the work of a step by name;
/// the scan runs beside the tracked proof and is
///  reported as the work it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const scanned = untrackedFile(context, root); endWork('untrackedScan', began);
/// ```
pub fn scan_for_untracked_file((context, root): (&GitContext, &Path)) -> Result<Option<String>, CleanRefusal> {
    let began: std::time::Instant = begin_work();
    let scanned: Result<Option<String>, CleanRefusal> = untracked_file(context, root);
    end_work("untrackedScan", began);
    return scanned;
}

/// What:
///  `scan_for_untracked_file` on the runtime:
/// the same work,
/// with the child awaited and the time it took still recorded.
/// Why:
///   A task of the runtime cannot start this child through the synchronous face,
/// which would have to enter the runtime to
///  register the step's pipes;
/// the timing is written down for a wait as it was for a thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const scanned = await untrackedFileAwaiting(context, root); endWork('untrackedScan', began);
/// ```
pub async fn scan_for_untracked_file_awaiting((context, root): (&GitContext, &Path)) -> Result<Option<String>, CleanRefusal> {
    let began: std::time::Instant = begin_work();
    let scanned: Result<Option<String>, CleanRefusal> = untracked_file_awaiting(context, root).await;
    end_work("untrackedScan", began);
    return scanned;
}
