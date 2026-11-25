//! What:
//!  Controls of one clone's retention line:
//!  a clone with no snapshot of its own,
//!  its own snapshots with their fields,
//!  an inherited history and the origin underlay,
//!  a field the reply does not carry,
//!  a dataset the pool does not hold,
//!  and a dataset that could not be read.
//! Why:
//!  ZFS counts one shared block in every dataset that holds it,
//!  so the line must print one object's own fields and sum nothing,
//!  and a field nobody read must never appear as zero.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkCloneRetention(facts).detail).toContain('sums none of them');
//! ```

use super::{AccountingFacts, CloneRetentionFacts, check_clone_retention};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::health::doctor_zfs_route::zfs_checks::Reading;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;

/// The clone these controls are about.
const CLONE: &str = "tank/wt/fix-flaky-test";

/// The origin snapshot the clone was made from.
const ORIGIN: &str = "tank/mono-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// One object's fields,
/// with every number the pool reported.
fn accounting(object: &str, used: u64, written: u64) -> AccountingFacts {
    return AccountingFacts {
        object: String::from(object),
        used: PropertyValue::Number(used),
        referenced: PropertyValue::Number(0),
        logical_referenced: PropertyValue::Number(used),
        written: PropertyValue::Number(written),
    };
}

/// The facts of one clone's retention.
fn facts(dataset: Reading<AccountingFacts>, snapshots: Vec<AccountingFacts>) -> CloneRetentionFacts {
    return CloneRetentionFacts { clone: String::from(CLONE), dataset, snapshots, inherited: Vec::new(), origin_snapshot: Some(String::from(ORIGIN)) };
}

/// A clone with no snapshot of its own passes,
/// names the four dataset fields and sums nothing.
#[test]
fn a_clone_without_snapshots_passes_and_sums_nothing() {
    let answer: CheckOutcome = check_clone_retention(&facts(Reading::Read(accounting(CLONE, 4_096, 0)), Vec::new()));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a clone without snapshots passes: {answer:?}");
    };
    assert!(detail.contains("`used` 4096"), "{detail}");
    assert!(detail.contains("`referenced` 0"), "{detail}");
    assert!(detail.contains("`logicalreferenced` 4096"), "{detail}");
    assert!(detail.contains("`written` 0"), "{detail}");
    assert!(detail.contains("sums none of them"), "{detail}");
    assert!(detail.contains("retained under the checkout's own name"), "{detail}");
}

/// An own snapshot's fields are printed as its own,
/// and the snapshot named.
///
/// The snapshot's `written` is the one the fixture gave it,
/// which is not the dataset's.
#[test]
fn an_own_snapshot_is_printed_as_own() {
    let answer: CheckOutcome = check_clone_retention(&facts(Reading::Read(accounting(CLONE, 8_192, 4_096)), vec![accounting("tank/wt/fix-flaky-test@own", 4_096, 0)]));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a clone with a snapshot passes: {answer:?}");
    };
    assert!(detail.contains("`tank/wt/fix-flaky-test@own` (own)"), "{detail}");
    assert!(detail.contains("`used` 4096"), "{detail}");
}

/// An inherited history is kept apart from the clone's own,
/// so a person does not expect a removal to destroy it.
#[test]
fn an_inherited_history_is_kept_apart() {
    let mut held: CloneRetentionFacts = facts(Reading::Read(accounting(CLONE, 1_024, 0)), Vec::new());
    held.inherited = vec![String::from("tank/wt/fix-flaky-test@origin")];
    let answer: CheckOutcome = check_clone_retention(&held);
    let CheckOutcome::Pass { detail } = answer else {
        panic!("an inherited history passes: {answer:?}");
    };
    assert!(detail.contains("tank/wt/fix-flaky-test@origin"), "{detail}");
    assert!(detail.contains("retained under the checkout's own name"), "{detail}");
}

/// A field the reply does not carry leaves the answer unknown,
/// naming the object and the field.
#[test]
fn a_missing_field_is_unknown_naming_it() {
    let mut held: AccountingFacts = accounting(CLONE, 4_096, 0);
    held.referenced = PropertyValue::Absent;
    let answer: CheckOutcome = check_clone_retention(&facts(Reading::Read(held), Vec::new()));
    let CheckOutcome::Unknown { reason } = answer else {
        panic!("a missing field is unknown: {answer:?}");
    };
    assert!(reason.contains("`referenced`"), "{reason}");
    assert!(reason.contains(CLONE), "{reason}");
}

/// One snapshot whose field is missing leaves the whole line unknown,
/// because a partial sum is the claim this line refuses to make.
#[test]
fn a_snapshot_without_its_used_field_is_unknown() {
    let mut snapshot: AccountingFacts = accounting("tank/wt/fix-flaky-test@own", 4_096, 0);
    snapshot.used = PropertyValue::Text(String::from("4096K"));
    let answer: CheckOutcome = check_clone_retention(&facts(Reading::Read(accounting(CLONE, 4_096, 4_096)), vec![snapshot]));
    let CheckOutcome::Unknown { reason } = answer else {
        panic!("a text field is unknown: {answer:?}");
    };
    assert!(reason.contains("4096K"), "{reason}");
}

/// A dataset the pool does not hold passes and says it retains no blocks.
#[test]
fn a_gone_dataset_passes_with_no_blocks() {
    let answer: CheckOutcome = check_clone_retention(&facts(Reading::Missing { why: String::from("dataset does not exist") }, Vec::new()));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a gone dataset passes: {answer:?}");
    };
    assert!(detail.contains("retains no blocks"), "{detail}");
}

/// A dataset that could not be read is unknown.
#[test]
fn an_unreadable_dataset_is_unknown() {
    let answer: CheckOutcome = check_clone_retention(&facts(Reading::Unreadable { why: String::from("permission denied") }, Vec::new()));
    let CheckOutcome::Unknown { reason } = answer else {
        panic!("an unreadable dataset is unknown: {answer:?}");
    };
    assert!(reason.contains("permission denied"), "{reason}");
}
