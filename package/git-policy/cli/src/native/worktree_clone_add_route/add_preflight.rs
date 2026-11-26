//! What:
//!  The route's preflight:
//!  everything A0 proves before the claim writes anything,
//!  and the sources it declines the whole route for.
//! Why:
//!  A0 is one step with no write,
//!  so its checks live together and the runner calls them once and hands what
//!  they proved to every step after it
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  A0).
//!  The handler's readiness and the source's eligibility are the two facts a
//!  later step cannot re-read cheaply,
//!  and the delegation is read here because a granted permission no later step
//!  can repair
//!  (`super::add_grants`).
//!  A source the fill's own restore would refuse is not this route's yet:
//!  nothing of this release settles a failed restore at a slot,
//!  so the claim,
//!  the clone and the snapshot would stay
//!  (`super::add_source_the_restore_cannot_prove`,
//!  and `SPEC.md`, "What this release does not yet do").
//!  The preflight answers nothing for such a source and the add reaches the
//!  ladder, which settles its own failed restore and falls back to Git's own
//!  checkout.
//!  That answer is judged before the route's own invocation rules, so the person's
//!  `--lock`, `--reason` or `--checkout` reaches the ladder with the add instead of
//!  failing in a rule of a route that is not serving it (issue 6453).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const preflight = preflightTheAdd(route, plan, context, arguments); // null: the ordinary add
//! ```

use super::add_git_words::person_lock_word;
use super::add_route_gate::RouteFacts;
use super::add_route_steps::StepRefusal;
use super::add_zfs::{ObjectIdentity, OpenedDevice};
use crate::doctor_zfs_route::{CheckOutcome, DatasetFacts, DatasetRole, Reading, check_dataset};
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_binding_record::plan_folder_read::refuse_while_a_plan_names_the_clone;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{ReadinessExpectation, ReadinessRecord};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_admission::{AdmissionRefusal, admit_attach};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_snapshot_source::{Refusal as SourceRefusal, SnapshotSource, snapshot_blocker, source_or_dirty};
use std::ffi::OsString;
use std::os::unix::fs::MetadataExt as _;
use std::path::PathBuf;

/// What:
///  Everything the preflight proves before the claim.
/// Why:
///  A0 writes nothing,
///  and each of its answers is handed to the steps rather than read twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Preflight = { source: SnapshotSource; device: OpenedDevice; checkoutGuid: DatasetGuid; poolGuid: DatasetGuid; warnings: string[] };
/// ```
pub(super) struct Preflight {
    /// The proven clean source.
    pub(super) source: SnapshotSource,
    /// The device opened behind the gate.
    pub(super) device: OpenedDevice,
    /// The checkout dataset's GUID.
    pub(super) checkout_guid: DatasetGuid,
    /// The pool's GUID.
    pub(super) pool_guid: DatasetGuid,
    /// The lines the add prints and goes on past,
    /// which a delegation missing a permission only a removal uses earns.
    pub(super) warnings: Vec<String>,
}

