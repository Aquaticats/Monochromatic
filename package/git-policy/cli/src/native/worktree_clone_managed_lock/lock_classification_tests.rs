//! What:
//!  Controls for the one classification:
//!  the wrapper's verified lock,
//!  its cut-short twin,
//!  a person's lock,
//!  no lock beside a standing registration,
//!  no pointer,
//!  no registration,
//!  a registration nothing explains,
//!  a half-made one,
//!  and another worktree's.
//! Why:
//!  This is the function `doctor` and the removal will call,
//!  so every host control here is one class of the prompt's three,
//!  built from a real file tree,
//!  and a state that read as another would send a removal at a lock it may not release.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(classifyTheManagedLock(commonDir, slot, slotPath).state.kind).toBe('managed');
//! ```

use super::classify_the_managed_lock;
use crate::worktree_clone_managed_lock::test_fixtures::{Fixture, RecordSpec};
use crate::worktree_clone_managed_lock::{LockOwnership, ManagedLockState, TheWordsCause};
use crate::worktree_clone_binding_record::EntryRow;
use std::path::PathBuf;

/// The fixture's nonce.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The other add's nonce,
/// which a copied lock would carry.
const OTHER_NONCE: &str = "00000000000000000000000000000000";

/// What:
///  A slot the add finished:
///  a record naming its registration and identity,
///  the registration with its three files,
///  and the slot's pointer,
///  with the lock the caller plants.
/// Why:
///  Every state below is this fixture with one fact moved,
///  so each control proves the fact it names rather than the fixture's shape.
fn a_finished_slot(label: &str, lock: Option<Vec<u8>>) -> (Fixture, PathBuf) {
    let fixture: Fixture = Fixture::new(label);
    let registration: PathBuf = fixture.write_registration("locked-slot");
    fixture.write_pointer(registration.as_path());
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    spec.registration = Some(registration.clone());
    spec.registration_identity = Some(String::from("42:7:8"));
    fixture.write_record(&spec);
    if let Some(bytes) = lock {
        fixture.write_lock(registration.as_path(), bytes.as_slice());
    }
    return (fixture, registration);
}

/// The class of one fixture's classification.
fn ownership_of(fixture: &Fixture) -> LockOwnership {
    return classify_the_managed_lock(fixture.common_dir.as_path(), fixture.slot.as_str(), fixture.slot_path.as_path()).ownership();
}

/// The state of one fixture's classification.
fn state_of(fixture: &Fixture) -> ManagedLockState {
    return classify_the_managed_lock(fixture.common_dir.as_path(), fixture.slot.as_str(), fixture.slot_path.as_path()).state;
}

/// S1: the sentence the record's nonce recomposes is the wrapper's managed lock,
/// wherever the pointer stands or not.
#[test]
fn the_verified_lock_is_the_wrappers_managed_class() {
    let (fixture, registration): (Fixture, PathBuf) = a_finished_slot("classify-verified", Some(Vec::new()));
    fixture.write_lock(registration.as_path(), fixture.the_sentence(NONCE).as_slice());
    assert_eq!(state_of(&fixture), ManagedLockState::Managed { nonce: crate::worktree_clone_mount_protocol::RequestNonce::parse(NONCE).expect("the nonce reads"), registration });
    assert_eq!(ownership_of(&fixture), LockOwnership::TheWrappersManaged);
}

/// S2: a covering prefix of the sentence is still the wrapper's own lock,
/// and the removal may complete it.
#[test]
fn the_cut_short_lock_is_still_the_wrappers() {
    let (fixture, registration): (Fixture, PathBuf) = a_finished_slot("classify-cut-short", None);
    let sentence: Vec<u8> = fixture.the_sentence(NONCE);
    let covering: usize = crate::worktree_clone_binding_record::LOCK_WORDS_PREFIX.len() + 32 + 1;
    fixture.write_lock(registration.as_path(), &sentence[..covering]);
    assert_eq!(
        state_of(&fixture),
        ManagedLockState::CutShort { nonce: crate::worktree_clone_mount_protocol::RequestNonce::parse(NONCE).expect("the nonce reads"), registration, at: covering as u64, of: sentence.len() as u64 }
    );
    assert_eq!(ownership_of(&fixture), LockOwnership::TheWrappersManaged);
}

