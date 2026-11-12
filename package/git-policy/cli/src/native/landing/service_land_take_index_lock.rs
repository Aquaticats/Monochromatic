//! What:
//!  Takes the index lock for a batch that is ready to land,
//! notes the wait in the timing and the log,
//! starts making the markers of the two batches after it,
//! and cleans up when the lock is refused.
//! Why:
//!  It is a self-contained step of a landing,
//! and a file of its own keeps `land_batch_body` short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = await takeIndexLock(...)
//! ```

use std::time::Duration;
use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than, run_blocking_job_within_the_budget};
use crate::service_land_git_steps_and_import_stream::{keep_unlanded, run};
use crate::service_land_git_steps_awaited::{keep_unlanded_awaiting, run_awaiting};
use crate::journald_native::Priority;
use crate::service_journal::OPEN_EXTENSION;
use crate::service_index_lock::{
    Announce, Announcer, IndexLock, LockFailure, TheLockWaitsMemory, WaitNotes, acquire_carrying, acquire_noting_wait_within, carry_the_lock_waits_memory,
    keep_the_lock_waits_memory,
};
use crate::service_timing::{LandTiming, wait_line};
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::transaction_git::{GitContext, NO_REFLOG_SETTING};
use crate::wait_ceilings::JOURNAL_WRITE_CEILING;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// What:
///  Takes the index lock for a batch,
/// noting how long it waited and who held it;
/// when the lock is refused the intent is forgotten,
///  the built commit is kept on a side ref and the scratch branch is deleted.
/// Why:
///  The lock is asked for after the objects and the intent are made,
/// so it is held for the branch move and the index build only;
///  a refusal leaves nothing that a restart of the service could land.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = await takeIndexLock(git, journal, lockCeiling, first, seq, announce, timing, newTip, scratch) // throws a failure code
/// ```
pub(crate) fn take_index_lock(
    git: &GitContext,
    journal: &Path,
    lock_ceiling: Duration,
    first: &CommitRequest,
    seq: u64,
    announce: &dyn Announcer,
    timing: &mut LandTiming,
    new: &str,
    scratch: &str,
) -> Result<IndexLock, FailureCode> {
    let repository: &Path = first.repository.as_path();
    let waiting_since: Instant = Instant::now();
    let mut notes: WaitNotes = WaitNotes::default();
    let taken = acquire_noting_wait_within(first.index.as_path(), seq, lock_ceiling, announce, &mut notes);
    let waited: std::time::Duration = waiting_since.elapsed();
    timing.lock_wait += waited;
    // A pass over the batch that found the lock free has nothing to add to what an earlier pass found held.
    if notes.looks > 0 || timing.attempts == 1 {
        timing.lock_holder.clone_from(&notes.holder);
        timing.lock_notes = notes.clone();
    }
    // The wait is written down when it ends, not with the answer, so a landing that fails later still leaves it.
    if let Some(line) = wait_line(seq, waited, &notes, taken.is_ok()) {
        // A lock that was refused fails the commit, so the line stands out from a wait that ended in the lock.
        let priority: Priority = if taken.is_ok() { Priority::Info } else { Priority::Error };
        crate::service_timing::log_entry(priority, line.as_str(), &[("CLI_GIT_BATCH", seq.to_string())]);
    }
    match taken {
        Ok(lock) => {
            // What:
            //  The markers of the two batches after this one are started now,
            // while this landing holds the lock and moves the branch.
            // Why:
            //  The making is seconds long on a loaded disk when its flush is the caller's,
            // and it used to start after the landing's answer,
            // so the next commit
            //  met it still running (`lock_stage_kind=pre`,
            // 9 to 13 s measured 2026-11-10).
            //  Starting them here spends the landing's own seconds on them and leaves the next commit a finished marker usually,
            //  and the one after it the spare.
            //  The batches after this one land on this index and are numbered one and two more, which is known now.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const held = await takeIndexLock(...); prestage.start(first.index, seq + 1); prestage.start(first.index, seq + 2);
            // ```
            crate::service_index_prestage::start(first.index.as_path(), seq.saturating_add(1));
            crate::service_index_prestage::start(first.index.as_path(), seq.saturating_add(2));
            return Ok(lock);
        }
        Err(failure) => {
            // Nothing moved: forget the intent, so a restart of the service cannot land the batch later (owner, 2026-11-03: "No,
            //  keep it for a retry"), keep the built commit on a side ref the message names, and forget the scratch branch.
            let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(journal, seq, OPEN_EXTENSION));
            if failure.code == FailureCode::IndexInUse {
                announce.announce(Announce::Waiting(keep_unlanded(git, repository, seq, new)));
            }
            let _deleted = run(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch], None);
            return Err(failure.code);
        }
    }
}

