//! What:
//!  What one `git worktree remove` of a slot a whole managed clone is bound to
//!  does,
//!  mounted or not:
//!  the whole removal,
//!  with the slot's state printed beside its report,
//!  or the refusal that proves nothing and changes nothing.
//! Why:
//!  Git's own removal deletes the registration and the slot's underlay folder,
//!  which is the pointer the add kept for `git worktree prune`,
//!  and the clone dataset with the person's checked-out files would then be
//!  reachable by nobody
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  Only the three states that prove a whole worktree take that path:
//!  a clone mounted at its slot,
//!  one unmounted from it,
//!  and a slot a destruction plan names,
//!  which the removal resumes: the plan is the record of what an earlier run
//!  destroyed,
//!  and its removal reads it,
//!  proves every name again and ends it
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "Which resume path a removal takes").
//!  A pending add's removal is the add's own way forward,
//!  and a slot nothing proves stays Git's unless the reading is the mount no
//!  record names,
//!  the record of another path,
//!  or a mount table nobody could read,
//!  which refuse as the owner answered on 2026-11-26.
//!  A record filed under the slot's name that cannot be read refuses too,
//!  as the move already refuses,
//!  because nothing then proves the removal would not delete a managed clone's
//!  slot.
//!  Every spelling of `git worktree remove` that names such a slot takes this
//!  path,
//!  whatever options stand before the operand:
//!  `SPEC.md` exempts none,
//!  and Git's own lock hint steers a person to the double force,
//!  which would otherwise delete the registration and the underlay pointer
//!  while the clone stands
//!  (`worktree_remove_plan::parse_removal_target`).
//!  The whole removal is the wrapper's own,
//!  and this file only decides that the slot holds a whole clone and prints what
//!  it proved about it:
//!  the order,
//!  the batch,
//!  the head destroy,
//!  the origin cleanup,
//!  the settle and the records are
//!  `worktree_clone_removal_proof_and_plan::removal_command_path`
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 16).
//!  Nothing here ends a holder or forces a removal:
//!  this release's removal refuses a holder and cancels itself,
//!  and `--force` stays refused in every spelling,
//!  `-f -f` among them
//!  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`).
//!  The clone's mount intent is read through the caller's reader,
//!  as the pool is,
//!  so a host control answers it without opening this machine's ZFS device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = theRemovalOfABoundClone(slotPath, commonDir); // undefined leaves the removal to Git
//! ```

use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::worktree_clone_binding_record::registration_home::home_folder;
use crate::worktree_clone_removal_proof_and_plan::removal_command_path::{
    TheRemovalAnswer,
    complete_the_bound_clones_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::RemovalEntry;
use crate::worktree_clone_removal_proof_and_plan::removal_reading::read_the_removal;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{
    LiveRemovalReads,
    RemovalPoolReads,
};
use crate::worktree_clone_slot_state::slot_state_classification::{
    ProofStoppedBy,
    SlotRouteState,
};
use crate::worktree_clone_slot_state::slot_state_live_read::{
    GatedCloneIntentReads,
    SlotReadingSources,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    read_binding_reading,
};
use crate::zfs_compatibility_gate::gate_verdict::LiveGate;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::path::{
    Path,
    PathBuf,
};

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
///  What one removal of a bound clone answers instead of running Git:
///  the lines the person reads and the status the command ends in.
/// Why:
///  A finished removal is no failure and no refusal,
///  and the forwarded command path has one shape for "this step answered and
///  Git does not run",
///  so the status travels with the lines
///  (`worktree_copying::worktree_copy_forward`,
///  `ForwardedRun::Failed`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BoundSlotRemovalAnswer = { stderr: string; code: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundSlotRemovalAnswer {
    /// The lines the person reads.
    pub stderr: String,
    /// The status the command ends in,
    /// which is `0` for a removal that finished and `1` for one that did not.
    pub code: i32,
}

