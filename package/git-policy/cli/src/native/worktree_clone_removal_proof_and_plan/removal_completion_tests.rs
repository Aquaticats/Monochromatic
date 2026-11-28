//! What:
//!  The whole-removal controls:
//!  the healthy removal,
//!  the origin record's moment,
//!  the holder refusals,
//!  a refused settle,
//!  a resumed removal that adopts its kept plan,
//!  and the head-gone path.
//! Why:
//!  The step's guarantee is the order of its writes and refusals,
//!  so each control asserts the state a person and the next attempt read:
//!  the plan's own file,
//!  the origin record's file,
//!  the requests the seams received,
//!  and whether the removal's records are still there
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = complete(theCompletion({ fixture, pool, check, probe, device, head, holders, settle }));
//! expect(device.sent).toHaveLength(1);
//! ```

//region The one fixture every control starts from

use crate::worktree_clone_binding_record::registration_home::{
    binding_name,
    claim_prefix,
    home_folder,
    origin_inventory_name,
    plans_folder,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::an_owed_record;
use crate::worktree_clone_origin_cleanup::origin_cleanup_publication::CleanupFolder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::{
    an_existing_cleanup_record,
    origin_cleanups_folder,
};
use crate::worktree_clone_removal_proof_and_plan::plan_names_and_states::the_names_not_destroyed;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanAttempt,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::{
    ClearingDevice,
    EpochProbe,
    LivePool,
    MarkedAfterTheBatch,
    ORIGIN_OWN_NAME,
    ScriptedHead,
    ScriptedHolders,
    ScriptedSettle,
    TheCompletion,
    a_clear_scan,
    a_fixture,
    a_marked_checks_pool,
    a_record_stands,
    a_scan_with_a_holder,
    an_intent_property,
    complete,
    the_budget,
    the_checks_pool,
    the_checks_pool_of,
    the_origin_full_name,
};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    ORIGIN_SNAPSHOT_GUID,
    OWN_CREATETXG,
    OWN_GUID,
    OWN_SNAPSHOT,
    RemovalFixture,
    SANOID_CREATETXG,
    SANOID_GUID,
    SANOID_SNAPSHOT,
};
use crate::worktree_clone_removal_proof_and_plan::removal_report::OriginCleanupDone;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::TheLiveSettle;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_removal_deferred_origin_check::fixtures::Pool;
use crate::worktree_removal_deferred_origin_check::origin_mark::MARK_OFF;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;
use std::cell::{
    Cell,
    RefCell,
};
use std::path::{
    Path,
    PathBuf,
};
use std::rc::Rc;

//endregion

/// The bytes a person's `git worktree lock --reason 'ZFS test underlay'` leaves,
/// which is the reason measurement item 6's guest gave and the newline Git's `write_file`
/// completes it with (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
/// "Where the lock lives,
/// and what Git reads it for").
const PERSON_LOCK_WORDS: &[u8] = b"ZFS test underlay\n";

#[test]
fn a_healthy_removal_destroys_the_snapshots_the_head_and_the_origin_and_frees_the_slot() {
    let fixture: RemovalFixture = a_fixture("completion-healthy");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let done = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect("a healthy removal finishes");
    assert_eq!(
        device.sent.borrow().len(),
        2,
        "one request for the clone's snapshots and one for the origin snapshot"
    );
    assert_eq!(
        head.sent.borrow().clone(),
        vec![String::from(CLONE)],
        "the head is destroyed once, by name"
    );
    assert_eq!(
        head.words.borrow().clone(),
        vec![String::from("head=requested")],
        "the plan said `requested` before the request"
    );
    assert_eq!(done.head, HeadWord::Gone);
    assert_eq!(
        done.destroyed.len(),
        2,
        "both of the clone's own snapshots are proved gone"
    );
    assert!(done.remaining.is_empty());
    match &done.origin {
        OriginCleanupDone::Finished {
            snapshot,
            ..
        } => assert_eq!(
            snapshot.as_deref(),
            Some(the_origin_full_name().as_str()),
            "the origin snapshot is the one the record named"
        ),
        other => panic!("the origin cleanup finishes, not {other:?}"),
    }
    assert!(
        done.settle.removed_the_registration && done.records.plan_removed,
        "the settle and the records finished"
    );
    assert!(
        !a_record_stands(&fixture, binding_name("fix-flaky-test").as_str()),
        "the binding and its inventories are gone"
    );
    assert!(!a_record_stands(
        &fixture,
        origin_inventory_name("fix-flaky-test").as_str()
    ));
    assert!(
        std::fs::symlink_metadata(plans_folder(fixture.common()).as_path()).is_err(),
        "the plan and its folder are gone"
    );
    assert!(
        an_existing_cleanup_record(fixture.common(), ORIGIN_OWN_NAME)
            .expect("the folder answers")
            .is_none(),
        "the cleanup record is gone"
    );
    let text: String = done.text();
    assert!(text.contains("is destroyed"), "{text}");
    assert!(
        text.contains("which frees the slot for the next add"),
        "{text}"
    );
}

