//! What:
//!  The destruction plan one removal leaves for one clone,
//!  as the line a person reads:
//!  the moment it started,
//!  the process that wrote it,
//!  the head's word,
//!  and every snapshot name with the outcome its own history ends on.
//! Why:
//!  The removal's refusals send a person to the plan again and again
//!  (`RemovalStop::TheSnapshotSetIsNotSendable`,
//!  `ThePlanCouldNotHoldTheAnswer`,
//!  `NoLiveIntent`,
//!  `ThePlanContradictsThePool` and `TheSettleRefused`),
//!  and plan step 18 is where that file's lines are printed
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "What it builds").
//!  A name's current state is the outcome on its highest attempt number,
//!  which is the record's own merge rule,
//!  so the rule is applied by [`recorded_outcomes`] and never restated by a
//!  check that reads the same plan.
//!  A plan that does not read is unknown and never an absence:
//!  it may hold the only record of a snapshot an earlier run destroyed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkRemovalPlan({ clone, reading }); // pass | finding('plan') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use std::path::PathBuf;

/// What:
///  What stands at the plan's own file name for one clone.
/// Why:
///  "No plan",
///  "a plan that reads" and "a file that does not read" are three
///  answers with three readers:
///  the first is the healthy state,
///  the second is a removal in flight or one that was killed,
///  and the third proves nothing about any snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanFileReading = { kind: 'none' } | { kind: 'standing'; path; plan } | { kind: 'unreadable'; path; reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanFileReading {
    /// No plan file is filed under this clone's GUID.
    NoPlan,
    /// A plan stands and reads.
    Standing {
        /// The plan's file.
        path: PathBuf,
        /// The plan the file's bytes read as.
        plan: Box<DestructionPlan>,
    },
    /// A file stands at the plan's name and does not read as one.
    Unreadable {
        /// The plan's file.
        path: PathBuf,
        /// Why the file is no plan,
        /// in the reader's own words.
        reason: String,
    },
}

/// What:
///  The plan one clone's removal wrote,
///  read from its own file.
/// Why:
///  The plan is found by the clone's GUID and no other record,
///  so the clone's name and the reading are one value,
///  and a control writes the reading it judges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalPlanFacts = { clone: string; reading: PlanFileReading };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovalPlanFacts {
    /// The clone dataset the plan would name.
    pub clone: String,
    /// What stands at the plan's file name.
    pub reading: PlanFileReading,
}

/// What:
///  One snapshot name and the outcome its own history ends on.
/// Why:
///  The plan's lines,
///  the clone's own snapshots and the blocked removal's
///  report all read the same merge rule,
///  so one value carries it and no caller re-derives it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedSnapshot = { name; guid; outcome; attempt; errno? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedSnapshot {
    /// The snapshot's full name.
    pub name: String,
    /// The GUID the plan's line carries for it.
    pub guid: DatasetGuid,
    /// The outcome on its highest attempt number.
    pub outcome: SnapshotOutcome,
    /// The attempt number that outcome belongs to.
    pub attempt: u64,
    /// The kernel's error number a `refused` line carries.
    pub errno: Option<u32>,
}

impl RecordedSnapshot {
    /// The clause a line prints for this name.
    pub fn clause(&self) -> String {
        let guid: String = self.guid.file_name();
        return match self.errno {
            Some(errno) => format!(
                "`{}` (GUID {guid}) {} on attempt {} (errno {errno})",
                self.name,
                self.outcome.word(),
                self.attempt
            ),
            None => format!(
                "`{}` (GUID {guid}) {} on attempt {}",
                self.name,
                self.outcome.word(),
                self.attempt
            ),
        };
    }
}

/// What:
///  Every name the plan holds a line for,
///  with the outcome its own history ends on,
///  in the plan's own name order.
/// Why:
///  One line per name is what a person compares with `zfs list`,
///  and the merge rule belongs here rather than at each reader,
///  so the plan's report,
///  the clone's snapshot line and the blocked removal
///  cannot disagree about one name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = recordedOutcomes(plan); // [{ name: 'clone@s1', outcome: 'destroyed', attempt: 1 }]
/// ```
pub fn recorded_outcomes(plan: &DestructionPlan) -> Vec<RecordedSnapshot> {
    let mut names: Vec<String> = plan
        .attempts
        .iter()
        .map(|line| return line.snapshot.name.clone())
        .collect();
    names.sort();
    names.dedup();
    let mut outcomes: Vec<RecordedSnapshot> = Vec::new();
    for name in names {
        let Some(line) = plan.current_state(name.as_str()) else {
            continue;
        };
        outcomes.push(RecordedSnapshot {
            name,
            guid: line.snapshot.guid,
            outcome: line.outcome,
            attempt: line.attempt,
            errno: line.errno,
        });
    }
    return outcomes;
}

/// What:
///  The plan standing for one clone,
///  or the unknown that says its file could not be read,
///  or the pass that says no removal is in flight.
/// Why:
///  A standing plan is a state a person must see:
///  it says which snapshots a removal already asked about and what the kernel
///  answered,
///  and it is the only record that outlives a killed removal.
///  The line names the head's word so the settle's refusals have something to
///  compare with,
///  and it names no recovery command this release does not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkRemovalPlan(facts); // pass | finding('plan') | unknown
/// ```
pub fn check_removal_plan(facts: &RemovalPlanFacts) -> CheckOutcome {
    let clone: &str = facts.clone.as_str();
    return match &facts.reading {
        PlanFileReading::NoPlan => pass(format!(
            "no destruction plan stands for `{clone}`, so no removal is in flight for it and every snapshot it holds is its own"
        )),
        PlanFileReading::Unreadable {
            path,
            reason,
        } => unknown(format!(
            "the destruction plan at {} cannot be read ({reason}), so no snapshot state is proved and the file may hold the only record of one an earlier run destroyed. The owner's answer of 2026-11-28 made this a state a removal refuses without force: run `git monochromatic-git-policy-cli settle-worktree-clone-record <slot> --confirm-removal-of <slot> --confirm-clone-guid <the last four decimal digits of cloneGuid=>`, which prints the plan it finds and acts on none it cannot read, or `git worktree remove --force <slot>`, which proves the clone by the record's GUID against the pool, destroys the clone and its snapshots from the pool's own list, and removes this file with its own records",
            path.display()
        )),
        PlanFileReading::Standing {
            path,
            plan,
        } => {
            let outcomes: Vec<RecordedSnapshot> = recorded_outcomes(plan);
            let named: Vec<String> = outcomes
                .iter()
                .map(|snapshot| return snapshot.clause())
                .collect();
            let head: String = format!("head={}", plan.head.word());
            let snapshots: String = if named.is_empty() {
                String::from("the plan holds no snapshot line")
            } else {
                format!("{} name(s): {}", named.len(), named.join("; "))
            };
            return finding(
                "plan",
                format!(
                    "a destruction plan for `{clone}` stands at {}: started {}, written by {}, {head}, {snapshots}. A name's state is the outcome on its highest attempt number, so a name the plan calls `destroyed` is gone and cannot be restored, and a name it calls `planned`, `intact` or `unknown` may still stand on the pool. The removal that wrote this plan is the one that resumes from it, and a `git worktree remove` of this slot runs it: it reads the plan, proves every name the plan records again, sends no request for a name the plan records as destroyed, carries the loss forward and ends the plan with the removal's own records.",
                    path.display(),
                    plan.facts.started,
                    plan.facts.owner
                ),
            );
        }
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_plan_checks_tests.rs"]
mod tests;
