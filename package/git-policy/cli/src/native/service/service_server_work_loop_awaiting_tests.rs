//! Controls for the notes the after-commit loop keeps for a restart,
//! when the job that writes one gives no answer:
//! the watch keeps trying it,
//! so no repository is forgotten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stateTheAnswer('noAnswer'); const watch = await rememberAwaiting(watch, gitDir); expect(watch.needsRetry()).toBe(true);
//! ```

use super::*;
use crate::blocking_jobs_within_the_budget::a_controls_answer::{AStatesAnswer, state_the_answer};
use std::os::unix::fs::PermissionsExt as _;

/// Runs one note's job to its end on a runtime of this thread,
/// which is the thread the control states its answer on.
fn run(noted: impl std::future::Future<Output = ClaimWatch>) -> ClaimWatch {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    return runtime.block_on(noted);
}

/// What:
///  A note whose write gave no answer is kept as one the watch tries again,
/// so the loop schedules that retry.
/// Why:
///  Finding 2 of the review of `b3d00b3be4`:
/// the write failed on the job's thread,
///  which kept the directory in the copy it
///  handed back,
///  and the caller kept the clone it made before the job.
/// That clone's list of notes it owes was empty,
/// so
///  `needs_retry` was false and no retry was scheduled:
/// the process could be killed with no restart note and the new
///  service would not know which directory held pending after-commit work.
#[test]
fn a_note_given_no_answer_is_tried_again_after_the_loops_pause() {
    // A journal directory this person may not write in, so the write itself fails: the review's own case is a note the started
    //  job could not write, which the caller's copy of the watch has no way to record.
    let note_directory: std::path::PathBuf = crate::test_support::fixture("a-note-given-no-answer");
    std::fs::set_permissions(note_directory.as_path(), std::fs::Permissions::from_mode(0o500)).expect("a journal nobody may write in");
    let watch: ClaimWatch = ClaimWatch::restored(note_directory.as_path(), None);
    assert!(!watch.needs_retry(), "a watch whose notes were all read owes nothing before the write");
    let git_dir: std::path::PathBuf = std::path::PathBuf::from("/srv/repo/.git");
    let _stated = state_the_answer(AStatesAnswer::NoAnswer);
    let kept: ClaimWatch = run(remember_awaiting(watch, git_dir.as_path()));
    let retried: bool = kept.needs_retry();
    std::fs::set_permissions(note_directory.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the journal is writable again");
    assert!(retried, "a note that gave no answer is tried again, so the loop keeps a retry for it in its schedule");
}
