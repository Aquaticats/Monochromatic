//! What:
//!  Controls for the settle:
//!  a GUID the inventory still carries is intact,
//!  an absent one is destroyed,
//!  a line the plan already wrote as destroyed is final,
//!  a `unknown` tuple refuses,
//!  and a name the pool carries with another GUID refuses.
//! Why:
//!  The command may not attach over an outcome nobody can prove,
//!  and the destroyed set it reports is the only record a person has of what is gone
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "Settling the destroy outcomes before anything moves").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(settleTheDestroyOutcomes(plan, live).destroyed).toEqual(['tank/clone@one']);
//! ```

use super::settle_the_destroy_outcomes;
use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_proof_and_plan::fixtures::{
    guid,
    sample_facts,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanAttempt,
    PlanSnapshot,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;

/// The two snapshots one plan records.
fn snapshots() -> Vec<PlanSnapshot> {
    return vec![
        PlanSnapshot {
            name: String::from("tank/clone@one"),
            guid: guid(1_000),
            createtxg: 18,
        },
        PlanSnapshot {
            name: String::from("tank/clone@two"),
            guid: guid(2_000),
            createtxg: 20,
        },
    ];
}

/// The plan a removal wrote before its first request.
fn the_plan() -> DestructionPlan {
    return DestructionPlan::first_write(sample_facts(), snapshots().as_slice());
}

/// One live snapshot of the clone.
fn live(
    name: &str,
    guid_value: u64,
) -> LiveObject {
    return LiveObject {
        kind: EntryKind::Snapshot,
        name: String::from(name),
        guid: guid(guid_value),
        createtxg: 18,
        owning_dataset: String::from("tank/clone"),
        owning_dataset_guid: guid(7_828_944_690_444_299_989),
    };
}

/// The plan with one tuple's highest line rewritten.
fn with_a_line(
    plan: &DestructionPlan,
    name: &str,
    outcome: SnapshotOutcome,
) -> DestructionPlan {
    let mut other: DestructionPlan = plan.clone();
    let snapshot: PlanSnapshot = other
        .named_snapshots()
        .into_iter()
        .find(|held| return held.name == name)
        .expect("the plan names it");
    other
        .record_line(PlanAttempt {
            snapshot,
            attempt: 1,
            outcome,
            errno: None,
        })
        .expect("the line is recorded");
    return other;
}

#[test]
fn a_guid_the_inventory_carries_is_intact_and_an_absent_one_is_destroyed() {
    let plan: DestructionPlan = the_plan();
    let live: Vec<LiveObject> = vec![live("one", 1_000)];
    let destroyed: Vec<String> =
        settle_the_destroy_outcomes(&plan, live.as_slice()).expect("both tuples settle");
    assert_eq!(
        destroyed,
        vec![String::from("tank/clone@two")],
        "the present one is not named"
    );
}

#[test]
fn a_plan_whose_every_snapshot_is_live_destroyed_nothing() {
    let plan: DestructionPlan = the_plan();
    let live: Vec<LiveObject> = vec![live("one", 1_000), live("two", 2_000)];
    let destroyed: Vec<String> =
        settle_the_destroy_outcomes(&plan, live.as_slice()).expect("both tuples settle");
    assert!(destroyed.is_empty());
}

#[test]
fn a_line_the_plan_wrote_as_destroyed_is_final_beside_a_live_snapshot() {
    let plan: DestructionPlan =
        with_a_line(&the_plan(), "tank/clone@one", SnapshotOutcome::Destroyed);
    let live: Vec<LiveObject> = vec![live("one", 1_000), live("two", 2_000)];
    let destroyed: Vec<String> =
        settle_the_destroy_outcomes(&plan, live.as_slice()).expect("the tuples settle");
    assert_eq!(
        destroyed,
        vec![String::from("tank/clone@one")],
        "a `destroyed` line is not undone by a live object of the same GUID"
    );
}

#[test]
fn a_tuple_marked_unknown_refuses_with_its_name_and_guid() {
    let plan: DestructionPlan =
        with_a_line(&the_plan(), "tank/clone@two", SnapshotOutcome::Unknown);
    match settle_the_destroy_outcomes(&plan, &[]).expect_err("an unknown outcome refuses") {
        CancelRefusal::AnUnknownOutcome {
            snapshot,
            guid: reported,
            ..
        } => {
            assert_eq!(snapshot, "tank/clone@two");
            assert_eq!(reported, guid(2_000).file_name());
        }
        other => panic!("an unknown outcome refuses as one, not {other:?}"),
    }
}

#[test]
fn a_name_the_pool_carries_with_another_guid_refuses() {
    let plan: DestructionPlan = the_plan();
    let live: Vec<LiveObject> = vec![live("one", 9_000)];
    match settle_the_destroy_outcomes(&plan, live.as_slice()).expect_err("a reused name refuses") {
        CancelRefusal::ASnapshotNameIsReused {
            name,
            recorded_guid,
            live_guid,
        } => {
            assert_eq!(name, "tank/clone@one");
            assert_eq!(recorded_guid, guid(1_000).file_name());
            assert_eq!(live_guid, guid(9_000).file_name());
        }
        other => panic!("a reused name refuses as one, not {other:?}"),
    }
}

#[test]
fn a_snapshot_carrying_the_recorded_guid_under_another_name_refuses() {
    let plan: DestructionPlan = the_plan();
    let live: Vec<LiveObject> = vec![live("a-newer-name", 1_000)];
    match settle_the_destroy_outcomes(&plan, live.as_slice()).expect_err("a renamed object refuses")
    {
        CancelRefusal::TheRecordsDisagree {
            first,
            second,
            ..
        } => {
            assert!(first.contains("one"), "{first}");
            assert!(second.contains("a-newer-name"), "{second}");
        }
        other => panic!("a renamed object refuses as a disagreement, not {other:?}"),
    }
}

#[test]
fn a_bookmark_carrying_the_guid_does_not_prove_the_snapshot_intact() {
    let plan: DestructionPlan = the_plan();
    let bookmark: LiveObject = LiveObject {
        kind: EntryKind::Bookmark,
        ..live("one", 1_000)
    };
    let destroyed: Vec<String> =
        settle_the_destroy_outcomes(&plan, &[bookmark]).expect("a bookmark settles as no snapshot");
    assert_eq!(
        destroyed.len(),
        2,
        "neither snapshot is proved intact by a bookmark"
    );
}

#[test]
fn the_settlement_writes_nothing_to_the_plan() {
    let plan: DestructionPlan = the_plan();
    let before: Vec<u8> = plan.encode().expect("the plan encodes");
    let _ = settle_the_destroy_outcomes(&plan, &[]).expect("the tuples settle");
    assert_eq!(
        plan.encode().expect("the plan encodes again"),
        before,
        "a settle is not an outcome"
    );
}
