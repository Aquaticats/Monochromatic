//! What:
//!  Controls of the check at both call sites and of the look before the unmount:
//!  a marked origin refuses,
//!  an unmarked one proceeds after exactly three reads in one order,
//!  every identity the plan recorded is compared,
//!  and a gone or unreadable object refuses rather than proceeding.
//! Why:
//!  These are plan step 15's host controls
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "The tests a build must have", controls 1 to 9 and 12 to 14,
//!  and controls 19 to 21 for the look).
//!  A control for what a refusal leaves unsent needs step 16's call sites and a control for the
//!  proof's own refusals needs step 14's hook,
//!  so those halves stay owed and are named in this round's report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkPlannedOrigin(unmarked(), planned())).toEqual({ kind: 'proceed' });
//! ```

use super::super::fixtures::{
    Answer, CHECKOUT, CLONE, CLONE_GUID, ORIGIN_CREATETXG, ORIGIN_DATASET_GUID, ORIGIN_SNAPSHOT, ORIGIN_SNAPSHOT_GUID, Pool, guid, gone, healthy, marked, origin_snapshot_stats, planned,
    pool_over, refused, refusal, stats, unmarked,
};
use super::super::origin_mark::{MARK_ON, OriginMark, UserHolds};
use super::super::origin_refusal::OriginRefusal;
use super::super::origin_refusals::{JudgedObject, RefusalForm, RefusalPoint, WhatARefusalLeaves};
use super::{Verdict, check_planned_origin, judge_the_look};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

//region The planned case and the mark

#[test]
fn an_unmarked_origin_proceeds_after_exactly_three_identity_reads() {
    let pool: Pool = unmarked();
    assert_eq!(check_planned_origin(&pool, &planned()), Verdict::Proceed);
    assert_eq!(pool.reads(), vec![String::from(CLONE), String::from(CHECKOUT), String::from(ORIGIN_SNAPSHOT)], "clone, origin dataset, origin snapshot, and no other read");
}

#[test]
fn a_marked_origin_refuses_with_the_two_counts_a_person_needs() {
    let cause: OriginRefusal = refusal(check_planned_origin(&marked(), &planned()));
    match &cause {
        OriginRefusal::OriginMarkedForDeferredDestroy { clones, holds, planned: recorded } => {
            assert_eq!(*clones, 1, "the reply's clone count travels with the cause");
            assert_eq!(*holds, UserHolds::Counted(0), "and so does its hold count");
            assert_eq!(recorded.clone_name, CLONE);
        }
        other => panic!("a marked origin must refuse with its own cause, not {other:?}"),
    }
    assert_eq!(cause.step(), "origin-mark");
    assert_eq!(cause.form(), RefusalForm::PendingAfterTheUnmount);
    assert_eq!(cause.what_it_leaves(), WhatARefusalLeaves::PendingWithItsRecord, "the removal keeps its intent and its plan");
}

#[test]
fn an_unreadable_mark_at_a_check_refuses_rather_than_proceeding() {
    let unreadable: Pool = healthy(None, Some(0));
    let cause: OriginRefusal = refusal(check_planned_origin(&unreadable, &planned()));
    match &cause {
        OriginRefusal::MarkUnreadable { point, .. } => assert_eq!(*point, RefusalPoint::AtACheck),
        other => panic!("an unreadable mark must refuse as itself, not {other:?}"),
    }
    assert_eq!(cause.form(), RefusalForm::PendingAfterTheUnmount);
    assert_eq!(cause.what_it_leaves(), WhatARefusalLeaves::PendingWithItsRecord);
}

#[test]
fn a_marked_origin_with_a_second_clone_still_refuses_and_says_the_origin_is_pinned() {
    let pool: Pool = pool_over(2, Some(MARK_ON), Some(0));
    let cause: OriginRefusal = refusal(check_planned_origin(&pool, &planned()));
    assert!(cause.message(&super::super::removal_history::RemovalHistory::NothingDestroyed).contains("because another clone of it is in place"), "a second clone pins the origin");
}

