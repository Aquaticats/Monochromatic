//! What:
//!  Controls for the walk of a slot's candidate names:
//!  one complete entry adopted,
//!  a bare entry cleared,
//!  two entries that both attribute themselves,
//!  a name another slot's binding owns,
//!  and a walk that finds nothing.
//! Why:
//!  The finder's rows are the binding design's,
//!  and this route must not read a stale counter entry,
//!  another slot's registration or a name Git could not derive as this slot's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(findTheSlotRegistration(commonDir, slot, slotPath, nonce).registration.kind).toBe('one');
//! ```

use super::{TheRegistration, find_the_slot_registration, the_slots_the_home_names};
use crate::worktree_clone_binding_record::registration_resume::EntryDecision;
use crate::worktree_clone_managed_lock::test_fixtures::{Fixture, RecordSpec};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::path::PathBuf;

/// The fixture's nonce.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The nonce the parser reads.
fn nonce() -> RequestNonce {
    return RequestNonce::parse(NONCE).expect("the fixture nonce reads");
}

/// A complete entry with the add's own lock is adopted.
#[test]
fn a_complete_entry_is_adopted() {
    let fixture: Fixture = Fixture::new("finding-adopt");
    let registration: PathBuf = fixture.write_registration("locked-slot");
    fixture.write_pointer(registration.as_path());
    fixture.write_lock(registration.as_path(), fixture.the_sentence(NONCE).as_slice());
    let finding = find_the_slot_registration(fixture.common_dir.as_path(), fixture.slot.as_str(), fixture.slot_path.as_path(), &nonce());
    assert_eq!(finding.registration, TheRegistration::One { name: String::from("locked-slot"), path: registration, decision: EntryDecision::Adopt { complete_the_lock: false } });
    assert_eq!(finding.names_seen, vec![String::from("locked-slot")]);
}

/// A bare directory at the slot's name,
/// with an empty slot, is the row the add's resume clears.
#[test]
fn a_bare_entry_is_cleared() {
    let fixture: Fixture = Fixture::new("finding-bare");
    std::fs::create_dir_all(fixture.worktrees().join("locked-slot")).expect("the bare registration is made");
    let finding = find_the_slot_registration(fixture.common_dir.as_path(), fixture.slot.as_str(), fixture.slot_path.as_path(), &nonce());
    assert_eq!(
        finding.registration,
        TheRegistration::One { name: String::from("locked-slot"), path: fixture.worktrees().join("locked-slot"), decision: EntryDecision::Clear { row: crate::worktree_clone_binding_record::EntryRow::Bare } }
    );
}

/// Two entries carrying one add's sentence are a refusal,
/// because only a copy could make two.
#[test]
fn two_attributing_entries_are_named() {
    let fixture: Fixture = Fixture::new("finding-two");
    let first: PathBuf = fixture.write_registration("locked-slot");
    fixture.write_pointer(first.as_path());
    fixture.write_lock(first.as_path(), fixture.the_sentence(NONCE).as_slice());
    let second: PathBuf = fixture.write_registration("locked-slot1");
    fixture.write_lock(second.as_path(), fixture.the_sentence(NONCE).as_slice());
    let finding = find_the_slot_registration(fixture.common_dir.as_path(), fixture.slot.as_str(), fixture.slot_path.as_path(), &nonce());
    assert_eq!(finding.registration, TheRegistration::Two { names: vec![String::from("locked-slot"), String::from("locked-slot1")] });
}

/// A name another slot's own binding owns is left alone,
/// and the home folder's listing is what says so.
#[test]
fn a_name_another_slot_owns_is_left_alone() {
    let fixture: Fixture = Fixture::new("finding-another-slot");
    let mut other: RecordSpec = RecordSpec::for_nonce("00000000000000000000000000000000");
    other.clone_dataset = String::from("pool/clones/topic1");
    let home: PathBuf = fixture.home();
    std::fs::create_dir_all(home.as_path()).expect("the record folder is made");
    std::fs::write(home.join("topic1.binding"), b"# a record of another slot\n").expect("another slot's record is written");
    assert_eq!(the_slots_the_home_names(fixture.common_dir.as_path()), vec![String::from("topic1")]);
    assert!(matches!(find_the_slot_registration(fixture.common_dir.as_path(), "topic", fixture.slot_path.as_path(), &nonce()).registration, TheRegistration::None));
}

/// A walk that finds nothing says so,
/// which is the state S6.
#[test]
fn a_walk_that_finds_nothing_answers_none() {
    let fixture: Fixture = Fixture::new("finding-none");
    let finding = find_the_slot_registration(fixture.common_dir.as_path(), fixture.slot.as_str(), fixture.slot_path.as_path(), &nonce());
    assert_eq!(finding.registration, TheRegistration::None);
    assert!(finding.names_seen.is_empty());
}
