//! Controls of the words a kind that answers with words is settled with:
//! what each verdict leaves behind in the queue.

use crate::service_claim_renewal_running::Renewal;
use crate::service_work::Verdict;
use crate::service_work_drain_pending_claims::drain_pending_awaiting;
use crate::service_work_handlers_replanning_and_acceptance::{MAX_TRIES, Replanning, WorkHandlers, work_directory};
use crate::test_support::{fixture, remove};
use crate::work_item::{DONE, FAILED, PENDING, WorkItem, enqueue, state_directory};
use crate::worktree_remove_hand_over::removal_item;
use std::cell::Cell;
use std::path::{Path, PathBuf};

/// A handler whose subvolume delete answers one verdict every time it is asked,
/// and counts the times it was asked.
struct OneVerdict {
    /// What the delete says,
    /// by the attempt's number counting from one.
    said: Box<dyn Fn(u32) -> Verdict>,
    /// How many times the delete was asked.
    asked: Cell<u32>,
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
        let attempt: u32 = self.asked.get() + 1;
        self.asked.set(attempt);
        return (self.said)(attempt);
    }
}

/// What:
///  One queued removal drained once by a handler that answers `said`,
/// in a fresh fixture,
/// answered with the fixture's root,
/// its work directory,
/// the record's id and how many times the handler was asked.
/// Why:
///   Every control below asks the same question of a different verdict,
/// so the queue is built,
/// drained and read in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const drained = (said) => { enqueue(removalItem(53)); drain(said); return { work, id, asked }; }
/// ```
fn drained_once(name: &str, said: impl Fn(u32) -> Verdict + 'static) -> (PathBuf, PathBuf, String, u32) {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let parent_path: PathBuf = root.join("parent");
    std::fs::create_dir(common.as_path()).expect("common");
    std::fs::create_dir(parent_path.as_path()).expect("parent");
    let item: WorkItem = removal_item(common.as_path(), parent_path.as_path(), 53);
    let work: PathBuf = work_directory(common.as_path());
    enqueue(work.as_path(), &item).expect("the record");
    let handlers = OneVerdict { said: Box::new(said), asked: Cell::new(0) };
    let mut replanning = Replanning { first: None, changes: Vec::new(), plans: None };
    crate::lazy_runtime::run_on_runtime(drain_pending_awaiting(common.as_path(), &handlers, std::time::Duration::from_secs(120), Renewal::Timed, &mut replanning))
        .expect("the pending drain runs");
    return (root, work, item.id(), handlers.asked.get());
}

/// Where the record `id` stands now.
fn settled_in(work: &Path, id: &str) -> &'static str {
    for state in [PENDING, DONE, FAILED] {
        if state_directory(work, state).join(id).is_file() {
            return state;
        }
    }
    return "no state";
}

/// What:
///  A deletion the handler answers with a retryable failure the first time and with a successful one afterwards is
///  deleted by the attempt that follows.
/// Why:
///  `Verdict::Retry` says the failure may pass,
/// so settling it where a failure the service gave up on goes ends the one attempt that would
///  have succeeded:
/// later rounds claim only `pending`,
/// and recovery reads `claimed` and `recovering`,
/// never `failed`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { work, asked } = drainedOnce((attempt) => attempt === 1 ? Retry('transient') : Done); assert(asked === 2); assert(state(work) === DONE);
/// ```
#[test]
fn a_retryable_deletion_is_tried_again_and_the_next_attempt_deletes_it() {
    let (root, work, id, asked) = drained_once("settling-retry-succeeds", |attempt: u32| {
        return if attempt == 1 { Verdict::Retry(String::from("a transient failure of the destroy")) } else { Verdict::Done };
    });
    assert_eq!(asked, 2, "the record is tried again after a retryable verdict");
    assert_eq!(settled_in(work.as_path(), id.as_str()), DONE, "the attempt that followed deleted the subvolume and the record is done");
    remove(root.as_path());
}

/// What:
///  A deletion the handler answers with a retryable failure every time is tried `MAX_TRIES` times and then given up on,
///  so nothing retries it for the life of the service.
/// Why:
///   A kernel that refuses the same call every time would keep a round busy forever;
/// the count of attempts in the record is what stops it,
///  and the failure the person reads names the directory that was left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { work, asked } = drainedOnce(() => Retry('a busy subvolume')); assert(asked === MAX_TRIES); assert(state(work) === FAILED);
/// ```
#[test]
fn a_deletion_that_always_asks_for_a_retry_is_tried_until_its_attempts_run_out() {
    let (root, work, id, asked) = drained_once("settling-retry-exhausted", |_attempt: u32| {
        return Verdict::Retry(String::from("the subvolume is still busy"));
    });
    assert_eq!(asked, MAX_TRIES, "the retries are bounded by the record's count of attempts");
    assert_eq!(settled_in(work.as_path(), id.as_str()), FAILED, "an item given up on is where a failure goes, with the words naming the directory left");
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
        let answered: Verdict = verdict.clone();
        let (root, work, id, asked) = drained_once("settling-other-verdicts", move |_attempt: u32| { return answered.clone(); });
        assert_eq!(asked, 1, "{verdict:?} is settled by the attempt that answered it");
        assert_eq!(settled_in(work.as_path(), id.as_str()), state, "{verdict:?} is settled in {state}");
        remove(root.as_path());
    }
}
