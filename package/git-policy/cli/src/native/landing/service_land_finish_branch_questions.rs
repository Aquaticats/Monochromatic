//! What:
//!  The branch questions a restart decides an open batch from:
//! what `reference` holds,
//! whether the batch's commit is
//!  that value or below it,
//! and the decision the two answers support.
//! Why:
//!  A restart drops the intent of a batch whose branch never moved,
//! and finishes one the branch shows;
//! every
//!  question Git could not answer leaves the batch open,
//! because the owner's rule is "never discard user data" and a
//!  Git that said nothing has not said the branch is unmoved.
//! Each question has two faces:
//! the synchronous one a caller on a thread
//!  asks,
//! and the awaited one a task of the runtime asks,
//! which awaits every Git child on the runtime.
//! They are
//!  this file rather than `service_land_finish.rs` to keep both under the file-size limit
//! (`AGENTS.md`,
//! `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const decision = await decisionForAwaiting(git, intent); // 'drop' | 'finish' | undefined
//! ```

use crate::service_journal::{Intent, Recovery, recovery_for};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;

/// The exit status `git rev-parse --verify --quiet` gives for a name that is no object:
/// the one answer that means "not there".
const NAME_IS_NO_OBJECT: i32 = 1;

/// The exit status `git merge-base --is-ancestor` gives when the first commit is not below the second.
const NOT_AN_ANCESTOR: i32 = 1;

/// What:
///  The value of `reference` in the repository:
/// its commit name,
/// an empty name when Git says for certain that the reference is
///  not there,
/// and nothing when Git gave no answer.
///  `Option<String>` is "an answer or none".
/// Why:
///   A restart drops the intent of a batch whose branch never moved.
/// A Git that could not be started,
/// or that failed in any way
///  other than "no such reference" (a lock,
/// a stuck mount,
/// a repository that cannot be read),
/// has not said the branch is
///  unmoved,
/// so the caller keeps the batch open.
/// `--quiet` makes a missing reference exit 1 with nothing on the standard error;
///  every other failure exits 128,
/// and a reference file Git cannot read exits 1 with a warning,
/// which is not "missing" either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentValue(git, intent): string | undefined // '' for a missing reference
/// ```
fn current_value(git: &GitContext, intent: &Intent) -> Option<String> {
    return value_answered(run_git(git, &current_value_request(intent)));
}

/// What:
///  `current_value` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and a restart decides an open batch once it is
///  a task.
/// The question and the reading of Git's answer are the synchronous face's,
/// so the two cannot decide a batch
///  from different answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function currentValueAwaiting(git, intent): Promise<string | undefined> // '' for a missing reference
/// ```
async fn current_value_awaiting(git: &GitContext, intent: &Intent) -> Option<String> {
    return value_answered(run_git_awaiting(git, &current_value_request(intent)).await);
}

/// The question `current_value` asks Git.
fn current_value_request(intent: &Intent) -> GitRequest {
    return GitRequest::new(intent.repository.as_path(), &["rev-parse", "--verify", "--quiet", intent.reference.as_str()]);
}

/// What:
///  What Git's answer to `current_value_request` says the reference holds.
/// Why:
///  Both faces read Git's answer through this one function,
/// so a missing reference,
/// a reference Git
///  could not read and a Git that could not be started stay three different answers for both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function valueAnswered(asked) { if (asked.error) return undefined; if (asked.code === 0) return asked.out.trim(); if (asked.code === 1 && asked.err === '') return ''; return undefined; }
/// ```
fn value_answered(answered: std::io::Result<GitOutput>) -> Option<String> {
    let Ok(output) = answered else {
        return None;
    };
    if output.succeeded() {
        return Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
    }
    if output.code == Some(NAME_IS_NO_OBJECT) && output.stderr.is_empty() {
        return Some(String::new());
    }
    return None;
}

/// What:
///  Whether the commit the batch landed is `current` or below it;
/// nothing when Git gave no answer.
/// Why:
///   Exit status 1 is Git's "not below";
/// any other failure (an object Git cannot find,
/// a lock,
/// a stuck mount) is no answer,
/// and
///  a batch is not dropped on no answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function landedBelow(git, intent, current): boolean | undefined
/// ```
fn landed_below(git: &GitContext, intent: &Intent, current: &str) -> Option<bool> {
    return below_answered(run_git(git, &landed_below_request(intent, current)));
}

/// What:
///  `landed_below` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the ancestry decides whether a landed batch is finished or
///  dropped once the restart is a task.
/// The question and the reading of Git's answer are the synchronous face's,
/// so the two
///  cannot decide differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function landedBelowAwaiting(git, intent, current): Promise<boolean | undefined>
/// ```
async fn landed_below_awaiting(git: &GitContext, intent: &Intent, current: &str) -> Option<bool> {
    return below_answered(run_git_awaiting(git, &landed_below_request(intent, current)).await);
}

/// The question `landed_below` asks Git.
fn landed_below_request(intent: &Intent, current: &str) -> GitRequest {
    return GitRequest::new(intent.repository.as_path(), &["merge-base", "--is-ancestor", intent.new.as_str(), current]);
}

/// What:
///  What Git's answer to `landed_below_request` says about the ancestry.
/// Why:
///  Both faces read Git's answer through this one function,
/// so "not below" is the one status that means no and
///  every other failure is no answer for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function belowAnswered(asked) { if (asked.error) return undefined; if (asked.code === 0) return true; return asked.code === 1 ? false : undefined; }
/// ```
fn below_answered(answered: std::io::Result<GitOutput>) -> Option<bool> {
    let Ok(output) = answered else {
        return None;
    };
    if output.succeeded() {
        return Some(true);
    }
    return if output.code == Some(NOT_AN_ANCESTOR) { Some(false) } else { None };
}

/// What:
///  The decision for an open batch from the branch as Git reports it now,
/// or nothing when Git gave no answer.
/// Why:
///   The question about the ancestry is asked only when it decides:
/// a branch at the old or the new value,
/// or one that is gone,
///  settles the batch without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decisionFor(git, intent): Recovery | undefined
/// ```
pub(crate) fn decision_for(git: &GitContext, intent: &Intent) -> Option<Recovery> {
    let current: String = current_value(git, intent)?;
    let decided_without_ancestry: bool = current.is_empty() || current == intent.old || current == intent.new;
    let ancestor: bool = if decided_without_ancestry { false } else { landed_below(git, intent, current.as_str())? };
    return Some(recovery_for(intent, current.as_str(), ancestor));
}

/// What:
///  `decision_for` with both Git questions awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and a restart decides every open batch once it is a task;
/// the ancestry is asked only when it decides,
/// and what the two answers mean is the synchronous face's,
/// so the
///  two cannot decide one batch differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function decisionForAwaiting(git, intent): Promise<Recovery | undefined>
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the recovery calls the synchronous decision until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn decision_for_awaiting(git: &GitContext, intent: &Intent) -> Option<Recovery> {
    let current: String = current_value_awaiting(git, intent).await?;
    let decided_without_ancestry: bool = current.is_empty() || current == intent.old || current == intent.new;
    let ancestor: bool = if decided_without_ancestry { false } else { landed_below_awaiting(git, intent, current.as_str()).await? };
    return Some(recovery_for(intent, current.as_str(), ancestor));
}
