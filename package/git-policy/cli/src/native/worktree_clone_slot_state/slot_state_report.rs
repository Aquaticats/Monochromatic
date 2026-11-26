//! What:
//!  One slot's report:
//!  its name,
//!  the state it is in,
//!  the dataset and pool its record names,
//!  the evidence the state came from,
//!  and the one next safe action.
//! Why:
//!  The design's report exists so no reader treats a known unmounted clone as
//!  deleted tracked files and none offers an ordinary add over a pending one
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees"),
//!  and `doctor` and the wrapper's commands print this one shape rather than
//!  each writing its own lines (plan step 12,
//!  and step 18's `doctor`).
//!  The lines are data until a caller prints them,
//!  so this build changes no stream and no exit status.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = reportOf(readings); // { slot, state, dataset, pool, evidence, nextSafeAction }
//! for (const line of report.lines()) console.log(line);
//! ```

use crate::worktree_clone_slot_state::slot_state_classification::{
    SlotRouteState,
    classify_slot,
};
use crate::worktree_clone_slot_state::slot_state_evidence::evidence;
use crate::worktree_clone_slot_state::slot_state_next_actions::next_safe_action;
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    SlotReadings,
};

/// What:
///  The words one report prints when the record names no dataset.
/// Why:
///  A slot the route holds nothing in has no dataset,
///  and an empty field would read as one nobody filled in.
pub const NO_DATASET: &str = "none";

/// What:
///  One slot's report,
///  ready to print.
/// Why:
///  A caller prints one value and cannot mix the state of one reading with the
///  evidence of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotStateReport = { slot: string; state: string; dataset: string; pool: string; evidence: string[]; nextSafeAction: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotStateReport {
    /// The slot's name.
    pub slot: String,
    /// The state in words.
    pub state: &'static str,
    /// The clone dataset the record names,
    /// or [`NO_DATASET`].
    pub dataset: String,
    /// The pool the record names,
    /// or [`NO_DATASET`].
    pub pool: String,
    /// What each source said,
    /// one line per source.
    pub evidence: Vec<String>,
    /// The one thing that is safe to do next.
    pub next_safe_action: String,
}

impl SlotStateReport {
    /// What:
    ///  The report as the lines a command prints.
    /// Why:
    ///  One shape for every reader,
    ///  and every line names the field it carries so a person can read the
    ///  report without the code that wrote it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lines = report.lines(); // ["slot `x`", "state: pending add", ...]
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = vec![
            format!("slot `{}`", self.slot),
            format!("state: {}", self.state),
            if self.dataset == NO_DATASET {
                format!("dataset: {NO_DATASET}")
            } else {
                format!("dataset: `{}` in pool `{}`", self.dataset, self.pool)
            },
        ];
        lines.extend(
            self.evidence
                .iter()
                .map(|line| return format!("evidence: {line}")),
        );
        lines.push(format!("next safe action: {}", self.next_safe_action));
        return lines;
    }
}

/// What:
///  The report for one slot's readings.
/// Why:
///  The state, the dataset and the action all come from the same readings,
///  and the record is the only source of the dataset and pool names,
///  so a report with no record says so rather than leaving them blank.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = reportOf(readings);
/// ```
pub fn report_of(readings: &SlotReadings) -> SlotStateReport {
    let state: SlotRouteState = classify_slot(readings);
    let named: Option<(&str, &str)> = match &readings.binding {
        BindingReading::Record(record) => {
            Some((record.clone_dataset.as_str(), record.pool.as_str()))
        }
        BindingReading::Absent
        | BindingReading::Unreadable {
            ..
        } => None,
    };
    let (dataset, pool): (String, String) = match named {
        Some((dataset, pool)) => (String::from(dataset), String::from(pool)),
        None => (String::from(NO_DATASET), String::from(NO_DATASET)),
    };
    return SlotStateReport {
        slot: readings.slot.clone(),
        state: state.word(),
        dataset,
        pool,
        evidence: evidence(readings),
        next_safe_action: next_safe_action(&state, readings),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_state_report_tests.rs"]
mod tests;
