//! What:
//!  The journal records a landing writes before its branch moves,
//! as faces a task of the runtime may await:
//! the batch's intent and the record of the index file it started from.
//! Why:
//!  Both are files in the batch's journal directory,
//! and their writes wait in the kernel on a disk that stopped answering;
//! a task that wrote them would hold a worker the runtime needs,
//! and no timer on that worker ends it.
//! The writes themselves are `service_journal.rs`'s,
//! one implementation each with the synchronous faces,
//! and the landing's deferral travels into the job and its syncs travel back
//! (`service_durable.rs`,
//! `WhatTheSyncsBecame`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeIntentAwaiting(dir, intent); await writeStartAwaiting(dir, seq, witness);
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::service_durable::WhatTheSyncsBecame;
use crate::service_journal::{Intent, write_intent_carrying, write_start};
use crate::service_land_start::IndexWitness;
use crate::wait_ceilings::JOURNAL_WRITE_CEILING;
use std::path::{Path, PathBuf};

/// What:
///  `write_intent` for a caller that is a task of the runtime:
/// the same write,
/// as a job of the blocking pool,
/// awaited no longer than `JOURNAL_WRITE_CEILING`.
/// Why:
///   A task may not do the write itself:
/// a disk that stopped answering would hold a worker the runtime needs,
/// and no timer on that worker ends it.
/// The landing's deferral travels into the job and the syncs the job made travel back,
/// so the intent is flushed where and when the synchronous face flushes it,
/// and the answer's figures are the same.
/// A job that gives no answer ends the landing it belongs to:
/// nothing lands on a batch whose intent may be missing,
/// and the next service decides the batch from the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeIntentAwaiting(dir, intent)
/// ```
///
/// # Errors
///
/// The error of the write,
/// of the flush or of the rename,
/// and the error of a job that did not run or gave no answer.
pub async fn write_intent_awaiting(dir: &Path, intent: &Intent) -> std::io::Result<()> {
    let job: TheIntentOfABatch = TheIntentOfABatch { dir: dir.to_path_buf(), intent: intent.clone(), deferring: crate::service_durable::syncs_are_deferred_here() };
    return match run_a_job_awaited_no_longer_than(JOURNAL_WRITE_CEILING, job, write_the_intent_of_a_batch).await {
        TheAnswerOfAJobAwaited::Ran((written, became)) => {
            became.keep();
            written
        }
        TheAnswerOfAJobAwaited::NotRun => Err(a_record_the_system_refused_a_thread_for("intent", intent.seq)),
        TheAnswerOfAJobAwaited::NoAnswer => Err(a_record_whose_write_gave_no_answer("intent", intent.seq)),
    };
}

/// The intent one job of the blocking pool writes,
/// with the landing's deferral as the task that asked for it found it.
pub(crate) struct TheIntentOfABatch {
    dir: PathBuf,
    intent: Intent,
    deferring: bool,
}

/// The intent of a batch,
/// written and flushed as one job of the blocking pool.
fn write_the_intent_of_a_batch(job: TheIntentOfABatch) -> (std::io::Result<()>, WhatTheSyncsBecame) {
    let mut became: WhatTheSyncsBecame = WhatTheSyncsBecame::default();
    let written: std::io::Result<()> = write_intent_carrying(job.dir.as_path(), &job.intent, job.deferring, &mut became);
    return (written, became);
}

/// What:
///  `write_start` for a caller that is a task of the runtime:
/// the same record,
/// as a job of the blocking pool,
/// awaited no longer than `JOURNAL_WRITE_CEILING`.
/// Why:
///   The landing writes this record while it holds the index lock and before its branch moves,
/// so a disk that stopped answering would hold that worker and the lock with it.
/// The record is not synced,
/// so the job carries no flush of the landing's;
/// a job that did not run or gave no answer leaves a batch no replay is made for,
/// which is the answer a record that could not be written already gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeStartAwaiting(dir, seq, witness)
/// ```
///
/// # Errors
///
/// The error of the write or of the rename,
/// and the error of a job that did not run or gave no answer.
pub async fn write_start_awaiting(dir: &Path, seq: u64, witness: &IndexWitness) -> std::io::Result<()> {
    let job: TheStartRecordOfABatch = TheStartRecordOfABatch { dir: dir.to_path_buf(), seq, witness: witness.clone() };
    return match run_a_job_awaited_no_longer_than(JOURNAL_WRITE_CEILING, job, write_the_start_record_of_a_batch).await {
        TheAnswerOfAJobAwaited::Ran(written) => written,
        TheAnswerOfAJobAwaited::NotRun => Err(a_record_the_system_refused_a_thread_for("start record", seq)),
        TheAnswerOfAJobAwaited::NoAnswer => Err(a_record_whose_write_gave_no_answer("start record", seq)),
    };
}

/// The record of the index file a batch started from,
/// as one job of the blocking pool writes it.
pub(crate) struct TheStartRecordOfABatch {
    dir: PathBuf,
    seq: u64,
    witness: IndexWitness,
}

/// The start record of a batch,
/// written as one job of the blocking pool.
fn write_the_start_record_of_a_batch(job: TheStartRecordOfABatch) -> std::io::Result<()> {
    return write_start(job.dir.as_path(), job.seq, &job.witness);
}

/// What:
///  The error of a journal record the system gave no thread to insert.
/// Why:
///   The answer is a failure the person reads,
/// not a wait:
/// the job will not run and changed nothing,
/// so the batch has no record of this step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(`the ${record} of batch ${seq} was not written: no thread could be started`)
/// ```
fn a_record_the_system_refused_a_thread_for(record: &str, seq: u64) -> std::io::Error {
    return std::io::Error::other(format!(
        "the {record} of batch {seq} was not written, because the system refused a thread for the step that writes it and took nothing of that step on. \
         Hint: check the process limit (`ulimit -u`) and the control group's `pids.max`, free some, and run the command again"
    ));
}

/// What:
///  The error of a journal record whose write gave no answer in time.
/// Why:
///   A step that started on another thread or outlasted `JOURNAL_WRITE_CEILING` may have written the record,
/// so nothing lands on the answer and the batch is decided from the branch the next time the service starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(`the ${record} of batch ${seq} may or may not be on disk`)
/// ```
fn a_record_whose_write_gave_no_answer(record: &str, seq: u64) -> std::io::Error {
    return std::io::Error::other(format!(
        "the {record} of batch {seq} was not written in time, so whether it is on disk is not known; that batch lands only if its branch moved, \
         and it did not, so the next command decides the batch from the branch. Hint: check the disk the Git directory is on, and run the command again"
    ));
}

/// Controls for the awaited journal writes stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_writes_awaited_tests.rs"]
mod tests;
