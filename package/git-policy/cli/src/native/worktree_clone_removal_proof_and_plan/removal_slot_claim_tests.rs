//! What:
//!  The controls of the per-slot claim a whole removal holds:
//!  a second removal and an add of the same slot refused while one removal works,
//!  the claim held to the order's last step,
//!  given back on every way the order ends,
//!  taken over from a removal that was killed,
//!  and the release a person reads when it cannot rewrite its file.
//! Why:
//!  Two removals of one slot write the same binding,
//!  plan folder and registration,
//!  and this path took no claim at all before this round,
//!  so the controls assert the state a second command meets rather than a return value:
//!  the claim file's own record,
//!  the owner `live_owner_of` names,
//!  and the records a refused removal left whole
//!  (`SPEC.md`,
//!  "The per-slot claim";
//!  `doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals",
//!  steps 1 and 14).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const first = takeTheRemovalsClaim(commonDir, slot);
//! expect(() => complete(theCompletion({...}))).toThrow(TheSlotIsClaimed);
//! expect(liveOwnerOf(home, prefix)).toBeUndefined(); first.release();
//! ```

use crate::single_owner_claim::{ClaimOutcome, ClaimRecord, decode, live_owner_of, plant_claim_for_tests, take_claim, unused_pid_for_tests};
use crate::worktree_clone_binding_record::registration_home::{binding_name, claim_generation_of, claim_prefix, home_folder};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, ReadFailure};
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_completion::{RemovalCompletion, complete_the_removal};
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::{
    ADoneUnmount, ClearingDevice, EpochProbe, LivePool, ORIGIN_OWN_NAME, ScriptedClock, ScriptedHead, ScriptedHolders, ScriptedSettle, TheCompletion, a_clear_scan, a_fixture, a_record_stands, a_scan_with_a_holder,
    complete, the_budget, the_checks_pool,
};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{RemovalEntry, enter_the_removal};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{RemovalFixture, ScriptedPool};
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{RegistrationSettle, SettleAnswer, SettleRequest};
use crate::worktree_clone_removal_proof_and_plan::removal_slot_claim::{ClaimEnd, release_at_the_last_step, take_the_removals_claim};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_removal_deferred_origin_check::fixtures::Pool;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::cell::{Cell, RefCell};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::rc::Rc;

/// The slot every control removes,
/// which the fixture files its records under.
const THE_SLOT: &str = "fix-flaky-test";

/// The claim file one generation of the slot's claim is filed under.
fn the_claim_file(fixture: &RemovalFixture, generation: u64) -> PathBuf {
    let mut name: OsString = claim_prefix(THE_SLOT);
    name.push(format!(".{generation}"));
    return home_folder(fixture.common()).join(name);
}

/// Every claim generation the record folder holds for the slot,
/// ascending.
fn the_claim_generations(fixture: &RemovalFixture) -> Vec<u64> {
    let folder: PathBuf = home_folder(fixture.common());
    let mut found: Vec<u64> = std::fs::read_dir(folder.as_path())
        .expect("the record folder lists")
        .flatten()
        .filter_map(|entry| return claim_generation_of(entry.file_name().to_str()?, THE_SLOT))
        .collect();
    found.sort_unstable();
    return found;
}

/// The record the newest claim file holds,
/// read the way every claimant reads it.
fn the_newest_claim_record(fixture: &RemovalFixture) -> ClaimRecord {
    let newest: u64 = *the_claim_generations(fixture).last().expect("a claim file stands");
    let text: String = std::fs::read_to_string(the_claim_file(fixture, newest).as_path()).expect("the claim file reads");
    return decode(text.as_str()).expect("the file is a claim this command wrote");
}

/// The owner the slot's claim names while one command holds it,
/// by process id,
/// and nothing when the claim was released.
fn the_claim_owner(fixture: &RemovalFixture) -> Option<i64> {
    return live_owner_of(home_folder(fixture.common()).as_path(), claim_prefix(THE_SLOT).as_os_str())
        .expect("the claim folder answers")
        .map(|owner| return owner.pid);
}