/// Prove the supported form,
/// the source,
/// the datasets,
/// the account's delegation,
/// the handler's readiness and the destination,
/// or answer nothing when the source holds a path the fill's restore would
/// refuse.
///
/// # Errors
/// The sentence each refusal prints,
/// which names the check and the way forward.
pub(super) fn preflight_the_add(route: &RouteFacts, plan: &AddPlan, context: &GitContext, arguments: &[OsString]) -> Result<Option<Preflight>, StepRefusal> {
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree { common_dir: route.common_dir.clone(), git_dir: route.git_dir.clone(), worktree_root: route.source_root.clone() };
    // What:
    //  A checkout with uncommitted edits is a source too.
    // Why:
    //  The clone route restores the copy to the commit the caller asked for, so it
    //  serves the same adds the ladder's other rungs serve
    //  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
    //  "The restore").
    //  This replaces the interim of 2026-11-19 (issue 6452), which sent such an add
    //  to rung 3 until this step landed.
    let source: SnapshotSource = source_or_dirty(context, &identity).map_err(|refusal: SourceRefusal| return StepRefusal::refused(format!("the main checkout cannot be the source of a clone: {}", refusal.reason)))?;
    if let Some(reason) = snapshot_blocker(&source) {
        return Err(StepRefusal::refused(format!("the main checkout cannot be the source of a clone: {reason}")));
    }
    if !source.trash_ignored {
        return Err(StepRefusal::refused(String::from(
            "the place the add sets the copied Git administration aside in, `.claude/cli-git-trash`, is not ignored by this repository's Git, so the new worktree would carry an untracked folder. Ways forward: add that path to `.gitignore` or `.git/info/exclude`, then run the add again",
        )));
    }
    let _plan_read: () = refuse_while_a_plan_names_the_clone(route.common_dir.as_path(), None, route.clone_dataset().as_str())
        .map_err(|stop| return StepRefusal::refused(String::from(stop.sentence())))?;
    verify_the_slot_parent(route)?;
    // What:
    //  A source whose copy the fill's restore would end after the clone is made
    //  makes the route decline instead of failing the add.
    // Why:
    //  Nothing of this release settles a failed restore at a slot,
    //  so the claim,
    //  the locked registration,
    //  the mounted clone and the origin snapshot would stay;
    //  the ladder's own restore destroys its stage and lets Git's checkout make
    //  the worktree
    //  (`super::add_source_the_restore_cannot_prove`).
    //  A question that could not be answered declines the same way:
    //  an unproven source is not this route's either.
    //  The checks above come first so every refusal a person can act on keeps its
    //  sentence;
    //  the device is opened after this, so a declined add asks it nothing.
    match super::add_source_the_restore_cannot_prove::reasons_the_restore_cannot_prove((context, &source)) {
        Ok(reasons) if reasons.is_empty() => {}
        Ok(reasons) => {
            crate::diagnostic_log::debug("addPreflight", format!("the route declines a source its own restore cannot prove ({} reasons, the first {:?})", reasons.len(), reasons.first()).as_str());
            return Ok(None);
        }
        Err(reason) => {
            crate::diagnostic_log::debug("addPreflight", format!("the route declines a source whose shapes it could not read ({reason})").as_str());
            return Ok(None);
        }
    }
    // What:
    //  The person's own `--lock`, `--reason` or `--checkout` is refused here,
    //  after the source's answer and before the device is opened.
    // Why:
    //  The sentence names a rule of the ZFS clone route, which takes the managed
    //  lock itself, and a rule of a route that is not serving this add stops an
    //  add another rung makes: the ladder hands the person's own words to Git,
    //  whose `worktree add` takes the lock and the reason by itself
    //  (`SPEC.md`, the ladder's forms).
    //  So a source the route declines, which the check just before this one
    //  answers, takes the ladder with its words instead of failing here (issue
    //  6453).
    //  The cost is the source check on every add this refusal ends, which the issue
    //  accepts for the routing to be right.
    if let Some(word) = person_lock_word(arguments) {
        return Err(StepRefusal::refused(format!(
            "the add names `{}`, and the ZFS clone route takes the managed lock itself. Ways forward: run the add again without `--lock`, `--reason` or `--checkout`",
            word.to_string_lossy()
        )));
    }
    let device: OpenedDevice = OpenedDevice::open().map_err(StepRefusal::refused)?;
    let checkout: ObjectIdentity = device.identity_of(route.checkout_dataset.as_str()).map_err(StepRefusal::refused)?;
    let pool: ObjectIdentity = device.identity_of(route.pool.as_str()).map_err(StepRefusal::refused)?;
    let container_facts: CloneObjectFacts = device.facts_of(route.container_dataset.as_str()).map_err(|reason| {
        return StepRefusal::refused(format!(
            "{reason}; the clone container {} the design places beside the checkout does not answer, so the route's layout is half-made. Ways forward: mount the container's dataset as setup does, then run the add again",
            route.container_dataset
        ));
    })?;
    let container: DatasetFacts = DatasetFacts { role: DatasetRole::Container, name: String::from(route.container_dataset.as_str()), reading: Reading::Read(container_facts) };
    // What:
    //  The container's own properties are judged here, before any write.
    // Why:
    //  A clone inherits them, so a container that is not `legacy`,
    //  mounts automatically,
    //  or carries set-user-ID or device nodes gives every clone below it the
    //  same fault,
    //  and the design refuses that before a claim, a slot, a registration, a
    //  snapshot or a clone is written
    //  (`doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`).
    refuse_a_container_that_cannot_serve_clones(&container)?;
    let warnings: Vec<String> = super::add_grants::refuse_without_the_decided_grants(&device, route)?;
    admit_the_handler(route)?;
    let _ = plan;
    return Ok(Some(Preflight { source, device, checkout_guid: checkout.guid, pool_guid: pool.guid, warnings }));
}

