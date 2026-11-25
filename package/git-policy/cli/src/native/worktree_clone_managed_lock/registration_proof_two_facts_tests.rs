//! What:
//!  Controls for the two-fact proof:
//!  a matching registration,
//!  a `gitdir` naming another path,
//!  a `commondir` naming another repository,
//!  a slot folder whose identity moved,
//!  a registration with no `gitdir`,
//!  a record with no identity to compare,
//!  a mount of the record's clone covering the slot,
//!  a replacement dataset under the record's clone name,
//!  a GUID the pool does not answer,
//!  a mounted clone whose record holds no GUID,
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
use crate::worktree_clone_managed_lock::test_fixtures::{Fixture, RecordSpec, ScriptedGuids, ScriptedMounts};
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
    let guids: ScriptedGuids = ScriptedGuids::never_asked();
    assert_eq!(prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids), Ok(()));
}

/// A `gitdir` naming another worktree is the proof's own refusal,
/// with both paths in it.
#[test]
fn a_gitdir_naming_another_path_is_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-other-path");
    std::fs::write(registration.join("gitdir"), format!("{}\n", fixture.root.join("elsewhere/.git").display()).as_bytes()).expect("the back-link is rewritten");
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let mounts: ScriptedMounts = ScriptedMounts::nothing();
    let guids: ScriptedGuids = ScriptedGuids::never_asked();
    let refused: TwoFactRefusal = prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids).expect_err("the proof refuses another path");
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
    let guids: ScriptedGuids = ScriptedGuids::never_asked();
    assert_eq!(prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids), Ok(()));
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
    let guids: ScriptedGuids = ScriptedGuids::never_asked();
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids),
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
    let guids: ScriptedGuids = ScriptedGuids::never_asked();
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids),
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
    let guids: ScriptedGuids = ScriptedGuids::never_asked();
    assert_eq!(prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids), Err(TwoFactRefusal::NoGitdir));
    fixture.write_registration("entry");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.registration = Some(registration.clone());
    fixture.write_record(&spec);
    let without: crate::worktree_clone_managed_lock::RecordedLockFacts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    assert_eq!(
        prove_the_registration(&without, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids),
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
    let recorded: crate::worktree_clone_mount_protocol::DatasetGuid = facts.clone.guid.expect("the fixture's record names the clone's GUID");
    let guids: ScriptedGuids = ScriptedGuids::answering(facts.clone.dataset.as_str(), recorded.file_name().as_str());
    let mounts: ScriptedMounts = ScriptedMounts::a_mount_of(facts.clone.dataset.as_str(), fixture.slot_path.as_path());
    assert_eq!(prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids), Ok(()));
    // The positive control: without the mount the same record is refused at the same path,
    // so the arm above sees a difference the covered slot makes.
    let uncovered: ScriptedMounts = ScriptedMounts::nothing();
    let unasked: ScriptedGuids = ScriptedGuids::never_asked();
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &uncovered, &unasked),
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
    let guids: ScriptedGuids = ScriptedGuids::never_asked();
    let other: ScriptedMounts = ScriptedMounts::a_mount_of("pool/clones/another", fixture.slot_path.as_path());
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &other, &guids),
        Err(TwoFactRefusal::AMountThatIsNotTheRecords { .. })
    ));
    let unreadable: ScriptedMounts = ScriptedMounts::unreadable("the table could not be read");
    assert!(matches!(
        prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &unreadable, &guids),
        Err(TwoFactRefusal::TheMountTableWasNotRead { .. })
    ));
}

/// What:
///  The mounted arm's second fact:
///  a dataset made again under the record's clone name answers another GUID,
///  so the mount is not the recorded clone and the proof refuses.
/// Why:
///  A name alone passes a replacement dataset,
///  and the lock of the destroyed clone would then be read as the replacement's;
///  the refusal names both GUIDs and the next safe command.
#[test]
fn a_dataset_made_again_under_the_recorded_name_is_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-clone-again");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(String::from("1:2:3"));
    spec.registration = Some(registration.clone());
    fixture.write_record(&spec);
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let recorded: crate::worktree_clone_mount_protocol::DatasetGuid = facts.clone.guid.expect("the fixture's record names the clone's GUID");
    let replacement: crate::worktree_clone_mount_protocol::DatasetGuid = crate::worktree_clone_mount_protocol::DatasetGuid::parse((recorded.value() / 2).to_string().as_str()).expect("the replacement's GUID reads");
    let guids: ScriptedGuids = ScriptedGuids::answering(facts.clone.dataset.as_str(), replacement.file_name().as_str());
    let mounts: ScriptedMounts = ScriptedMounts::a_mount_of(facts.clone.dataset.as_str(), fixture.slot_path.as_path());
    let refused: TwoFactRefusal = prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids).expect_err("the replacement clone is not the recorded one");
    match &refused {
        TwoFactRefusal::TheDatasetAtTheSlotHasAnotherGuid { recorded: held, found, .. } => {
            assert_eq!(*held, recorded);
            assert_eq!(*found, replacement);
        }
        other => panic!("the refusal names the GUID difference: {other:?}"),
    }
    let words: String = refused.words(registration.as_path());
    assert!(words.contains(recorded.file_name().as_str()) && words.contains(replacement.file_name().as_str()), "the words name both GUIDs: {words}");
    assert!(words.contains("Ways forward:"), "the words name the next safe command: {words}");
}

/// A GUID read the pool refuses is its own reason,
/// and never a pass and never a folder comparison.
#[test]
fn a_guid_the_pool_does_not_answer_is_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-guid-unread");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(String::from("1:2:3"));
    spec.registration = Some(registration.clone());
    fixture.write_record(&spec);
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let guids: ScriptedGuids = ScriptedGuids::refusing(facts.clone.dataset.as_str(), "the pool is not answering");
    let mounts: ScriptedMounts = ScriptedMounts::a_mount_of(facts.clone.dataset.as_str(), fixture.slot_path.as_path());
    let refused: TwoFactRefusal = prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids).expect_err("an unread GUID proves nothing");
    match &refused {
        TwoFactRefusal::TheClonesGuidWasNotRead { dataset, reason } => {
            assert_eq!(dataset, &facts.clone.dataset);
            assert!(reason.contains("the pool is not answering"), "{reason}");
        }
        other => panic!("the refusal carries the read's own reason: {other:?}"),
    }
    assert!(refused.words(registration.as_path()).contains("Ways forward:"), "the words name the next safe command");
}

/// A record with no `cloneGuid=` cannot compare a mounted clone,
/// so the arm refuses without asking the pool.
#[test]
fn a_mounted_clone_whose_record_holds_no_guid_is_refused() {
    let (fixture, registration): (Fixture, PathBuf) = a_recorded_fixture("proof-mounted-no-guid");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(String::from("1:2:3"));
    spec.registration = Some(registration.clone());
    spec.clone_guid = None;
    fixture.write_record(&spec);
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    assert_eq!(facts.clone.guid, None, "the control's record really holds no GUID");
    let guids: ScriptedGuids = ScriptedGuids::never_asked();
    let mounts: ScriptedMounts = ScriptedMounts::a_mount_of(facts.clone.dataset.as_str(), fixture.slot_path.as_path());
    let refused: TwoFactRefusal = prove_the_registration(&facts, fixture.common_dir.as_path(), registration.as_path(), &mounts, &guids).expect_err("no GUID compares nothing");
    assert!(matches!(refused, TwoFactRefusal::NoRecordedCloneGuid { .. }), "the refusal says the record holds no GUID: {refused:?}");
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