#[test]
fn the_origin_record_is_published_before_the_first_snapshot_request() {
    let fixture: RemovalFixture = a_fixture("completion-origin-first");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let seen: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let watcher: WatchingDevice<'_> = WatchingDevice {
        inner: &device,
        folder: origin_cleanups_folder(fixture.common()),
        seen: Rc::clone(&seen),
    };
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let _ = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &watcher,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect("the removal finishes");
    let answers: Vec<bool> = seen.borrow().clone();
    assert!(!answers.is_empty(), "the watch saw the requests");
    assert!(
        answers[0],
        "the origin cleanup record stood before the first request"
    );
}

#[test]
fn a_holder_seen_before_the_first_request_stops_the_removal_and_sends_nothing() {
    let fixture: RemovalFixture = a_fixture("completion-holder-first");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_scan_with_a_holder());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let stop: RemovalStop = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect_err("a holder blocks destruction");
    match &stop {
        RemovalStop::HoldersSeeTheClone {
            holders: named,
            ..
        } => assert_eq!(named.len(), 1, "the scan named the one holder"),
        other => panic!("the holder refusal is the one that stops, not {other:?}"),
    }
    assert!(
        device.sent.borrow().is_empty(),
        "no snapshot request follows a holder"
    );
    assert!(
        head.sent.borrow().is_empty(),
        "no head request follows a holder"
    );
    let plan: DestructionPlan =
        read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan stands");
    assert!(
        plan.named_snapshots().iter().all(|snapshot| return plan
            .current_state(snapshot.name.as_str())
            .map(|held| return held.outcome == SnapshotOutcome::Intact)
            .unwrap_or(false)),
        "every name is settled as `intact` by the run's own fresh read, and no line claims a request"
    );
    assert!(
        locate(&fixture, binding_name("fix-flaky-test").as_str()),
        "the binding stands"
    );
}

/// What:
///  A mark set on the origin between the batch's request and the head destroy refuses the
///  head and sends no request of its own.
/// Why:
///  `SPEC.md` promises the deferred-origin check again before the head,
///  after a fresh enumeration of the clone,
///  and the mark is the one field only that second check reads:
///  the batch's check met an unmarked origin and went ahead,
///  so a removal that kept that answer and destroyed the head would reap a snapshot a mark
///  had just been set on.
///  The control that marks the origin before the head in `removal_destruction_tests` drives the
///  destruction step,
///  not this one,
///  so this is the only control that holds the second check's place in the whole removal.
#[test]
fn a_mark_set_after_the_batch_refuses_before_the_head() {
    let fixture: RemovalFixture = a_fixture("completion-mark-before-the-head");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let unmarked: Pool = the_checks_pool();
    let marked: Pool = a_marked_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: the_checks_pool(),
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    // The batch's chunk is checked unmarked, and the head's check meets the mark.
    let check: MarkedAfterTheBatch<'_> = MarkedAfterTheBatch {
        before: &unmarked,
        after: &marked,
        device: &device,
    };
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let stop: RemovalStop = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &check,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect_err("a mark set after the batch stops the head destroy");
    match &stop {
        RemovalStop::OriginCheckRefused {
            refusal,
            ..
        } => {
            assert_eq!(
                refusal.cause_name(),
                "origin-marked-for-deferred-destroy",
                "the refusal is the marked origin's own cause"
            );
        }
        other => panic!("a marked origin refuses as the check's cause, not {other:?}"),
    }
    assert_eq!(
        device.sent.borrow().len(),
        1,
        "the batch's one request was sent and no other"
    );
    assert!(
        head.sent.borrow().is_empty(),
        "the head request is not sent"
    );
    let plan: DestructionPlan =
        read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan stands");
    assert_eq!(plan.head, HeadWord::Present, "no head request was recorded");
    assert!(
        the_names_not_destroyed(&plan).is_empty(),
        "the batch's names are destroyed"
    );
    assert!(
        locate(&fixture, binding_name("fix-flaky-test").as_str()),
        "the binding stands"
    );
}

