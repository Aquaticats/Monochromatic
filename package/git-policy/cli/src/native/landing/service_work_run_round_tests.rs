//! Controls for the plan of the pushes waiting for a new try,
//! when the job that writes it gives no answer:
//! the retry a worker promised is never dropped from the plan the loop reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stateTheAnswer('noAnswer'); const plan = await applyAwaiting(plan, [plannedAttempt]); expect(plan.nextWait(dir)).toBeDefined();
//! ```

use super::*;
use crate::blocking_jobs_within_the_budget::a_controls_answer::{AStatesAnswer, state_the_answer};
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

/// Runs one round's plan job to its end on a runtime of this thread,
/// which is the thread the control states its answer on.
fn run(applied: impl std::future::Future<Output = PushRetries>) -> PushRetries {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    return runtime.block_on(applied);
}

/// What:
///  A plan job that started on another thread,
/// or that gave no answer within its ceiling,
/// leaves the plan it was handed with the retry the worker promised.
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
/// so the
///  list keeps them here,
/// and no task of the runtime writes the journal.
#[test]
fn a_plan_job_given_no_answer_keeps_the_retry_its_worker_promised() {
    let directory: PathBuf = PathBuf::from("/srv/repo/.git");
    let plan: PushRetries = PushRetries::with_first_pause(Duration::from_secs(10));
    let changes: Vec<RetryChange> =
        vec![RetryChange::Plan { git_dir: directory.clone(), item: item("refs/heads/main"), attempt: 1, due: Instant::now() + Duration::from_secs(10) }];
    let _stated = state_the_answer(AStatesAnswer::NoAnswer);
    let kept: PushRetries = run(apply_awaiting(plan, changes));
    assert!(
        kept.next_wait(directory.as_path(), Instant::now()).is_some(),
        "a plan job that gave no answer kept the retry its worker promised, so that round's directory has a wake"
    );
    assert_eq!(
        kept.directories(),
        vec![directory.clone()],
        "the plan names the directory of the retry it kept, which is what the loop schedules a wait for"
    );
    // The refusal that promised the job will not run leaves the same list,
    // and writes no journal from this task either.
    let _refused = state_the_answer(AStatesAnswer::NotRun);
    let refused_plan: PushRetries = PushRetries::with_first_pause(Duration::from_secs(10));
    let refused_changes: Vec<RetryChange> =
        vec![RetryChange::Plan { git_dir: directory.clone(), item: item("refs/heads/main"), attempt: 1, due: Instant::now() + Duration::from_secs(10) }];
    let after_a_refusal: PushRetries = run(apply_awaiting(refused_plan, refused_changes));
    assert!(
        after_a_refusal.next_wait(directory.as_path(), Instant::now()).is_some(),
        "a plan job the system gave no thread for kept the retry its worker promised, so that round's directory has a wake"
    );
}

/// What:
///  A round whose plan job outlived its ceiling does not take a plan a later round wrote.
/// Why:
///   The job of a round's own plan writes is a clone of the plan,
/// and a caller that stops waiting for it keeps a clone of its own and goes on;
/// the job is never cancelled, so it may write long after the round ended.
/// A later wake drains the same branch, its push fails retryably, and its worker writes the newer plan.
/// A forget of the abandoned round names only the plans of its own round,
/// so it must not take a statement it never read:
/// otherwise a kill after the late removal leaves nothing for a restart to retry,
/// and the failure record's promise of a new try is false.
#[test]
fn a_forget_of_an_abandoned_round_keeps_the_plan_a_later_round_wrote() {
    use crate::push_retry_store::{StoredPlan, load, save};

    let root: PathBuf = crate::test_support::fixture("abandoned-round-keeps-a-later-plan");
    let store: PathBuf = root.join("push-retries");
    let here: PathBuf = root.join("repo/.git");
    let main: &str = "refs/heads/main";
    let now: Instant = Instant::now();
    // Round one: a push that failed for a reason which can pass, its worker's plan written before the person was told.
    let mut first: PushRetries = PushRetries::restored(Duration::from_secs(1), store.as_path());
    let pause: Duration = first.schedule(here.as_path(), &item(main), 1, now);
    let _taken: Vec<Assignment> = plan_the_round(std::slice::from_ref(&here), &mut first, now + pause);
    // The round's job holds this clone, and it is still running when the caller stops waiting for it.
    let abandoned: PushRetries = first.clone();
    assert_eq!(load(store.as_path()).len(), 1, "round one's plan is on disk");
    // A later wake drains the same branch again, its push fails retryably, and its worker writes that plan.
    let later: StoredPlan = StoredPlan { git_dir: here.clone(), item: item(main), attempt: 2 };
    save(store.as_path(), &later).expect("the later round's worker keeps its plan before the person is told");
    // The abandoned round's job resumes and applies the forget of the push it took.
    let mut resumed: PushRetries = abandoned;
    resumed.apply(vec![RetryChange::Forget { git_dir: here.clone(), branch_ref: String::from(main) }]);
    assert_eq!(load(store.as_path()), vec![later], "the plan the later round wrote is still on disk after the late forget");
    let restarted: PushRetries = PushRetries::restored(Duration::from_secs(1), store.as_path());
    assert_eq!(restarted.directories(), vec![here], "a kill and a restart still find the retry the later round promised");
    crate::test_support::remove(root.as_path());
}
