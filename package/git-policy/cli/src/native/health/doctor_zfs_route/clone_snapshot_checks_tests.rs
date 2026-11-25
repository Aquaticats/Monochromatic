//! What:
//!  Controls of one clone's snapshot line:
//!  a clone with no snapshot of its own,
//!  its own snapshots beside the plan's states,
//!  a recorded `destroyed` name the pool still lists,
//!  a recorded `planned` name the pool lost,
//!  an own snapshot no plan names,
//!  a name whose GUID moved,
//!  the history it inherited,
//!  a dataset the pool does not hold,
//!  and a pool that could not be read.
//! Why:
//!  A removal stop sends a person to the clone's own snapshots beside the plan,
//!  so each disagreement is named in both directions and the inherited history
//!  is never counted as the clone's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkCloneSnapshots({ ...facts, recorded: [{ name, outcome: 'destroyed' }] })).toMatchObject({ kind: 'finding' });
//! ```

use super::{CloneSnapshotFacts, check_clone_snapshots};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::health::doctor_zfs_route::removal_plan_checks::RecordedSnapshot;
use crate::health::doctor_zfs_route::zfs_checks::Reading;
use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::{LiveInventory, LiveObject};

/// The clone these controls are about.
const CLONE: &str = "tank/wt/fix-flaky-test";

/// The clone's own transaction group,
/// which every own snapshot stands above.
const CLONE_CREATETXG: u64 = 17;

/// One dataset GUID built from its decimal number.
fn guid(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str()).expect("a number is a canonical GUID");
}

/// One snapshot of the clone's own history,
/// by the leaf name a listing reports.
fn own(leaf: &str, value: u64, createtxg: u64) -> LiveObject {
    return LiveObject { kind: EntryKind::Snapshot, name: String::from(leaf), guid: guid(value), createtxg, owning_dataset: String::from(CLONE), owning_dataset_guid: guid(7) };
}

/// One snapshot the clone inherited from its origin,
/// whose group is at or below the clone's own.
fn inherited(name: &str, value: u64) -> LiveObject {
    return own(name, value, CLONE_CREATETXG - 1);
}

/// One name the plan recorded.
fn recorded(name: &str, value: u64, outcome: SnapshotOutcome) -> RecordedSnapshot {
    return RecordedSnapshot { name: String::from(name), guid: guid(value), outcome, attempt: 0, errno: None };
}

/// The facts of one clone's inventory.
fn facts(objects: Vec<LiveObject>, recorded_names: Vec<RecordedSnapshot>) -> CloneSnapshotFacts {
    return CloneSnapshotFacts { clone: String::from(CLONE), clone_createtxg: CLONE_CREATETXG, live: Reading::Read(LiveInventory { dataset: String::from(CLONE), dataset_guid: guid(7), objects }), recorded: recorded_names };
}

/// A clone with no snapshot of its own passes,
/// and says so.
#[test]
fn a_clone_without_a_snapshot_of_its_own_passes() {
    let answer: CheckOutcome = check_clone_snapshots(&facts(vec![inherited("origin", 11)], Vec::new()));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a clone without a snapshot of its own passes: {answer:?}");
    };
    assert!(detail.contains("no snapshot of its own"), "{detail}");
    assert!(detail.contains("1 inherited snapshot"), "{detail}");
}

/// An own snapshot the plan recorded as `planned` and the pool holds passes,
/// with its GUID and group.
#[test]
fn own_snapshots_the_plan_agrees_with_pass() {
    let answer: CheckOutcome = check_clone_snapshots(&facts(vec![own("own", 21, 18)], vec![recorded("tank/wt/fix-flaky-test@own", 21, SnapshotOutcome::Planned)]));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("an agreed snapshot passes: {answer:?}");
    };
    assert!(detail.contains("GUID 21"), "{detail}");
    assert!(detail.contains("group 18"), "{detail}");
    assert!(detail.contains("agrees with the pool"), "{detail}");
}