#[test]
fn a_holder_seen_only_before_the_head_stops_after_the_batch() {
    let fixture: RemovalFixture = a_fixture("completion-holder-head");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    // The chunk's check is clear and the head's check meets a holder.
    let holders: ScriptedHolders =
        ScriptedHolders::in_order(vec![a_clear_scan(), a_scan_with_a_holder()]);
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let stop: RemovalStop = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect_err("a holder before the head stops the head destroy");
    assert!(
        matches!(stop, RemovalStop::HoldersSeeTheClone { .. }),
        "the head's own refusal"
    );
    assert_eq!(device.sent.borrow().len(), 1, "the batch was sent");
    assert!(
        head.sent.borrow().is_empty(),
        "the head request is not sent"
    );
    let plan: DestructionPlan =
        read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan stands");
    assert_eq!(plan.head, HeadWord::Present, "no head request was recorded");
    assert!(
        the_names_not_destroyed(&plan).is_empty(),
        "the batch's names are destroyed"
    );
}

#[test]
fn a_refused_head_keeps_the_clone_and_the_word_refused() {
    let fixture: RemovalFixture = a_fixture("completion-head-refused");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::refusing(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let stop: RemovalStop = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect_err("a clone that still answers blocks the settle");
    assert!(
        matches!(stop, RemovalStop::TheHeadDestroyWasRefused { .. }),
        "a head destroy the pool still answers is a refusal of its own, before any settle is asked"
    );
    let plan: DestructionPlan =
        read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan stands");
    assert_eq!(
        plan.head,
        HeadWord::Refused,
        "the head's word is the refusal"
    );
    assert!(
        a_record_stands(&fixture, binding_name("fix-flaky-test").as_str()),
        "the binding still stands"
    );
}

#[test]
fn a_refused_settle_keeps_the_plan_and_its_binding_and_the_next_run_finishes_without_a_request() {
    let fixture: RemovalFixture = a_fixture("completion-resume");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let refusing: ScriptedSettle =
        ScriptedSettle::refusing(String::from("the registration refused"));
    let first: RemovalStop = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &refusing,
        cancelling: None,
        budget: the_budget(),
    })
    .expect_err("the settle refuses");
    assert!(matches!(first, RemovalStop::TheSettleRefused { .. }));
    let kept: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid)
        .expect("the plan stands after the refusal");
    assert_eq!(kept.head, HeadWord::Gone, "the head's own proof is kept");
    assert!(
        a_record_stands(&fixture, binding_name("fix-flaky-test").as_str()),
        "the binding stands for the next run"
    );
    let succeeding: ScriptedSettle = ScriptedSettle::succeeding();
    let done = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &succeeding,
        cancelling: None,
        budget: the_budget(),
    })
    .expect("the second run finishes the removal");
    assert_eq!(
        device.sent.borrow().len(),
        2,
        "the first run destroyed the clone's snapshots and the origin snapshot, and the second sent nothing"
    );
    assert_eq!(head.sent.borrow().len(), 1, "the head was destroyed once");
    assert!(done.records.plan_removed, "the second run removed the plan");
}