#[test]
fn a_marked_origin_under_a_user_hold_still_refuses_and_says_the_reap_will_not_happen() {
    let pool: Pool = pool_over(1, Some(MARK_ON), Some(1));
    let message: String = refusal(check_planned_origin(&pool, &planned())).message(&super::super::removal_history::RemovalHistory::NothingDestroyed);
    assert!(message.contains("because one user hold pins it"), "a hold keeps the snapshot: {message}");
}

//endregion

//region The clone

#[test]
fn a_replaced_clone_refuses_before_the_origin_is_read() {
    let pool: Pool = unmarked().with(CLONE, Answer::of(stats(CLONE_GUID + 1, false, 7, 0, ORIGIN_SNAPSHOT), NvList::new()));
    let cause: OriginRefusal = refusal(check_planned_origin(&pool, &planned()));
    match cause {
        OriginRefusal::CloneIsNotThePlannedOne { live, .. } => assert_eq!(live, CLONE_GUID + 1, "the live GUID travels with the cause"),
        other => panic!("a replaced clone must refuse with its own cause, not {other:?}"),
    }
    assert_eq!(pool.reads(), vec![String::from(CLONE)], "the origin is never read after the clone disagrees");
}

#[test]
fn a_clone_whose_live_origin_differs_refuses_before_the_origin_is_read() {
    let other: &str = "zpcachyos/nosnap/checkout@git-worktree-clone-origin-ffeeddccbbaa9988";
    let pool: Pool = unmarked().with(CLONE, Answer::of(stats(CLONE_GUID, false, 7, 0, other), NvList::new()));
    match refusal(check_planned_origin(&pool, &planned())) {
        OriginRefusal::CloneOriginDiffers { live, .. } => assert_eq!(live, other),
        other_cause => panic!("a clone over another origin must refuse with its own cause, not {other_cause:?}"),
    }
    assert_eq!(pool.reads().len(), 1, "no origin was read");
}

#[test]
fn a_clone_with_no_origin_refuses() {
    let pool: Pool = unmarked().with(CLONE, Answer::of(stats(CLONE_GUID, false, 7, 0, ""), NvList::new()));
    let cause: OriginRefusal = refusal(check_planned_origin(&pool, &planned()));
    assert!(matches!(cause, OriginRefusal::CloneHasNoOrigin { .. }), "a dataset with no origin is no clone: {cause:?}");
    assert_eq!(cause.step(), "clone-origin");
}

#[test]
fn a_name_that_is_not_the_planned_kind_refuses() {
    let clone_as_snapshot: Pool = unmarked().with(CLONE, Answer::of(stats(CLONE_GUID, true, 7, 0, ORIGIN_SNAPSHOT), NvList::new()));
    match refusal(check_planned_origin(&clone_as_snapshot, &planned())) {
        OriginRefusal::CloneIsNotAFilesystem { snapshot, .. } => assert!(snapshot, "the cause says the kernel called it a snapshot"),
        other => panic!("a snapshot at the clone's name must refuse as the wrong kind, not {other:?}"),
    }
    let origin_as_snapshot: Pool = unmarked().with(CHECKOUT, Answer::of(stats(ORIGIN_DATASET_GUID, true, 3, 1, ""), NvList::new()));
    assert!(matches!(refusal(check_planned_origin(&origin_as_snapshot, &planned())), OriginRefusal::OriginDatasetIsNotAFilesystem { snapshot: true, .. }));
    let origin_snapshot_as_filesystem: Pool = unmarked().with(
        ORIGIN_SNAPSHOT,
        Answer::of(stats(ORIGIN_SNAPSHOT_GUID, false, ORIGIN_CREATETXG, 1, ""), NvList::new()),
    );
    assert_eq!(refusal(check_planned_origin(&origin_snapshot_as_filesystem, &planned())).step(), "origin-snapshot-type");
}

//endregion

//region The origin's two objects

