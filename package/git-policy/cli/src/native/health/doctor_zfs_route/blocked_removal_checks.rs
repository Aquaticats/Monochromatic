//! What:
//!  The blocked-removal state one clone stands in:
//!  the `unmount` intent that stands on it,
//!  the snapshots a removal already destroyed,
//!  the objects that still stand,
//!  the namespaces whose holders stop it,
//!  the caller's own session among the processes a removal spares,
//!  and whether anything here may end those holders.
//! Why:
//!  The design asks the report to show all of this for a blocked removal,
//!  including "whether the removal will cancel itself without `--force` or end
//!  the holders with it once the owed VM item shows that works"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds").
//!  Three facts decide the words:
//!  a holder the scan saw blocks the removal,
//!  a scan that could not read every task proves no absence,
//!  and the forced arm's holder-ending has no measurement yet,
//!  so nothing here offers it until that flag is set.
//!  A removal without `--force` cancels itself,
//!  which the project decided,
//!  and the caller and its session are never ended:
//!  where the caller's own
//!  session holds the clone the removal refuses instead
//!  (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`).
//!  The kept `cancel-worktree-removal <slot>` command is the design's way back,
//!  this release's management grammar admits it,
//!  and a removal run again while a holder stands cancels itself,
//!  so this line names the command and says nothing is destroyed until the
//!  holders let go.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkBlockedRemoval(facts); // pass | finding('removal'|'intent') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use super::holder_checks::HolderFacts;
use super::removal_plan_checks::RecordedSnapshot;
use crate::worktree_clone_binding_record::add_state_commands::THE_REMOVALS_WORDS;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;

/// What:
///  Everything the blocked-removal line reads.
/// Why:
///  The intent,
///  the plan's names,
///  the scan and the caller's session are four
///  different reads of one clone,
///  and a line that took three of them would describe a state nobody saw.
///  The plan's presence is its own fact beside its lines,
///  because a removal of a clone that holds no snapshot of its own writes a
///  plan whose line set is empty,
///  and reading "no lines" as "no plan" would call a removal in flight an
///  intent with nothing behind it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BlockedRemovalFacts = { slot; clone; intent?: string; planStands: boolean; recorded: RecordedSnapshot[]; holders: HolderFacts; inTheCallersSession: number[]; forcedArmMeasured: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockedRemovalFacts {
    /// The slot the removal is for.
    pub slot: String,
    /// The clone dataset the removal is about.
    pub clone: String,
    /// The nonce of the `unmount` intent standing on the clone,
    /// and nothing when no intent of the clone's own stands.
    pub intent: Option<String>,
    /// Whether a destruction plan stands for the clone.
    pub plan_stands: bool,
    /// Every name the destruction plan holds,
    /// with the outcome its history ends on,
    /// and nothing when no plan stands.
    pub recorded: Vec<RecordedSnapshot>,
    /// The all-namespace scan's answer for this clone.
    pub holders: HolderFacts,
    /// The representative tasks that belong to the caller's own session.
    pub in_the_callers_session: Vec<u32>,
    /// Whether the forced arm's ending of holders has been measured.
    pub the_forced_arm_is_measured: bool,
}

