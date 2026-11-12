//! Controls for the storage steps of a drain:
//! a step whose own job outlasted its ceiling says so,
//! and never that it did not run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said = await aStorageStep(shortCeiling, 'the step', 'nothing was done', aJobHeldOpen); expect(said).toContain('gave no answer');
//! ```

use super::*;
use crate::parallel_test_support::PATIENCE;

/// How long the control gives a step's place and then its job,
/// short enough that the job cannot finish inside it while the control holds the job open.
const THE_CONTROLS_CEILING: Duration = Duration::from_millis(200);

/// What:
///  A storage step whose own job outlasted its ceiling is answered as a step that gave no answer,
///  never as a step that did not run.
/// Why:
///   The two answers decide what the drain may do next and they are not the same:
/// a step that did not run changed nothing,
/// so its records keep their lease for the next round,
/// while a step that outlasted its ceiling may still be finishing what it began,
/// and a caller told "nothing happened" could claim the same records beside it.
/// The job is held open by this control,
/// so the answer cannot come from the work ending inside the ceiling.
#[test]
fn a_step_whose_own_job_outlasted_its_ceiling_says_so() {
    let (started, has_started) = std::sync::mpsc::channel::<()>();
    let (let_go, waits) = std::sync::mpsc::channel::<()>();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime for the control");
    let answered = runtime.block_on(a_storage_step_awaited_no_longer_than(THE_CONTROLS_CEILING, "the step", "nothing was done", move || -> u8 {
        let _told = started.send(());
        let _held = waits.recv();
        return 7;
    }));
    assert!(
        has_started.recv_timeout(PATIENCE).is_ok(),
        "the step's job started, so its ceiling is the only thing that can end this step's wait"
    );
    let said: String = answered.expect_err("a job the control held open outlasted the ceiling, so the step gave no answer");
    assert!(said.contains("the step gave no answer"), "the step says it gave no answer: {said}");
    assert!(said.contains("nothing was done"), "the step says what it did not do: {said}");
    assert!(
        said.contains("may still be being written"),
        "the step says what it began may still be finishing, which is what decides that nothing may claim its records: {said}"
    );
    assert!(!said.contains("did not run"), "a job that started did not fail to run: {said}");
    // The job is let go, and the runtime's own drop waits for it.
    let _let_go = let_go.send(());
    drop(runtime);
}

/// The branch and commit every item of these controls names.
const BRANCH: &str = "refs/heads/main";
const OID: &str = "0123456789abcdef0123456789abcdef01234567";

/// A work item for one push of `git_dir`.
fn a_push(git_dir: &Path) -> crate::work_item::WorkItem {
    return crate::work_item::WorkItem {
        kind: crate::work_item::WorkKind::Push,
        repository: git_dir.parent().expect("the repository of a Git directory").to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(BRANCH),
        oid: String::from(OID),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
}

/// What:
///  A look that could not answer is its own answer,
///  and it asks for another round rather than for nothing.
/// Why:
///   Its answer is what wakes the service once another instance's lease runs out,
///  and nothing else wakes it when there is no push plan and no journal to read:
/// reading a look that gave no answer as "no claim is held" leaves a push claimed for good,
///  waiting for a command the person may never type.
/// The only place of the process's bound is held here,
/// which is the state a store that answers slowly leaves behind,
/// so the look cannot run at all.
#[test]
fn a_look_that_gave_no_answer_asks_for_another_round() {
    let root: PathBuf = crate::test_support::fixture("storage-claim-look-no-answer");
    let work_dir: PathBuf = root.join("work");
    let repository: PathBuf = root.join("repository");
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let lease: Duration = crate::service_work::CLAIM_LEASE;
    crate::work_item::enqueue(work_dir.as_path(), &a_push(git_dir.as_path())).expect("enqueue");
    let _claimed = crate::work_item::claim_next_of(work_dir.as_path(), &ClaimTime::read(), &[crate::work_item::WorkKind::Push]).expect("claim").expect("one claim");
    assert!(
        crate::work_item_recovery::next_claim_expiry(work_dir.as_path(), &ClaimTime::read(), lease).is_some(),
        "positive control: the claim another instance holds runs out its lease, so a wake is owed"
    );
    assert_eq!(ClaimLease::NoneHeld.the_wake_it_asks_for(), None, "positive control: no claim asks for no wake");
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(crate::parallel_test_support::ONE_PLACE);
    let (started, has_started) = std::sync::mpsc::channel::<()>();
    let (release, waiting) = std::sync::mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let answer = crate::lazy_runtime::run_on_runtime(crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
            let _told = started.send(());
            let _held = waiting.recv();
        }))
        .expect("the holder's runtime");
        answer.expect("the holder's job");
    });
    assert!(has_started.recv_timeout(PATIENCE).is_ok(), "the only place of the process's bound is held");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime for the control");
    let answered: ClaimLease = runtime.block_on(next_claim_expiry_within_awaiting(THE_CONTROLS_CEILING, work_dir.as_path(), &ClaimTime::read(), lease));
    let _let_go = release.send(());
    holder.join().expect("the holder");
    drop(runtime);
    assert_eq!(answered, ClaimLease::Unknown, "a look that did not run is neither a wait nor an answer that no claim is held");
    assert_eq!(
        answered.the_wake_it_asks_for(),
        Some(LOOK_AGAIN_AFTER_A_STEP_THAT_GAVE_NO_ANSWER),
        "the look gave no answer, so the round must be woken again rather than told that no claim is held"
    );
    crate::test_support::remove(root.as_path());
}