/// The sentence for a record no reader may act on.
/// Why:
///  A record that cannot be read may name this very slot,
///  so the refusal says what refused and what to repair before the removal runs
///  again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = unreadableRecordSentence('one', 'one.binding: not a record');
/// ```
fn unreadable_record_sentence(
    slot: &str,
    reason: &str,
) -> String {
    return format!(
        "`git worktree remove` is refused: a record is filed under the slot name `{slot}` and cannot be read ({reason}), so nothing proves this removal would not delete a managed clone's slot. Nothing was removed and Git did not run. No command of this release settles such a record. Deleting it makes the removal Git's own for a slot nothing then proves: with no record filed under the slot name and no mount at the slot, Git deletes the registration and the slot folder, and a clone bound to that slot is then reachable by nobody. Restore it from a copy if that is not what you mean, then run the removal again."
    );
}

/// What:
///  The sentence one refusal of a removal prints where nothing proves what the
///  slot holds:
///  what could not be proved,
///  the repair that ends it,
///  and the promise that nothing was removed.
/// Why:
///  The owner's answer of 2026-11-26 refuses the removal in each of the three
///  cases and names what could not be proved,
///  and each case has its own repair,
///  so the sentence is written once from the classification's own words
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = unprovedSlotSentence('one', 'no record is filed ...', 'a-mount-no-record-names');
/// ```
fn unproved_slot_sentence(
    slot: &str,
    reason: &str,
    stopped_by: ProofStoppedBy,
) -> String {
    let repair: &str = match stopped_by {
        ProofStoppedBy::AMountNoRecordNames => {
            "If the mount at the slot belongs to a managed clone, restore that clone's binding record; if a person mounted it, unmount it; then run the removal again."
        }
        ProofStoppedBy::ARecordOfAnotherPath => {
            "The record belongs to another slot, so a record filed under this slot's name must be corrected or settled before this slot can be removed; then run the removal again."
        }
        ProofStoppedBy::AMountThatCannotBeRead => {
            "Read this process's mount table again, and the mount at the slot, so the slot's state can be proved; then run the removal again."
        }
        ProofStoppedBy::AnotherReading => "Read what the reason names, then run the removal again.",
    };
    return format!(
        "`git worktree remove` is refused: nothing proves what the slot `{slot}` holds ({reason}), so nothing proves this removal would not delete a managed clone's slot. Nothing was removed and Git did not run. {repair}"
    );
}

/// What:
///  What one `git worktree remove` of a slot a whole managed clone is bound to
///  does,
///  over the pool and the slot sources the release reads.
/// Why:
///  A command that answers instead of running Git carries its own lines and
///  status,
///  and Git never runs against the slot.
///  The target and the common directory are what the step needs and all it
///  needs,
///  so a command that stands in no worktree,
///  where the settled run never starts,
///  answers through the same call as one that does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = theRemovalOfABoundClone(slotPath, commonDir);
/// ```
pub fn the_removal_of_a_bound_clone(
    target: &Path,
    common_dir: &Path,
) -> Option<BoundSlotRemovalAnswer> {
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    let gate: LiveGate = LiveGate;
    let intents: GatedCloneIntentReads<'_> = GatedCloneIntentReads::new(&gate);
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    return the_removal_with(target, common_dir, RemovalReads::Live, &sources);
}