#[test]
fn an_incomplete_namespace_read_is_carried_into_the_report() {
    let fixture: RemovalFixture = a_fixture("completion-incomplete-read");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::a_scan_with_an_incomplete_read());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let done = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect("an incomplete read does not refuse by itself");
    assert!(
        done.holder_reads_unfinished >= 1,
        "the read the scan could not finish is reported, never read as a clear namespace"
    );
    assert!(
        done.text()
            .contains("namespace read(s) of the fresh scans could not be completed"),
        "{}",
        done.text()
    );
}

/// What:
///  A kept plan is resumed:
///  the name it records as destroyed is never requested again,
///  the loss it records is carried into the report,
///  and the removal's own records end the plan.
/// Why:
///  This is the resume the accepted records ask for:
///  the next removal of that clone reads the record,
///  proves it,
///  carries the loss forward and ends it
///  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
///  "How long the plan a cancellation keeps stays";
///  `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "Which resume path a removal takes").
///  The plan's head stands and its clone answers,
///  which is the head-present path,
///  and the clone's own intent is the `unmount` request a person's cancellation
///  left.
#[test]
fn a_kept_plan_is_resumed_without_a_request_for_a_name_it_records_as_destroyed() {
    let fixture: RemovalFixture = a_fixture("completion-resume-kept");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    // The earlier run destroyed the clone's own snapshot, so the pool no longer lists it.
    for listed in pool.lists.borrow_mut().iter_mut() {
        if listed.0 == CLONE {
            listed.1.retain(|object| return object.name != OWN_SNAPSHOT);
        }
    }
    let plan: PathBuf = fixture.plant_a_plan(
        HeadWord::Present,
        vec![
            fixture.a_line_of_the_plan(
                SANOID_SNAPSHOT,
                SANOID_GUID,
                SANOID_CREATETXG,
                0,
                SnapshotOutcome::Planned,
            ),
            fixture.a_line_of_the_plan(
                OWN_SNAPSHOT,
                OWN_GUID,
                OWN_CREATETXG,
                0,
                SnapshotOutcome::Destroyed,
            ),
        ],
    );
    let check: Pool = the_checks_pool_of(MARK_OFF, an_intent_property("unmount"));
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let done = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect("the kept plan is resumed and finished");
    assert_eq!(
        device.sent.borrow().clone(),
        vec![
            vec![format!("{CLONE}@{SANOID_SNAPSHOT}")],
            vec![the_origin_full_name()],
        ],
        "the name the plan records as destroyed is never requested again"
    );
    assert!(
        done.destroyed.contains(&format!("{CLONE}@{OWN_SNAPSHOT}")),
        "the loss the plan recorded is carried into the report: {:?}",
        done.destroyed
    );
    assert!(
        done.destroyed
            .contains(&format!("{CLONE}@{SANOID_SNAPSHOT}")),
        "the name this run destroyed is reported beside it: {:?}",
        done.destroyed
    );
    assert!(
        done.records.plan_removed,
        "the removal's own records end the plan"
    );
    assert!(
        std::fs::symlink_metadata(plan.as_path()).is_err(),
        "the plan's file is gone"
    );
}

