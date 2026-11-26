//! What:
//!  Controls for the whole command over one throwaway repository:
//!  a reversal that reports the destroyed set,
//!  the already-cancelled state that writes no request,
//!  a caller's mistake that takes no claim,
//!  a live removal that takes none either,
//!  and the head-gone state.
//! Why:
//!  The command acts only on what it proves,
//!  and each of these arms leaves a different record or none
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The proof" and "What it refuses" sections).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = cancelTheRemoval(input); expect(outcome.report().destroyed).toHaveLength(1);
//! ```

use super::{TheCancelCommand, TheCancellation, cancel_the_removal};
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, ReadFailure};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanAttempt};
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{CLONE, CLONE_GUID, RemovalFixture, ScriptedPool, SLOT, guid};
use crate::worktree_clone_removal_cancel::cancel_arguments::CancelRequest;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::cancel_stand_ins::{
    NeverRead, RecordingMountRequest, ScriptedClaim, ScriptedStaged, a_pool_carrying, a_repository_with_its_pointer, an_order, an_whole_answer, steps,
};
use crate::worktree_clone_removal_cancel::the_slot_mount::TheSlotMount;
use crate::worktree_clone_removal_cancel::the_staged_state::StagedSettlement;
use crate::worktree_clone_slot_state::slot_state_mounts::CloneMount;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::path::{Path, PathBuf};

/// What:
///  A mount table the control writes.
/// Why:
///  No control on a person's machine may read that machine's mounts.
struct ATable {
    rows: Vec<MountedDatasetInstance>,
    at_the_slot: bool,
}

impl TheSlotMount for ATable {
    fn where_the_clone_stands(&self, clone_dataset: &str, slot_path: &Path) -> CloneMount {
        let _ = (clone_dataset, slot_path);
        if self.at_the_slot {
            return CloneMount::AtItsSlot;
        }
        if self.rows.is_empty() {
            return CloneMount::Nowhere;
        }
        return CloneMount::Elsewhere { described: self.rows[0].described() };
    }
}

/// The table of a clone mounted at its slot.
fn at_its_slot() -> ATable {
    return ATable { rows: Vec::new(), at_the_slot: true };
}

/// The table of a clone mounted nowhere,
/// which is where a removal leaves it before the answer.
fn nowhere() -> ATable {
    return ATable { rows: Vec::new(), at_the_slot: false };
}

/// The plan a removal leaves behind with one snapshot destroyed.
fn a_plan_with_a_destroyed_snapshot(repository: &RemovalFixture) -> DestructionPlan {
    let mut plan: DestructionPlan = DestructionPlan::first_write(
        crate::worktree_clone_removal_proof_and_plan::fixtures::sample_facts(),
        crate::worktree_clone_removal_proof_and_plan::fixtures::sample_snapshots().as_slice(),
    );
    plan.facts.clone_guid = guid(CLONE_GUID);
    plan.facts.clone_dataset = String::from(CLONE);
    let snapshot = plan.named_snapshots()[0].clone();
    plan.record_line(PlanAttempt { snapshot, attempt: 1, outcome: SnapshotOutcome::Destroyed, errno: None }).expect("the destroyed line");
    let folder: PathBuf = plans_folder(repository.common_dir.as_path());
    crate::private_storage::create_private_directory_all(folder.as_path()).expect("the plan folder");
    std::fs::write(folder.join(plan.facts.clone_guid.file_name()), plan.encode().expect("the plan encodes")).expect("the plan file");
    return plan;
}

/// One run of the command over one repository.
#[allow(clippy::too_many_arguments)]
fn a_run<'a>(
    repository: &'a RemovalFixture,
    checkout: &'a Path,
    home: &'a Path,
    pool: &'a ScriptedPool,
    table: &'a dyn TheSlotMount,
    absence: &'a dyn DatasetReader,
    request: &'a RecordingMountRequest,
    staged: &'a ScriptedStaged,
    claim: &'a ScriptedClaim,
    slot: &'a CancelRequest,
) -> TheCancelCommand<'a> {
    return TheCancelCommand {
        checkout,
        common_dir: repository.common_dir.as_path(),
        home,
        request: slot,
        reads: pool,
        absence,
        mounts: table,
        claim,
        mount_request: request,
        staged,
    };
}