/// What:
///  The same step over the pool reads and the slot sources a control answers.
/// Why:
///  A host control may not open the machine's ZFS device,
///  and the intent is the second raw ZFS request this step makes,
///  so every control of it runs through this call and the printed report is the
///  command's own.
///  The mount table is a source too,
///  so a control can prove which read the step makes and which it does not.
///  The target is the directory the planner read from the command's own words,
///  in every spelling Git accepts,
///  so the step and the reading that fills it cannot disagree.
///  A control that scripts the pool,
///  or one whose pool cannot be read,
///  is answered with the proof's own lines rather than with a removal:
///  a control may not open this machine's ZFS device,
///  and the whole removal needs one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = theRemovalWith(slotPath, commonDir, scriptedPool, scriptedSources);
/// ```
pub fn the_removal_with(
    target: &Path,
    common_dir: &Path,
    reads: RemovalReads<'_>,
    sources: &SlotReadingSources<'_>,
) -> Option<BoundSlotRemovalAnswer> {
    // A repository with no record folder holds no bound slot, so the answer is nothing and the mount table is never read:
    // this is every removal in a repository the route has never served.
    let home: PathBuf = home_folder(common_dir);
    if !home.is_dir() {
        return None;
    }
    let slot: String =
        crate::worktree_clone_binding_record::registration_home::slot_of(target).ok()?;
    // A record that cannot be read refuses before the mount table is read: it may name this very slot, and Git's own
    //  `-f -f` removal would delete the registration and the slot folder while nothing proved it safe
    //  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
    if let BindingReading::Unreadable {
        reason,
    } = read_binding_reading(home.as_path(), slot.as_str())
    {
        return Some(BoundSlotRemovalAnswer {
            stderr: format!(
                "{}\n",
                unreadable_record_sentence(slot.as_str(), reason.as_str())
            ),
            code: REFUSED_EXIT_CODE,
        });
    }
    let at = crate::worktree_clone_link_repair::the_slot_at_with(
        common_dir,
        slot.as_str(),
        target,
        sources.mounts,
        sources.intents,
    );
    if let SlotRouteState::CannotProve {
        reason,
        stopped_by,
    } = &at.state
        && stopped_by.refuses_a_removal()
    {
        return Some(BoundSlotRemovalAnswer {
            stderr: format!(
                "{}\n",
                unproved_slot_sentence(slot.as_str(), reason.as_str(), *stopped_by)
            ),
            code: REFUSED_EXIT_CODE,
        });
    }
    let whole_clone: bool = matches!(
        at.state,
        SlotRouteState::MountedManagedClone { .. }
            | SlotRouteState::KnownUnmountedClone { .. }
            | SlotRouteState::PendingRemoval { .. }
    );
    if !whole_clone {
        return None;
    }
    // A standing plan is the record of what an earlier run destroyed, so the removal this spelling runs is the one
    //  that resumes it rather than a fresh one that would write a second plan beside it.
    let resuming: String = match &at.state {
        SlotRouteState::PendingRemoval {
            plan,
            ..
        } => format!(
            " A removal's destruction plan at {} stands, so this run resumes it: it reads the plan, proves every name the plan records again, sends no request for a name the plan records as destroyed, carries the loss forward and ends the plan with the removal's own records.",
            plan.display()
        ),
        _ => String::new(),
    };
    // The slot holds a whole managed clone, so the wrapper removes it rather than Git. The slot's own report is printed
    //  first either way, because it is what proves the step is about this clone; the removal's report or the control's
    //  reading follows it.
    let mut stderr: String = match reads {
        RemovalReads::Live => format!(
            "the removal of `{}` runs here: a whole managed clone is bound to this slot, and {}.{resuming} Git does not run, and this release's removal ends no holder and forces nothing, so `-f -f` and every other spelling take this path.\n",
            target.display(),
            crate::worktree_clone_binding_record::THE_REMOVALS_WORDS
        ),
        _ => format!(
            "the removal of `{}` is refused: a whole managed clone is bound to this slot, and {}.{resuming} The pool here is the caller's own answer and not this machine's device, which the whole removal needs, so nothing was removed and Git did not run; the lines after this one say what a removal would prove and lose.\n",
            target.display(),
            crate::worktree_clone_binding_record::THE_REMOVALS_WORDS
        ),
    };
    for line in at.lines() {
        stderr.push_str(line.as_str());
        stderr.push('\n');
    }
    return match reads {
        RemovalReads::Live => {
            let answer: TheRemovalAnswer = complete_the_bound_clones_removal(
                common_dir,
                slot.as_str(),
                crate::worktree_clone_add_route::the_removals_cancelling(),
            );
            stderr.push_str(answer.lines.as_str());
            stderr.push('\n');
            Some(BoundSlotRemovalAnswer {
                stderr,
                code: answer.code,
            })
        }
        scripted => {
            // A control answers the pool itself, and the whole removal needs this machine's device, so a control reads
            //  what a removal would prove and lose rather than removing anything.
            stderr.push_str(the_removals_reading(common_dir, slot.as_str(), scripted).as_str());
            Some(BoundSlotRemovalAnswer {
                stderr,
                code: REFUSED_EXIT_CODE,
            })
        }
    };
}

/// What:
///  The lines a control's own pool answer prints about the removal that was not
///  made:
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