/// What:
///  A Git settle that records the claim's owner at the moment it is asked,
///  which is the order's step 13.
/// Why:
///  The claim must be held from the first step to the last,
///  and only a seam this late can show the hold outlasted the destroys and the origin attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WatchingSettle { settle(request) { this.seen.push(liveOwnerOf(home, prefix)); return this.inner.settle(request) } }
/// ```
struct WatchingSettle<'a> {
    inner: &'a dyn RegistrationSettle,
    home: PathBuf,
    prefix: OsString,
    seen: Rc<RefCell<Vec<Option<i64>>>>,
}

impl RegistrationSettle for WatchingSettle<'_> {
    fn settle_the_registration(&self, request: &SettleRequest<'_>) -> Result<SettleAnswer, RemovalStop> {
        let owner: Option<i64> = live_owner_of(self.home.as_path(), self.prefix.as_os_str()).expect("the claim folder answers").map(|found| return found.pid);
        self.seen.borrow_mut().push(owner);
        return self.inner.settle_the_registration(request);
    }
}

/// A reader whose clone name and parent's children answer nothing,
/// which is the pool a removal meets after its head destroy.
struct AnAbsentClone;

impl DatasetReader for AnAbsentClone {
    fn identity(&self, _dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        return Err(ReadFailure { said: String::from("no such dataset"), errno: Some(libc::ENOENT) });
    }

    fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// The one whole-removal call over a fixture whose origin snapshot a record can name.
fn the_healthy_input<'a>(
    fixture: &'a RemovalFixture,
    pool: &'a LivePool,
    probe: &'a dyn DatasetReader,
    device: &'a dyn crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice,
    head: &'a dyn crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::DatasetDestroyDevice,
    holders: &'a dyn crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::FreshHolderScan,
    settle: &'a dyn RegistrationSettle,
) -> TheCompletion<'a> {
    return TheCompletion { fixture, pool, check: probe, probe, device, head, holders, settle, budget: the_budget() };
}