/// What:
///  A plan that reads and names another origin than the add's record refuses the
///  removal and changes nothing.
/// Why:
///  No record wins when the plan and the binding disagree:
///  the removal refuses before the first delete,
///  names the field,
///  and rewrites neither
///  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "6.
/// The binding record and the removal intent",
///  "When they disagree").
#[test]
fn a_plan_that_names_another_origin_refuses_and_changes_nothing() {
    let fixture: RemovalFixture = a_fixture("completion-plan-of-another-origin");
    let plan: PathBuf = fixture.plant_a_plan(HeadWord::Present, Vec::new());
    let planted: String = std::fs::read_to_string(plan.as_path()).expect("the planted plan reads");
    // The plan still reads as one: only the field's value is another canonical GUID.
    let another: String = String::from("11050373805378911968");
    let rewritten: String = planted.replace(
        format!("originSnapshotGuid={ORIGIN_SNAPSHOT_GUID}").as_str(),
        format!("originSnapshotGuid={another}").as_str(),
    );
    assert_ne!(rewritten, planted, "the plan carries the field");
    std::fs::write(plan.as_path(), rewritten).expect("the plan of another origin");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let stop: RemovalStop = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    })
    .expect_err("a plan of another origin is refused");
    match &stop {
        RemovalStop::IdentityDiffers {
            object,
            field,
            ..
        } => {
            assert_eq!(object, "the standing destruction plan");
            assert_eq!(*field, "originSnapshotGuid");
        }
        other => panic!("the plan's own identity refusal, not {other:?}"),
    }
    assert!(
        device.sent.borrow().is_empty(),
        "no snapshot request was sent: {:?}",
        device.sent.borrow()
    );
    assert!(
        head.sent.borrow().is_empty(),
        "no head destroy was sent: {:?}",
        head.sent.borrow()
    );
    assert!(plan.is_file(), "the refusal rewrote no plan");
    assert!(
        a_record_stands(&fixture, binding_name("fix-flaky-test").as_str()),
        "the refusal changed no binding"
    );
}

/// What:
///  The order a removal meets a lock a person wrote,
///  measured through the scripted seams:
///  it refuses before a single request,
///  and the sentence names the lock,
///  what was not removed,
///  and the one command that clears it.
/// Why:
///  Git refuses its own removal for a lock and steers a person to the double force,
///  which here runs this wrapper's removal and destroys a clone and its snapshots
///  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
///  "What Git 2.56.1 does").
///  Measured at `8bdae8a0d8`:
///  with a person's lock on the fixture's registration,
///  this same removal destroyed the clone's own snapshots,
///  the origin snapshot and the head,
///  and only then did the Git settle refuse the lock's words,
///  because the settle is the order's only step that reads `locked`.
///  So the lock is read before the first write now,
///  and this control holds that:
///  no request of either device,
///  no intent and no plan.
#[test]
fn a_persons_lock_refuses_the_removal_before_any_request() {
    let fixture: RemovalFixture = a_fixture("completion-persons-lock");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let locked: PathBuf = fixture.registration.join("locked");
    std::fs::write(locked.as_path(), PERSON_LOCK_WORDS).expect("the person's lock");
    let settle: TheLiveSettle = TheLiveSettle;
    let stopped = complete(&TheCompletion {
        fixture: &fixture,
        pool: &pool,
        check: &probe,
        probe: &probe,
        device: &device,
        head: &head,
        holders: &holders,
        settle: &settle,
        cancelling: None,
        budget: the_budget(),
    });
    let stop: RemovalStop = match stopped {
        Ok(done) => panic!(
            "a person's lock refuses the removal, so nothing finishes: {}",
            done.text()
        ),
        Err(stop) => stop,
    };
    match &stop {
        RemovalStop::TheLocksWordsAreNotThisAdds {
            slot,
            cause,
        } => {
            assert_eq!(slot, "fix-flaky-test");
            assert!(
                cause.contains("ZFS test underlay"),
                "the refusal names the lock's own words: {cause}"
            );
        }
        other => panic!("the lock's own refusal, not {other:?}"),
    }
    assert!(
        device.sent.borrow().is_empty(),
        "no snapshot request was sent: {:?}",
        device.sent.borrow()
    );
    assert!(
        head.sent.borrow().is_empty(),
        "no head destroy was sent: {:?}",
        head.sent.borrow()
    );
    assert!(
        std::fs::symlink_metadata(plans_folder(fixture.common()).as_path()).is_err(),
        "no removal intent was filed and no plan was written"
    );
    let message: String = stop.message();
    for said in [
        "a person's lock may stand there",
        "before it unmounts or destroys anything",
        "claim",
        "Nothing was unmounted, nothing was destroyed",
        "`git worktree unlock fix-flaky-test`",
        "`-f -f` among them",
    ] {
        assert!(
            message.contains(said),
            "the sentence says {said}: {message}"
        );
    }
    // The order's first write is the slot's claim, so a sentence that promised the removal wrote nothing before
    //  this point would promise something no run keeps: the claim stands in the record folder, written back as
    //  released by the drop the refusal takes.
    assert!(
        !message.contains("before its first write"),
        "the sentence does not promise a removal that wrote nothing, because the claim was written: {message}"
    );
    let claims: Vec<PathBuf> = claim_files_of(fixture.common(), "fix-flaky-test");
    assert_eq!(
        claims.len(),
        1,
        "the slot's claim file is the only record this run wrote: {claims:?}"
    );
    let claim: String = std::fs::read_to_string(claims[0].as_path()).expect("the claim file reads");
    assert_eq!(
        crate::single_owner_claim::decode(claim.as_str()),
        Some(crate::single_owner_claim::ClaimRecord::Released),
        "stopping gave the slot back, so the claim file records a released claim"
    );
    assert!(
        locked.is_file(),
        "the person's lock stands where it was, and the removal never rewrote it"
    );
}

