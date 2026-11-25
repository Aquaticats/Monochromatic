//! What:
//!  Controls for the two-fact proof:
//!  a matching registration,
//!  a `gitdir` naming another path,
//!  a `commondir` naming another repository,
//!  a slot folder whose identity moved,
//!  a registration with no `gitdir`,
//!  a record with no identity to compare,
//!  a mount of the record's clone covering the slot,
//!  a mount of anything else there,
//!  and a mount table that cannot be read.
//! Why:
//!  The proof is what lets a lock be judged after the slot's pointer is gone,
//!  so each fact must fail on its own and none may pass by another's agreement.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(proveTheRegistration(facts, repository, registration)).toBeUndefined();
//! ```

use super::{TwoFactRefusal, prove_the_registration};
use crate::worktree_clone_managed_lock::test_fixtures::{Fixture, RecordSpec, ScriptedMounts};
use crate::worktree_clone_managed_lock::read_the_recorded_facts;
use std::path::PathBuf;

/// The fixture's nonce.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// A fixture whose record holds the slot's own identity and names one registration.
fn a_recorded_fixture(label: &str) -> (Fixture, PathBuf) {
    let fixture: Fixture = Fixture::new(label);
    let registration: PathBuf = fixture.write_registration("entry");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    spec.registration = Some(registration.clone());
    fixture.write_record(&spec);
    return (fixture, registration);
}

/// The proof holds when the `gitdir` names this slot's `.git`,
/// the `commondir` names this repository,
/// and the folder at the recorded path is the recorded one.
#[test]
fn the_proof_holds_for_its_own_registration() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-holds");
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let mounts: ScriptedMounts = ScriptedMounts::nothing();
    assert_eq!(prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts), Ok(()));
}

/// A `gitdir` naming another worktree is the proof's own refusal,
/// with both paths in it.
#[test]
fn a_gitdir_naming_another_path_is_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-other-path");
    std::fs::write(registration.join("gitdir"), format!("{}\n", fixture.root.join("elsewhere/.git").display()).as_bytes()).expect("the back-link is rewritten");
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let mounts: ScriptedMounts = ScriptedMounts::nothing();
    let refused: TwoFactRefusal = prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts).expect_err("the proof refuses another path");
    assert!(matches!(refused, TwoFactRefusal::GitdirNamesAnotherPath { .. }), "the refusal names the path it compared: {refused:?}");
}

/// A relative `gitdir` is resolved against the registration,
/// which is the spelling another command may write.
#[test]
fn a_relative_gitdir_is_resolved() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-relative");
    let relative: PathBuf = pathdiff(fixture.slot_path.join(".git").as_path(), registration.as_path());
    std::fs::write(registration.join("gitdir"), format!("{}\n", relative.display()).as_bytes()).expect("the back-link is written relative");
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let mounts: ScriptedMounts = ScriptedMounts::nothing();
    assert_eq!(prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts), Ok(()));
}

/// A `commondir` naming another repository is refused.
#[test]
fn a_commondir_naming_another_repository_is_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-other-repository");
    let elsewhere: PathBuf = fixture.root.join("elsewhere");
    std::fs::create_dir_all(elsewhere.as_path()).expect("another repository is made");
    std::fs::write(registration.join("commondir"), format!("{}\n", elsewhere.display()).as_bytes()).expect("the common directory's record is rewritten");
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let mounts: ScriptedMounts = ScriptedMounts::nothing();
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts),
        Err(TwoFactRefusal::CommondirNamesAnotherRepository { .. })
    ));
}

/// A folder at the recorded slot path whose identity moved is not the recorded folder,
/// which is how a mount covering the slot refuses rather than passing.
#[test]
fn a_slot_folder_whose_identity_moved_is_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-identity-moved");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(String::from("1:2:3"));
    spec.registration = Some(registration.clone());
    fixture.write_record(&spec);
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let mounts: ScriptedMounts = ScriptedMounts::nothing();
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts),
        Err(TwoFactRefusal::TheSlotIsNotTheRecordedFolder { .. })
    ));
}

/// A registration with no `gitdir` proves nothing,
/// and a record with no identity cannot compare the folder.
#[test]
fn a_missing_gitdir_and_a_missing_identity_are_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-missing-facts");
    std::fs::remove_file(registration.join("gitdir")).expect("the back-link goes");
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let mounts: ScriptedMounts = ScriptedMounts::nothing();
    assert_eq!(prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts), Err(TwoFactRefusal::NoGitdir));
    fixture.write_registration("entry");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.registration = Some(registration.clone());
    fixture.write_record(&spec);
    let without: crate::worktree_clone_managed_lock::RecordedLockFacts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    assert_eq!(
        prove_the_registration(&without, fixture.common_dir.as_path(), registration.as_path(), &mounts),
        Err(TwoFactRefusal::NoRecordedSlotIdentity)
    );
}

/// What:
///  The mounted arm of measurement `M2`:
///  while a mount of the clone the record names covers the slot,
///  the recorded underlay identity no longer answers there,
///  and the proof passes through the mount instead of refusing.
/// Why:
///  A mounted clone's slot path is the clone's root,
///  so comparing `slotIdentity=` there would call this slot another folder,
///  and the lock of a mounted clone would read as a stranger's.
#[test]
fn a_mount_of_the_records_clone_proves_the_registration() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-mounted");
    // The identity the record holds is the underlay's, which the mount now hides.
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(String::from("1:2:3"));
    spec.registration = Some(registration.clone());
    fixture.write_record(&spec);
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let mounts: ScriptedMounts = ScriptedMounts::a_mount_of(facts.clone.dataset.as_str(), fixture.slot_path.as_path());
    assert_eq!(prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts), Ok(()));
    // The positive control: without the mount the same record is refused at the same path,
    // so the arm above sees a difference the covered slot makes.
    let uncovered: ScriptedMounts = ScriptedMounts::nothing();
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &uncovered),
        Err(TwoFactRefusal::TheSlotIsNotTheRecordedFolder { .. })
    ));
}

/// A mount of anything else at the slot is refused,
/// and so is a mount table that cannot be read.
/// Why:
///  A bind mount or another dataset standing at the slot says nothing about this slot's clone,
///  and a table nothing read proves neither the mounted nor the unmounted fact.
#[test]
fn a_mount_of_something_else_and_an_unreadable_table_are_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-mount-others");
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let other: ScriptedMounts = ScriptedMounts::a_mount_of("pool/clones/another", fixture.slot_path.as_path());
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &other),
        Err(TwoFactRefusal::AMountThatIsNotTheRecords { .. })
    ));
    let unreadable: ScriptedMounts = ScriptedMounts::unreadable("the table could not be read");
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &unreadable),
        Err(TwoFactRefusal::TheMountTableWasNotRead { .. })
    ));
}

/// What:
///  One path spelled relative to another,
///  for the control that plants a relative `gitdir`.
/// Why:
///  Git itself may write either spelling,
///  and this control needs an honest relative one rather than a hand-counted run of `..`.
fn pathdiff(target: &std::path::Path, from: &std::path::Path) -> PathBuf {
    let mut up: PathBuf = PathBuf::new();
    for _component in from.components() {
        up.push("..");
    }
    return up.join(target.strip_prefix("/").unwrap_or(target));
}