/// What:
///  What one clone's removal is doing,
///  as a pass when nothing is pending,
///  a finding that names the blocked state,
///  or unknown when the scan could not prove the absence of holders.
/// Why:
///  A person who finds a clone unmounted with a plan beside it needs to know
///  what is already gone,
///  what still stands,
///  what stops the removal,
///  and whether anything in this release can finish or undo it.
///  A removal that is in flight and not blocked is the plan's own line and not
///  this one,
///  so this check answers only the blocked state and the state where an intent
///  stands with no plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkBlockedRemoval(facts); // pass | finding('removal'|'intent') | unknown
/// ```
pub fn check_blocked_removal(facts: &BlockedRemovalFacts) -> CheckOutcome {
    let clone: &str = facts.clone.as_str();
    if !facts.plan_stands {
        return match &facts.intent {
            None => pass(format!(
                "no destruction plan stands for `{clone}` and no unmount intent does, so no removal is pending for it and nothing is blocked"
            )),
            Some(nonce) => finding(
                "intent",
                format!(
                    "the clone `{clone}` carries an `unmount` intent under nonce `{nonce}` and no destruction plan was written for it, which is the state a removal killed between its intent and its first write leaves, or one whose unmount the handler has not answered. Nothing about the pool changed by itself, no snapshot is proved destroyed, and {THE_REMOVALS_WORDS}."
                ),
            ),
        };
    }
    if facts.holders.holders.is_empty() && !facts.holders.incomplete.is_empty() {
        return unknown(format!(
            "a destruction plan stands for `{clone}` and the scan found no holder of it, and {} task(s) or table(s) could not be read ({}), so whether this removal is blocked is unproved",
            facts.holders.incomplete.len(),
            facts.holders.incomplete.join("; ")
        ));
    }
    if facts.holders.holders.is_empty() {
        return pass(format!(
            "a destruction plan stands for `{clone}` and no namespace holds the clone, so the removal is not blocked by a holder"
        ));
    }
    let destroyed: Vec<String> = facts
        .recorded
        .iter()
        .filter(|line| return line.outcome == SnapshotOutcome::Destroyed)
        .map(|line| return line.name.clone())
        .collect();
    let remaining: Vec<String> = facts
        .recorded
        .iter()
        .filter(|line| return line.outcome != SnapshotOutcome::Destroyed)
        .map(|line| return line.name.clone())
        .collect();
    let loss: String = if destroyed.is_empty() {
        String::from("no snapshot of the clone is recorded as destroyed, so nothing is lost yet")
    } else {
        format!(
            "{} snapshot(s) are recorded as destroyed and cannot be restored: {}",
            destroyed.len(),
            destroyed.join(", ")
        )
    };
    let standing: String = if remaining.is_empty() {
        format!("The clone's head `{clone}` and no other name this plan holds")
    } else {
        format!(
            "The clone's head `{clone}` and {} name(s) the plan does not record as destroyed: {}",
            remaining.len(),
            remaining.join(", ")
        )
    };
    let intent: String = match &facts.intent {
        Some(nonce) => format!(
            "The clone's own `unmount` intent stands under nonce `{nonce}`, so this is the state a removal asked for."
        ),
        None => String::from(
            "No `unmount` intent of the clone's own stands, so nothing the wrapper wrote keeps it unmounted.",
        ),
    };
    let blocked_by: Vec<String> = facts
        .holders
        .holders
        .iter()
        .map(|holder| {
            return format!(
                "{} (task {}) mounts it at {}",
                holder.namespace,
                holder.representative_pid,
                holder.mount_point.display()
            );
        })
        .collect();
    let caller: String = if facts.in_the_callers_session.is_empty() {
        String::from("No holder belongs to the caller's own session.")
    } else {
        format!(
            "{} holder(s) belong to the caller's own session ({}), and a removal never ends the caller or its session even with `--force`, so it refuses here instead: leave the folder and run the command again from outside it.",
            facts.in_the_callers_session.len(),
            facts
                .in_the_callers_session
                .iter()
                .map(u32::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    };
    let force: String = if facts.the_forced_arm_is_measured {
        String::from(
            "Without `--force` the removal cancels itself; with it, every holder other than the caller and its session is ended, which is the measured arm.",
        )
    } else {
        String::from(
            "Without `--force` the removal cancels itself. The arm that ends the holders with `--force` has no measurement yet, so nothing here offers it and the holders have to let go for the removal to go on.",
        )
    };
    let cancel: String = format!(
        "The cancel command the design keeps as the way back is `git monochromatic-git-policy-cli cancel-worktree-removal {}`: it puts the clone back at its slot and ends the removal's records without destroying anything, and a removal run again while a holder stands cancels itself and puts the clone back the same way, so nothing is destroyed until the holders let go.",
        facts.slot
    );
    return finding(
        "removal",
        format!(
            "the removal of `{clone}` at the slot `{}` is blocked by a holder: {} namespace(s) hold it: {}. {intent} {loss}. {standing}. {caller} {force} {cancel}",
            facts.slot,
            facts.holders.holders.len(),
            blocked_by.join("; ")
        ),
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "blocked_removal_checks_tests.rs"]
mod tests;