#[test]
fn a_reversal_reports_the_destroyed_set_and_the_mount_it_observed() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("cancel-command-reversed");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Unmount);
    let plan: DestructionPlan = a_plan_with_a_destroyed_snapshot(&repository);
    let home: PathBuf = fixture("cancel-command-home");
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_whole_answer(repository.slot_path.as_path(), 11)) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    let claim: ScriptedClaim = ScriptedClaim { order: order.clone(), refusal: None };
    let slot: CancelRequest = CancelRequest { slot: String::from(SLOT) };
    let table: ATable = at_its_slot();
    let input: TheCancelCommand<'_> = a_run(&repository, checkout.as_path(), home.as_path(), &pool, &table, &NeverRead, &request, &staged, &claim, &slot);
    let outcome: TheCancellation = cancel_the_removal(&input).expect("the command reverses the removal");
    let report = outcome.report();
    assert_eq!(report.exit_code(), 0);
    assert!(!report.already_cancelled);
    assert_eq!(report.destroyed, vec![plan.named_snapshots()[0].name.clone()]);
    assert_eq!(report.mount_id, Some(11));
    assert_eq!(report.guid, guid(CLONE_GUID).file_name());
    let text: String = report.lines().join("\n");
    assert!(text.contains("cancelled the removal of `fix-flaky-test`"), "{text}");
    assert!(text.contains(plan.named_snapshots()[0].name.as_str()), "{text}");
    assert!(text.contains("do not come back"), "{text}");
    assert!(text.contains("mounted at its slot"), "{text}");
    assert!(text.contains("doctor"), "{text}");
    let taken: Vec<String> = steps(&order);
    assert_eq!(taken.first().map(|step| return step.clone()), Some(format!("claim:{SLOT}")), "the claim is taken by the slot the binding is filed under: {taken:?}");
    assert!(taken.iter().any(|step| return step.starts_with("request:mount:")), "{taken:?}");
    assert!(taken.contains(&String::from("staged")), "{taken:?}");
}

#[test]
fn a_removal_already_cancelled_writes_no_request_and_still_removes_a_plan_that_witnessed_nothing() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("cancel-command-already");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Mount);
    let mut plan: DestructionPlan = DestructionPlan::first_write(
        crate::worktree_clone_removal_proof_and_plan::fixtures::sample_facts(),
        crate::worktree_clone_removal_proof_and_plan::fixtures::sample_snapshots().as_slice(),
    );
    plan.facts.clone_guid = guid(CLONE_GUID);
    plan.facts.clone_dataset = String::from(CLONE);
    let folder: PathBuf = plans_folder(repository.common_dir.as_path());
    crate::private_storage::create_private_directory_all(folder.as_path()).expect("the plan folder");
    let plan_path: PathBuf = folder.join(plan.facts.clone_guid.file_name());
    std::fs::write(plan_path.as_path(), plan.encode().expect("the plan encodes")).expect("the plan file");
    let home: PathBuf = fixture("cancel-command-home-already");
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_whole_answer(repository.slot_path.as_path(), 11)) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    let claim: ScriptedClaim = ScriptedClaim { order: order.clone(), refusal: None };
    let slot: CancelRequest = CancelRequest { slot: String::from(SLOT) };
    let table: ATable = at_its_slot();
    let input: TheCancelCommand<'_> = a_run(&repository, checkout.as_path(), home.as_path(), &pool, &table, &NeverRead, &request, &staged, &claim, &slot);
    let outcome: TheCancellation = cancel_the_removal(&input).expect("the command reports the state");
    let report = outcome.report();
    assert!(report.already_cancelled);
    assert_eq!(report.nonce, None);
    assert!(report.destroyed.is_empty());
    assert!(!plan_path.exists(), "the plan that witnessed nothing leaves");
    let text: String = report.lines().join("\n");
    assert!(text.contains("was already cancelled"), "{text}");
    assert!(text.contains("No new request was written"), "{text}");
    assert!(!steps(&order).iter().any(|step| return step.starts_with("request:")), "no request is written: {:?}", steps(&order));
}

