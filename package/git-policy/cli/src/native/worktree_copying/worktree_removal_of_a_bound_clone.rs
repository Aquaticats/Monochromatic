//! What:
//!  The refusal of one `git worktree remove` of a slot a whole managed clone is
//!  bound to,
//!  mounted or not,
//!  with the slot's state printed beside it,
//!  and the removal's lineage proof,
//!  inventory and bookmark rule run read-only inside the refusal.
//! Why:
//!  Git's own removal deletes the registration and the slot's underlay folder,
//!  which is the pointer the add kept for `git worktree prune`,
//!  and the clone dataset with the person's checked-out files would then be
//!  reachable by nobody
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  Only the two states that prove a whole worktree refuse:
//!  a pending add's removal is the add's own way forward,
//!  and a slot nothing proves stays Git's.
//!  Every spelling of `git worktree remove` that names such a slot is refused,
//!  whatever options stand before the operand:
//!  `SPEC.md` exempts none,
//!  and Git's own lock hint steers a person to the double force,
//!  which would otherwise delete the registration and the underlay pointer
//!  while the clone stands
//!  (`worktree_remove_plan::parse_removal_target`).
//!  A refusal of a whole clone also owes the person what a removal would have
//!  done,
//!  so the proof a removal makes runs here and its loss is printed
//!  (`worktree_clone_removal_proof_and_plan::removal_reading`,
//!  plan step 14).
//!  This release has no snapshot batch,
//!  head destroy,
//!  origin cleanup or Git settle,
//!  so nothing here writes a removal intent,
//!  unmounts the clone or writes the destruction plan:
//!  plan step 16's wiring switches those on,
//!  and until then a clone's removal that got that far would leave an unmounted
//!  clone and a plan no command of this release could finish or cancel.
//!  The reading therefore writes nothing and prints what it proved.
//!  The clone's mount intent is read through the caller's reader,
//!  as the pool is,
//!  so a host control answers it without opening this machine's ZFS device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = refuseTheRemovalOfABoundClone(facts); // undefined leaves the removal to Git
//! ```

use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::worktree_clone_binding_record::registration_home::home_folder;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::RemovalEntry;
use crate::worktree_clone_removal_proof_and_plan::removal_reading::read_the_removal;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{
    LiveRemovalReads,
    RemovalPoolReads,
};
use crate::worktree_clone_slot_state::slot_state_classification::SlotRouteState;
use crate::worktree_clone_slot_state::slot_state_live_read::{
    GatedCloneIntentReads,
    SlotReadingSources,
};
use crate::worktree_copy_forward::{
    CopyFacts,
    ForwardedRun,
};
use crate::worktree_identity::common_directory;
use crate::zfs_compatibility_gate::gate_verdict::LiveGate;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::path::Path;