/// A name the plan calls `destroyed` that the pool still lists is a finding.
#[test]
fn a_destroyed_name_the_pool_still_lists_is_a_finding() {
    let answer: CheckOutcome = check_clone_snapshots(&facts(vec![own("own", 21, 18)], vec![recorded("tank/wt/fix-flaky-test@own", 21, SnapshotOutcome::Destroyed)]));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a destroyed name the pool lists is a finding: {answer:?}");
    };
    assert_eq!(field, "snapshots");
    assert!(detail.contains("was destroyed on attempt 0"), "{detail}");
    assert!(detail.contains("still lists it"), "{detail}");
}

/// A name the plan still expects that the pool does not list is a finding.
#[test]
fn a_planned_name_the_pool_lost_is_a_finding() {
    let answer: CheckOutcome = check_clone_snapshots(&facts(Vec::new(), vec![recorded("tank/wt/fix-flaky-test@own", 21, SnapshotOutcome::Planned)]));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a planned name the pool lost is a finding: {answer:?}");
    };
    assert_eq!(field, "snapshots");
    assert!(detail.contains("as `planned` on attempt 0"), "{detail}");
    assert!(detail.contains("lists no such snapshot"), "{detail}");
}

/// A `destroyed` name the pool lost agrees with the plan.
#[test]
fn a_destroyed_name_the_pool_lost_agrees() {
    let answer: CheckOutcome = check_clone_snapshots(&facts(Vec::new(), vec![recorded("tank/wt/fix-flaky-test@own", 21, SnapshotOutcome::Destroyed)]));
    assert!(matches!(answer, CheckOutcome::Pass { .. }), "{answer:?}");
}

/// An own snapshot no line of the plan names is a finding.
#[test]
fn an_unrecorded_own_snapshot_is_a_finding() {
    let answer: CheckOutcome = check_clone_snapshots(&facts(vec![own("late", 31, 30)], Vec::new()));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("an unrecorded own snapshot is a finding: {answer:?}");
    };
    assert_eq!(field, "snapshots");
    assert!(detail.contains("no line of the plan names it"), "{detail}");
}

/// A name whose GUID moved is a finding naming both GUIDs.
#[test]
fn a_name_whose_guid_moved_is_a_finding() {
    let answer: CheckOutcome = check_clone_snapshots(&facts(vec![own("own", 22, 18)], vec![recorded("tank/wt/fix-flaky-test@own", 21, SnapshotOutcome::Planned)]));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a moved GUID is a finding: {answer:?}");
    };
    assert_eq!(field, "snapshots");
    assert!(detail.contains("GUID 21"), "{detail}");
    assert!(detail.contains("GUID 22"), "{detail}");
}

/// A dataset the pool does not hold is a finding,
/// because the clone's files are gone with it.
#[test]
fn a_gone_dataset_is_a_finding() {
    let mut held: CloneSnapshotFacts = facts(Vec::new(), Vec::new());
    held.live = Reading::Missing { why: String::from("dataset does not exist") };
    let answer: CheckOutcome = check_clone_snapshots(&held);
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a gone dataset is a finding: {answer:?}");
    };
    assert_eq!(field, "snapshots");
    assert!(detail.contains("no dataset"), "{detail}");
}

/// A pool that could not be read is unknown.
#[test]
fn an_unreadable_pool_is_unknown() {
    let mut held: CloneSnapshotFacts = facts(Vec::new(), Vec::new());
    held.live = Reading::Unreadable { why: String::from("permission denied") };
    let answer: CheckOutcome = check_clone_snapshots(&held);
    let CheckOutcome::Unknown { reason } = answer else {
        panic!("an unreadable pool is unknown: {answer:?}");
    };
    assert!(reason.contains("permission denied"), "{reason}");
}

/// An inventory of another dataset proves nothing about this clone,
/// so the answer is unknown rather than that clone's states.
#[test]
fn an_inventory_of_another_dataset_is_unknown() {
    let mut held: CloneSnapshotFacts = facts(Vec::new(), Vec::new());
    held.live = Reading::Read(LiveInventory { dataset: String::from("tank/wt/another"), dataset_guid: guid(7), objects: Vec::new() });
    let answer: CheckOutcome = check_clone_snapshots(&held);
    assert!(matches!(answer, CheckOutcome::Unknown { .. }), "{answer:?}");
}