#[test]
fn a_slot_that_is_not_being_removed_is_the_callers_mistake_and_takes_no_claim() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("cancel-command-not-a-removal");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Mount);
    let home: PathBuf = fixture("cancel-command-home-idle");
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_whole_answer(repository.slot_path.as_path(), 11)) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    let claim: ScriptedClaim = ScriptedClaim { order: order.clone(), refusal: None };
    let slot: CancelRequest = CancelRequest { slot: String::from(SLOT) };
    let table: ATable = nowhere();
    let input: TheCancelCommand<'_> = a_run(&repository, checkout.as_path(), home.as_path(), &pool, &table, &NeverRead, &request, &staged, &claim, &slot);
    match cancel_the_removal(&input).expect_err("a slot with no removal record refuses") {
        CancelRefusal::NotBeingRemoved { slot: named, .. } => assert_eq!(named, SLOT),
        other => panic!("a slot that is not being removed refuses as one, not {other:?}"),
    }
    assert!(steps(&order).is_empty(), "no claim is taken for a caller's mistake: {:?}", steps(&order));
}

#[test]
fn a_live_removal_refuses_and_writes_no_request() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("cancel-command-live");
    let pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Unmount);
    let _plan: DestructionPlan = a_plan_with_a_destroyed_snapshot(&repository);
    let home: PathBuf = fixture("cancel-command-home-live");
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_whole_answer(repository.slot_path.as_path(), 11)) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    let claim: ScriptedClaim = ScriptedClaim { order: order.clone(), refusal: Some(CancelRefusal::ALiveRemoval { owner: String::from("pid 4242") }) };
    let slot: CancelRequest = CancelRequest { slot: String::from(SLOT) };
    let table: ATable = nowhere();
    let input: TheCancelCommand<'_> = a_run(&repository, checkout.as_path(), home.as_path(), &pool, &table, &NeverRead, &request, &staged, &claim, &slot);
    assert!(matches!(cancel_the_removal(&input), Err(CancelRefusal::ALiveRemoval { .. })));
    assert!(!steps(&order).iter().any(|step| return step.starts_with("request:")), "a live removal leaves the request unwritten: {:?}", steps(&order));
}

#[test]
fn a_clone_whose_head_is_gone_refuses_and_mounts_nothing() {
    let (repository, checkout): (RemovalFixture, PathBuf) = a_repository_with_its_pointer("cancel-command-head-gone");
    let mut pool: ScriptedPool = a_pool_carrying(&repository, MountOperation::Unmount);
    pool.identities.retain(|(name, _)| return name != CLONE);
    pool.snapshots = Vec::new();
    let _plan: DestructionPlan = a_plan_with_a_destroyed_snapshot(&repository);
    let home: PathBuf = fixture("cancel-command-home-gone");
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_whole_answer(repository.slot_path.as_path(), 11)) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    let claim: ScriptedClaim = ScriptedClaim { order: order.clone(), refusal: None };
    let slot: CancelRequest = CancelRequest { slot: String::from(SLOT) };
    let table: ATable = nowhere();
    let input: TheCancelCommand<'_> = a_run(&repository, checkout.as_path(), home.as_path(), &pool, &table, &TheCloneWasDestroyed, &request, &staged, &claim, &slot);
    match cancel_the_removal(&input).expect_err("a gone head refuses") {
        CancelRefusal::TheCloneDoesNotAnswer { .. } | CancelRefusal::TheHeadIsGone { .. } => {}
        other => panic!("a clone that does not answer refuses as one, not {other:?}"),
    }
    assert!(!steps(&order).iter().any(|step| return step.starts_with("request:")), "nothing is mounted for a gone head: {:?}", steps(&order));
}

/// What:
///  A pool whose clone name does not answer and whose parent has no children,
///  which is the head-gone state.
/// Why:
///  The head-gone refusal rests on the absence proof's two facts,
///  so the arm's probe answers `ENOENT` for the name and ends the parent's listing at once.
struct TheCloneWasDestroyed;

impl DatasetReader for TheCloneWasDestroyed {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        let _ = dataset;
        return Err(ReadFailure { said: String::from("no such dataset"), errno: Some(libc::ENOENT) });
    }

    fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}
