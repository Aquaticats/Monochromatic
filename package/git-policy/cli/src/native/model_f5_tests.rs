//! What:
//!  The controls of finding F5 of the sixth review and its seventh-review follow-up: the attachment guard.
//! Why:
//!  The seventh review's schedule is replayed against revision 7 (the first-try path has no guard); revision 6's
//!  reflog-growth condition is red for declining a valid follow-up; the candidate guards both paths and still reaches
//!  the accepted residue.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision seven: the first-try path skips the guard', ...);
//! ```

// What:     imports the model's entry points and the explorer's helpers.
// Why:      The tests replay schedules and explore the model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, initial, steps, invariant, Rules } from './model_f5_attachment';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f5_attachment::{Rules, check, initial, invariant, steps};

/// Revision 7: the switch completes before the critical section; the first-try lock skips the guard; the delta is
/// staged on `topic`.
#[test]
fn revision_seven_first_try_path_stages_on_the_switched_branch() {
    let reached = replay(
        initial(Rules::Seven),
        steps,
        &[
            "person: switches the worktree to topic, carrying an edit",
            "lane: takes index.lock on the first try, swaps the ref and applies the delta",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("the delta was staged on a worktree attached to another branch with no guard read".to_string())
    );
}

/// Revision 6: the publication itself grew the `HEAD` reflog, so the follow-up declines though `HEAD` never left `main`.
#[test]
fn revision_six_reflog_growth_declines_a_valid_follow_up() {
    let reached = replay(
        initial(Rules::Six),
        steps,
        &[
            "foreign: takes index.lock",
            "lane: swaps the ref without the index lock",
            "foreign: releases index.lock",
            "lane: declines the follow-up because the HEAD reflog grew since capture",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("a valid follow-up was declined although HEAD never left the branch".to_string())
    );
}

/// The exhaustive search finds a violation under each of the two earlier rules.
#[test]
fn the_search_finds_a_violation_under_revisions_six_and_seven() {
    for rules in [Rules::Six, Rules::Seven] {
        let report: Report = check(rules);
        assert!(report.violation.is_some(), "no violation under {:?}", rules);
    }
}

/// The candidate guards both paths: nothing is staged without a guard read that saw `main`.
#[test]
fn candidate_guards_both_paths() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F5 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// The accepted residue stays reachable under the candidate (the positive control of the concession): a switch after
/// the last guard read stages the delta on `topic`, and the model does not call it a violation.
#[test]
fn candidate_still_reaches_the_accepted_residue() {
    let reached = replay(
        initial(Rules::Candidate),
        steps,
        &[
            "lane: reads HEAD before its critical section",
            "person: switches the worktree to topic, carrying an edit",
            "lane: takes index.lock on the first try, swaps the ref and applies the delta",
        ],
    );
    assert!(reached.accepted_residue);
    assert_eq!(invariant(&reached), None);
}

/// Revision 8 draft, M1: one path, the guard bracket after the publication; no schedule stages without a guard read that
/// saw `main`, and the accepted residue stays reachable.
#[test]
fn eight_has_one_guarded_path_and_keeps_the_accepted_residue() {
    let report: Report = check(Rules::Eight);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F5 eight: {} states, deepest {}", report.states, report.deepest);
    let reached = replay(
        initial(Rules::Eight),
        steps,
        &[
            "lane: swaps the ref without the index lock",
            "lane: reads HEAD before reading the index",
            "lane: reads the index and then HEAD again",
            "person: switches the worktree to topic, carrying an edit",
            "lane: applies the follow-up",
        ],
    );
    assert!(reached.accepted_residue);
}
