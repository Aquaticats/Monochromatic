//! Controls for the plan of the pushes waiting for a new try,
//! when a round applies what its workers asked:
//! the list the loop reads keeps the retry a worker promised,
//! and the round takes no plan's file away from a later round.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = applyWithoutAJob(plan, [plannedAttempt]); expect(kept.nextWait(dir)).toBeDefined();
//! ```

use super::*;
use crate::work_item::WorkItem;
use crate::work_item::WorkKind;

/// A push item of `branch`.
fn item(branch: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: PathBuf::from("/srv/repo"),
        git_dir: PathBuf::from("/srv/repo/.git"),
        branch_ref: branch.to_owned(),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
}

/// One change a worker of a round asked of the plan.
fn a_planned_attempt(directory: &Path) -> RetryChange {
    return RetryChange::Plan { git_dir: directory.to_path_buf(), item: item("refs/heads/main"), attempt: 1, due: Instant::now() + Duration::from_secs(10) };
}

/// What:
///  The retry a worker promised is in the list the loop reads after the round,
/// and the round writes no plan's file itself.
/// Why:
///  Finding 1 of the review of `b3d00b3be4`:
/// the round took the due push out of the plan,
///  the worker returned the
///  plan of the next try,
///  the job that writes it started anyway,
///  and the caller kept the clone it made before the job.
/// That clone
///  held neither the due push nor its new plan,
/// so the round named no wait for the directory and the loop's schedule dropped
///  its entry:
/// a push stayed unpushed with nothing to wake it.
/// The worker of each plan writes the file it promised,
/// so the list keeps them here,
/// and the round's own apply writes nothing at all,
/// which is what leaves a later round's plan alone (`a_forget_applied_after_a_later_plan_leaves_that_plan_on_disk`).
#[test]
fn a_round_keeps_the_retry_its_worker_promised_and_writes_no_file() {
    let directory: PathBuf = PathBuf::from("/srv/repo/.git");
    let mut plan: PushRetries = PushRetries::with_first_pause(Duration::from_secs(10));
    plan.apply(vec![a_planned_attempt(directory.as_path())]);
    assert!(
        plan.next_wait(directory.as_path(), Instant::now()).is_some(),
        "the round keeps the retry its worker promised, so that round's directory has a wake"
    );
    assert_eq!(plan.directories(), vec![directory], "the plan names the directory of the retry it kept, which is what the loop schedules a wait for");
}