#[test]
fn a_replaced_origin_dataset_refuses() {
    let pool: Pool = unmarked().with(CHECKOUT, Answer::of(stats(ORIGIN_DATASET_GUID + 1, false, 3, 1, ""), NvList::new()));
    match refusal(check_planned_origin(&pool, &planned())) {
        OriginRefusal::OriginDatasetIsNotThePlannedOne { live, .. } => assert_eq!(live, ORIGIN_DATASET_GUID + 1),
        other => panic!("a replaced origin dataset must refuse with its own cause, not {other:?}"),
    }
    assert_eq!(pool.reads().len(), 2, "the origin snapshot is never read after the dataset disagrees");
}

#[test]
fn a_replaced_origin_snapshot_refuses() {
    let (_, properties) = origin_snapshot_stats(1, Some(0), Some(0));
    let pool: Pool = unmarked().with(ORIGIN_SNAPSHOT, Answer::of(stats(ORIGIN_SNAPSHOT_GUID + 1, true, ORIGIN_CREATETXG, 1, ""), properties));
    match refusal(check_planned_origin(&pool, &planned())) {
        OriginRefusal::OriginSnapshotIsNotThePlannedOne { live, .. } => assert_eq!(live, ORIGIN_SNAPSHOT_GUID + 1),
        other => panic!("a replaced origin snapshot must refuse with its own cause, not {other:?}"),
    }
}

#[test]
fn an_origin_whose_createtxg_differs_refuses() {
    let (_, properties) = origin_snapshot_stats(1, Some(0), Some(0));
    let pool: Pool = unmarked().with(ORIGIN_SNAPSHOT, Answer::of(stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG + 1, 1, ""), properties));
    let cause: OriginRefusal = refusal(check_planned_origin(&pool, &planned()));
    match cause {
        OriginRefusal::OriginSnapshotCreatetxgDiffers { live, .. } => assert_eq!(live, ORIGIN_CREATETXG + 1),
        other => panic!("a snapshot made in another transaction must refuse with its own cause, not {other:?}"),
    }
    assert_eq!(cause.step(), "origin-snapshot-createtxg");
}

//endregion

//region Gone objects and failed reads

#[test]
fn a_gone_clone_or_origin_dataset_or_origin_snapshot_refuses_and_names_the_settle() {
    let gone_clone: Pool = unmarked().with(CLONE, gone(CLONE));
    let clone_cause: OriginRefusal = refusal(check_planned_origin(&gone_clone, &planned()));
    match &clone_cause {
        OriginRefusal::CloneGone { planned: recorded } => assert_eq!(recorded.clone_guid, guid(CLONE_GUID), "the plan's tuple travels with the cause"),
        other => panic!("a gone clone must refuse with its own cause, not {other:?}"),
    }
    assert!(clone_cause.message(&super::super::removal_history::RemovalHistory::NothingDestroyed).contains("a later run settles only after it has proved that clone is gone"), "the settle rule is named");
    assert_eq!(clone_cause.form(), RefusalForm::NothingCompared);
    let gone_dataset: Pool = unmarked().with(CHECKOUT, gone(CHECKOUT));
    assert_eq!(refusal(check_planned_origin(&gone_dataset, &planned())).step(), "origin-dataset");
    let gone_snapshot: Pool = unmarked().with(ORIGIN_SNAPSHOT, gone(ORIGIN_SNAPSHOT));
    assert_eq!(refusal(check_planned_origin(&gone_snapshot, &planned())).step(), "origin-snapshot");
    assert_eq!(gone_snapshot.reads().len(), 3, "the snapshot's own read was the third and it answered not there");
}

