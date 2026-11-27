//! What:
//!  The record of where one wrapped `git worktree add` spent its time,
//! step by step,
//! appended to a size-capped file in the
//!  common Git directory beside the commit timing record.
//! Why:
//!  An add that took thirteen seconds could not be split afterwards without a `strace` that distorted it.
//! Each step is a
//!  stretch of the command (a walk of the source,
//! a Git run,
//! the snapshot call,
//! a durable write),
//! timed on the monotonic
//!  clock;
//! a step inside another is taken out of the outer one,
//! so the steps are disjoint and what none covers is reported
//!  as `otherMs`.
//! The record goes to the file only,
//! in one append when the command ends,
//! with no flush to disk,
//! and it
//!  never changes what the add does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! begin(commonDir);
//! const entered = beginStep('walkSource');
//! const value = walk();
//! endStep(entered);
//! finish(exitCode); // appends JSON.stringify(record) + '\n' to join(commonDir, 'monochromatic-git-policy-cli-worktree-add-timing.jsonl')
//! ```
//!
//! This file is the module's index:
//!  the clock that starts and ends a stretch of work,
//!  the work of other threads,
//!  and the one append of the record to its file.
//!  The accounting and the JSON line are in `add_timing_record.rs`,
//!  the recording of one thread and the steps it publishes are in `add_timing_steps.rs`,
//!  and both were split out because this module was over the line limit,
//!  with every name they defined re-exported here so no caller changes.

use crate::add_timing_steps::TAG;
use crate::add_timing_steps::{
    ACTIVE,
    NOTICE,
    WORK,
};
use crate::diagnostic_log::debug;
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use crate::log_lines_appended_without_a_lock::append_line;
use crate::quiet_ledger::drain;
use crate::wrapper_names_old_and_new::WORKTREE_ADD_TIMING_LOG;
use std::path::PathBuf;
use std::time::{
    Duration,
    Instant,
};

/// The accounting and the JSON line that is written for it.
pub use crate::add_timing_record::{
    ADD_TIMING_SCHEMA_VERSION,
    Recorder,
};
/// Whether a global Git option is followed by its value,
/// which the step name of a Git run reads.
pub(crate) use crate::add_timing_steps::option_takes_value;
/// The recording of one thread,
/// the steps it publishes,
/// the note of one fact,
/// and the name of one Git run.
pub use crate::add_timing_steps::{
    begin,
    begin_on_route,
    begin_step,
    end_step,
    git_step_label,
    note,
};

/// The file inside the common Git directory that collects the add timing records of every worktree of the repository.
pub const ADD_TIMING_FILE: &str = WORKTREE_ADD_TIMING_LOG.current;

/// What:
///  The clock reading that starts a stretch of work on any thread,
/// added to the name's sum when `end_work` is called.
/// Why:
///   The repair of package managers runs its tools on worker threads,
/// which the thread's own steps cannot see;
/// the sum over
///  threads says how much work a step held,
/// and the step that waited for it says how long the add waited.
/// The body runs in
///  place between `begin_work` and `end_work`,
/// so it reads the caller's variables without a closure to carry them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork();
/// const value = scan();
/// endWork('untrackedScan', began);
/// ```
pub fn begin_work() -> Instant {
    return Instant::now();
}

/// What:
///  Add the time since `began` to the work named `name`.
/// Why:
///   The second half of a stretch of work:
/// the sum per name says how much work a step held across threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endWork('untrackedScan', began);
/// ```
pub fn end_work(
    name: &str,
    began: Instant,
) {
    note_work(name, began.elapsed());
}

/// What:
///  Add a stretch of work the caller already measured to the work named `name`.
/// Why:
///   Work done once per entry of a walk is summed by the worker that walked them
/// and noted once,
/// so a run of twenty thousand entries costs one note per name and not twenty thousand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteWork('cloneFiles', spentAcrossTheSlice);
/// ```
pub fn note_work(
    name: &str,
    spent: Duration,
) {
    if let Ok(mut work) = WORK.lock()
        && let Some(list) = work.as_mut()
    {
        list.push((name.to_string(), spent));
    }
}

/// What:
///  End the recording and append its record to the repository's add timing file,
/// best effort.
/// Why:
///   The command's result never depends on the record:
/// a file that cannot be written is only a debug line.
/// The directory is
///  never created and nothing is flushed to disk,
/// so the record costs one `write`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finish(exitCode);
/// ```
pub fn finish(exit_code: i32) {
    // The add answered, so its notice ends here and nothing it says can outlive the answer.
    NOTICE.set(None);
    let taken: Option<(Instant, Recorder)> = ACTIVE.take();
    let drained: Option<Vec<(String, Duration)>> = match WORK.lock() {
        Ok(mut work) => work.take(),
        Err(_poisoned) => None,
    };
    let Some((began, mut recorder)) = taken else {
        return;
    };
    for (name, spent) in drained.unwrap_or_default() {
        recorder.add_work(name.as_str(), spent);
    }
    recorder.quiet = drain();
    // No step is published once the add answered, so a notice that arrives after the answer cannot name one.
    crate::add_standing_notice::the_add_ended();
    let target: PathBuf = recorder.common_dir().join(ADD_TIMING_FILE);
    let line: String = recorder.line(began.elapsed(), exit_code);
    if let Err(error) = append_line(target.as_path(), line.as_bytes(), &UNPINNED_LOG) {
        debug(
            TAG,
            &format!(
                "the add timing record could not be written to {}: {error}",
                target.display()
            ),
        );
    }
}

/// The accounting,
/// the line and the writer.
#[cfg(test)]
#[path = "add_timing_tests.rs"]
mod tests;