/// What:
///  How one command's refusal reads the pool.
/// Why:
///  The release answer opens the kernel's ZFS device,
///  and a control answers the same three reads without one,
///  so the reading a person meets and the reading a control proves are the same
///  call over one seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reads = { kind: 'live' } | { kind: 'scripted'; pool } | { kind: 'unreadable'; reason };
/// ```
pub enum RemovalReads<'a> {
    /// The release answer:
    /// the device behind the start-time compatibility gate.
    Live,
    /// A pool a control scripts.
    Scripted(&'a dyn RemovalPoolReads),
    /// A pool that cannot be read,
    /// with the reason its reader answered.
    Unreadable(&'a str),
}

/// What:
///  The refusal of a removal of a slot a whole managed clone is bound to,
///  as the run the command ends in.
/// Why:
///  A command that prints its own report and changes nothing is not a failure
///  of a step,
///  so it ends with the wrapper's refusal status,
///  and Git never runs against the slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = refuseTheRemovalOfABoundClone(facts);
/// ```
pub fn refuse_the_removal_of_a_bound_clone(facts: &CopyFacts) -> Option<ForwardedRun> {
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    let gate: LiveGate = LiveGate;
    let intents: GatedCloneIntentReads<'_> = GatedCloneIntentReads::new(&gate);
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    return refuse_the_removal_with(facts, RemovalReads::Live, &sources);
}

/// What:
///  The same refusal over the pool reads and the slot sources a control
///  answers.
/// Why:
///  A host control may not open the machine's ZFS device,
///  and the intent is the second raw ZFS request this refusal makes,
///  so every control of this refusal runs through this call and the printed
///  report is the command's own.
///  The mount table is a source too,
///  so a control can prove which read the refusal makes and which it does not.
///  The target is the directory the planner read from the command's own words,
///  in every spelling Git accepts,
///  so the refusal and the reading that fills it cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = refuseTheRemovalWith(facts, scriptedPool, scriptedSources);
/// ```
pub fn refuse_the_removal_with(
    facts: &CopyFacts,
    reads: RemovalReads<'_>,
    sources: &SlotReadingSources<'_>,
) -> Option<ForwardedRun> {
    // The refusal reads the worktree the command names whatever its options, which is the target the planner took from every
    // spelling Git accepts; the plan above serves only the plain form.
    let target: &Path = facts.removal_target.as_deref()?;
    let common_dir: &Path = common_directory(&facts.identity)?;
    // A repository with no record folder holds no bound slot, so the answer is nothing and the mount table is never read:
    // this is every removal in a repository the route has never served.
    if !home_folder(common_dir).is_dir() {
        return None;
    }
    let slot: String =
        crate::worktree_clone_binding_record::registration_home::slot_of(target).ok()?;
    let at = crate::worktree_clone_link_repair::the_slot_at_with(
        common_dir,
        slot.as_str(),
        target,
        sources.mounts,
        sources.intents,
    );
    if !matches!(
        at.state,
        SlotRouteState::MountedManagedClone { .. } | SlotRouteState::KnownUnmountedClone { .. }
    ) {
        return None;
    }
    let mut stderr: String = format!(
        "the removal of `{}` is refused: a whole managed clone is bound to this slot, and {}. Nothing was removed, and Git did not run. Every spelling of `git worktree remove` is refused here, `-f -f` included: `git worktree unlock {slot}` releases the registration Git holds, and being rid of the clone after that is a person's own destroy, which takes its snapshots and bookmarks with it.\n",
        target.display(),
        crate::worktree_clone_binding_record::NO_REMOVAL_WORDS
    );
    for line in at.lines() {
        stderr.push_str(line.as_str());
        stderr.push('\n');
    }
    stderr.push_str(the_removals_reading(common_dir, slot.as_str(), reads).as_str());
    return Some(ForwardedRun::Failed {
        stderr,
        code: REFUSED_EXIT_CODE,
    });
}

/// What:
///  The lines one refusal prints about the removal that was not made:
///  what its proof matched and what it would lose,
///  or the check that stopped it,
///  or the pool that could not be read.
/// Why:
///  Every arm ends with the same promise,
///  that nothing was written and nothing was deleted,
///  and each names the step a person's next command belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = theRemovalsReading(commonDir, slot, reads);
/// ```
fn the_removals_reading(
    common_dir: &Path,
    slot: &str,
    reads: RemovalReads<'_>,
) -> String {
    let entry: RemovalEntry<'_> = RemovalEntry {
        common_dir,
        slot,
    };
    let reading = match reads {
        RemovalReads::Scripted(pool) => {
            read_the_removal(&entry, pool).map_err(|stop| return stop.message())
        }
        RemovalReads::Unreadable(reason) => return unreadable_pool(reason),
        RemovalReads::Live => match LiveRemovalReads::open() {
            Ok(live) => read_the_removal(&entry, &live).map_err(|stop| return stop.message()),
            Err(reason) => return unreadable_pool(reason.as_str()),
        },
    };
    return match reading {
        Ok(proved) => {
            let mut lines: Vec<String> = proved.lines();
            lines.push(String::new());
            format!("{}\n", lines.join("\n"))
        }
        Err(sentence) => format!(
            "the removal's own checks stopped it before anything was written:\n{sentence}\n"
        ),
    };
}

/// The lines one refusal prints when the pool raised no proof at all.
fn unreadable_pool(reason: &str) -> String {
    return format!(
        "the removal's lineage proof could not run, because {reason}\nNothing was unmounted and nothing was written, so every snapshot, bookmark, record and registration still stands.\n"
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_of_a_bound_clone_tests.rs"]
mod tests;
