//! What:
//!  The controls of the destruction step's one order:
//!  the check before every snapshot request,
//!  the batch and the plan's own record of each answer,
//!  the fresh enumeration,
//!  and the check again before the head destroy.
//! Why:
//!  The step's whole guarantee is where the checks sit:
//!  a marked origin refuses before the request it guards,
//!  a mark that appears between two chunks is caught by the next chunk's check,
//!  and the message names what the plan already recorded as lost
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "One check,
//! called at every snapshot request and before the head destroy" and
//!  "The refusals";
//!  `doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`).
//!  Every control reads the plan back from its file,
//!  because the plan is what a person and the next attempt see,
//!  and counts the device's requests,
//!  because "no request was sent" is only worth something from a seam that could have shown
//!  one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const underway = destroyTheRemoval({ entry, reads, unmount, clock, check, device, budget });
//! expect(device.sent).toHaveLength(1);
//! ```

//region The seams a control answers

use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, ReadFailure};
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{HeadWord, SnapshotOutcome};
use crate::worktree_clone_removal_proof_and_plan::removal_destruction::{DestructionUnderway, RemovalDestruction, destroy_the_removal};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{RemovalClock, RemovalEntry};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE, CLONE_CREATETXG, CLONE_GUID, ORIGIN_CREATETXG, ORIGIN_DATASET, ORIGIN_DATASET_GUID, ORIGIN_SNAPSHOT, ORIGIN_SNAPSHOT_GUID, OWN_SNAPSHOT, POOL, RemovalFixture, SANOID_SNAPSHOT,
    ScriptedPool, the_stats,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{RemovalUnmount, UnmountAnswer};
use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use crate::worktree_clone_snapshot_batch::snapshot_destroy_request::encoded_request_bytes;
use crate::worktree_removal_deferred_origin_check::fixtures::{Answer, Pool, origin_properties, pair_text};
use crate::worktree_removal_deferred_origin_check::origin_mark::{MARK_OFF, MARK_ON};
use crate::worktree_removal_deferred_origin_check::origin_refusals::{RefusalForm, WhatARefusalLeaves};
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, DeviceError, NvList};
use std::cell::{Cell, RefCell};

/// The run nonce every control's plan carries.
const RUN_NONCE: &str = "00112233445566778899aabbccddeeff";

/// What:
///  The clock a control answers,
///  so the plan's three process facts are the control's and not the test runner's.
/// Why:
///  The owner and the moment are facts about the writing process,
///  and a control that read its own would make the assertion about the runner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clock = { run: () => nonce, owner: () => '48213:linux:9481733', started: () => '2026-11-22T18:40:12-05:00' };
/// ```
struct ScriptedClock;

impl RemovalClock for ScriptedClock {
    fn run(&self) -> Result<RequestNonce, RemovalStop> {
        return Ok(RequestNonce::parse(RUN_NONCE).expect("the control's nonce is canonical"));
    }

    fn owner(&self) -> Result<String, RemovalStop> {
        return Ok(String::from("48213:linux:9481733"));
    }

    fn started(&self) -> String {
        return String::from("2026-11-22T18:40:12-05:00");
    }
}

/// The unmount answer a control gives:
/// the intent's own nonce and the clone's GUID,
/// so the step's order runs without a mount handler.
struct ADoneUnmount {
    clone_guid: DatasetGuid,
}

impl RemovalUnmount for ADoneUnmount {
    fn request_the_unmount(&self, intent: &MountIntent) -> Result<UnmountAnswer, RemovalStop> {
        return Ok(UnmountAnswer { dataset: self.clone_guid, operation: intent.operation, nonce: intent.nonce.clone(), outcome: ReceiptOutcome::Done, mount_id: None, failed_step: None, errno: None });
    }
}

/// What:
///  The device a control answers with:
///  the names of every request it received,
///  and the answer the control scripted.
/// Why:
///  A control's claim that a refusal sent no request is a claim about this seam,
///  so the seam keeps every request and lets the control answer each one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const device = { sent: [], answer: (names) => Result.ok(none) };
/// ```
struct ScriptedDevice {
    sent: RefCell<Vec<Vec<String>>>,
    /// What every request answers.
    answer: Box<dyn Fn() -> Result<Option<NvList>, DeviceError>>,
}

