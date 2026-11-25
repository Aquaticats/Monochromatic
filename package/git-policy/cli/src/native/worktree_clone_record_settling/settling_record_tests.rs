//! Controls for the binding read and the two confirmations:
//!  a published record reads with its own identities,
//!  a slot typed for another record refuses,
//!  digits that are not the record's own refuse,
//!  a record holding no clone names the add again,
//!  and a missing record refuses by name.

use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_record_settling::settling_fixtures::{SLOT, TheFixture, binding_path, publish_the_record, the_fixture};
use crate::worktree_clone_record_settling::settling_record::{confirm_the_record, read_the_record};
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_request::SettlingRequest;

/// One request, as the design's line gives it.
fn request(slot: &str, digits: &str) -> SettlingRequest {
    return SettlingRequest { slot: String::from(slot), confirmed_slot: String::from(slot), digits: String::from(digits) };
}

/// A record of any phase holding a clone reads with the identity the fixture published.
#[test]
fn a_published_record_reads() {
    let made: TheFixture = the_fixture("settling-record-reads");
    publish_the_record(&made, Phase::Filled);
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    assert_eq!(record.phase, Phase::Filled);
    assert_eq!(record.recorded.clone_guid, crate::worktree_clone_record_settling::settling_fixtures::a_guid(crate::worktree_clone_record_settling::settling_fixtures::CLONE_GUID));
    assert_eq!(record.recorded.slot_path, made.slot_path);
    confirm_the_record(&record.recorded, &request(SLOT, "3102")).expect("the record's own values confirm it");
}

/// A record filed under this slot whose own `slot=` derives another name refuses,
/// so a slot typed for another record never settles this one.
#[test]
fn a_record_of_another_slot_name_refuses() {
    let made: TheFixture = the_fixture("settling-record-other-slot");
    publish_the_record(&made, Phase::Filled);
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    let refused = confirm_the_record(&record.recorded, &request("another-slot", "3102")).expect_err("another slot refuses");
    assert_eq!(refused, SettlingStop::TheSlotIsNotThisRecord { asked: String::from("another-slot"), recorded: String::from(SLOT) });
    assert!(refused.sentence().contains("this is not `another-slot`'s record"), "{}", refused.sentence());
}

/// Digits that are not the record's own refuse,
/// and the sentence prints the digits the record holds.
#[test]
fn the_digits_must_be_the_records_own() {
    let made: TheFixture = the_fixture("settling-record-wrong-digits");
    publish_the_record(&made, Phase::Filled);
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    let refused = confirm_the_record(&record.recorded, &request(SLOT, "3103")).expect_err("other digits refuse");
    assert_eq!(refused, SettlingStop::TheCloneGuidDiffers { typed: String::from("3103"), wanted: String::from("3102") });
    assert!(refused.sentence().contains("`3102`"), "{}", refused.sentence());
}

/// A record the grammar lets stand without a clone names the add again.
#[test]
fn a_record_without_a_clone_names_the_add() {
    for phase in [Phase::Registered, Phase::Snapshotted] {
        let made: TheFixture = the_fixture(&format!("settling-record-no-clone-{}", phase.word()));
        let mut record = crate::worktree_clone_record_settling::settling_fixtures::the_record(&made, phase);
        record.clone_guid = None;
        record.clone_createtxg = None;
        let home: std::fs::File = crate::worktree_clone_binding_record::registration_home::open_or_create_home(made.common.as_path()).expect("the home opens");
        crate::worktree_clone_binding_record::record_publication::write_binding(&home, SLOT, &record).expect("the record writes");
        let refused = read_the_record(made.common.as_path(), SLOT).expect_err("no clone refuses");
        assert_eq!(refused, SettlingStop::NoCloneRecorded { slot: String::from(SLOT), phase: String::from(phase.word()) });
        assert!(refused.sentence().contains("run the add again"), "{}", refused.sentence());
    }
}

/// A `settling` record holding no clone GUID cannot stand at all:
/// the record's own grammar requires `cloneGuid` there,
/// so the reader refuses it before this command sees it,
/// and that refusal is the project's answer for a record that cannot be read.
#[test]
fn a_settling_record_without_a_clone_is_unreadable() {
    let made: TheFixture = the_fixture("settling-record-no-clone-settling");
    let mut record = crate::worktree_clone_record_settling::settling_fixtures::the_record(&made, Phase::Settling);
    record.clone_guid = None;
    record.clone_createtxg = None;
    let home: std::fs::File = crate::worktree_clone_binding_record::registration_home::open_or_create_home(made.common.as_path()).expect("the home opens");
    crate::worktree_clone_binding_record::record_publication::write_binding(&home, SLOT, &record).expect("the record writes");
    let refused = read_the_record(made.common.as_path(), SLOT).expect_err("the grammar refuses it");
    let SettlingStop::RecordUnreadable { said, .. } = &refused else { panic!("the grammar's refusal is the unreadable arm, and this answered {refused:?}") };
    assert!(said.contains("holds no `cloneGuid` field"), "{said}");
    assert!(said.contains("run the command again"), "{said}");
}

/// No record at all refuses by the file's own name.
#[test]
fn a_missing_record_refuses_by_name() {
    let made: TheFixture = the_fixture("settling-record-missing");
    let refused = read_the_record(made.common.as_path(), SLOT).expect_err("no record refuses");
    let SettlingStop::RecordUnreadable { path, said } = &refused else { panic!("the missing file is the unreadable arm, and this answered {refused:?}") };
    assert_eq!(path, &binding_path(&made));
    assert!(said.contains("does not exist"), "{said}");
}