/// What:
///  `take_index_lock` for a caller that is a task of the runtime:
/// the same lock of the same batch,
/// with the wait a job of the blocking pool and every notice it makes told to the task that awaits it.
/// Why:
///   Git's own rule is that the index lock is held before the branch moves and given up when the new index is in place,
/// so the landing waits for a foreign holder for up to its own bound (`LANDING_LOCK_CEILING`,
/// `SETTLE_LOCK_CEILING`).
/// That wait reads the lock's directory and the process table and sleeps on an inotify watch,
/// which a worker the runtime polls may not do:
/// a disk that stopped answering would hold the worker and the clients queued behind it.
///  What the wait remembers between takes is the landing's (the markers made ahead,
/// the lock an earlier commit gave up
///  on,
/// the claims a reclaim could not read back),
/// so the task carries it into the job and keeps what the job answers.
/// The notices of the wait are the clients' while they wait,
/// so the job hands each of them to the task,
/// which tells the clients as it awaits:
/// an announcement that waited for the job's end would reach the person after the lock was already taken.
/// The bound is the one the synchronous face is given:
/// the job ends with the wait,
/// and a wait that reaches its ceiling answers `IndexInUse` with the holder named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = await takeIndexLockAwaiting(git, journal, lockCeiling, first, seq, announce, timing, newTip, scratch)
/// ```
///
/// # Errors
///
/// As `take_index_lock`,
/// and `Unavailable` when the system gave no thread for the wait.
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio-landing-batch-body-storage-steps-awaited.md)"))]
pub(crate) async fn take_index_lock_awaiting(
    git: &GitContext,
    journal: &Path,
    lock_ceiling: Duration,
    first: &CommitRequest,
    seq: u64,
    announce: &dyn Announcer,
    timing: &mut LandTiming,
    new: &str,
    scratch: &str,
) -> Result<IndexLock, FailureCode> {
    let repository: &Path = first.repository.as_path();
    let waiting_since: Instant = Instant::now();
    let (heard, mut hearing): (tokio::sync::mpsc::UnboundedSender<Announce>, tokio::sync::mpsc::UnboundedReceiver<Announce>) = tokio::sync::mpsc::unbounded_channel();
    let job: TheLockTake = TheLockTake { index: first.index.clone(), seq, ceiling: lock_ceiling, heard, memory: carry_the_lock_waits_memory() };
    let waiting = run_blocking_job_within_the_budget(move || { return take_the_lock(job); });
    tokio::pin!(waiting);
    let mut notices_heard: bool = true;
    let answer: Result<TheLockTakeAnswer, BlockingJobError> = loop {
        tokio::select! {
            // The notices are the clients' while they wait, so each goes on as the job makes it.
            heard_note = hearing.recv(), if notices_heard => match heard_note {
                Some(said) => announce.announce(said),
                // The job has ended or will never run, so it has nothing more to say.
                None => notices_heard = false,
            },
            answered = &mut waiting => break answered,
        }
    };
    let waited: Duration = waiting_since.elapsed();
    let TheLockTakeAnswer { taken, notes, memory } = match answer {
        Ok(answered) => answered,
        // A job that did not run changed nothing, and one that started and gave no answer may hold the lock still: what came
        //  back of the landing's memory is nothing, so the next take stages its own marker and waits for a holder in full.
        Err(refusal) => {
            crate::service_error!("batch {seq}: the index lock could not be waited for ({refusal}), so this batch lands nothing");
            return Err(FailureCode::Unavailable);
        }
    };
    keep_the_lock_waits_memory(memory);
    timing.lock_wait += waited;
    // A pass over the batch that found the lock free has nothing to add to what an earlier pass found held.
    if notes.looks > 0 || timing.attempts == 1 {
        timing.lock_holder.clone_from(&notes.holder);
        timing.lock_notes = notes.clone();
    }
    // The wait is written down when it ends, not with the answer, so a landing that fails later still leaves it.
    if let Some(line) = wait_line(seq, waited, &notes, taken.is_ok()) {
        // A lock that was refused fails the commit, so the line stands out from a wait that ended in the lock.
        let priority: Priority = if taken.is_ok() { Priority::Info } else { Priority::Error };
        crate::service_timing::log_entry(priority, line.as_str(), &[("CLI_GIT_BATCH", seq.to_string())]);
    }
    match taken {
        Ok(lock) => {
            // The markers of the two batches after this one are started now, while this landing holds the lock and moves the branch,
            //  for the reason `take_index_lock` writes beside these two calls.
            crate::service_index_prestage::start(first.index.as_path(), seq.saturating_add(1));
            crate::service_index_prestage::start(first.index.as_path(), seq.saturating_add(2));
            return Ok(lock);
        }
        Err(failure) => {
            // Nothing moved: forget the intent, keep the built commit on a side ref the message names, and forget the scratch branch,
            //  as the synchronous face does.
            forget_the_intent_of_a_batch(journal, seq).await;
            if failure.code == FailureCode::IndexInUse {
                announce.announce(Announce::Waiting(keep_unlanded_awaiting(git, repository, seq, new).await));
            }
            let _deleted = run_awaiting(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch], None).await;
            return Err(failure.code);
        }
    }
}

