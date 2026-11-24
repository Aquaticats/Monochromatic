//! Controls for the live reads:
//!  a record published under a fixture's common directory is found and read,
//!  a scripted intent source fills the intent facts,
//!  a scripted mount table decides the mounted and unmounted arms,
//!  the registration the record names is read from the filesystem,
//!  and a refused intent read is the unreadable arm rather than an absence.

use crate::testing::mounted_dataset_table::{ScriptedMountTable, row};
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_mount_protocol::RequestNonce;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{IntentReading, MountIntent, MountOperation};
use crate::worktree_clone_slot_state::slot_state_classification::classify_slot;
use crate::worktree_clone_slot_state::slot_state_live_read::{CloneIntentReads, SlotReadingSources, read_slot};
use crate::worktree_clone_slot_state::slot_state_readings::{BindingReading, IntentFacts, SlotReadings};
use crate::worktree_clone_slot_state::slot_state_report::report_of;
use crate::worktree_clone_slot_state::slot_state_test_records::{CLONE_DATASET, publish, record_at};
use std::path::{Path, PathBuf};

/// The fixture's slot name.
fn slot() -> &'static str {
    return "topic";
}

/// The fixture's slot path.
fn slot_path() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/topic");
}

/// The fixture's nonce.
fn nonce() -> RequestNonce {
    return RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f").expect("the fixture nonce reads");
}

/// One scripted intent source.
struct ScriptedIntent {
    /// The reading every question is answered with,
    /// or the refusal the device would have given.
    answer: Result<IntentReading, String>,
}

impl CloneIntentReads for ScriptedIntent {
    fn intent_of(&self, _clone: &str) -> Result<IntentReading, String> {
        return self.answer.clone();
    }
}

/// The readings of one published record and one scripted mount table.
fn read(common: &Path, table: &dyn crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets, intent: &dyn CloneIntentReads) -> SlotReadings {
    let reads: SlotReadingSources<'_> = SlotReadingSources { mounts: table, intents: intent };
    return read_slot(common, slot(), slot_path().as_path(), &reads);
}

/// A filled record whose clone no row mounts is the known unmounted clone,
/// read from a real file and the release mount parser.
#[test]
fn a_filled_record_with_no_mount_is_a_known_unmounted_clone() {
    let root: PathBuf = fixture("slot-state-live-unmounted");
    let common: PathBuf = root.join(".git");
    publish(common.as_path(), &record_at(Phase::Filled, slot_path().as_path()), slot());
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[row(40, 1, "/", "/repo", "zfs", "tank/monochromatic-git-checkout")]);
    let intent: ScriptedIntent = ScriptedIntent { answer: Ok(IntentReading::Absent) };
    let readings: SlotReadings = read(common.as_path(), &table, &intent);
    assert_eq!(readings.slot, slot());
    assert_eq!(readings.slot_path, slot_path());
    assert_eq!(readings.intent, IntentFacts::Absent);
    assert_eq!(readings.registration_stands, Some(false));
    assert_eq!(classify_slot(&readings), crate::worktree_clone_slot_state::slot_state_classification::SlotRouteState::KnownUnmountedClone { clone_dataset: String::from(CLONE_DATASET) });
    let report = report_of(&readings);
    assert!(report.next_safe_action.contains("nothing under the slot is a person's file"), "{}", report.next_safe_action);
}

/// The clone's own root at the slot makes the same record a mounted clone.
#[test]
fn a_filled_record_with_the_clone_row_is_mounted() {
    let root: PathBuf = fixture("slot-state-live-mounted");
    let common: PathBuf = root.join(".git");
    publish(common.as_path(), &record_at(Phase::Filled, slot_path().as_path()), slot());
    let table: ScriptedMountTable =
        ScriptedMountTable::holding(&[row(40, 1, "/", "/repo", "zfs", "tank/monochromatic-git-checkout"), row(41, 40, "/", slot_path().to_str().expect("utf8 slot"), "zfs", CLONE_DATASET)]);
    let intent: ScriptedIntent = ScriptedIntent { answer: Ok(IntentReading::Absent) };
    let readings: SlotReadings = read(common.as_path(), &table, &intent);
    assert_eq!(classify_slot(&readings), crate::worktree_clone_slot_state::slot_state_classification::SlotRouteState::MountedManagedClone { clone_dataset: String::from(CLONE_DATASET) });
}