/// What:
///  The claim files one repository's record folder holds.
/// Why:
///  The claim is the removal's first write,
///  so a control that says what a refusal left reads the folder rather than assuming it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claims = claimFilesOf(commonDir); // every `<slot>.claim.<generation>`
/// ```
fn claim_files_of(
    common_dir: &Path,
    slot: &str,
) -> Vec<PathBuf> {
    let home: PathBuf = home_folder(common_dir);
    let prefix: String = format!("{}.", claim_prefix(slot).to_string_lossy());
    let entries = match std::fs::read_dir(home.as_path()) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut claims: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with(prefix.as_str()) {
            claims.push(entry.path());
        }
    }
    return claims;
}

/// What:
///  A removal resumed from a plan whose clone is already gone reads a person's lock
///  before it writes the plan and before it destroys the origin snapshot,
///  for every head word whose path the clone's absence takes.
/// Why:
///  The alpha.53 read's M1:
///  `start_the_removal` answered `TheStart::Gone` for those head words without reading the lock,
///  and the head-gone path then published the plan and sent the origin destroy,
///  so a person's lock was met at the settle and after the two writes.
///  That contradicts what the records promise a person's lock
///  (`SPEC.md`,
///  "Removal of a ZFS worktree clone"),
///  and the release's own sentence about it.
///  The control plants the plan a killed removal leaves, a cleanup record an earlier run left
///  so a removal that ran would destroy the origin snapshot,
///  a clone the pool proves gone,
///  and a person's lock on the registration,
///  then reads what the run left.
#[test]
fn a_persons_lock_refuses_a_resumed_removal_whose_clone_is_gone_before_any_write() {
    for head in [
        HeadWord::Requested,
        HeadWord::Unknown,
        HeadWord::Gone,
    ] {
        let fixture: RemovalFixture =
            a_fixture(format!("completion-resumed-lock-{}", head.word()).as_str());
        // The plan a killed removal leaves: the head's request recorded and every name still `planned`. The writer
        //  files the lines in the order of their names and reads the record back, so a control files them that way.
        let mut lines: Vec<PlanAttempt> = vec![
            fixture.a_line_of_the_plan(
                OWN_SNAPSHOT,
                OWN_GUID,
                OWN_CREATETXG,
                0,
                SnapshotOutcome::Planned,
            ),
            fixture.a_line_of_the_plan(
                SANOID_SNAPSHOT,
                SANOID_GUID,
                SANOID_CREATETXG,
                0,
                SnapshotOutcome::Planned,
            ),
        ];
        lines.sort_by(|left, right| return left.snapshot.name.cmp(&right.snapshot.name));
        let plan_path: PathBuf = fixture.plant_a_plan(head, lines);
        let planted: Vec<u8> = std::fs::read(plan_path.as_path()).expect("the planted plan reads");
        let cleanups: CleanupFolder =
            CleanupFolder::open_or_create(fixture.common()).expect("the cleanup folder opens");
        let record: PathBuf = cleanups
            .publish(&an_owed_record())
            .expect("the origin cleanup record an earlier run left stands");
        let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
        let check: Pool = the_checks_pool();
        // The clone is already gone, which is what makes the resume take the head-gone path.
        let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(true));
        let probe: EpochProbe = EpochProbe {
            pool: check,
            hidden: Rc::clone(&hidden),
        };
        let device: ClearingDevice = ClearingDevice::over(&pool);
        let head_device: ScriptedHead = ScriptedHead::succeeding(
            fixture.plan_folder(),
            fixture.recorded().clone_guid,
            Rc::clone(&hidden),
        );
        let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
        let locked: PathBuf = fixture.registration.join("locked");
        std::fs::write(locked.as_path(), PERSON_LOCK_WORDS).expect("the person's lock");
        let settle: ScriptedSettle = ScriptedSettle::succeeding();
        let stopped = complete(&TheCompletion {
            fixture: &fixture,
            pool: &pool,
            check: &probe,
            probe: &probe,
            device: &device,
            head: &head_device,
            holders: &holders,
            settle: &settle,
            cancelling: None,
            budget: the_budget(),
        });
        let stop: RemovalStop = match stopped {
            Ok(done) => panic!(
                "a person's lock refuses the resumed removal of a {} plan, so nothing finishes: {}",
                head.word(),
                done.text()
            ),
            Err(stop) => stop,
        };
        match &stop {
            RemovalStop::TheLocksWordsAreNotThisAdds {
                slot,
                cause,
            } => {
                assert_eq!(slot, "fix-flaky-test");
                assert!(
                    cause.contains("ZFS test underlay"),
                    "the refusal names the lock's own words: {cause}"
                );
            }
            other => panic!(
                "the lock's own refusal for a {} plan, not {other:?}",
                head.word()
            ),
        }
        assert!(
            device.sent.borrow().is_empty(),
            "no origin snapshot was destroyed for a {} plan: {:?}",
            head.word(),
            device.sent.borrow()
        );
        assert_eq!(
            std::fs::read(plan_path.as_path()).expect("the plan still reads"),
            planted,
            "the plan a {} plan's resume would publish is exactly the one that stood",
            head.word()
        );
        assert!(
            record.is_file(),
            "the origin cleanup record an earlier run left stands, for a {} plan",
            head.word()
        );
        assert!(
            locked.is_file(),
            "the person's lock stands where it was, for a {} plan",
            head.word()
        );
        assert!(
            settle.calls.get() == 0,
            "the settle that reads `locked` never ran, for a {} plan",
            head.word()
        );
        assert!(
            stop.message()
                .contains("`git worktree unlock fix-flaky-test`"),
            "the sentence names the way out: {}",
            stop.message()
        );
    }
}

/// What:
///  A device that records whether the origin cleanup record stood when each request arrived.
/// Why:
///  The design's moment for that write is before the first destroy,
///  and only a seam at the request can show it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WatchingDevice { destroySnaps(pool, names) { this.seen.push(exists(folder)); ... } }
/// ```
struct WatchingDevice<'a> {
    inner: &'a ClearingDevice,
    folder: std::path::PathBuf,
    seen: Rc<RefCell<Vec<bool>>>,
}

impl crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice
    for WatchingDevice<'_>
{
    fn destroy_snaps(
        &self,
        pool: &str,
        names: &[&str],
    ) -> Result<Option<NvList>, monochromatic_zfs_request_to_openzfs_kernel_device::DeviceError>
    {
        let record: std::path::PathBuf = self.folder.join(ORIGIN_OWN_NAME);
        self.seen
            .borrow_mut()
            .push(std::fs::symlink_metadata(record.as_path()).is_ok());
        return self.inner.destroy_snaps(pool, names);
    }
}

/// Whether one of the wrapper's record files stands.
fn locate(
    fixture: &RemovalFixture,
    name: &str,
) -> bool {
    return a_record_stands(fixture, name);
}
