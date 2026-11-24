//! What:
//!  The route's preflight:
//!  everything A0 proves before the claim writes anything.
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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const preflight = preflightTheAdd(route, plan, context, arguments); // throws the sentence a person reads
//! ```

use super::add_git_words::person_lock_word;
use super::add_route_gate::RouteFacts;
use super::add_route_steps::StepRefusal;
use super::add_zfs::{ObjectIdentity, OpenedDevice};
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_binding_record::plan_folder_read::refuse_while_a_plan_names_the_clone;
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
/// the clean source,
/// the datasets,
/// the account's delegation,
/// the handler's readiness and the destination.
///
/// # Errors
/// The sentence each refusal prints,
/// which names the check and the way forward.
pub(super) fn preflight_the_add(route: &RouteFacts, plan: &AddPlan, context: &GitContext, arguments: &[OsString]) -> Result<Preflight, StepRefusal> {
    if let Some(word) = person_lock_word(arguments) {
        return Err(StepRefusal::refused(format!(
            "the add names `{}`, and the ZFS clone route takes the managed lock itself. Ways forward: run the add again without `--lock`, `--reason` or `--checkout`",
            word.to_string_lossy()
        )));
    }
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree { common_dir: route.common_dir.clone(), git_dir: route.git_dir.clone(), worktree_root: route.source_root.clone() };
    // A checkout with uncommitted edits is a source too: the clone route restores
    // the copy to the commit the caller asked for, so it serves the same adds the
    // ladder's other rungs serve (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
    // "The restore"; the interim that sent a dirty checkout to rung 3 ends here).
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
    let device: OpenedDevice = OpenedDevice::open().map_err(StepRefusal::refused)?;
    let checkout: ObjectIdentity = device.identity_of(route.checkout_dataset.as_str()).map_err(StepRefusal::refused)?;
    let pool: ObjectIdentity = device.identity_of(route.pool.as_str()).map_err(StepRefusal::refused)?;
    device.identity_of(route.container_dataset.as_str()).map_err(|reason| {
        return StepRefusal::refused(format!(
            "{reason}; the clone container {} the design places beside the checkout does not answer, so the route's layout is half-made. Ways forward: mount the container's dataset as setup does, then run the add again",
            route.container_dataset
        ));
    })?;
    let warnings: Vec<String> = super::add_grants::refuse_without_the_decided_grants(&device, route)?;
    admit_the_handler(route)?;
    let _ = plan;
    return Ok(Preflight { source, device, checkout_guid: checkout.guid, pool_guid: pool.guid, warnings });
}

/// Verify the folder the slot goes in,
/// when it stands.
fn verify_the_slot_parent(route: &RouteFacts) -> Result<(), StepRefusal> {
    let parent: PathBuf = route.worktrees_parent();
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
