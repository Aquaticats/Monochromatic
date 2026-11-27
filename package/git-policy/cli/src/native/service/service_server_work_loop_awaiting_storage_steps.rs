//! What:
//!  The storage steps the after-commit loop of the service makes as jobs of the blocking pool:
//! writing and reading again the notes a restart looks at,
//! reading again the plan of the pushes a service owes,
//! and looking the notes up.
//! Why:
//!  A task of the runtime that read or wrote a file itself would hold a worker of the runtime
//! while a disk,
//! a network file system or a FUSE mount answers,
//! so each step is a job,
//! and the value it works on comes back to the loop (`service_server_work_loop_awaiting.rs`).
//! The watch and the plan each have one owner at a time,
//! so a job takes its value and hands it back,
//! and nothing locks it.
//!  The three answers of a job are told apart here:
//! it ran,
//! the system gave no thread for it,
//! or it gave no answer within its ceiling,
//! and only the last one is owed again after a longer pause.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [watch, found] = await retryAwaiting(watch); // each step gives its value back
//! ```

use crate::blocking_jobs_within_the_budget::{
    TheAnswerOfAJobAwaited,
    run_a_job_awaited_no_longer_than,
};
use crate::push_retry::PushRetries;
use crate::service_claim_watch::ClaimWatch;
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Write `git_dir` down as a repository to look at when this service starts again,
/// as a job of the process's blocking pool,
///  and give the watch back with the note in it.
/// Why:
///   The note is a file in the instance's journal,
/// so writing it on a worker would hold a thread the runtime needs while storage waits.
/// The watch has one
///  owner at a time,
/// so the job takes it and hands it back,
/// and nothing locks it.
/// A task of the runtime never writes the note itself:
///  a filesystem that stopped answering would hold a worker the runtime needs,
/// and a write that gives no answer is kept as a note the watch tries again after its pause,
/// so a note is not lost to a refusal.
/// A watch kept in memory only writes nothing,
/// so it is not a job at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = await rememberAwaiting(watch, gitDir)
/// ```
pub(crate) async fn remember_awaiting(
    watch: ClaimWatch,
    git_dir: &Path,
) -> ClaimWatch {
    if watch.key().is_none() {
        let mut kept: ClaimWatch = watch;
        kept.remember(git_dir);
        return kept;
    }
    let unchanged: ClaimWatch = watch.clone();
    return match run_a_job_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        (watch, git_dir.to_path_buf()),
        remember_in_a_job,
    )
    .await
    {
        TheAnswerOfAJobAwaited::Ran(back) => back,
        // The system gave no thread, so nothing was written and the watch is as it was: this directory is kept as one whose
        //  note may be missing, so `retry` writes it after the pause and the work goes on either way.
        TheAnswerOfAJobAwaited::NotRun => {
            crate::service_warn!(
                "{} could not be written down for a restart, because the system gave no thread for it; it is tried again after a pause",
                git_dir.display()
            );
            a_watch_that_tries_again(unchanged, git_dir)
        }
        // The job started on another thread or gave no answer within its ceiling, so whether the note was written is not
        //  known; writing it again at worst writes the same note twice.
        TheAnswerOfAJobAwaited::NoAnswer => {
            crate::service_warn!(
                "{} may not have been written down for a restart, because the note gave no answer; it is tried again after a pause",
                git_dir.display()
            );
            a_watch_that_tries_again(unchanged, git_dir)
        }
    };
}

/// What:
///  `watch` keeping `git_dir` as a directory whose note may be missing.
/// Why:
///  A write that did not run,
/// or whose answer cannot be read,
/// leaves the journal without the note this call asked for;
/// the watch tries it again after its pause,
/// and the work goes on either way,
/// because a failed note must not stop a backup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watch.aNoteThatMayBeMissing(gitDir)
/// ```
pub(crate) fn a_watch_that_tries_again(
    mut watch: ClaimWatch,
    git_dir: &Path,
) -> ClaimWatch {
    watch.a_note_that_may_be_missing(git_dir);
    return watch;
}

/// `remember_awaiting`'s work,
/// as the job of the pool takes it.
pub(crate) fn remember_in_a_job((mut watch, git_dir): (ClaimWatch, PathBuf)) -> ClaimWatch {
    watch.remember(git_dir.as_path());
    return watch;
}

/// What:
///  Try again the notes that could not be written or read,
/// as a job of the process's blocking pool,
/// and give the watch back with the directories this made known.
/// Why:
///   The steps are a write and a reading of the journal's files,
/// so they are storage like the write above,
/// and the watch is the job's and comes back.
/// A watch kept in memory only has nothing to
///  try again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [watch, found] = await retryAwaiting(watch)
/// ```
pub(crate) async fn retry_awaiting(watch: ClaimWatch) -> (ClaimWatch, Vec<PathBuf>) {
    if watch.key().is_none() {
        let mut kept: ClaimWatch = watch;
        let found: Vec<PathBuf> = kept.retry();
        return (kept, found);
    }
    let unchanged: ClaimWatch = watch.clone();
    return match run_a_job_awaited_no_longer_than(AFTER_COMMIT_STATE_CEILING, watch, retry_in_a_job)
        .await
    {
        TheAnswerOfAJobAwaited::Ran((back, found)) => (back, found),
        // Nothing was read or written, so the pause for this failure is not grown either: the next round of the schedule
        //  tries the same notes again.
        TheAnswerOfAJobAwaited::NotRun => {
            crate::service_warn!(
                "the notes this service keeps for a restart could not be tried again, because the system gave no thread for it"
            );
            (unchanged, Vec::new())
        }
        // Whether the notes were read or written is not known, and a watch without them would schedule nothing:
        //  the schedule keeps this pause, so they are tried again after it,
        //  and the job that gave no answer still holds a place of the blocking bound, so that pause grows.
        TheAnswerOfAJobAwaited::NoAnswer => {
            crate::service_warn!(
                "the notes this service keeps for a restart gave no answer, so they are tried again after a longer pause"
            );
            (notes_owed_again(unchanged), Vec::new())
        }
    };
}