#[test]
fn a_read_that_failed_for_another_reason_is_not_a_gone_object() {
    let pool: Pool = unmarked().with(CLONE, refused("the device answered EACCES"));
    let cause: OriginRefusal = refusal(check_planned_origin(&pool, &planned()));
    match &cause {
        OriginRefusal::ReadFailed { object, name, said, .. } => {
            assert_eq!(*object, JudgedObject::Clone);
            assert_eq!(name, CLONE);
            assert!(said.contains("EACCES"), "the kernel's words travel with the cause: {said}");
        }
        other => panic!("a failed read must refuse as itself, not {other:?}"),
    }
    assert_eq!(cause.step(), "read");
    let dataset: Pool = unmarked().with(CHECKOUT, refused("the pool is not imported"));
    assert_eq!(refusal(check_planned_origin(&dataset, &planned())).step(), "read");
    let snapshot: Pool = unmarked().with(ORIGIN_SNAPSHOT, refused("the pool is not imported"));
    assert_eq!(refusal(check_planned_origin(&snapshot, &planned())).step(), "read");
}

//endregion

//region The look before the unmount

#[test]
fn the_look_refuses_a_marked_origin_at_the_point_that_took_nothing_down() {
    let marked_mark: OriginMark = OriginMark::Marked { clones: 1, holds: UserHolds::Counted(0) };
    let cause: OriginRefusal = refusal(judge_the_look(CLONE, "fix-flaky-test", ORIGIN_SNAPSHOT, &marked_mark));
    assert!(matches!(cause, OriginRefusal::OriginMarkedBeforeTheUnmount { .. }), "the look has its own cause: {cause:?}");
    assert_eq!(cause.step(), "origin-mark");
    assert_eq!(cause.form(), RefusalForm::NothingTakenDown);
    assert_eq!(cause.what_it_leaves(), WhatARefusalLeaves::NothingPending, "the look runs before the intent and the unmount");
}

#[test]
fn an_unmarked_mark_at_the_look_proceeds() {
    assert_eq!(judge_the_look(CLONE, "fix-flaky-test", ORIGIN_SNAPSHOT, &OriginMark::Unmarked), Verdict::Proceed);
}

#[test]
fn the_look_refuses_an_unreadable_mark_at_its_own_point() {
    let unreadable: OriginMark = OriginMark::Unreadable { said: String::from("`defer_destroy` is not in the reply") };
    let cause: OriginRefusal = refusal(judge_the_look(CLONE, "fix-flaky-test", ORIGIN_SNAPSHOT, &unreadable));
    match &cause {
        OriginRefusal::MarkUnreadable { point, .. } => assert_eq!(*point, RefusalPoint::BeforeTheUnmount),
        other => panic!("an unreadable mark at the look must refuse as itself, not {other:?}"),
    }
    assert_eq!(cause.form(), RefusalForm::NothingTakenDown);
    assert_eq!(cause.what_it_leaves(), WhatARefusalLeaves::NothingPending);
}

#[test]
fn the_look_can_only_report_the_mark_it_judged() {
    let marks: Vec<OriginMark> = vec![
        OriginMark::Unmarked,
        OriginMark::Marked { clones: 1, holds: UserHolds::Counted(0) },
        OriginMark::Unreadable { said: String::from("nothing") },
    ];
    for mark in marks {
        match judge_the_look(CLONE, "fix-flaky-test", ORIGIN_SNAPSHOT, &mark) {
            Verdict::Proceed => assert_eq!(mark, OriginMark::Unmarked, "only an unmarked snapshot goes on"),
            Verdict::Refused(cause) => assert!(
                matches!(cause, OriginRefusal::OriginMarkedBeforeTheUnmount { .. } | OriginRefusal::MarkUnreadable { .. }),
                "the look judges the mark and no identity, so it can report no identity cause: {cause:?}"
            ),
        }
    }
}

#[test]
fn the_check_and_the_look_name_one_state_at_two_points() {
    let at_a_check: OriginRefusal = refusal(check_planned_origin(&marked(), &planned()));
    let at_the_look: OriginRefusal = refusal(judge_the_look(CLONE, "fix-flaky-test", ORIGIN_SNAPSHOT, &OriginMark::Marked { clones: 1, holds: UserHolds::Counted(0) }));
    assert_ne!(at_a_check, at_the_look, "a receipt can say which point refused");
    assert_eq!(at_a_check.step(), at_the_look.step(), "both file under the mark's step");
}