impl ScriptedDevice {
    /// A device that answers every request with success.
    fn succeeding() -> ScriptedDevice {
        return ScriptedDevice { sent: RefCell::new(Vec::new()), answer: Box::new(|| return Ok(None)) };
    }
}

impl SnapshotDestroyDevice for ScriptedDevice {
    fn destroy_snaps(&self, _pool: &str, names: &[&str]) -> Result<Option<NvList>, DeviceError> {
        self.sent.borrow_mut().push(names.iter().map(|name| return String::from(*name)).collect());
        return (self.answer)();
    }
}

/// What:
///  A reader that answers the first reads from one pool and every later read from another.
/// Why:
///  The mark has to appear between two chunks,
///  and the check reads the same three names each time,
///  so the seam that changes its answer is the reader itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChangingReader { first; later; after; reads = 0; identity(n) { return (this.reads++ < this.after ? this.first : this.later).identity(n); } }
/// ```
struct ChangingReader<'a> {
    first: &'a Pool,
    later: &'a Pool,
    /// How many reads the first pool answers.
    after: usize,
    reads: Cell<usize>,
}

impl DatasetReader for ChangingReader<'_> {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        let seen: usize = self.reads.get();
        self.reads.set(seen + 1);
        return if seen < self.after { self.first.identity(dataset) } else { self.later.identity(dataset) };
    }

    fn next_child(&self, parent: &str, cursor: u64) -> Result<Option<ListedChild>, String> {
        return self.first.next_child(parent, cursor);
    }
}

//endregion

//region The check's own pool

/// What:
///  The scripted pool the check reads,
///  built over the removal fixture's tuple.
/// Why:
///  The check compares the live names and GUIDs with the plan's,
///  so a pool scripted from another fixture's tuple would refuse every field before the one a
///  control changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = theChecksPool({ mark: MARK_OFF, holds: 0, clones: 1 });
/// ```
fn the_checks_pool(mark: u64, holds: u64, clones: u64) -> Pool {
    return Pool::new(vec![
        (CLONE, Answer::of(the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT), NvList::new())),
        (ORIGIN_DATASET, Answer::of(the_stats(ORIGIN_DATASET_GUID, false, 2, 1, ""), NvList::new())),
        (ORIGIN_SNAPSHOT, Answer::of(the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG, clones, ""), origin_properties(Some(mark), Some(holds)))),
    ]);
}

/// The pool whose origin reads unmarked and unheld.
fn an_unmarked_pool() -> Pool {
    return the_checks_pool(MARK_OFF, 0, 1);
}

/// The pool whose origin snapshot is marked with one clone and no hold,
/// which is the state a removal would reap.
fn a_marked_pool() -> Pool {
    return the_checks_pool(MARK_ON, 0, 1);
}

//endregion

//region Running the step over one fixture

/// What:
///  Run the destruction step over one fixture,
///  the pool a control scripted,
///  the reader the checks use and the device it gave.
/// Why:
///  Every control differs in one of the three,
///  so one call keeps the rest of the arrangement out of each control.
fn destroy(fixture: &RemovalFixture, pool: &ScriptedPool, check: &dyn DatasetReader, device: &ScriptedDevice, budget: RequestBudget) -> Result<DestructionUnderway, RemovalStop> {
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount { clone_guid: recorded.clone_guid };
    let common: std::path::PathBuf = fixture.common().to_path_buf();
    let at: RemovalEntry<'_> = RemovalEntry { common_dir: common.as_path(), slot: recorded.slot.as_str() };
    return destroy_the_removal(&RemovalDestruction { entry: &at, reads: pool, unmount: &unmount, clock: &ScriptedClock, check, device, budget });
}

/// The fixture's own snapshots,
/// in the order the batch sorts them.
fn the_own_snapshots() -> Vec<String> {
    let mut names: Vec<String> = vec![format!("{CLONE}@{OWN_SNAPSHOT}"), format!("{CLONE}@{SANOID_SNAPSHOT}")];
    names.sort();
    return names;
}

