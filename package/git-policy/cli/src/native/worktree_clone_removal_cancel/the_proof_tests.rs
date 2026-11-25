//! What:
//!  Controls for the proof:
//!  the slot's pointer,
//!  its binding record,
//!  the plan,
//!  the clone's own request,
//!  and the at-least-one-removal-record rule,
//!  each with the refusal it owes.
//! Why:
//!  The command may not reverse a removal it cannot prove,
//!  and a person who typed another slot's name must not be told a removal was reversed
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The proof" section).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(() => proveTheSlotIsARemovals(input)).toThrow(/is not a slot name/);
//! ```

use super::{TheRecordsProof, TheRecordsThatStand, prove_the_slot_is_a_removals};
use crate::testing::test_support::fixture as a_fixture_directory;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, ReadFailure};
use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, MountOperation};
use crate::worktree_clone_removal_cancel::cancel_stand_ins::an_intent_property;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{CLONE, CLONE_CREATETXG, CLONE_GUID, ORIGIN_SNAPSHOT, RemovalFixture, ScriptedPool, SLOT, the_stats};
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_proof::WORKTREES_FOLDER;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::path::{Path, PathBuf};

/// The nonce the fixture's clone carries.
const THE_NONCE: &str = "3f2a0000000000000000000003f2a000";

/// The nonce as the request carries it.
fn a_nonce() -> RequestNonce {
    return RequestNonce::parse(THE_NONCE).expect("a nonce");
}

/// One repository with a `.git` pointer naming its own registration.
fn a_repository(name: &str) -> (RemovalFixture, PathBuf) {
    let fixture: RemovalFixture = RemovalFixture::healthy(name);
    let checkout: PathBuf = fixture.common_dir.parent().expect("the common directory is inside the checkout").to_path_buf();
    write_pointer(&fixture, checkout.as_path(), SLOT);
    return (fixture, checkout);
}

/// The pointer one slot's folder holds.
fn write_pointer(fixture: &RemovalFixture, checkout: &Path, slot: &str) {
    let registration: PathBuf = checkout.join(".git").join("worktrees").join(slot);
    std::fs::write(fixture.slot_path.join(".git").as_path(), format!("gitdir: {}\n", registration.display())).expect("the slot's pointer");
}

/// The fixture's pool with the clone's own request planted.
fn a_pool(fixture: &RemovalFixture, operation: MountOperation) -> ScriptedPool {
    let intent: MountIntent = MountIntent { operation, nonce: a_nonce() };
    return fixture
        .healthy_pool()
        .with(CLONE, DatasetIdentity { stats: the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT), properties: an_intent_property(&intent) });
}

/// A reader the records' proof never needs.
struct NeverRead;

impl DatasetReader for NeverRead {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        return Err(ReadFailure { said: format!("the control does not read `{dataset}`"), errno: None });
    }

    fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
        return Err(String::from("the control does not walk the parent"));
    }
}

/// One records proof over the fixture.
fn a_proof<'a>(fixture: &'a RemovalFixture, home: &'a Path, checkout: &'a Path, pool: &'a ScriptedPool, slot: &'a str) -> TheRecordsProof<'a> {
    return TheRecordsProof { checkout, common_dir: fixture.common_dir.as_path(), home, slot, reads: pool, absence: &NeverRead };
}

#[test]
fn a_slot_name_the_rule_refuses_is_the_slots_first_refusal() {
    let (fixture, checkout): (RemovalFixture, PathBuf) = a_repository("proof-bad-name");
    let pool: ScriptedPool = a_pool(&fixture, MountOperation::Unmount);
    let home: PathBuf = a_fixture_directory("proof-home");
    match prove_the_slot_is_a_removals(&a_proof(&fixture, home.as_path(), checkout.as_path(), &pool, "two words")).expect_err("a name that is no slot name") {
        CancelRefusal::NotBeingRemoved { .. } => {}
        other => panic!("a name that is no slot name refuses as no removal, not {other:?}"),
    }
}

