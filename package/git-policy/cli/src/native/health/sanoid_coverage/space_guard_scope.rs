//! What:
//!  The report of what the space guard can actually prune,
//!  measured from its configured dataset,
//!  the snapshot-name prefix it destroys
//!  and whether it recurses.
//! Why:
//!  The project kept the guard's current scope and asked the route to report
//!  its limit
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Space-pressure pruning" under "Project decisions",
//!  answered "Current scope (Recommended)";
//!  plan step 20,
//!  "What it builds":
//!  "the report of the space guard's actual scope").
//!  The guard prunes `<dataset>@autosnap_*` of its one configured dataset and
//!  nothing else,
//!  so it cannot free a descendant dataset's hourlies or the
//!  `git-worktree-clone-origin-` snapshots a living clone pins,
//!  which is the space that fills when home is snapshotted recursively.
//!  A guard that recursed would destroy a living worktree's history before
//!  its expiry,
//!  so that reading is a finding rather than a pass.
//!  The facts are values,
//!  so nothing here runs the guard or reads `/usr/local/sbin`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = checkSpaceGuardScope(facts); // { outcome, lines: ['the guard cannot prune ...'] }
//! ```

use crate::doctor_zfs_route::{
    CheckOutcome,
    finding,
    pass,
};
use crate::worktree_clone_mount_handler::sweep_clone_check::CLONE_ORIGIN_SNAPSHOT_PREFIX;

use super::decided_coverage::CoverageDecision;
use super::measured_dataset_tree::MeasuredTree;

/// The check's stable name in a report line.
pub const SCOPE_CHECK_NAME: &str = "space-guard-scope";

/// The stable name of the fact a scope finding names.
pub const FIELD_GUARD_RECURSION: &str = "guard-recursion";

/// The stable name of the fact a configured-dataset finding names.
pub const FIELD_GUARD_DATASET: &str = "guard-dataset";

/// What:
///  What one machine read about its space guard.
/// Why:
///  Three facts decide the scope:
///  which dataset it prunes,
///  which snapshot names it destroys,
///  and whether it descends,
///  and each is read from the guard rather than assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SpaceGuardFacts = { configuredDataset: string; prunedPrefix: string; recursive: boolean; decision; tree };
/// ```
pub struct SpaceGuardFacts {
    /// The dataset the guard prunes.
    pub configured_dataset: String,
    /// The snapshot-name prefix the guard destroys.
    pub pruned_prefix: String,
    /// Whether the guard's prune descends into child datasets.
    pub recursive: bool,
    /// The coverage the project decided,
    /// which names the dataset the guard should cover.
    pub decision: CoverageDecision,
    /// The dataset tree the machine measured.
    pub tree: MeasuredTree,
}

/// What:
///  The scope report:
///  the check's answer and every line a person reads to know what the guard
///  cannot free.
/// Why:
///  The answer is one line in the report,
///  and the account of the datasets it cannot reach is what a person deciding
///  about space needs,
///  so both travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScopeAnswer = { outcome: CheckOutcome; lines: string[] };
/// ```
pub struct ScopeAnswer {
    /// The report's answer for this check.
    pub outcome: CheckOutcome,
    /// What the guard cannot prune,
    /// one dataset class per line.
    pub lines: Vec<String>,
}

/// What:
///  The scope report for one machine.
/// Why:
///  It is the one entry the wiring step calls for this check,
///  and it answers a finding for a guard that reaches beyond the decided
///  scope,
///  so an expanded guard is never reported as the decided one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkSpaceGuardScope(facts); // ScopeAnswer
/// ```
pub fn check_space_guard_scope(facts: &SpaceGuardFacts) -> ScopeAnswer {
    let dataset: &str = facts.configured_dataset.as_str();
    let descendants: Vec<String> = facts
        .tree
        .datasets_under(facts.decision.recursive_root.as_str())
        .iter()
        .filter(|measured| return !facts.decision.excludes(measured.name.as_str()))
        .map(|measured| return measured.name.clone())
        .collect();
    let mut lines: Vec<String> = Vec::new();
    for child in &descendants {
        lines.push(format!("the guard cannot prune a snapshot of {child}, including its {} names and the {CLONE_ORIGIN_SNAPSHOT_PREFIX} snapshots a living clone pins", facts.pruned_prefix));
    }
    lines.push(format!("the guard prunes only names beginning {} on {dataset}, so any other snapshot of {dataset} stays", facts.pruned_prefix));
    if facts.recursive {
        let detail: String = format!(
            "the guard descends into child datasets while pruning, and the project kept the current scope of one dataset so a living worktree's history is not destroyed before its expiry"
        );
        return ScopeAnswer {
            outcome: finding(FIELD_GUARD_RECURSION, detail),
            lines,
        };
    }
    if dataset != facts.decision.recursive_root {
        let detail: String = format!(
            "the guard prunes {dataset} and the recursive section snapshots {}, so the hourlies of {} itself are never pruned while they fill the pool",
            facts.decision.recursive_root, facts.decision.recursive_root
        );
        return ScopeAnswer {
            outcome: finding(FIELD_GUARD_DATASET, detail),
            lines,
        };
    }
    let detail: String = format!(
        "the guard prunes only {dataset}@{}* and cannot free the hourlies of {} dataset(s) under it or the {CLONE_ORIGIN_SNAPSHOT_PREFIX} snapshots a living clone pins",
        facts.pruned_prefix,
        descendants.len()
    );
    return ScopeAnswer {
        outcome: pass(detail),
        lines,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "space_guard_scope_tests.rs"]
mod tests;
