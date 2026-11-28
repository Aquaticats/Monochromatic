//! What:
//!  One mount's turn of the shutdown pass:
//!  the slot's claim,
//!  the comparisons that prove the mount is the clone's own at its slot,
//!  the unmount from the held descriptor without force or lazy detach,
//!  the busy answer that names the clone and its holders,
//!  and the claim's release.
//! Why:
//!  The design forbids conflicting actions on one mount,
//!  so the claim comes before the identity is read and every later mount's
//!  turn is another claim
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering":
//!  "Recheck desired state and identities under a per-slot root claim").
//!  A slot another sweep holds is left to that sweep rather than queued,
//!  and a mount the kernel refuses to let go is reported with its holders and
//!  left standing:
//! the design says "report which clone holds the mount,
//!  and never force or
//!  detach it"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  The pass writes no receipt and changes no durable intent here:
//! it answered no request,
//!  so there is no nonce to file an answer under,
//!  and
//!  the clone's own desired state is what the next boot reads
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  where a receipt answers a request).
//! A removal's staged record is the sweep's own recovery and not this pass's:
//!  a slot whose removal stopped part-way has no clone mount standing at it,
//!  so nothing here would reach it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reached = unmountOne(request, configuration, recordedParent, candidate);
//! reached.ending; // unmounted | held | left
//! ```

use super::one_mounts_checks::{
    CheckedMount,
    verify_the_mount,
};
use super::shutdown_unmount_answer::{
    Holder,
    Holdings,
    MountEnding,
    ReachedMount,
};
use super::shutdown_unmount_request::ShutdownUnmountRequest;
use super::unmountable_mounts::CandidateMount;
use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_mount_handler::per_slot_root_claim::HeldSlotClaim;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;

/// What:
///  Unmount the mount standing at one slot,
///  or say why it stands.
/// Why:
///   Each slot's answer is its own line and never a stop,
///  because one clone that refuses must not leave every later clone mounted
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Boot and shutdown unit").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reached = unmountOne(request, configuration, recordedParent, candidate);
/// ```
pub(crate) fn unmount_one(
    request: &ShutdownUnmountRequest<'_>,
    configuration_checkout: &std::path::Path,
    recorded_parent: &CloneFolderIdentity,
    candidate: &CandidateMount,
) -> ReachedMount {
    let claim: HeldSlotClaim = match request.places.claims.claim(candidate.slot.as_str()) {
        Ok(claim) => claim,
        Err(refusal) => {
            return left(
                candidate,
                format!(
                    "another handler holds this slot's claim, so this pass left the slot to it: {}",
                    refusal.message()
                ),
            );
        }
    };
    let ending: MountEnding =
        unmount_under_the_claim(request, configuration_checkout, recorded_parent, candidate);
    claim.release();
    return ReachedMount {
        slot: candidate.slot.clone(),
        clone: candidate.clone.clone(),
        slot_path: candidate.slot_path.clone(),
        ending,
    };
}

/// One slot's turn with the claim held,
/// so two handlers cannot act on one clone.
///
/// Why:
///  The release happens however this answers,
///  which is why the caller holds it.
fn unmount_under_the_claim(
    request: &ShutdownUnmountRequest<'_>,
    configuration_checkout: &std::path::Path,
    recorded_parent: &CloneFolderIdentity,
    candidate: &CandidateMount,
) -> MountEnding {
    let checked: CheckedMount = match verify_the_mount(
        candidate,
        configuration_checkout,
        recorded_parent,
        request.lookups,
        request.walk,
        request.calls,
    ) {
        Ok(checked) => checked,
        Err(said) => {
            return MountEnding::Left {
                said,
            };
        }
    };
    // A descriptor on the clone would hold it busy, and the route's own unmount drops that descriptor before the call
    // for the same reason (`worktree_clone_unmount/unmount_pair.rs`): "A descriptor on the clone would hold it busy ...
    // so nothing needs to keep this one alive". The call is named by the underlay folder's own descriptor, which is on
    // the checkout dataset, so unmounting what stands at the slot takes down the clone's mount and not the checkout's.
    let mount_id: u64 = checked.mount_id;
    drop(checked.target);
    let underlay: std::fs::File = match request
        .walk
        .open_the_slot_uncrossed(configuration_checkout, candidate.slot.as_str())
    {
        Ok(underlay) => underlay,
        Err(refusal) => {
            return MountEnding::Left {
                said: refusal.message(
                    "the underlay folder at the slot could not be opened without crossing the clone's mount",
                ),
            };
        }
    };
    return match request.calls.unmount_top(&underlay) {
        Ok(()) => MountEnding::Unmounted {
            mount_id,
        },
        // A mount a task holds is `EBUSY`, and the kernel's number is what tells
        // that from a mount that is not there any more.
        Err(refusal) if refusal.errno == libc::EBUSY => MountEnding::Held {
            mount_id,
            said: refusal.detail,
            holdings: holders_of(request, candidate),
        },
        Err(refusal) => MountEnding::Left {
            said: refusal.message("the kernel would not unmount the clone's own mount"),
        },
    };
}

/// What:
///  Who holds one clone the kernel would not let go,
///  or why no scan could be made.
/// Why:
///   The busy answer's whole purpose is to name what holds the mount,
/// so a scan that could not run says so rather than reading as a machine with
///  no holder
///  (`crate::clone_holder_scan::holder_scan::HolderScan`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdings = holdersOf(request, candidate); // { holders, incompleteReads } or a reason
/// ```
fn holders_of(
    request: &ShutdownUnmountRequest<'_>,
    candidate: &CandidateMount,
) -> Result<Holdings, String> {
    // The scan is bound to the dataset's GUID, so a GUID this pool cannot
    // answer stops the scan rather than producing a result no reader may match.
    let dataset: DatasetGuid = request.guids.live_guid(candidate.clone.as_str()).map_err(
        |said| {
            return format!(
                "no holder scan could be made, because `{}` answered no GUID to bind one to ({said})",
                candidate.clone
            );
        },
    )?;
    let scan: HolderScan = request
        .holders
        .holders_of(candidate.clone.as_str(), dataset)
        .map_err(|said| {
            return format!(
                "no holder scan could be made, because it could answer under no nonce of its own ({said})"
            );
        })?;
    let holders: Vec<Holder> = scan
        .holders
        .iter()
        .map(|namespace| {
            return Holder {
                namespace: namespace.namespace().to_string(),
                task: namespace.task.clone(),
                mounts: namespace.mount_count(),
            };
        })
        .collect();
    return Ok(Holdings {
        holders,
        own_mounts: scan.own_mounts.len(),
        incomplete_reads: scan.incomplete.len(),
    });
}

/// One mount the pass left standing,
/// with the reason it gives a person.
///
/// Why:
///  Three of this turn's answers are a mount that stays,
///  and each names its own cause rather than a shared word.
fn left(
    candidate: &CandidateMount,
    said: String,
) -> ReachedMount {
    return ReachedMount {
        slot: candidate.slot.clone(),
        clone: candidate.clone.clone(),
        slot_path: candidate.slot_path.clone(),
        ending: MountEnding::Left {
            said,
        },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "one_mounts_unmount_tests.rs"]
mod tests;
