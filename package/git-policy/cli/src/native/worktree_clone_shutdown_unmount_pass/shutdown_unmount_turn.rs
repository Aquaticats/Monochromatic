//! What:
//!  The whole shutdown pass in one place:
//!  withdraw this boot's readiness,
//!  read the names the boot pass read,
//!  decide what the withdrawn record proves,
//!  read this process's mount table,
//!  take the mounts of this boot's container that stand at a slot,
//!  read the `.claude/worktrees` identity the route recorded,
//!  and answer one line per mount.
//! Why:
//!  The order is the design's:
//!  readiness withdrawn and admission closed before anything in flight is
//!  drained,
//!  and the verified managed mounts unmounted after it,
//!  with no durable intent changed
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  One function holds that order so no caller can run the unmount half on a
//!  machine whose readiness still admits attaches,
//!  and every place it reads,
//!  every call it makes and the clock are the request's,
//!  so a value test decides everything this pass decides and a guest arm
//!  drives the same code as root on a real pool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = runThePass(request); // throws a refusal when the pass may not act at all
//! outcome.lines();
//! ```

use super::one_mounts_unmount::unmount_one;
use super::shutdown_unmount_answer::{
    PassOutcome,
    ReachedMount,
};
use super::shutdown_unmount_request::ShutdownUnmountRequest;
use super::the_boots_readiness::{
    BootsReadiness,
    Verified,
    what_the_record_verified,
    withdraw_before_draining,
};
use super::unmountable_mounts::{
    CandidateMount,
    mounts_at_slots,
};
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_mount_handler::sweep_configuration::{
    SweepConfiguration,
    read_configuration,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;

/// What:
///  Run one shutdown pass.
/// Why:
/// A unit,
///  a person and a guest arm each ask the same question of one machine,
///  and the answer carries every mount's ending so a summary is a count of
///  lines rather than a second reading
///  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
///  step 19,
///  "Acceptance tests":
///  an idle clone is unmounted and a busy one is named).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = runThePass(request); // throws when the pass may not act at all
/// ```
///
/// # Errors
/// This boot's readiness record could not be withdrawn,
/// the configuration the boot pass read cannot be used,
/// the record and the configuration name different places,
/// this process's mount table cannot be read,
/// or the `.claude/worktrees` identity the route recorded cannot be read.
pub fn run_the_pass(request: &ShutdownUnmountRequest<'_>) -> Result<PassOutcome, String> {
    let readiness: BootsReadiness = withdraw_before_draining(request.places)?;
    let configuration: SweepConfiguration = read_the_names(request)?;
    let verified: Verified = what_the_record_verified(&readiness, &configuration)?;
    let verified_nothing: Option<String> = match verified {
        Verified::Nothing {
            said,
        } => Some(said),
        Verified::Everything => None,
    };
    let candidates: Vec<CandidateMount> = match &verified_nothing {
        // No mount is read on a boot that proved nothing, so nothing this pass
        // answers can be mistaken for a reading of a machine it did not verify.
        Some(_) => Vec::new(),
        None => the_standing_mounts(request, &configuration)?,
    };
    let recorded_parent: Option<CloneFolderIdentity> = match candidates.is_empty() {
        true => None,
        false => Some(request.worktrees.recorded()?),
    };
    let reached: Vec<ReachedMount> = match &recorded_parent {
        None => Vec::new(),
        Some(parent) => candidates
            .iter()
            .map(|candidate| {
                return unmount_one(request, configuration.checkout.as_path(), parent, candidate);
            })
            .collect(),
    };
    return Ok(PassOutcome {
        boot: readiness.boot,
        withdrawal: readiness.withdrawal,
        verified_nothing,
        reached,
    });
}

/// What:
///  The mounts of this boot's container that stand at a slot of the
///  checkout.
/// Why:
///   The table is read once and every candidate comes from those rows,
///  so two mounts' turns can never disagree about what the kernel listed
///  (`crate::worktree_clone_slot_state::slot_state_mounts`:
/// the rows are one process's own table,
///  in mount order).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const candidates = theStandingMounts(request, configuration); // throws when the table cannot be read
/// ```
fn the_standing_mounts(
    request: &ShutdownUnmountRequest<'_>,
    configuration: &SweepConfiguration,
) -> Result<Vec<CandidateMount>, String> {
    let rows: Vec<MountedDatasetInstance> = request.mounts.mounted_datasets().map_err(
        |reason| {
            return format!(
                "this process's mount table could not be read ({reason}), so no mount standing now can be proved and nothing was unmounted. Ways forward: run the mode in the mount namespace the boot unit runs in, as its `ExecStart` does"
            );
        },
    )?;
    return Ok(mounts_at_slots(
        rows.as_slice(),
        configuration.parent_dataset.as_str(),
        configuration.checkout.as_path(),
    ));
}

/// What:
///  The names the boot pass's own `ExecStart` read:
///  the container,
///  the checkout and its dataset.
/// Why:
///   The shutdown unit runs the same words the start does,
///  so a pass that read another configuration would unmount mounts another
///  unit made
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Boot and shutdown unit").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configuration = readTheNames(request); // throws when the file is unusable
/// ```
fn read_the_names(request: &ShutdownUnmountRequest<'_>) -> Result<SweepConfiguration, String> {
    let path: &std::path::Path = request.places.configuration.as_path();
    return read_configuration(path, request.mounts).map_err(|refusal| {
        return format!(
            "{}so no mount was unmounted and no durable intent was changed. Ways forward: write the container and the checkout the boot unit reads, then run the boot unit's `boot-reconcile`",
            refusal.message()
        );
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "shutdown_unmount_turn_tests.rs"]
mod tests;
