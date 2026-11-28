//! What:
//!  The removal a person's `git worktree remove` runs:
//!  one order over the live pool,
//!  the live unmount request,
//!  the live holder scan,
//!  the live settle and the one request budget.
//! Why:
//!  Plan steps 14 to 16 built the order,
//!  the batch,
//!  the head destroy,
//!  the origin cleanup,
//!  the settle and the finish pass,
//!  and no command reached any of them:
//!  `SPEC.md` said the switch "waits on two things besides this build",
//!  `cancel-worktree-removal` and a release whose printed sentences name only
//!  what it has,
//!  and both are here
//!  (`package/git-policy/cli/SPEC.md`,
//!  "Removal of a ZFS worktree clone").
//!  The seams are the live ones release code already carries,
//!  so this file only chooses them and turns the order's two answers into a
//!  line and a status:
//!  a finished removal prints its report and exits `0`,
//!  and a refusal or a cancellation prints its own sentence and exits `1`.
//!  It lives in the order's own folder because the wrapper's forwarded command
//!  path may name this folder and may not name the cancellation's,
//!  and the layout allow list may only shrink
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = completeTheBoundClonesRemoval(commonDir, slot, cancelling);
//! console.error(answer.lines);
//! process.exit(answer.code);
//! ```

use crate::foundation::iso_time::now_iso;
use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::TheLiveHolderScan;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::TheLiveClock;
use crate::worktree_clone_removal_proof_and_plan::removal_completion::complete_the_removal;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::RemovalEntry;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::read_the_recorded_binding;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::LiveRemovalReads;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::TheLiveSettle;
use crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheRemovalCancelling;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::LiveRemovalUnmount;
use crate::worktree_clone_snapshot_batch::request_budget::the_removals_request_budget;
use monochromatic_zfs_request_to_openzfs_kernel_device::ZfsDevice;
use std::path::Path;

/// The status a removal that finished ends in,
/// because the worktree is gone.
pub const REMOVED_EXIT_CODE: i32 = 0;

/// What:
///  What one command's removal step answers instead of running Git:
///  the lines the person reads and the status the command ends in.
/// Why:
///  A finished removal is no failure and no refusal,
///  and the forwarded command path has one shape for "this step answered and
///  Git does not run",
///  so the status travels with the lines rather than the shape deciding it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { lines: string; code: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheRemovalAnswer {
    /// The lines the person reads,
    /// which is the report or the sentence that stopped the removal.
    pub lines: String,
    /// The status the command ends in.
    pub code: i32,
}

impl TheRemovalAnswer {
    /// One sentence and the status a removal that changed nothing ends in.
    fn stopped(said: String) -> TheRemovalAnswer {
        return TheRemovalAnswer {
            lines: said,
            code: REFUSED_EXIT_CODE,
        };
    }
}

/// What:
///  The whole removal of one slot a whole managed clone is bound to,
///  over the live seams of this machine.
/// Why:
///  The person's command is the only caller,
///  and it has already proved the slot holds a whole clone,
///  so this file opens what the order drives and nothing else.
///  A pool that cannot be read and a binding that cannot be read answer
///  before anything is written,
///  with the order's own sentence where it has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = completeTheBoundClonesRemoval(commonDir, slot, cancelling);
/// ```
pub fn complete_the_bound_clones_removal(
    common_dir: &Path,
    slot: &str,
    cancelling: &dyn TheRemovalCancelling,
) -> TheRemovalAnswer {
    let entry: RemovalEntry<'_> = RemovalEntry {
        common_dir,
        slot,
    };
    // The unmount request names the clone before the order takes the claim, and the order reads the binding again
    //  under it, so a record rewritten between the two reads still decides the order and not this line.
    let recorded = match read_the_recorded_binding(common_dir, slot) {
        Ok(recorded) => recorded,
        Err(stop) => return TheRemovalAnswer::stopped(stop.message()),
    };
    let unmount = match LiveRemovalUnmount::open(
        recorded.clone_dataset.as_str(),
        recorded.clone_guid,
    ) {
        Ok(unmount) => unmount,
        Err(stop) => return TheRemovalAnswer::stopped(stop.message()),
    };
    let reads = match LiveRemovalReads::open() {
        Ok(reads) => reads,
        Err(reason) => {
            return TheRemovalAnswer::stopped(format!(
                "the removal's lineage proof could not run, because {reason}\nNothing was unmounted and nothing was written, so every snapshot, bookmark, record and registration still stands."
            ));
        }
    };
    let check: ZfsDatasetReader = match ZfsDatasetReader::open() {
        Ok(check) => check,
        Err(error) => {
            return TheRemovalAnswer::stopped(format!(
                "the removal's checks could not read the pool, because {error}\nNothing was unmounted and nothing was written, so every snapshot, bookmark, record and registration still stands."
            ));
        }
    };
    let snapshots: ZfsDevice = match ZfsDevice::open() {
        Ok(device) => device,
        Err(error) => {
            return TheRemovalAnswer::stopped(format!(
                "the removal's snapshot requests could not be sent, because {error}\nNothing was unmounted and nothing was written, so every snapshot, bookmark, record and registration still stands."
            ));
        }
    };
    // The moment the records carry is read once, so one run's plan lines agree with each other.
    let now: String = now_iso();
    let input: RemovalCompletion<'_> = RemovalCompletion {
        entry: &entry,
        reads: &reads,
        unmount: &unmount,
        clock: &TheLiveClock,
        check: &check,
        probe: &check,
        snapshots: &snapshots,
        head: &snapshots,
        holders: &TheLiveHolderScan,
        settle: &TheLiveSettle,
        cancelling: Some(cancelling),
        budget: the_removals_request_budget(),
        now: now.as_str(),
    };
    return match complete_the_removal(&input) {
        Ok(done) => TheRemovalAnswer {
            lines: done.text(),
            code: REMOVED_EXIT_CODE,
        },
        Err(stop) => TheRemovalAnswer::stopped(stop.message()),
    };
}