/// What:
///  The refusal for a container whose properties the design does not serve for
///  the clones below it.
/// Why:
///  `doctor` already judges a container against exactly these properties,
///  and the add must judge the same container the same way rather than grow a
///  second check that could disagree
///  (`doc/planning/cli-git-zfs-clone-mountpoint-without-the-forbidden-grant.md`,
///  Choice 2);
///  the sentence names the container, the property and the value it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkDataset({ role: 'container', name: container, reading: { kind: 'read', value: facts } });
/// if (outcome.kind !== 'pass') throw new Error(outcome.detail + waysForward);
/// ```
fn refuse_a_container_that_cannot_serve_clones(facts: &DatasetFacts) -> Result<(), StepRefusal> {
    let container: &str = facts.name.as_str();
    let answer: CheckOutcome = check_dataset(facts);
    return match answer {
        CheckOutcome::Pass { .. } => Ok(()),
        CheckOutcome::Finding { detail, .. } => Err(StepRefusal::refused(format!(
            "the clone container cannot serve this add: {detail}. No claim, slot, Git registration, snapshot or clone was written. Ways forward: as root, set that property of `{container}` to the value the sentence names, then run the add again"
        ))),
        CheckOutcome::Unknown { reason } => Err(StepRefusal::refused(format!(
            "the clone container `{container}` cannot be judged from this account: {reason}. No claim, slot, Git registration, snapshot or clone was written. Ways forward: as root, read the container's properties (`zfs get mountpoint,canmount,setuid,devices {container}`), then run the add again"
        ))),
    };
}

/// Verify the folder the slot goes in,
/// when it stands.
fn verify_the_slot_parent(route: &RouteFacts) -> Result<(), StepRefusal> {    let parent: PathBuf = route.worktrees_parent();
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(parent.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(StepRefusal::refused(format!("the worktrees folder {} cannot be examined ({error})", parent.display()))),
    };
    if !metadata.is_dir() {
        return Err(StepRefusal::refused(format!("the worktrees folder {} is no folder", parent.display())));
    }
    // SAFETY: `geteuid` reads this process's own identity and has no preconditions.
    let ours: u32 = unsafe { libc::geteuid() };
    if metadata.uid() != ours {
        return Err(StepRefusal::refused(format!("the worktrees folder {} belongs to account {}, and this is account {ours}", parent.display(), metadata.uid())));
    }
    return Ok(());
}

/// Require a current handler readiness report for this boot,
/// pool and checkout.
fn admit_the_handler(route: &RouteFacts) -> Result<(), StepRefusal> {
    let boot: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").map_err(|error| return StepRefusal::refused(format!("this boot's identifier cannot be read ({error}), so the handler's readiness cannot be judged")))?;
    let answer: Option<ReadinessRecord> = ReadinessFile::live().read().map_err(|refusal| return StepRefusal::refused(refusal.message()))?;
    let Some(record) = answer else {
        return Err(StepRefusal::refused(format!(
            "no handler readiness report stands for this boot, so no attach is admitted ({}). Ways forward: start the boot unit's `boot-reconcile` and run the add again",
            ReadinessFile::live().record().display()
        )));
    };
    let checkout: String = route.source_root.display().to_string();
    let expectation: ReadinessExpectation<'_> = ReadinessExpectation { boot: boot.trim(), pool: route.pool.as_str(), checkout: checkout.as_str() };
    return admit_attach(&record, &expectation).map_err(|refusal: AdmissionRefusal| return StepRefusal::refused(refusal.message()));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_preflight_tests.rs"]
mod tests;
