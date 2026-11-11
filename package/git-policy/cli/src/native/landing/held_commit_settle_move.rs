//! What:
//!  The one branch write of a settlement,
//!  and what the batch owes around it.
//! Why:
//!  The settlement reads the branch,
//!  decides from it,
//!  and moves it by one compare-and-swap
//!  (`held_commit_settle.rs`).
//!  What the write owes before it and gives back when it is refused is one step with it,
//!  so both stand here and the settlement's file stays short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! whatABatchOwes(written); const moved = moveBranch(...); if (!moved.ok) forgetWhatABatchOwes(written);
//! ```

use crate::held_commit_record::Held;
use crate::held_commit_settle::kept_reference;
use crate::service_journal::Intent;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::ffi::OsStr;

/// What a Git that could not be started is said to be,
/// in one sentence the callers share.
pub(crate) fn git_did_not_start(error: std::io::Error) -> String {
    return format!("Git could not be started ({error})");
}

/// What:
///  Moves the branch from the value it was read at to `new`,
///  and removes the side reference the commit was kept on.
/// Why:
///  The compare-and-swap is the one step that decides whether this settlement or another one
///  (another command,
///  or a landing) moved the branch,
///  so a commit can never be applied twice.
///  The message is one argument of `-m`,
///  never a line of the transaction,
///  so no byte of it can change the grammar Git reads on standard input
///  (`AGENTS.md` rules `SYB` and `STB`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// updateRef(['-m', reflog, '--stdin'], `start\nupdate ${ref} ${new} ${old}\ndelete ${scratch}\nprepare\ncommit\n`)
/// ```
pub(crate) fn move_branch(git: &GitContext, held: &Held, intent: &Intent, new: &str, expected: &str) -> Result<bool, String> {
    let scratch: String = kept_reference(held.seq);
    // The name is bytes and the rest of the line is the wrapper's own text, so a name that is not text reaches Git whole.
    let mut text: Vec<u8> = b"start\nupdate ".to_vec();
    text.extend_from_slice(intent.reference.bytes());
    text.extend_from_slice(format!(" {new} {expected}\ndelete {scratch}\nprepare\ncommit\n").as_bytes());
    let mut request: GitRequest = GitRequest::new(held.repository.as_path(), &["update-ref"]);
    request.arguments.push(OsStr::new("-m").to_os_string());
    request.arguments.push(crate::path_bytes::os_from_path_bytes(held.reflog.as_slice()));
    request.arguments.push(OsStr::new("--stdin").to_os_string());
    request.input = Some(text);
    let output: GitOutput = run_git(git, &request).map_err(git_did_not_start)?;
    if output.succeeded() {
        return Ok(true);
    }
    return Err(String::from_utf8_lossy(output.stderr.as_slice()).trim().to_string());
}

/// What:
///  Moves the branch for a settlement,
///  taking everything the batch owes before that move and giving it back when the write is refused.
/// Why:
///  A kill between the move and the index install leaves the record of the index update and the held push of every commit
///  the move lands,
///  and the push is held before the branch names it (`work_item_holding.rs`).
///  A branch that did not move owes nothing,
///  so a write that was refused or that lost a race takes the records back.
///  The push is held again by `finish` once the branch names the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whatABatchOwes(written); const moved = moveBranch(...); if (!moved.ok) forgetWhatABatchOwes(written); moved
/// ```
pub(crate) fn move_the_branch(git: &GitContext, held: &Held, written: &Intent, commit: &str, current: &str) -> Result<bool, String> {
    crate::service_server_land_group::what_a_batch_owes(written);
    let moved: Result<bool, String> = move_branch(git, held, written, commit, current);
    if matches!(moved, Ok(false) | Err(_)) {
        crate::service_server_land_group::forget_what_a_batch_owes(written);
    }
    return moved;
}