/// The budget one of the fixture's two snapshot names fills alone,
/// measured from the exact request it becomes.
fn one_name_budget() -> RequestBudget {
    let names: Vec<String> = the_own_snapshots();
    let bytes: usize = encoded_request_bytes(POOL, &names[..1]).expect("a name packs into a request");
    return RequestBudget::of_input_bytes(bytes);
}

/// The budget both of the fixture's snapshot names fill together,
/// measured from the exact request they become.
fn both_names_budget() -> RequestBudget {
    let names: Vec<String> = the_own_snapshots();
    let bytes: usize = encoded_request_bytes(POOL, names.as_slice()).expect("both names pack into one request");
    return RequestBudget::of_input_bytes(bytes);
}

//endregion

#[test]
fn a_healthy_removal_sends_one_request_and_records_every_answer() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-healthy");
    let pool: ScriptedPool = fixture.healthy_pool();
    let check: Pool = an_unmarked_pool();
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    let underway: DestructionUnderway = destroy(&fixture, &pool, &check, &device, both_names_budget()).expect("a healthy removal destroys its own snapshots");
    let sent: Vec<Vec<String>> = device.sent.borrow().clone();
    assert_eq!(sent.len(), 1, "the whole set fits one request");
    assert_eq!(sent[0], the_own_snapshots(), "both of the clone's own snapshots are submitted, in the plan's order");
    assert_eq!(underway.requests.len(), 1, "the kernel's answer is kept");
    for name in the_own_snapshots() {
        let state = underway.plan.current_state(name.as_str()).expect("the plan holds the name the request carried");
        assert_eq!(state.outcome, SnapshotOutcome::Destroyed, "every submitted name's answer is the state its history ends on");
        assert_eq!(state.attempt, 1, "the answer is the line above the first write's `planned` one");
    }
    let filed: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan reads back");
    assert_eq!(filed, underway.plan, "the file holds every answer this run received");
    assert_eq!(filed.head, HeadWord::Present, "no head request was recorded");
    assert_eq!(underway.remaining_snapshots, the_own_snapshots(), "the fresh enumeration answers what the scripted pool holds, and the guest arm is where a real batch empties it");
    assert_eq!(underway.lost.snapshots, the_own_snapshots(), "the loss names the clone's own snapshots");
}

#[test]
fn the_check_reads_the_clone_first_and_the_origin_snapshot_last_before_the_request() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-order");
    let pool: ScriptedPool = fixture.healthy_pool();
    let check: Pool = an_unmarked_pool();
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    let _ = destroy(&fixture, &pool, &check, &device, both_names_budget()).expect("the healthy removal goes on");
    let one_check: Vec<String> = vec![String::from(CLONE), String::from(ORIGIN_DATASET), String::from(ORIGIN_SNAPSHOT)];
    let both_checks: Vec<String> = one_check.iter().chain(one_check.iter()).cloned().collect();
    assert_eq!(check.reads(), both_checks, "each check reads the clone, the origin dataset and the origin snapshot, in that order, and nothing else");
    assert_eq!(device.sent.borrow().len(), 1, "the request follows the check");
}

#[test]
fn a_marked_origin_refuses_before_the_first_request_and_names_nothing_lost() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-marked-first");
    let pool: ScriptedPool = fixture.healthy_pool();
    let check: Pool = a_marked_pool();
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    let stop: RemovalStop = destroy(&fixture, &pool, &check, &device, both_names_budget()).expect_err("a marked origin refuses before the first request");
    match &stop {
        RemovalStop::OriginCheckRefused { refusal, history } => {
            assert_eq!(refusal.cause_name(), "origin-marked-for-deferred-destroy");
            assert_eq!(refusal.form(), RefusalForm::PendingAfterTheUnmount, "a check's refusal says the clone is unmounted and the removal stays pending");
            assert_eq!(refusal.what_it_leaves(), WhatARefusalLeaves::PendingWithItsRecord, "the intent and the plan are kept");
            assert_eq!(*history, RemovalHistory::NothingDestroyed);
        }
        other => panic!("the first check's refusal is step 15's own, not {other:?}"),
    }
    let message: String = stop.message();
    assert!(message.contains("This removal has not destroyed anything yet."), "the history sentence follows the plan's outcomes: {message}");
    assert!(message.contains("the clone stays unmounted"), "the message says the clone stays unmounted: {message}");
    assert!(message.contains("cancel-worktree-removal "), "the message names the way back: {message}");
    assert!(device.sent.borrow().is_empty(), "no request follows a refusal");
    let filed: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan stands");
    assert_eq!(filed.attempts.len(), 2, "the plan holds the first write's two lines and no answer");
    assert!(filed.attempts.iter().all(|line| return line.outcome == SnapshotOutcome::Planned), "no line claims a request this run never sent");
}

