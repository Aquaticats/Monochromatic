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

/// Runs any awaited step to its end on a runtime of this thread,
/// which is the thread a control states its answer on.
fn run_any<Awaited: std::future::Future>(awaited: Awaited) -> Awaited::Output {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    return runtime.block_on(awaited);
}

/// A plan file of one branch of one Git directory,
/// as the worker that promised the try published it.
fn a_saved_plan() -> crate::push_retry_store::StoredPlan {
    return crate::push_retry_store::StoredPlan {
        git_dir: PathBuf::from("/srv/repo/.git"),
        item: crate::work_item::WorkItem {
            kind: crate::work_item::WorkKind::Push,
            repository: PathBuf::from("/srv/repo"),
            git_dir: PathBuf::from("/srv/repo/.git"),
            branch_ref: String::from("refs/heads/main"),
            oid: String::from("0123456789abcdef0123456789abcdef01234567"),
            parent: String::new(),
            created: String::from("2026-11-03T05:40:00.000Z"),
            tries: 0,
        },
        attempt: 2,
    };
}

/// What:
///  The pushes a start could not read are read again by the step the loop takes,
/// and the directories they name are looked at.
/// Why:
///  Finding 1 of the review of `bc3d54f9cd`:
/// a start whose read of the plan gave no answer began with an empty list and nothing read the journal again,
/// so a push an earlier service had promised to try was not tried for the life of that service,
/// although its plan stood in the journal.
/// The loop keeps a wake for the plan's directory while it is unread (`PushRetries::needs_restore`),
/// and this is the step that wake runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await save(dir, plan); const [retries, found] = await restoreThePlan(PushRetries.unread(first, dir)); expect(found).toEqual([gitDir]);
/// ```
#[test]
fn a_plan_a_start_could_not_read_is_read_again_and_tried() {
    let root: PathBuf = crate::test_support::fixture("a-plan-read-again");
    let saved: crate::push_retry_store::StoredPlan = a_saved_plan();
    crate::push_retry_store::save(root.as_path(), &saved).expect("the plan an earlier service promised");
    let unread: PushRetries = PushRetries::unread(Duration::from_secs(10), root.as_path());
    assert!(unread.needs_restore(), "a start whose read gave no answer has the plan to read");
    let (kept, found): (PushRetries, Vec<PathBuf>) = run_any(restore_the_plan(unread));
    assert_eq!(found, vec![saved.git_dir.clone()], "the repository the plan names is looked at");
    assert!(!kept.needs_restore(), "the plan was read, so the loop keeps no wake for reading it again");
    assert!(kept.next_wait(saved.git_dir.as_path(), Instant::now()).is_some(), "and the push an earlier service promised is tried");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A plan given no thread for its second read keeps the list it had,
/// and the loop's pause reads it again.
/// Why:
///   A refusal promises the read did not run,
/// so the list is exactly what it was and still needs its read;
/// a caller that took the refusal for a read would drop the pushes an earlier service left from its schedule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stateTheAnswer('notRun'); const [kept, found] = await restoreThePlan(unread); expect(kept.needsRestore()).toBe(true);
/// ```
#[test]
fn a_plan_given_no_thread_for_its_second_read_keeps_its_read_pending() {
    let root: PathBuf = crate::test_support::fixture("a-plan-given-no-thread");
    let unread: PushRetries = PushRetries::unread(Duration::from_secs(10), root.as_path());
    let _refused = state_the_answer(AStatesAnswer::NotRun);
    let (kept, found): (PushRetries, Vec<PathBuf>) = run_any(restore_the_plan(unread));
    assert!(found.is_empty(), "nothing was read, so no repository is looked at by this step");
    assert!(kept.needs_restore(), "the plan is still unread, so the loop reads it after its pause");
    assert_eq!(kept.restore_pause(), Duration::from_secs(10), "a refusal that promises nothing ran does not grow the pause");
    crate::test_support::remove(root.as_path());
}