/// The mount request standing on the clone is read for a filled record.
#[test]
fn a_mount_request_on_a_filled_clone_is_read() {
    let root: PathBuf = fixture("slot-state-live-intent");
    let common: PathBuf = root.join(".git");
    publish(common.as_path(), &record_at(Phase::Filled, slot_path().as_path()), slot());
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[row(40, 1, "/", "/repo", "zfs", "tank/monochromatic-git-checkout")]);
    let intent: ScriptedIntent = ScriptedIntent { answer: Ok(IntentReading::Request(MountIntent { operation: MountOperation::Mount, nonce: nonce() })) };
    let readings: SlotReadings = read(common.as_path(), &table, &intent);
    assert_eq!(readings.intent, IntentFacts::Mounting { nonce: String::from("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f") });
}

/// A refused property read is the unreadable arm, never an absence.
#[test]
fn a_refused_intent_read_is_reported_as_unreadable() {
    let root: PathBuf = fixture("slot-state-live-intent-refused");
    let common: PathBuf = root.join(".git");
    publish(common.as_path(), &record_at(Phase::Filled, slot_path().as_path()), slot());
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[row(40, 1, "/", "/repo", "zfs", "tank/monochromatic-git-checkout")]);
    let intent: ScriptedIntent = ScriptedIntent { answer: Err(String::from("the compatibility gate refused")) };
    let readings: SlotReadings = read(common.as_path(), &table, &intent);
    match readings.intent {
        IntentFacts::Unreadable { reason } => assert!(reason.contains("gate refused"), "{reason}"),
        found => panic!("expected an unreadable intent, got {found:?}"),
    }
}

/// An add that has not cloned yet does not ask the clone's property at all.
#[test]
fn an_unfinished_add_does_not_ask_the_intent() {
    let root: PathBuf = fixture("slot-state-live-unasked");
    let common: PathBuf = root.join(".git");
    publish(common.as_path(), &record_at(Phase::Snapshotted, slot_path().as_path()), slot());
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[row(40, 1, "/", "/repo", "zfs", "tank/monochromatic-git-checkout")]);
    let intent: ScriptedIntent = ScriptedIntent { answer: Err(String::from("nothing should ask this")) };
    let readings: SlotReadings = read(common.as_path(), &table, &intent);
    assert_eq!(readings.intent, IntentFacts::Unasked);
    assert_eq!(classify_slot(&readings), crate::worktree_clone_slot_state::slot_state_classification::SlotRouteState::PendingAdd { phase: Phase::Snapshotted, clone_dataset: String::from(CLONE_DATASET) });
}

/// The registration the record names is read from the filesystem,
/// so a directory the record names that exists reports it standing.
#[test]
fn a_registration_the_record_names_is_read_from_the_filesystem() {
    let root: PathBuf = fixture("slot-state-live-registration");
    let common: PathBuf = root.join(".git");
    let registration: PathBuf = root.join(".git/worktrees/topic");
    std::fs::create_dir_all(registration.as_path()).expect("the fixture registration");
    let mut record = record_at(Phase::Filled, slot_path().as_path());
    record.registration = Some(std::fs::canonicalize(registration.as_path()).expect("the fixture registration resolves").as_os_str().as_encoded_bytes().to_vec());
    publish(common.as_path(), &record, slot());
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[row(40, 1, "/", "/repo", "zfs", "tank/monochromatic-git-checkout")]);
    let intent: ScriptedIntent = ScriptedIntent { answer: Ok(IntentReading::Absent) };
    let readings: SlotReadings = read(common.as_path(), &table, &intent);
    assert_eq!(readings.registration_stands, Some(true));
}

/// A common directory with no record is the absent arm,
/// and the report says the route holds nothing there.
#[test]
fn no_record_is_read_as_the_absent_arm() {
    let root: PathBuf = fixture("slot-state-live-absent");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the fixture common directory");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[row(40, 1, "/", "/repo", "zfs", "tank/monochromatic-git-checkout")]);
    let intent: ScriptedIntent = ScriptedIntent { answer: Err(String::from("nothing should ask this")) };
    let readings: SlotReadings = read(common.as_path(), &table, &intent);
    assert_eq!(readings.binding, BindingReading::Absent);
    assert_eq!(readings.intent, IntentFacts::Unasked);
    assert_eq!(readings.registration_stands, None);
}
