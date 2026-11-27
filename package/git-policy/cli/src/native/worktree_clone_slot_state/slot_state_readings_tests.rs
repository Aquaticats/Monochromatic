//! Controls for the slot's readings:
//!  no record,
//!  a readable record,
//!  a record no reader may act on,
//!  a plan that names the clone,
//!  an unreadable plan,
//!  every intent arm,
//!  and an inherited intent that authorizes nothing.

use crate::testing::test_support::fixture;
use crate::testing::the_destruction_plan_a_control_plants::{
    ACloneAPlanNames,
    APlanAControlPlants,
    plant,
};
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_mount_protocol::RequestNonce;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{
    IntentReading,
    IntentRefusal,
    MountIntent,
    MountOperation,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    IntentFacts,
    PlanReading,
    SlotReadings,
    intent_facts_of,
    read_binding_reading,
    read_plan,
};
use crate::worktree_clone_slot_state::slot_state_test_records::{
    publish,
    record_at,
};
use std::path::{
    Path,
    PathBuf,
};

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
    return RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
        .expect("the fixture nonce reads");
}

/// No record and a record the grammar accepts are told apart by the reader.
#[test]
fn a_missing_record_is_an_absence_and_a_written_one_is_read() {
    let root: PathBuf = fixture("slot-state-readings");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the fixture common directory");
    let home: PathBuf =
        crate::worktree_clone_binding_record::registration_home::home_folder(common.as_path());
    std::fs::create_dir_all(home.as_path()).expect("the fixture home");
    assert_eq!(
        read_binding_reading(home.as_path(), slot()),
        BindingReading::Absent
    );
    publish(
        common.as_path(),
        &record_at(Phase::Filled, slot_path().as_path()),
        slot(),
    );
    let read: BindingReading = read_binding_reading(home.as_path(), slot());
    match &read {
        BindingReading::Record(record) => assert_eq!(record.phase, Phase::Filled),
        found => panic!("expected the written record, got {found:?}"),
    }
}

/// A file under the record's name that is not a record refuses with its reason.
#[test]
fn a_record_no_reader_may_act_on_names_why() {
    let root: PathBuf = fixture("slot-state-unreadable-record");
    let common: PathBuf = root.join(".git");
    let home: PathBuf =
        crate::worktree_clone_binding_record::registration_home::home_folder(common.as_path());
    std::fs::create_dir_all(home.as_path()).expect("the fixture home");
    std::fs::write(home.join("topic.binding"), b"not a record\n").expect("the foreign file writes");
    let read: BindingReading = read_binding_reading(home.as_path(), slot());
    match &read {
        BindingReading::Unreadable {
            reason,
        } => assert!(reason.contains("topic.binding"), "{reason}"),
        found => panic!("expected an unreadable record, got {found:?}"),
    }
}

/// A plan filed under the clone's GUID names the clone.
#[test]
fn a_plan_under_the_clone_guid_names_it() {
    let root: PathBuf = fixture("slot-state-plan-names");
    let guid =
        crate::worktree_clone_slot_state::slot_state_test_records::guid("4821193048571903102");
    let planted_plan: APlanAControlPlants =
        APlanAControlPlants::a_removals_first_write(ACloneAPlanNames::a_fixture_clone(
            root.as_path(),
            crate::worktree_clone_slot_state::slot_state_test_records::CLONE_DATASET,
            guid,
        ));
    let planted: PathBuf = plant(&planted_plan);
    let reading: PlanReading = read_plan(
        root.as_path(),
        Some(guid),
        crate::worktree_clone_slot_state::slot_state_test_records::CLONE_DATASET,
    );
    match &reading {
        PlanReading::NamesTheClone {
            plan,
        } => assert_eq!(plan, &planted),
        found => panic!("expected the plan to name the clone, got {found:?}"),
    }
}

/// A plan folder holding a file that is not a plan refuses rather than reading finding nothing.
#[test]
fn an_unreadable_plan_names_the_file() {
    let root: PathBuf = fixture("slot-state-plan-unreadable");
    let plans: PathBuf =
        crate::worktree_clone_binding_record::registration_home::plans_folder(root.as_path());
    std::fs::create_dir_all(plans.as_path()).expect("the fixture plan folder");
    let foreign: PathBuf = plans.join("4821193048571903102");
    std::fs::write(foreign.as_path(), b"someone's notes\n").expect("the foreign file writes");
    let reading: PlanReading = read_plan(
        root.as_path(),
        None,
        crate::worktree_clone_slot_state::slot_state_test_records::CLONE_DATASET,
    );
    match &reading {
        PlanReading::Unreadable {
            plan,
            ..
        } => assert_eq!(plan, &foreign),
        found => panic!("expected the unreadable plan, got {found:?}"),
    }
}

/// A folder with no plan folder is no plan at all.
#[test]
fn an_absent_plan_folder_is_no_plan() {
    let root: PathBuf = fixture("slot-state-plan-absent");
    assert_eq!(
        read_plan(
            root.as_path(),
            None,
            crate::worktree_clone_slot_state::slot_state_test_records::CLONE_DATASET
        ),
        PlanReading::NoPlan
    );
}

/// Every intent arm keeps the request's operation and nonce or its refusal.
#[test]
fn each_intent_arm_is_read_as_its_facts() {
    let mounting: IntentReading = IntentReading::Request(MountIntent {
        operation: MountOperation::Mount,
        nonce: nonce(),
    });
    assert_eq!(
        intent_facts_of(&mounting),
        IntentFacts::Mounting {
            nonce: String::from("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
        }
    );
    let unmounting: IntentReading = IntentReading::Request(MountIntent {
        operation: MountOperation::Unmount,
        nonce: nonce(),
    });
    assert_eq!(
        intent_facts_of(&unmounting),
        IntentFacts::Unmounting {
            nonce: String::from("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
        }
    );
    assert_eq!(intent_facts_of(&IntentReading::Absent), IntentFacts::Absent);
    assert_eq!(
        intent_facts_of(&IntentReading::Inherited {
            from: String::from("tank")
        }),
        IntentFacts::Inherited {
            from: String::from("tank")
        }
    );
    let refused: IntentReading = IntentReading::Refused(IntentRefusal::UnknownOperation {
        word: String::from("park"),
    });
    match intent_facts_of(&refused) {
        IntentFacts::Unreadable {
            reason,
        } => assert!(reason.contains("park"), "{reason}"),
        found => panic!("expected an unreadable intent, got {found:?}"),
    }
}

/// A reading with no record says nothing about a registration.
#[test]
fn a_reading_without_a_record_names_no_registration() {
    let readings: SlotReadings = SlotReadings {
        slot: String::from(slot()),
        slot_path: slot_path(),
        binding: BindingReading::Absent,
        plan: PlanReading::NoPlan,
        intent: IntentFacts::Unasked,
        mounts: crate::worktree_clone_slot_state::slot_state_mounts::MountRows::Rows(Vec::new()),
        slot_mount: crate::worktree_clone_slot_state::slot_state_mounts::SlotMount::Nothing,
        registration_stands: None,
    };
    assert_eq!(readings.registration_stands, None);
    assert!(Path::new(readings.slot_path.as_path()).ends_with(slot()));
}