/// The watch a retry that outlasted its ceiling came to nothing for,
/// owed again after a longer pause.
pub(crate) fn notes_owed_again(mut watch: ClaimWatch) -> ClaimWatch {
    watch.retry_came_to_nothing();
    return watch;
}

/// `retry_awaiting`'s work,
/// as the job of the pool takes it.
pub(crate) fn retry_in_a_job(mut watch: ClaimWatch) -> (ClaimWatch, Vec<PathBuf>) {
    let found: Vec<PathBuf> = watch.retry();
    return (watch, found);
}

/// What:
///  Read the plan's journal again,
/// as a job of the process's blocking pool,
/// and give the plan back with the Git directories that read made known.
/// Why:
///   The read is storage,
/// so a task of the runtime never makes it itself,
///  and the plan has one owner at a time,
/// so the job takes it and hands it back
///  (`PushRetries::restore`).
/// Either way the job gives no answer,
/// the plan it was handed comes back as it was and that is said:
/// the files stand in the journal,
/// so the loop's next wake reads them again.
///   The two answers are told apart:
/// a refusal proves that nothing ran and holds no place of the blocking bound,
/// so asking again at the shortest pause costs nothing,
/// while a read that outlasted its ceiling is still holding its place until the journal answers,
/// so that one is owed again after a longer pause and the attempts thin out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [plan, found] = await restoreThePlan(plan)
/// ```
pub(crate) async fn restore_the_plan(retries: PushRetries) -> (PushRetries, Vec<PathBuf>) {
    let unchanged: PushRetries = retries.clone();
    return match run_a_job_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        retries,
        restore_in_a_job,
    )
    .await
    {
        TheAnswerOfAJobAwaited::Ran((back, found)) => (back, found),
        TheAnswerOfAJobAwaited::NotRun => {
            crate::service_warn!(
                "the plan of the pushes waiting for a new try could not be read again, because the system gave no thread for it; it is read again after a pause"
            );
            (unchanged, Vec::new())
        }
        // The job may have read the journal before its answer was lost, and the plan it read is not known here: the files are
        //  still there, so the next read names the same pushes and loses none of them.
        TheAnswerOfAJobAwaited::NoAnswer => {
            crate::service_warn!(
                "the plan of the pushes waiting for a new try gave no answer to its second read, so it is read again after a longer pause"
            );
            (plan_owed_again(unchanged), Vec::new())
        }
    };
}

/// The plan a read that outlasted its ceiling came to nothing for,
/// owed again after a longer pause.
/// Why:
///   That read is still holding a place of the blocking bound,
/// so asking for another one at the shortest pause would fill the pool with reads of a journal that never answers.
pub(crate) fn plan_owed_again(mut retries: PushRetries) -> PushRetries {
    retries.restore_came_to_nothing();
    return retries;
}

/// `restore_the_plan`'s work,
/// as the job of the pool takes it.
pub(crate) fn restore_in_a_job(mut retries: PushRetries) -> (PushRetries, Vec<PathBuf>) {
    let found: Vec<PathBuf> = retries.restore();
    return (retries, found);
}

/// What:
///  Ask the store which repositories a record waits in,
/// as a job of the process's blocking pool,
/// and give the watch back
///  with them.
/// Why:
///   The look lists the notes and one work directory for each,
/// so it is storage like the notes themselves,
/// and a
///  filesystem that stopped answering would hold a worker the runtime needs if a task did it.
/// The watch has one owner at a
///  time,
/// so the job takes it and hands it back,
/// and nothing locks it.
/// A look that could not run says nothing:
/// it is
///  asked again at the next bound,
/// and a journal that stopped answering is said by the notes' own retry
///  (`retry_awaiting`),
/// which grows its pause instead of printing a line every few seconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [watch, waiting] = await lookAwaiting(watch)
/// ```
pub(crate) async fn look_awaiting(watch: ClaimWatch) -> (ClaimWatch, Vec<PathBuf>) {
    if watch.key().is_none() {
        let mut kept: ClaimWatch = watch;
        let waiting: Vec<PathBuf> = kept.look_again();
        return (kept, waiting);
    }
    let unchanged: ClaimWatch = watch.clone();
    return match run_a_job_awaited_no_longer_than(AFTER_COMMIT_STATE_CEILING, watch, look_in_a_job)
        .await
    {
        TheAnswerOfAJobAwaited::Ran((back, waiting)) => (back, waiting),
        // Nothing was read, so nothing is owed: the next bound asks again, and a store that keeps refusing is named by the
        //  notes' own retry.
        TheAnswerOfAJobAwaited::NotRun => (unchanged, Vec::new()),
        // The job may still hold a place of the blocking bound, so the store owes a read that thins its attempts out
        //  (`notes_owed_again`); the look's own pause grows with it here.
        TheAnswerOfAJobAwaited::NoAnswer => {
            let owed: ClaimWatch = notes_owed_again(unchanged);
            (owed, Vec::new())
        }
    };
}

/// `look_awaiting`'s work,
/// as the job of the pool takes it.
pub(crate) fn look_in_a_job(mut watch: ClaimWatch) -> (ClaimWatch, Vec<PathBuf>) {
    let waiting: Vec<PathBuf> = watch.look_again();
    return (watch, waiting);
}