#[test]
fn a_mark_that_appears_after_the_first_chunk_refuses_before_the_second() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-marked-second-chunk");
    let pool: ScriptedPool = fixture.healthy_pool();
    let first: Pool = an_unmarked_pool();
    let later: Pool = a_marked_pool();
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    // The first check's three reads are answered unmarked, and every later read is marked.
    let counted: ChangingReader<'_> = ChangingReader { first: &first, later: &later, after: 3, reads: Cell::new(0) };
    let stop: RemovalStop = destroy(&fixture, &pool, &counted, &device, one_name_budget()).expect_err("the second chunk's check refuses");
    assert_eq!(device.sent.borrow().len(), 1, "the second chunk is never sent");
    let message: String = stop.message();
    assert!(message.contains("has already lost"), "the sentence names what the first chunk destroyed: {message}");
    assert!(message.contains("does not come back"), "the sentence says the loss is permanent: {message}");
    let filed: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan stands");
    let destroyed: Vec<String> = filed.named_snapshots().iter().filter(|snapshot| return filed.current_state(snapshot.name.as_str()).map(|state| return state.outcome == SnapshotOutcome::Destroyed).unwrap_or(false)).map(|snapshot| return snapshot.name.clone()).collect();
    assert_eq!(destroyed.len(), 1, "the plan records the one name the first chunk destroyed");
    assert!(message.contains(destroyed[0].as_str()), "the message names the same snapshot the plan does");
    let still_planned: Vec<String> = filed.named_snapshots().iter().filter(|snapshot| return filed.current_state(snapshot.name.as_str()).map(|state| return state.outcome == SnapshotOutcome::Planned).unwrap_or(false)).map(|snapshot| return snapshot.name.clone()).collect();
    assert_eq!(still_planned.len(), 1, "the unsent name is still `planned`, and the destroyed one is not");
    assert_eq!(filed.attempts.len(), 3, "two names' first lines and one answer");
}

#[test]
fn the_head_check_refuses_after_the_batch_and_the_plan_keeps_what_it_destroyed() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-marked-head");
    let pool: ScriptedPool = fixture.healthy_pool();
    let first: Pool = an_unmarked_pool();
    let later: Pool = a_marked_pool();
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    // The batch's one chunk is checked unmarked, and the head check meets the mark.
    let counted: ChangingReader<'_> = ChangingReader { first: &first, later: &later, after: 3, reads: Cell::new(0) };
    let stop: RemovalStop = destroy(&fixture, &pool, &counted, &device, both_names_budget()).expect_err("the head check refuses");
    assert_eq!(device.sent.borrow().len(), 1, "the batch's one request was sent and no other");
    let message: String = stop.message();
    assert!(message.contains("has already lost"), "the head check's message names the batch's loss: {message}");
    let filed: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan stands");
    for name in the_own_snapshots() {
        let state = filed.current_state(name.as_str()).expect("the plan holds every name the batch submitted");
        assert_eq!(state.outcome, SnapshotOutcome::Destroyed, "the plan keeps every name the batch destroyed");
    }
    assert_eq!(filed.head, HeadWord::Present, "no head request was recorded");
}

