//! Controls of the words a kind that answers with words is settled with:
//! what each verdict leaves behind in the queue.

use crate::service_claim_renewal_running::Renewal;
use crate::service_work::Verdict;
use crate::service_work_drain_pending_claims::drain_pending_awaiting;
use crate::service_work_handlers_replanning_and_acceptance::{Replanning, WorkHandlers, work_directory};
use crate::test_support::{fixture, remove};
use crate::work_item::{DONE, FAILED, PENDING, WorkItem, enqueue, state_directory};
use crate::worktree_remove_hand_over::removal_item;
use std::path::PathBuf;

/// A handler whose subvolume delete answers one verdict every time it is asked.
struct OneVerdict {
    /// What the delete says.
    said: Verdict,
}

impl WorkHandlers for OneVerdict {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn subvolume_delete(&self, _item: &WorkItem) -> Verdict {
        return self.said.clone();
    }
}

/// What:
///  One queued removal drained by a handler that answers `verdict`,
/// in a fresh fixture,
/// answered with where the record stands afterwards.
/// Why:
///   Every control below asks the same question of a different verdict,
/// so the queue is built,
/// drained and read in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stateAfter = (verdict) => { enqueue(removalItem(59)); drain(verdict); return states.filter(exists); }
/// ```
fn state_after_a_drain(name: &str, verdict: Verdict) -> (PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let parent_path: PathBuf = root.join("parent");
    std::fs::create_dir(common.as_path()).expect("common");
    std::fs::create_dir(parent_path.as_path()).expect("parent");
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), 53);
    let work: PathBuf = work_directory(common.as_path());
    enqueue(work.as_path(), &item).expect("the record");
    let handlers = OneVerdict { said: verdict };
    let mut replanning = Replanning { first: None, changes: Vec::new(), plans: None };
    crate::lazy_runtime::run_on_runtime(drain_pending_awaiting(
        common.as_path(),
        &handlers,
        std::time::Duration::from_secs(120),
        Renewal::Timed,
        &mut replanning,
    ))
    .expect("the pending drain runs");
    return (root, work, item.id());
}

/// What:
///  A deletion the handler answers with a retryable failure stays where a later round finds it.
/// Why:
///  `Verdict::Retry` says the failure may pass,
/// so settling it where a failure the service gave up on goes ends the one attempt that would
///  have succeeded:
/// later rounds claim only `pending`,
/// and recovery reads `claimed` and `recovering`, never `failed`.
/// A record given up on also keeps the worktree's files under the hidden name,
///  which no further attempt deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(removalItem); drain(() => Retry('a transient failure')); assert(exists(pending/<id>)); assert(!exists(failed/<id>));
/// ```
#[test]
fn a_retryable_deletion_stays_available_for_another_attempt() {
    let (root, work, id) = state_after_a_drain("settling-retry-stays-pending", Verdict::Retry(String::from("a transient failure of the destroy")));
    let pending: PathBuf = state_directory(work.as_path(), PENDING).join(id.as_str());
    let failed: PathBuf = state_directory(work.as_path(), FAILED).join(id.as_str());
    assert!(pending.is_file(), "a retryable deletion is queued for the next attempt: {}", pending.display());
    assert!(!failed.exists(), "a retryable deletion is not given up on: {}", failed.display());
    assert!(!state_directory(work.as_path(), DONE).join(id.as_str()).exists(), "nothing was deleted, so the record is not done");
    remove(root.as_path());
}

/// What:
///  The other verdicts keep the places they had:
/// a done one goes to `done`,
/// a failure the handler gave up on goes to `failed`,
/// and a skip is done.
/// Why:
///   Only the retry may change;
/// a settlement that sent every verdict to `pending` would pass the control above and fail the queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(stateAfter(Done) === DONE); assert(stateAfter(Failed('x')) === FAILED); assert(stateAfter(Skipped('x')) === DONE);
/// ```
#[test]
fn the_other_verdicts_keep_the_state_they_had() {
    let cases: [(Verdict, &str); 3] = [
        (Verdict::Done, DONE),
        (Verdict::Failed(String::from("this mount does not allow the destroy")), FAILED),
        (Verdict::Skipped(String::from("the record names no subvolume")), DONE),
    ];
    for (verdict, state) in cases {
        let (root, work, id) = state_after_a_drain("settling-other-verdicts", verdict.clone());
        assert!(
            state_directory(work.as_path(), state).join(id.as_str()).is_file(),
            "{verdict:?} is settled in {state}, not in {PENDING} or {}",
            [DONE, FAILED].join("/")
        );
        remove(root.as_path());
    }
}
