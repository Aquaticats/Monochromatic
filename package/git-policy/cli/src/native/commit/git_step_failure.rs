//! What:
//!  The last Git step of this process that failed,
//!  with the status it exited with,
//!  kept until a commit's timing record takes it.
//! Why:
//!  A failed step reaches the command as words on standard error,
//! and the words hold the paths the person named,
//!  so the record cannot keep them;
//! what it can keep is which program and subcommand failed and what status it gave,
//! which is what tells a machine failure from a refusal of the content.
//! The note is dropped as soon as another step starts,
//!  so a step that failed and did not stop the command is never read as the reason it stopped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! let last: { name: string; status?: number } | null = null;
//! export const noteFailedStep = (name, status) => { last = { name: plainName(name), status }; };
//! export const clearFailedStep = () => { last = null; };
//! export const drainFailedStep = () => { const taken = last; last = null; return taken; };
//! ```

/// The step name made safe to write into a record.
use crate::transaction_step_durations::plain_name;
/// A lock around the one note.
use std::sync::Mutex;

/// One Git step that failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailedStep {
    /// The program and the subcommand,
    /// for example `git add`.
    pub name: String,
    /// The status the step exited with;
    /// absent when it never exited by itself,
    /// which is a step that could not start or was ended at its ceiling.
    pub status: Option<i32>,
}

/// The step that failed since another step started,
/// or nothing.
static LAST: Mutex<Option<FailedStep>> = Mutex::new(None);

/// What:
///  Keeps the step that has just failed.
/// Why:
///  The Git runner calls it where a started step's non-zero exit becomes a failure,
/// and the scratch commit where its child exits with one,
/// so the note is the step itself and not a reading of its words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteFailedStep('git add', 128);
/// ```
pub fn note_failed_step(name: &str, status: Option<i32>) {
    // A poisoned lock means another thread panicked while holding it; the note is still one note.
    let mut last = LAST.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *last = Some(FailedStep { name: plain_name(name), status });
}

/// What:
///  Forgets the note,
///  because another step is starting.
/// Why:
///  A step that failed and let the command go on is not the reason a later stop happened,
/// and only the note of the step that was running last answers what stopped the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearFailedStep();
/// ```
pub fn clear_failed_step() {
    let mut last = LAST.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *last = None;
}

/// What:
///  Takes the note,
///  leaving none behind.
/// Why:
///  One command writes one timing record,
/// and a record must not read a note another record already stated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failed = drainFailedStep();
/// ```
pub fn drain_failed_step() -> Option<FailedStep> {
    let mut last = LAST.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    return std::mem::take(&mut *last);
}

/// The note is written,
/// cleared and taken as controls in the test module.
#[cfg(test)]
#[path = "git_step_failure_tests.rs"]
mod tests;