/// A person's words are the operator class,
/// and the sentence says which words they are.
#[test]
fn a_persons_lock_is_the_operator_class() {
    let (fixture, _registration): (Fixture, PathBuf) = a_finished_slot("classify-operator", Some(b"initializing\n".to_vec()));
    assert_eq!(state_of(&fixture), ManagedLockState::NotTheAddsWords { cause: TheWordsCause::AnothersWords { first_line: String::from("initializing") } });
    assert_eq!(ownership_of(&fixture), LockOwnership::AnOperatorsLock);
}

/// An empty lock is a person's too,
/// which is the deliberate cost of a proof that compares bytes.
#[test]
fn an_empty_lock_is_the_operator_class() {
    let (fixture, _registration): (Fixture, PathBuf) = a_finished_slot("classify-empty", Some(Vec::new()));
    assert!(matches!(state_of(&fixture), ManagedLockState::NotTheAddsWords { cause: TheWordsCause::AnothersWords { .. } }));
    assert_eq!(ownership_of(&fixture), LockOwnership::AnOperatorsLock);
}

/// Another add's nonce is not this add's lock,
/// so a copied sentence is nobody's managed lock here.
#[test]
fn another_adds_sentence_is_the_operator_class() {
    let (fixture, registration): (Fixture, PathBuf) = a_finished_slot("classify-copied", None);
    fixture.write_lock(registration.as_path(), fixture.the_sentence(OTHER_NONCE).as_slice());
    assert_eq!(ownership_of(&fixture), LockOwnership::AnOperatorsLock);
}

/// A person's `git worktree lock --reason` carrying the wrapper's own sentence reads as the managed lock,
/// which is the deliberate cost of a proof that compares bytes rather than identity (`V5`).
#[test]
fn a_person_who_writes_the_sentence_makes_the_managed_state() {
    let (fixture, registration): (Fixture, PathBuf) = a_finished_slot("classify-a-copied-sentence", None);
    fixture.write_lock(registration.as_path(), fixture.the_sentence(NONCE).as_slice());
    assert!(matches!(state_of(&fixture), ManagedLockState::Managed { .. }), "the words are all the proof reads");
    assert_eq!(ownership_of(&fixture), LockOwnership::TheWrappersManaged);
}

/// A record that gave no nonce recomposes nothing,
/// so every lock is the operator class with that cause named.
#[test]
fn a_slot_with_no_record_cannot_verify_any_lock() {
    let fixture: Fixture = Fixture::new("classify-no-record");
    let registration: PathBuf = fixture.write_registration("locked-slot");
    fixture.write_pointer(registration.as_path());
    fixture.write_lock(registration.as_path(), fixture.the_sentence(NONCE).as_slice());
    assert!(matches!(state_of(&fixture), ManagedLockState::NotTheAddsWords { cause: TheWordsCause::NoRecordToRecompose { .. } }));
    assert_eq!(ownership_of(&fixture), LockOwnership::AnOperatorsLock);
}

/// S4: a standing registration with no lock is the state the put-back answers,
/// and the pointer may still stand.
#[test]
fn a_standing_registration_with_no_lock_is_the_unlocked_state() {
    let (fixture, registration): (Fixture, PathBuf) = a_finished_slot("classify-unlocked", None);
    assert_eq!(state_of(&fixture), ManagedLockState::NoLock { nonce: crate::worktree_clone_mount_protocol::RequestNonce::parse(NONCE).expect("the nonce reads"), registration });
    assert_eq!(ownership_of(&fixture), LockOwnership::NoLockStands);
}

/// S5: with the clone unmounted and its pointer deleted,
/// the lock still reads as the wrapper's own through the two-fact proof,
/// and the missing pointer is a second sentence.
#[test]
fn the_lock_survives_the_lost_pointer() {
    let (fixture, registration): (Fixture, PathBuf) = a_finished_slot("classify-no-pointer", Some(Vec::new()));
    fixture.write_lock(registration.as_path(), fixture.the_sentence(NONCE).as_slice());
    std::fs::remove_file(fixture.slot_path.join(".git")).expect("the pointer goes, as item 6's arm deletes it");
    let classification = classify_the_managed_lock(fixture.common_dir.as_path(), fixture.slot.as_str(), fixture.slot_path.as_path());
    assert!(matches!(classification.state, ManagedLockState::Managed { .. }), "the lock still reads as the add's: {:?}", classification.state);
    assert!(classification.the_slot_shows_no_pointer());
    assert_eq!(crate::worktree_clone_managed_lock::the_lines(&classification, fixture.slot.as_str()).len(), 2);
    assert_eq!(classification.ownership(), LockOwnership::TheWrappersManaged);
}

