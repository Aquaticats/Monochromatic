//! What:
//!  Controls of the space guard's scope report:
//!  the decided guard,
//!  a guard that descends,
//!  and a guard configured for another dataset.
//! Why:
//!  The project kept the guard's current scope,
//!  so an expanded guard must never read as the decided one,
//!  and the report has to name the space the guard cannot free,
//!  which is the whole point of printing its scope.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = checkSpaceGuardScope(decidedGuard());
//! expect(answer.outcome.kind).toBe('pass');
//! ```

use crate::doctor_zfs_route::CheckOutcome;
use crate::health::sanoid_coverage::coverage_test_facts::{
    CACHE,
    CONTAINER,
    NPM,
    PROBE,
    ROOT,
    WORKTREES,
    decided_guard,
};
use crate::health::sanoid_coverage::{
    FIELD_GUARD_DATASET,
    FIELD_GUARD_RECURSION,
    ScopeAnswer,
    SpaceGuardFacts,
};

/// The detail of one answer's report line.
fn detail_of(answer: &ScopeAnswer) -> &str {
    return match &answer.outcome {
        CheckOutcome::Pass {
            detail,
        }
        | CheckOutcome::Finding {
            detail,
            ..
        } => detail.as_str(),
        CheckOutcome::Unknown {
            reason,
        } => reason.as_str(),
    };
}

/// A guard that prunes the ancestor's `autosnap_` names and does not descend
/// passes and names every dataset it cannot prune.
#[test]
fn the_decided_guard_passes_and_names_what_it_cannot_prune() {
    let answer: ScopeAnswer = super::check_space_guard_scope(&decided_guard());
    assert!(
        matches!(answer.outcome, CheckOutcome::Pass { .. }),
        "{:?}",
        answer.outcome
    );
    assert!(detail_of(&answer).contains("1 dataset(s) under it"));
    assert!(detail_of(&answer).contains("git-worktree-clone-origin-"));
    let listed: Vec<&str> = answer
        .lines
        .iter()
        .filter(|line| return line.contains("cannot prune a snapshot of"))
        .map(|line| return line.as_str())
        .collect();
    assert_eq!(
        listed.len(),
        1,
        "only {} is under the ancestor and not excluded",
        WORKTREES
    );
    assert!(listed[0].contains(WORKTREES));
    assert!(
        answer
            .lines
            .iter()
            .any(|line| return line.contains(&format!("any other snapshot of {ROOT} stays")))
    );
    for exclusion in [CACHE, CONTAINER, NPM, PROBE] {
        assert!(
            !answer
                .lines
                .iter()
                .any(|line| return line.contains(exclusion)),
            "{exclusion} is excluded and the guard need not prune it"
        );
    }
}

/// A guard that descends is a finding naming the scope the project kept.
#[test]
fn a_guard_that_descends_is_a_finding() {
    let mut facts: SpaceGuardFacts = decided_guard();
    facts.recursive = true;
    let answer: ScopeAnswer = super::check_space_guard_scope(&facts);
    assert!(
        matches!(answer.outcome, CheckOutcome::Finding { .. }),
        "{:?}",
        answer.outcome
    );
    assert!(
        matches!(&answer.outcome, CheckOutcome::Finding { field, .. } if field == FIELD_GUARD_RECURSION)
    );
    assert!(detail_of(&answer).contains("current scope"));
}

/// A guard configured for another dataset is a finding naming both datasets.
#[test]
fn a_guard_for_another_dataset_is_a_finding() {
    let mut facts: SpaceGuardFacts = decided_guard();
    facts.configured_dataset = String::from("tank/other");
    let answer: ScopeAnswer = super::check_space_guard_scope(&facts);
    assert!(
        matches!(&answer.outcome, CheckOutcome::Finding { field, .. } if field == FIELD_GUARD_DATASET)
    );
    assert!(detail_of(&answer).contains("tank/other") && detail_of(&answer).contains(ROOT));
}

/// A machine with no dataset under the ancestor passes and lists only the
/// ancestor's own limit.
#[test]
fn a_machine_with_no_descendant_lists_only_the_ancestors_own_limit() {
    let mut facts: SpaceGuardFacts = decided_guard();
    facts
        .tree
        .datasets
        .retain(|dataset| return !dataset.name.starts_with("tank/home/"));
    let answer: ScopeAnswer = super::check_space_guard_scope(&facts);
    assert!(
        matches!(answer.outcome, CheckOutcome::Pass { .. }),
        "{:?}",
        answer.outcome
    );
    assert!(detail_of(&answer).contains("0 dataset(s) under it"));
    assert_eq!(answer.lines.len(), 1);
}
