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
//!  "The proof: which slot, which clone, which claim").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(() => proveTheSlotIsARemovals(input)).toThrow(/is not a slot name/);
//! ```

use super::{TheRecordsProof, WORKTREES_FOLDER, prove_the_slot_is_a_removals};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{RemovalFixture, ScriptedPool, SLOT, guid};
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::cancel_stand_ins::{NeverRead, a_pool_carrying, a_repository_with_its_pointer};
use std::path::{Path, PathBuf};

/// One records proof over the fixture.
fn a_proof<'a>(repository: &'a RemovalFixture, home: &'a Path, checkout: &'a Path, pool: &'a ScriptedPool, slot: &'a str) -> TheRecordsProof<'a> {
    return TheRecordsProof { checkout, common_dir: repository.common_dir.as_path(), home, slot, reads: pool, absence: &NeverRead };
}

#[test]
fn a_slot_name_the_rule_refuses_is_the_slots_first_refusal() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("proof-bad-name");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Unmount);
    let home: PathBuf = fixture("proof-home-bad-name");
    match prove_the_slot_is_a_removals(&a_proof(&repository, home.as_path(), checkout.as_path(), &pool, "two words")).expect_err("a name that is no slot name") {
        CancelRefusal::NotBeingRemoved { .. } => {}
        other => panic!("a name that is no slot name refuses as no removal, not {other:?}"),
    }
}

#[test]
fn a_slot_folder_with_no_pointer_is_a_damaged_pointer() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("proof-no-pointer");
    std::fs::remove_file(repository.slot_path.join(".git").as_path()).expect("the pointer leaves");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Unmount);
    let home: PathBuf = fixture("proof-home-no-pointer");
    match prove_the_slot_is_a_removals(&a_proof(&repository, home.as_path(), checkout.as_path(), &pool, SLOT)).expect_err("a missing pointer refuses") {
        CancelRefusal::ThePointerIsNotOneGitdirLine { slot, .. } => assert_eq!(slot, SLOT),
        other => panic!("a missing pointer refuses as a damaged one, not {other:?}"),
    }
}

#[test]
fn a_pointer_to_another_checkout_is_the_callers_mistake() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("proof-other-checkout");
    std::fs::write(repository.slot_path.join(".git").as_path(), "gitdir: /elsewhere/.git/worktrees/fix-flaky-test\n").expect("the other pointer");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Unmount);
    let home: PathBuf = fixture("proof-home-other");
    let refusal: CancelRefusal = prove_the_slot_is_a_removals(&a_proof(&repository, home.as_path(), checkout.as_path(), &pool, SLOT)).expect_err("another checkout refuses");
    assert!(matches!(refusal, CancelRefusal::ASlotOfAnotherRepository { .. }), "{refusal:?}");
    assert_eq!(refusal.exit_code(), 2);
}

#[test]
fn a_pointer_with_two_lines_is_no_pointer() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("proof-two-lines");
    let registration: PathBuf = checkout.join(".git").join("worktrees").join(SLOT);
    std::fs::write(repository.slot_path.join(".git").as_path(), format!("gitdir: {}\ngitdir: {}\n", registration.display(), registration.display())).expect("the two-line pointer");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Unmount);
    let home: PathBuf = fixture("proof-home-two-lines");
    match prove_the_slot_is_a_removals(&a_proof(&repository, home.as_path(), checkout.as_path(), &pool, SLOT)).expect_err("a two-line pointer refuses") {
        CancelRefusal::ThePointerIsNotOneGitdirLine { .. } => {}
        other => panic!("a two-line pointer refuses as a damaged one, not {other:?}"),
    }
}

#[test]
fn a_requested_removal_proves_its_records_and_names_the_clone() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("proof-healthy");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Unmount);
    let home: PathBuf = fixture("proof-home-healthy");
    let claimed: DatasetGuid =
        prove_the_slot_is_a_removals(&a_proof(&repository, home.as_path(), checkout.as_path(), &pool, SLOT)).expect("a requested removal proves its records");
    assert_eq!(claimed, guid(crate::worktree_clone_removal_proof_and_plan::removal_fixtures::CLONE_GUID));
    assert_eq!(repository.the_plan_folder_holds(), 0, "the records' proof writes no plan");
    assert_eq!(checkout.join(WORKTREES_FOLDER).join(SLOT), repository.slot_path, "the slot the proof reads is the fixture's own");
}

#[test]
fn a_slot_whose_clone_asks_for_a_mount_with_no_other_record_is_not_being_removed() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("proof-pending-add");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Mount);
    let home: PathBuf = fixture("proof-home-pending");
    match prove_the_slot_is_a_removals(&a_proof(&repository, home.as_path(), checkout.as_path(), &pool, SLOT)).expect_err("a pending add is not a removal") {
        CancelRefusal::NotBeingRemoved { found, .. } => assert!(found.contains("`mount`"), "the refusal names the operation it found: {found}"),
        other => panic!("a pending add refuses as no removal, not {other:?}"),
    }
}