/// S6: a record that names a registration which is gone,
/// with no entry the finder can attribute, is the state nothing can restore.
#[test]
fn a_registration_the_record_names_and_nothing_holds_is_gone() {
    let fixture: Fixture = Fixture::new("classify-gone");
    let gone: PathBuf = fixture.worktrees().join("locked-slot");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    spec.registration = Some(gone.clone());
    fixture.write_record(&spec);
    assert_eq!(state_of(&fixture), ManagedLockState::NoRegistration { nonce: crate::worktree_clone_mount_protocol::RequestNonce::parse(NONCE).expect("the nonce reads"), recorded: Some(gone) });
    assert_eq!(ownership_of(&fixture), LockOwnership::NoRegistrationStands);
}

/// S7: a registration whose folder is not the recorded one explains nothing,
/// and neither do two entries carrying one sentence.
#[test]
fn a_registration_the_record_cannot_explain_is_unexplained() {
    let moved: Fixture = Fixture::new("classify-moved-folder");
    let registration: PathBuf = moved.write_registration("locked-slot");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(String::from("1:2:3"));
    spec.registration = Some(registration.clone());
    moved.write_record(&spec);
    moved.write_lock(registration.as_path(), moved.the_sentence(NONCE).as_slice());
    assert!(matches!(state_of(&moved), ManagedLockState::NoBindingExplains { .. }));
    assert_eq!(ownership_of(&moved), LockOwnership::Unexplained);
    let twice: Fixture = Fixture::new("classify-two-entries");
    let first: PathBuf = twice.write_registration("locked-slot");
    twice.write_lock(first.as_path(), twice.the_sentence(NONCE).as_slice());
    let second: PathBuf = twice.write_registration("locked-slot1");
    twice.write_lock(second.as_path(), twice.the_sentence(NONCE).as_slice());
    let mut unnamed: RecordSpec = RecordSpec::for_nonce(NONCE);
    unnamed.slot_identity = Some(twice.slot_identity());
    twice.write_record(&unnamed);
    assert!(matches!(state_of(&twice), ManagedLockState::NoBindingExplains { .. }));
}

/// S8: a bare registration at the slot's name is the residue the add's own resume clears.
#[test]
fn a_half_made_registration_is_its_own_state() {
    let fixture: Fixture = Fixture::new("classify-half-made");
    std::fs::create_dir_all(fixture.worktrees().join("locked-slot")).expect("the bare registration is made");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    fixture.write_record(&spec);
    assert_eq!(state_of(&fixture), ManagedLockState::HalfMade { nonce: crate::worktree_clone_mount_protocol::RequestNonce::parse(NONCE).expect("the nonce reads"), name: String::from("locked-slot"), row: EntryRow::Bare });
    assert_eq!(ownership_of(&fixture), LockOwnership::Unexplained);
}

/// S9: a registration whose `gitdir` names another worktree is somebody else's,
/// and its lock is none of this slot's business.
#[test]
fn another_worktrees_registration_is_its_own_state() {
    let (fixture, registration): (Fixture, PathBuf) = a_finished_slot("classify-another-worktree", Some(Vec::new()));
    fixture.write_lock(registration.as_path(), fixture.the_sentence(NONCE).as_slice());
    std::fs::write(registration.join("gitdir"), format!("{}\n", fixture.root.join("elsewhere/.git").display()).as_bytes()).expect("the back-link is rewritten");
    assert!(matches!(state_of(&fixture), ManagedLockState::SomebodyElsesWorktree { .. }));
    assert_eq!(ownership_of(&fixture), LockOwnership::Unexplained);
}

/// A stale counter entry is found by the finder's candidates,
/// because a killed add makes Git name the next one `<name>1`.
#[test]
fn a_counter_entry_is_found_beside_the_slots_name() {
    let fixture: Fixture = Fixture::new("classify-counter");
    let registration: PathBuf = fixture.write_registration("locked-slot1");
    fixture.write_pointer(registration.as_path());
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    fixture.write_record(&spec);
    fixture.write_lock(registration.as_path(), fixture.the_sentence(NONCE).as_slice());
    assert!(matches!(state_of(&fixture), ManagedLockState::Managed { .. }));
}
