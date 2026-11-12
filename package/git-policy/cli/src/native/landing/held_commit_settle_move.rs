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
//!  Each face stands twice:
//!  the synchronous one a command that is not a task calls,
//!  and the awaited one a task of the runtime calls,
//!  over one request and one reading of Git's answer
//!  (`doc/planning/git-policy-cli-full-tokio.md`,
//!  "the settlement's own face").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! whatABatchOwes(written); const moved = await moveBranchAwaiting(...); if (!moved.ok) forgetWhatABatchOwes(written);
//! ```

use crate::held_commit_record::Held;
use crate::held_commit_settle::kept_reference;
use crate::service_journal::Intent;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::ffi::OsStr;

/// What a Git that could not be started is said to be,
/// in one sentence the callers share.
pub(crate) fn git_did_not_start(error: std::io::Error) -> String {
    return format!("Git could not be started ({error})");
}

/// What:
///  The question `move_branch` asks Git:
///  the branch moved from `expected` to `new`,
///  and the side reference the commit was kept on removed with it.
/// Why:
///  Both faces ask the one question,
/// so the compare-and-swap and the reflog message are the same for the two,
/// and a name that is not text reaches Git whole on either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: held.repository, arguments: ['update-ref', '-m', reflog, '--stdin'], input: `start\nupdate ${ref} ${new} ${expected}\ndelete ${scratch}\nprepare\ncommit\n` };
/// ```
fn move_branch_request(held: &Held, intent: &Intent, new: &str, expected: &str) -> GitRequest {
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
    return request;
}

/// What:
///  What Git's answer to `move_branch_request` came to:
///  the branch moved,
///  or the words of its refusal.
/// Why:
///  Both faces read the answer through this one function,
/// so a branch that moved between the read and the write is the same answer for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveBranchAnswered(output) { return output.ok ? { moved: true } : { refused: output.stderr.trim() } }
/// ```
fn move_branch_answered(output: GitOutput) -> Result<bool, String> {
    if output.succeeded() {
        return Ok(true);
    }
    return Err(String::from_utf8_lossy(output.stderr.as_slice()).trim().to_string());
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
    let output: GitOutput = run_git(git, &move_branch_request(held, intent, new, expected)).map_err(git_did_not_start)?;
    return move_branch_answered(output);
}

/// What:
///  `move_branch` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and this is the one write that decides whether this settlement or another moved the branch.
/// The compare-and-swap,
/// the reflog message and the words of a refusal are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await moveBranchAwaiting(git, held, intent, new, expected)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the settlement's own face is a slice of its own (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn move_branch_awaiting(git: &GitContext, held: &Held, intent: &Intent, new: &str, expected: &str) -> Result<bool, String> {
    let output: GitOutput = run_git_awaiting(git, &move_branch_request(held, intent, new, expected)).await.map_err(git_did_not_start)?;
    return move_branch_answered(output);
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
