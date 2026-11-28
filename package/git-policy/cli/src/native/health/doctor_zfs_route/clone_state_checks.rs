//! What:
//!  What one managed slot is,
//!  for the ZFS section of `doctor`'s report:
//!  the clone mounted at its slot,
//!  the clone unmounted from its slot,
//!  the clone a removal left mounted in a staging folder under the account's
//!  cache,
//!  the add that stopped part-way,
//!  the removal in flight,
//!  and the state no reading proves.
//! Why:
//!  The design asks the report to name "unmounted and staged clones,
//!  ...,
//!  interrupted actions,
//!  damaged or pruned Git registrations,
//!  and each living clone's recorded origin"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds").
//!  Step 12 already decides the state of one slot from its binding record,
//!  its plan,
//!  its intent and the mount table,
//!  and prints the state with one evidence line per source and the one next
//!  safe action,
//!  so this check reads that decision rather than deciding a second time.
//!  One arm is this check's own:
//!  a clone the mount table finds mounted somewhere else is no state step 12
//!  can name,
//!  and when that somewhere is a staging folder the clone is staged,
//!  not unproven,
//!  because a removal moved it there and a reattach or the removal's own end is
//!  what a person needs to know.
//!  A state that is neither is unknown with step 12's reason,
//!  because a report that guessed between "mounted" and "unmounted" would tell
//!  a person their files are gone when the mount could not be read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkCloneState({ readings, stagingBinds }); // pass | finding('state'|'staging') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use super::staging_folder_checks::StagingBind;
use crate::worktree_clone_slot_state::slot_state_classification::{
    SlotRouteState,
    classify_slot,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    SlotReadings,
};
use crate::worktree_clone_slot_state::slot_state_report::{
    NO_DATASET,
    SlotStateReport,
    report_of,
};
use std::path::PathBuf;

/// What:
///  The two readings one slot's state is printed from:
///  step 12's readings and every live mount under the account's cache with a
///  staging name.
/// Why:
///  The staging rows are what tells a staged clone from a clone mounted at a
///  place no removal made,
///  and both values are handed in,
///  so a control writes the state it judges and reads no mount table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneStateFacts = { readings: SlotReadings; stagingBinds: StagingBind[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneStateFacts {
    /// Step 12's readings for one slot.
    pub readings: SlotReadings,
    /// Every live mount under the cache whose name a removal derived.
    pub staging_binds: Vec<StagingBind>,
}

/// What:
///  The staging folder this clone is mounted at,
///  when one holds it.
/// Why:
///  A removal moves the clone onto a staging folder and unmounts it there,
///  so the clone's own dataset appearing among the staging rows is the one fact
///  that tells a staged clone from one a person moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = stagedAt(facts, cloneDataset); // PathBuf | undefined
/// ```
fn staged_at(
    facts: &CloneStateFacts,
    clone: &str,
) -> Option<PathBuf> {
    for bind in &facts.staging_binds {
        if bind.source == clone {
            return Some(bind.mount_point.clone());
        }
    }
    return None;
}

/// What:
///  The one line a slot's report prints,
///  in the shape the design fixes:
///  the state,
///  the dataset and pool,
///  one piece of evidence per source,
///  and the one next safe action.
/// Why:
///  `doctor` prints one line per check,
///  so the report's block is written as one line here and the wiring run keeps
///  the order of the fields a person reads the block in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = described(report); // 'mounted managed clone: the clone ... ; binding: ...; next safe action: ...'
/// ```
fn described(report: &SlotStateReport) -> String {
    let dataset: String = if report.dataset == NO_DATASET {
        String::from("no clone dataset is recorded")
    } else {
        format!("the clone `{}` in pool `{}`", report.dataset, report.pool)
    };
    let evidence: String = if report.evidence.is_empty() {
        String::new()
    } else {
        format!("; {}", report.evidence.join("; "))
    };
    return format!(
        "{}: {dataset}{evidence}; next safe action: {}",
        report.state, report.next_safe_action
    );
}

/// What:
///  What one managed slot is,
///  as a pass,
///  a finding that names the state a person must act on,
///  or unknown with the source that could not be read.
/// Why:
///  A clone mounted at its slot and a clone unmounted from it are the two
///  states the route intends,
///  so each passes with its dataset and the action that is safe from it.
///  A staged clone,
///  an add that stopped part-way and a removal in flight are states the design
///  asks this report to name,
///  so each is a finding whose field says which of the three it is.
///  A state step 12 cannot prove stays unknown rather than passing for the
///  unmounted clone it might not be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCloneState(facts); // pass | finding('staging'|'phase'|'plan') | unknown
/// ```
pub fn check_clone_state(facts: &CloneStateFacts) -> CheckOutcome {
    let report: SlotStateReport = report_of(&facts.readings);
    let clone: Option<&str> = match &facts.readings.binding {
        BindingReading::Record(record) => Some(record.clone_dataset.as_str()),
        BindingReading::Absent
        | BindingReading::Unreadable {
            ..
        } => None,
    };
    if let Some(dataset) = clone
        && let Some(folder) = staged_at(facts, dataset)
    {
        {
            return finding(
                "staging",
                format!(
                    "the slot `{}` holds a staged clone: `{dataset}` is mounted at the staging folder {} and nothing of it stands at the slot, which is the state a removal leaves when it moved the clone there and stopped before the unmount. The worktree's files are that mount and are reachable at the folder until something moves the mount back. Do not run `git worktree remove` at the slot: the wrapper's own removal takes a clone mounted at its slot or unmounted from it, so Git's own removal answers for a clone mounted elsewhere, and it would drop the registration while the clone stands at the staging folder. Evidence: {}.",
                    report.slot,
                    folder.display(),
                    describe_the_evidence(&report)
                ),
            );
        }
    }
    return match classify_slot(&facts.readings) {
        SlotRouteState::MountedManagedClone {
            ..
        } => pass(described(&report)),
        SlotRouteState::KnownUnmountedClone {
            ..
        } => pass(described(&report)),
        SlotRouteState::NotAManagedClone => pass(described(&report)),
        SlotRouteState::PendingAdd {
            ..
        } => finding(
            "phase",
            format!(
                "the slot `{}` holds an unfinished add: {}",
                report.slot,
                described(&report)
            ),
        ),
        SlotRouteState::PendingRemoval {
            ..
        } => finding(
            "plan",
            format!(
                "the slot `{}` holds a removal in flight: {}",
                report.slot,
                described(&report)
            ),
        ),
        SlotRouteState::CannotProve {
            reason,
            ..
        } => unknown(format!(
            "the state of the slot `{}` cannot be proved: {reason}",
            report.slot
        )),
    };
}

/// What:
///  The evidence half of a slot's report,
///  on its own.
/// Why:
///  The staged arm prints the sources it read beside its own state,
///  and a second wording of them would let two lines disagree about the same
///  record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const evidence = describeTheEvidence(report); // 'binding: phase=filled ...; plan: ...'
/// ```
fn describe_the_evidence(report: &SlotStateReport) -> String {
    if report.evidence.is_empty() {
        return String::from("no source was read");
    }
    return report.evidence.join("; ");
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_state_checks_tests.rs"]
mod tests;
