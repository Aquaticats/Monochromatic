//! What:
//!  The settlement's own reads as faces a task of the runtime may await:
//!  a batch's intent,
//!  the record of a batch that finished,
//!  and the question whether a batch's commit is below the branch.
//! Why:
//!  Each of them reads a file of the journal or starts a Git child,
//! and a task of the runtime may neither wait on the disk nor enter the runtime to wait for a child.
//! The answers,
//! the job structures and the questions are here,
//! and the synchronous faces of `held_commit_settle.rs` keep the caller they have.
//! A reading that gave nothing in time is told apart from a record that is not there,
//! because "no record" is an answer a settlement decides from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const intent = await intentAwaiting(journal, seq); const done = await finishedAwaiting(journal, seq);
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::held_commit_record::Held;
use crate::service_journal::{DONE_EXTENSION, Intent, OPEN_EXTENSION, batch_path, parse_intent};
use crate::transaction_git::GitContext;
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::COMMIT_OWN_STATE_CEILING;
use std::path::{Path, PathBuf};

use super::{landed_below_answered, landed_below_request};

/// The record one intent question names,
/// owned so that a job of the blocking pool can take it.
struct AnIntentLook {
    /// The directory of the batch's journal.
    journal: PathBuf,
    /// The batch whose intent is read.
    seq: u64,
}

/// The read one job of the pool does:
/// `intent_of` over the question's own record.
fn intent_here(work: AnIntentLook) -> Result<Option<Intent>, String> {
    let path: PathBuf = batch_path(work.journal.as_path(), work.seq, OPEN_EXTENSION);
    let Ok(text) = std::fs::read_to_string(path.as_path()) else {
        return Ok(None);
    };
    return match parse_intent(text.as_str()) {
        Some(intent) => Ok(Some(intent)),
        None => Err(format!("the record of the batch at `{}` cannot be read", path.display())),
    };
}

/// What one look at a batch's intent answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IntentLook = { kind: 'found'; intent: Intent } | { kind: 'absent' } | { kind: 'damaged'; why: string } | { kind: 'no answer' };
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the settlement face reads these fields (doc/planning/git-policy-cli-full-tokio.md)"))]
enum IntentLook {
    /// The record of the batch,
    /// read.
    Found(Box<Intent>),
    /// No record is there,
    /// or a record is there that this build cannot read.
    Absent,
    /// A record is there and cannot be read,
    /// for the reason given.
    Damaged(String),
    /// The look gave no answer in time.
    NoAnswer,
}

/// What:
///  The intent of a held batch,
/// as a job of the blocking pool awaited no longer than `COMMIT_OWN_STATE_CEILING`.
/// Why:
///   The record is a file of the journal,
/// which waits in the kernel on a mount that stopped answering,
/// and a task of the
///  runtime may not wait there.
/// A look that gave no answer is told apart from one that found no record:
/// a batch with no
///  intent is settled from the branch,
/// and a look nobody answered must decide nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intent = await intentWithin(ceiling, journal, seq); // 'found' | 'absent' | 'no answer'
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the settlement face waits for the finish awaited faces (doc/planning/git-policy-cli-full-tokio.md)"))]
async fn intent_awaiting(journal: &Path, seq: u64) -> IntentLook {
    return intent_within(COMMIT_OWN_STATE_CEILING, journal, seq).await;
}

/// `intent_awaiting` with the wait `ceiling` grants the look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await intentWithin(100, journal, seq)
/// ```
async fn intent_within(ceiling: std::time::Duration, journal: &Path, seq: u64) -> IntentLook {
    let work: AnIntentLook = AnIntentLook { journal: journal.to_path_buf(), seq };
    return match run_a_job_awaited_no_longer_than(ceiling, work, intent_here).await {
        TheAnswerOfAJobAwaited::Ran(Ok(Some(intent))) => IntentLook::Found(Box::new(intent)),
        TheAnswerOfAJobAwaited::Ran(Ok(None)) => IntentLook::Absent,
        TheAnswerOfAJobAwaited::Ran(Err(because)) => IntentLook::Damaged(because),
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => IntentLook::NoAnswer,
    };
}

/// The record one finished-batch question names,
/// owned so that a job of the blocking pool can take it.
struct AFinishedLook {
    /// The directory of the batch's journal.
    journal: PathBuf,
    /// The batch whose finished record is read.
    seq: u64,
}

/// The read one job of the pool does:
/// `finished_of` over the question's own record.
fn finished_here(work: AFinishedLook) -> Option<Intent> {
    let path: PathBuf = batch_path(work.journal.as_path(), work.seq, DONE_EXTENSION);
    let text: String = std::fs::read_to_string(path.as_path()).ok()?;
    return parse_intent(text.as_str());
}

/// What:
///  The record of the batch a held record belongs to once that batch ended,
/// as a job of the blocking pool awaited no longer
///  than `COMMIT_OWN_STATE_CEILING`,
///  and nothing when the look gave no answer.
/// Why:
///   The read waits on the disk the journal is on,
/// and a task of the runtime may not wait there;
/// a read that gave no answer
///  must not be taken for "the batch wrote no finished record",
/// because that is the answer that says the batch ended without
///  landing its commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = await finishedWithin(ceiling, journal, seq); // Intent | undefined
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the settlement face waits for the finish awaited faces (doc/planning/git-policy-cli-full-tokio.md)"))]
async fn finished_awaiting(journal: &Path, seq: u64) -> Option<Intent> {
    return finished_within(COMMIT_OWN_STATE_CEILING, journal, seq).await;
}

/// `finished_awaiting` with the wait `ceiling` grants the look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await finishedWithin(100, journal, seq)
/// ```
async fn finished_within(ceiling: std::time::Duration, journal: &Path, seq: u64) -> Option<Intent> {
    let work: AFinishedLook = AFinishedLook { journal: journal.to_path_buf(), seq };
    return match run_a_job_awaited_no_longer_than(ceiling, work, finished_here).await {
        TheAnswerOfAJobAwaited::Ran(found) => found,
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}

/// What:
///  `landed_below` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and this is the question a settlement asks after every read of the
///  branch.
/// The question and the reading of its answer are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landedBelowAwaiting(git, held, intent, current)
/// ```
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the settlement face waits for the finish awaited faces (doc/planning/git-policy-cli-full-tokio.md)"))]
async fn landed_below_awaiting(git: &GitContext, held: &Held, intent: &Intent, current: &str) -> bool {
    return landed_below_answered(run_git_awaiting(git, &landed_below_request(held, intent, current)).await, intent, current);
}