#[test]
fn a_second_removal_of_the_slot_is_refused_while_the_first_holds_the_claim() {
    let fixture: RemovalFixture = a_fixture("claim-second-removal");
    let mut first = take_the_removals_claim(fixture.common(), THE_SLOT).expect("the first removal takes the claim");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe { pool: check, hidden: Rc::clone(&hidden) };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(fixture.plan_folder(), fixture.recorded().clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let input: TheCompletion<'_> = the_healthy_input(&fixture, &pool, &probe, &device, &head, &holders, &settle);
    let stop: RemovalStop = complete(&input).expect_err("a held slot is refused rather than queued");
    match &stop {
        RemovalStop::TheSlotIsClaimed { slot, pid, path } => {
            assert_eq!(slot, THE_SLOT, "the refusal names the slot the first removal holds");
            assert_eq!(*pid, i64::from(std::process::id()), "the refusal names the owner's process");
            assert_eq!(path.file_name(), the_claim_file(&fixture, 1).file_name(), "the refusal names the claim file that holds the slot");
        }
        other => panic!("the claim refusal is the one that stops the second removal, not {other:?}"),
    }
    let message: String = stop.message();
    assert!(message.contains("Nothing was destroyed"), "{message}");
    assert!(message.contains("Ways forward:"), "{message}");
    assert!(device.sent.borrow().is_empty(), "the second removal sent no snapshot request");
    assert!(head.sent.borrow().is_empty(), "the second removal sent no head request");
    assert!(a_record_stands(&fixture, binding_name(THE_SLOT).as_str()), "the binding the first removal works from stands");
    assert_eq!(the_claim_owner(&fixture), Some(i64::from(std::process::id())), "the first removal's claim is untouched");
    assert_eq!(release_at_the_last_step(&mut first), ClaimEnd::Released);
}

#[test]
fn an_add_of_the_slot_meets_the_claim_a_removal_holds() {
    let fixture: RemovalFixture = RemovalFixture::healthy("claim-add-meets");
    let home: PathBuf = home_folder(fixture.common());
    let mut claim = take_the_removals_claim(fixture.common(), THE_SLOT).expect("the removal takes the claim");
    // The add route's own call, so an add that changed its claim's key or folder would fail this control.
    let refused = crate::worktree_clone_add_route::add_route_run::take_the_claim(home.as_path(), THE_SLOT).expect_err("an add must not take the claim a removal holds");
    assert!(refused.message.contains("another command holds the claim on"), "the add's own refusal is the one that stops it, and it says: {}", refused.message);
    assert!(refused.message.contains(format!("(process {})", std::process::id()).as_str()), "the add names the live owner's process: {}", refused.message);
    assert_eq!(release_at_the_last_step(&mut claim), ClaimEnd::Released);
    match take_claim(home.as_path(), claim_prefix(THE_SLOT).as_os_str()).expect("the claim folder answers") {
        ClaimOutcome::Held(_next) => assert_eq!(the_claim_generations(&fixture), vec![2], "the released claim is replaced by the next generation, which removes the older one"),
        ClaimOutcome::Busy(owner) => panic!("a released claim stops no add, and process {} still holds it", owner.pid),
    }
}

#[test]
fn a_healthy_removal_holds_the_claim_to_its_last_step_and_releases_it() {
    let fixture: RemovalFixture = a_fixture("claim-healthy");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe { pool: check, hidden: Rc::clone(&hidden) };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(fixture.plan_folder(), fixture.recorded().clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let seen: Rc<RefCell<Vec<Option<i64>>>> = Rc::new(RefCell::new(Vec::new()));
    let inner: ScriptedSettle = ScriptedSettle::succeeding();
    let settle: WatchingSettle<'_> = WatchingSettle { inner: &inner, home: home_folder(fixture.common()), prefix: claim_prefix(THE_SLOT), seen: Rc::clone(&seen) };
    let done = complete(&the_healthy_input(&fixture, &pool, &probe, &device, &head, &holders, &settle)).expect("a healthy removal finishes");
    assert_eq!(seen.borrow().clone(), vec![Some(i64::from(std::process::id()))], "the claim was held when the order reached its settle");
    assert_eq!(done.claim, ClaimEnd::Released, "the order's last step released the claim");
    assert_eq!(the_claim_owner(&fixture), None, "no live owner stands after the removal returns");
    assert_eq!(the_newest_claim_record(&fixture), ClaimRecord::Released, "the claim file says `released`");
    assert!(done.text().contains("the slot's per-slot claim is released"), "{}", done.text());
}

#[test]
fn a_removal_refused_at_a_holder_gives_the_claim_back() {
    let fixture: RemovalFixture = a_fixture("claim-holder-refused");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe { pool: check, hidden: Rc::clone(&hidden) };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(fixture.plan_folder(), fixture.recorded().clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_scan_with_a_holder());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let stop: RemovalStop = complete(&the_healthy_input(&fixture, &pool, &probe, &device, &head, &holders, &settle)).expect_err("a holder stops the removal");
    assert!(matches!(stop, RemovalStop::HoldersSeeTheClone { .. }), "the holder refusal is the one that stops, not {stop:?}");
    assert!(device.sent.borrow().is_empty(), "no snapshot request follows a holder");
    assert_eq!(the_claim_owner(&fixture), None, "the refusal gave the slot back");
    assert_eq!(the_newest_claim_record(&fixture), ClaimRecord::Released, "the claim file says `released`");
}

#[test]
fn a_removal_refused_at_the_settle_gives_the_claim_back_and_the_next_run_finishes() {
    let fixture: RemovalFixture = a_fixture("claim-settle-refused");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe { pool: check, hidden: Rc::clone(&hidden) };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(fixture.plan_folder(), fixture.recorded().clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let refusing: ScriptedSettle = ScriptedSettle::refusing(String::from("the registration refused"));
    let stop: RemovalStop = complete(&the_healthy_input(&fixture, &pool, &probe, &device, &head, &holders, &refusing)).expect_err("the settle refuses");
    assert!(matches!(stop, RemovalStop::TheSettleRefused { .. }), "the settle refusal is the one that stops, not {stop:?}");
    assert_eq!(the_claim_owner(&fixture), None, "the error gave the slot back");
    let succeeding: ScriptedSettle = ScriptedSettle::succeeding();
    let done = complete(&the_healthy_input(&fixture, &pool, &probe, &device, &head, &holders, &succeeding)).expect("the next run finishes the removal");
    assert_eq!(device.sent.borrow().len(), 2, "the first run destroyed the clone's snapshots and the origin snapshot, and the second sent nothing");
    assert_eq!(head.sent.borrow().len(), 1, "the head was destroyed once");
    assert_eq!(done.claim, ClaimEnd::Released, "the next run took the claim and released it too");
    assert_eq!(the_claim_generations(&fixture), vec![2], "the second run took the next generation, and a take removes the older one");
}

#[test]
fn a_claim_a_killed_removal_left_is_taken_over_and_released() {
    let fixture: RemovalFixture = a_fixture("claim-killed");
    let dead: i64 = unused_pid_for_tests();
    plant_claim_for_tests(home_folder(fixture.common()).as_path(), claim_prefix(THE_SLOT).as_os_str(), 1, dead, "1:linux:1");
    assert_eq!(the_claim_owner(&fixture), None, "a claim whose owner no longer runs is not live");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe { pool: check, hidden: Rc::clone(&hidden) };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(fixture.plan_folder(), fixture.recorded().clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let done = complete(&the_healthy_input(&fixture, &pool, &probe, &device, &head, &holders, &settle)).expect("a killed removal's claim stops nothing");
    assert_eq!(done.claim, ClaimEnd::Released);
    assert_eq!(the_claim_generations(&fixture), vec![2], "the new claim is the next generation and the killed one's file is gone");
    assert_eq!(the_newest_claim_record(&fixture), ClaimRecord::Released);
}

#[test]
fn a_claim_file_no_wrapper_command_wrote_stops_the_removal_and_is_named() {
    let fixture: RemovalFixture = a_fixture("claim-foreign-file");
    let foreign: PathBuf = the_claim_file(&fixture, 1);
    std::fs::write(foreign.as_path(), "not a claim this command wrote\n").expect("a file no wrapper command wrote");
    let stop: RemovalStop = take_the_removals_claim(fixture.common(), THE_SLOT).expect_err("a file that is not a claim stops the removal");
    match &stop {
        RemovalStop::TheClaimCannotBeTaken { slot, said } => {
            assert_eq!(slot, THE_SLOT);
            assert!(said.contains("not a claim file this command wrote"), "{said}");
        }
        other => panic!("the take refusal is the one that stops, not {other:?}"),
    }
    assert_eq!(std::fs::read_to_string(foreign.as_path()).expect("the file stands"), "not a claim this command wrote\n", "the removal changed nothing of the file it refused");
}

#[test]
fn the_claim_is_taken_before_the_first_read_so_a_refused_removal_leaves_it_released() {
    let fixture: RemovalFixture = a_fixture("claim-before-the-first-read");
    let clone_guid = fixture.recorded().clone_guid;
    std::fs::remove_file(home_folder(fixture.common()).join(binding_name(THE_SLOT)).as_path()).expect("the binding goes, as a slot whose add never wrote one");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe { pool: check, hidden: Rc::clone(&hidden) };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(fixture.plan_folder(), clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    // The completion is called directly: the fixture's own `complete` reads the binding to build the entry, which is the
    // very read this control removes.
    let entry: RemovalEntry<'_> = RemovalEntry { common_dir: fixture.common(), slot: THE_SLOT };
    let unmount: ADoneUnmount = ADoneUnmount { clone_guid };
    let input = RemovalCompletion { entry: &entry, reads: &pool, unmount: &unmount, clock: &ScriptedClock, check: &probe, probe: &probe, snapshots: &device, head: &head, holders: &holders, settle: &settle, budget: the_budget(), now: "2026-11-25T09:12:44-05:00" };
    let stop: RemovalStop = complete_the_removal(&input).expect_err("a slot with no binding refuses");
    assert!(matches!(stop, RemovalStop::NoBinding { .. }), "the binding's own refusal is the one that stops, not {stop:?}");
    assert_eq!(the_claim_generations(&fixture), vec![1], "the claim was taken at the very first step, before the binding was read");
    assert_eq!(the_newest_claim_record(&fixture), ClaimRecord::Released, "and the refusal gave it back");
}

#[test]
fn the_head_gone_removal_takes_the_claim_and_releases_it_at_its_last_step() {
    let fixture: RemovalFixture = RemovalFixture::healthy("claim-head-gone");
    let pool: ScriptedPool = fixture.healthy_pool();
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount { clone_guid: recorded.clone_guid };
    let entry: RemovalEntry<'_> = RemovalEntry { common_dir: fixture.common(), slot: THE_SLOT };
    let _written = enter_the_removal(&entry, &pool, &unmount, &ScriptedClock).expect("the entry writes the plan");
    let mut plan: DestructionPlan = read_the_plan(fixture.common(), recorded.clone_guid).expect("the plan stands");
    plan.head = HeadWord::Gone;
    PlanFolder::open_or_create(fixture.common()).expect("the plan folder opens").publish(&plan).expect("the plan is rewritten as a killed head destroy leaves it");
    let absent: AnAbsentClone = AnAbsentClone;
    let empty: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let device: ClearingDevice = ClearingDevice::over(&empty);
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let head: ScriptedHead = ScriptedHead::succeeding(fixture.plan_folder(), recorded.clone_guid, Rc::clone(&hidden));
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let done = complete(&TheCompletion { fixture: &fixture, pool: &pool, check: &absent, probe: &absent, device: &device, head: &head, holders: &holders, settle: &settle, budget: the_budget() })
        .expect("the head-gone removal finishes");
    assert_eq!(done.head, HeadWord::Gone);
    assert!(done.records.plan_removed, "the plan went, which is the order's step before the release");
    assert_eq!(done.claim, ClaimEnd::Released, "the head-gone path releases the claim at its last step too");
    assert_eq!(the_claim_owner(&fixture), None);
    assert!(the_claim_generations(&fixture).len() == 1, "the head-gone removal took one claim");
}

#[test]
fn a_release_that_cannot_rewrite_its_file_is_reported_and_leaves_the_owner_named() {
    let fixture: RemovalFixture = RemovalFixture::healthy("claim-release-refused");
    let home: PathBuf = home_folder(fixture.common());
    let mut claim = take_the_removals_claim(fixture.common(), THE_SLOT).expect("the claim is taken");
    std::fs::set_permissions(home.as_path(), std::fs::Permissions::from_mode(0o500)).expect("the folder refuses a new file");
    let end: ClaimEnd = release_at_the_last_step(&mut claim);
    std::fs::set_permissions(home.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the folder takes its mode back");
    match &end {
        ClaimEnd::RefusedToRelease { said } => assert!(said.contains("could not be released"), "{said}"),
        other => panic!("a release that cannot write its file is a refusal, not {other:?}"),
    }
    assert_eq!(the_claim_owner(&fixture), Some(i64::from(std::process::id())), "the claim names this process until it ends");
    assert_ne!(the_newest_claim_record(&fixture), ClaimRecord::Released, "the file was not rewritten as released");
}