#[test]
fn a_slot_folder_with_no_pointer_is_a_damaged_pointer() {
    let (fixture, checkout): (RemovalFixture, PathBuf) = a_repository("proof-no-pointer");
    std::fs::remove_file(fixture.slot_path.join(".git").as_path()).expect("the pointer leaves");
    let pool: ScriptedPool = a_pool(&fixture, MountOperation::Unmount);
    let home: PathBuf = a_fixture_directory("proof-home-no-pointer");
    match prove_the_slot_is_a_removals(&a_proof(&fixture, home.as_path(), checkout.as_path(), &pool, SLOT)).expect_err("a missing pointer refuses") {
        CancelRefusal::ThePointerIsNotOneGitdirLine { slot, .. } => assert_eq!(slot, SLOT),
        other => panic!("a missing pointer refuses as a damaged one, not {other:?}"),
    }
}

#[test]
fn a_pointer_to_another_checkout_is_the_callers_mistake() {
    let (fixture, checkout): (RemovalFixture, PathBuf) = a_repository("proof-other-checkout");
    std::fs::write(fixture.slot_path.join(".git").as_path(), "gitdir: /elsewhere/.git/worktrees/fix-flaky-test\n").expect("the other pointer");
    let pool: ScriptedPool = a_pool(&fixture, MountOperation::Unmount);
    let home: PathBuf = a_fixture_directory("proof-home-other");
    let refusal: CancelRefusal = prove_the_slot_is_a_removals(&a_proof(&fixture, home.as_path(), checkout.as_path(), &pool, SLOT)).expect_err("another checkout refuses");
    assert!(matches!(refusal, CancelRefusal::ASlotOfAnotherRepository { .. }), "{refusal:?}");
    assert_eq!(refusal.exit_code(), 2);
}

#[test]
fn a_pointer_with_two_lines_is_no_pointer() {
    let (fixture, checkout): (RemovalFixture, PathBuf) = a_repository("proof-two-lines");
    let registration: PathBuf = checkout.join(".git").join("worktrees").join(SLOT);
    std::fs::write(fixture.slot_path.join(".git").as_path(), format!("gitdir: {}\ngitdir: {}\n", registration.display(), registration.display())).expect("the two-line pointer");
    let pool: ScriptedPool = a_pool(&fixture, MountOperation::Unmount);
    let home: PathBuf = a_fixture_directory("proof-home-two-lines");
    match prove_the_slot_is_a_removals(&a_proof(&fixture, home.as_path(), checkout.as_path(), &pool, SLOT)).expect_err("a two-line pointer refuses") {
        CancelRefusal::ThePointerIsNotOneGitdirLine { .. } => {}
        other => panic!("a two-line pointer refuses as a damaged one, not {other:?}"),
    }
}

#[test]
fn the_records_that_stand_carry_the_clone_the_guid_and_the_request() {
    let (fixture, checkout): (RemovalFixture, PathBuf) = a_repository("proof-healthy");
    let pool: ScriptedPool = a_pool(&fixture, MountOperation::Unmount);
    let home: PathBuf = a_fixture_directory("proof-home-healthy");
    let standing: TheRecordsThatStand = prove_the_slot_is_a_removals(&a_proof(&fixture, home.as_path(), checkout.as_path(), &pool, SLOT)).expect("a requested removal proves its records");
    assert_eq!(standing.recorded.clone_dataset, CLONE);
    assert_eq!(standing.recorded.clone_guid, crate::worktree_clone_removal_proof_and_plan::removal_fixtures::guid(CLONE_GUID));
    assert_eq!(standing.intent.operation, MountOperation::Unmount);
    assert_eq!(standing.intent.nonce.as_str(), THE_NONCE);
    assert_eq!(standing.slot_path, checkout.join(WORKTREES_FOLDER).join(SLOT));
    assert!(standing.plan.is_none());
}

#[test]
fn a_slot_whose_clone_asks_for_a_mount_with_no_other_record_is_not_being_removed() {
    let (fixture, checkout): (RemovalFixture, PathBuf) = a_repository("proof-pending-add");
    let pool: ScriptedPool = a_pool(&fixture, MountOperation::Mount);
    let home: PathBuf = a_fixture_directory("proof-home-pending");
    match prove_the_slot_is_a_removals(&a_proof(&fixture, home.as_path(), checkout.as_path(), &pool, SLOT)).expect_err("a pending add is not a removal") {
        CancelRefusal::NotBeingRemoved { found, .. } => assert!(found.contains("`mount`"), "the refusal names the operation it found: {found}"),
        other => panic!("a pending add refuses as no removal, not {other:?}"),
    }
}