#[test]
fn an_unreadable_mark_at_a_check_refuses_rather_than_proceeding() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-mark-unreadable");
    let pool: ScriptedPool = fixture.healthy_pool();
    let mut properties: NvList = NvList::new();
    properties.add_nvlist("defer_destroy", pair_text("maybe"));
    let check: Pool = Pool::new(vec![
        (CLONE, Answer::of(the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT), NvList::new())),
        (ORIGIN_DATASET, Answer::of(the_stats(ORIGIN_DATASET_GUID, false, 2, 1, ""), NvList::new())),
        (ORIGIN_SNAPSHOT, Answer::of(the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG, 1, ""), properties)),
    ]);
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    let stop: RemovalStop = destroy(&fixture, &pool, &check, &device, both_names_budget()).expect_err("a mark that cannot be read is never an off mark");
    match &stop {
        RemovalStop::OriginCheckRefused { refusal, .. } => {
            assert_eq!(refusal.cause_name(), "mark-unreadable");
            assert_eq!(refusal.form(), RefusalForm::PendingAfterTheUnmount, "an unreadable mark at a check takes the check's form");
        }
        other => panic!("an unreadable mark refuses as step 15's cause, not {other:?}"),
    }
    assert!(device.sent.borrow().is_empty(), "no request follows a refusal");
}

#[test]
fn a_replacement_clone_refuses_before_any_request() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-replacement");
    let pool: ScriptedPool = fixture.healthy_pool();
    let check: Pool = Pool::new(vec![(CLONE, Answer::of(the_stats(CLONE_GUID + 1, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT), NvList::new()))]);
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    let stop: RemovalStop = destroy(&fixture, &pool, &check, &device, both_names_budget()).expect_err("a replacement clone is not the planned one");
    match &stop {
        RemovalStop::OriginCheckRefused { refusal, .. } => {
            assert_eq!(refusal.cause_name(), "clone-is-not-the-planned-one");
            assert_eq!(refusal.form(), RefusalForm::NothingCompared, "the message claims no request");
        }
        other => panic!("a replaced clone refuses as step 15's cause, not {other:?}"),
    }
    let message: String = stop.message();
    assert!(message.contains("it sent no request"), "form C says the check sent no request: {message}");
    assert!(message.contains("reports the same mismatch"), "form C says the cancel command reports the mismatch: {message}");
    assert!(device.sent.borrow().is_empty(), "no request follows a refusal");
}

#[test]
fn an_empty_set_sends_no_request_and_the_head_check_still_runs() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-empty-set");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.snapshots = Vec::new();
    let check: Pool = a_marked_pool();
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    let stop: RemovalStop = destroy(&fixture, &pool, &check, &device, both_names_budget()).expect_err("the head check refuses over an empty set");
    assert!(device.sent.borrow().is_empty(), "an empty set makes no request");
    let message: String = stop.message();
    assert!(message.contains("This removal has not destroyed anything yet."), "no snapshot was sent, so nothing was lost: {message}");
    match &stop {
        RemovalStop::OriginCheckRefused { refusal, .. } => assert_eq!(refusal.cause_name(), "origin-marked-for-deferred-destroy", "the head check is the point that refused"),
        other => panic!("the head check refused, not {other:?}"),
    }
    let filed: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan stands");
    assert!(filed.attempts.is_empty(), "the entry recorded no snapshot, so the plan holds no line");
}

#[test]
fn a_set_the_budget_cannot_hold_refuses_before_any_request() {
    let fixture: RemovalFixture = RemovalFixture::healthy("destruction-over-budget");
    let pool: ScriptedPool = fixture.healthy_pool();
    let check: Pool = an_unmarked_pool();
    let device: ScriptedDevice = ScriptedDevice::succeeding();
    let stop: RemovalStop = destroy(&fixture, &pool, &check, &device, RequestBudget::of_input_bytes(1)).expect_err("a set no request can carry is refused");
    match &stop {
        RemovalStop::TheSnapshotSetIsNotSendable { sent, .. } => assert_eq!(*sent, 0, "no request was sent"),
        other => panic!("an over-budget set stops the set and not the check, not {other:?}"),
    }
    assert!(device.sent.borrow().is_empty(), "no request was sent");
    assert!(stop.message().contains("No snapshot request was sent"), "the message says the request was not sent");
}