/// The lock one job of the blocking pool waits for,
/// with the landing's memory and the channel its notices go to.
struct TheLockTake {
    index: PathBuf,
    seq: u64,
    ceiling: Duration,
    heard: tokio::sync::mpsc::UnboundedSender<Announce>,
    memory: TheLockWaitsMemory,
}

/// What one awaited take answered.
struct TheLockTakeAnswer {
    taken: Result<IndexLock, LockFailure>,
    notes: WaitNotes,
    memory: TheLockWaitsMemory,
}

/// Takes the lock as one job of the blocking pool,
/// answering what the wait found and what the landing remembers.
fn take_the_lock(job: TheLockTake) -> TheLockTakeAnswer {
    let mut notes: WaitNotes = WaitNotes::default();
    let mut memory: TheLockWaitsMemory = job.memory;
    let telling: AnnouncingToTheAcquirer = AnnouncingToTheAcquirer { heard: job.heard };
    let taken: Result<IndexLock, LockFailure> = acquire_carrying(job.index.as_path(), job.seq, job.ceiling, &telling, &mut notes, &mut memory);
    return TheLockTakeAnswer { taken, notes, memory };
}

/// What:
///  Where the wait of a job of the blocking pool says what it waits for.
/// Why:
///   The wait is the job's and the clients are the task's,
/// so each notice crosses to the task that awaits the job
/// (`take_index_lock_awaiting`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AnnouncingToTheAcquirer implements Announcer { announce(note) { this.heard.send(note) } }
/// ```
struct AnnouncingToTheAcquirer {
    heard: tokio::sync::mpsc::UnboundedSender<Announce>,
}

impl Announcer for AnnouncingToTheAcquirer {
    fn announce(&self, note: Announce) {
        // A task that stopped listening gave up on the lock: the notice is then nobody's, and the wait goes on to its own bound.
        let _told = self.heard.send(note);
    }
}

/// What:
///  Forgets the intent of a batch whose lock was refused,
/// as a job of the blocking pool.
/// Why:
///   Nothing moved,
/// so a restart must not land the batch later;
/// the record is a file,
/// which a task may not remove.
/// A record that could not be removed is said:
/// the next start settles the batch from the branch,
/// which did not move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forgetTheIntentOfABatch(journal, seq)
/// ```
async fn forget_the_intent_of_a_batch(journal: &Path, seq: u64) {
    let job: (PathBuf, u64) = (journal.to_path_buf(), seq);
    match run_a_job_awaited_no_longer_than(JOURNAL_WRITE_CEILING, job, remove_the_intent_of_a_batch).await {
        TheAnswerOfAJobAwaited::Ran(()) => {}
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => {
            crate::service_warn!("batch {seq}: the intent of the batch a refused index lock ended could not be removed, so a later start settles it from the branch");
        }
    }
}

/// The intent of a batch whose lock was refused,
/// removed as one job of the blocking pool.
fn remove_the_intent_of_a_batch(job: (PathBuf, u64)) {
    let (journal, seq) = job;
    let _removed = std::fs::remove_file(crate::service_journal::batch_path(journal.as_path(), seq, OPEN_EXTENSION));
}

/// Controls for taking the index lock stay out of the release executable.
#[cfg(test)]
#[path = "service_land_take_index_lock_tests.rs"]
mod tests;

/// Controls for the awaited take stay out of the release executable.
#[cfg(test)]
#[path = "service_land_take_index_lock_awaiting_tests.rs"]
mod awaiting_tests;
