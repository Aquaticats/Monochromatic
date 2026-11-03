//! What:
//!  The controls of finding F8 of the sixth review and its seventh-review follow-ups: maintenance coordination across
//!  worktrees of one object store.
//! Why:
//!  Each schedule is replayed literally against the rule it breaks; the exhaustive search must find a violation under
//!  revisions 6 and 7 and none under the candidate, whose observe-empty mutation must stay red.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision seven: an empty scope before launch is counted as finished', ...);
//! ```

// What:     imports the model's entry points and the explorer's helpers.
// Why:      The tests replay schedules and explore the model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, initial, steps, invariant, Rules } from './model_f8_maintenance';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f8_maintenance::{Rules, check, initial, invariant, steps};

/// The message of the harm every schedule below ends in.
const LOSS: &str = "a request's recovery copy was released and its objects were then deleted by wrapper-started maintenance";

/// The message of the property check: release while destructive execution could still occur.
const POSSIBLE: &str = "a recovery copy was released while destructive execution could still occur";

/// Revision 6, the reviewer's schedule: worktree A records the run in its own lane; worktree B does not look, checks
/// connectivity and releases; the prune's early view then deletes B's objects.
#[test]
fn revision_six_other_worktree_releases_during_a_prune() {
    let reached = replay(
        initial(Rules::Six, true),
        steps,
        &[
            "launcher: records the run in its own lane and starts the destructive process",
            "prune: takes its reachability view",
            "lane B: accepts the request and imports its objects",
            "lane B: publishes the request",
            "lane B: the connectivity check passes",
            "lane B: releases the spool",
            "prune: deletes the objects its view found unreachable",
        ],
    );
    assert_eq!(invariant(&reached), Some(LOSS.to_string()));
}

/// Revision 7, F8A: the entry is durable, the scope is not created yet; lane B finds the scope absent, closes the entry
/// and releases; the live launcher then starts the destructive process.
#[test]
fn revision_seven_counts_an_absent_scope_before_launch_as_finished() {
    let reached = replay(
        initial(Rules::Seven, true),
        steps,
        &[
            "launcher: creates the registry entry with O_EXCL and fsyncs it",
            "lane B: finds the scope empty or absent and closes the entry with an observation",
            "lane B: accepts the request and imports its objects",
            "lane B: publishes the request",
            "lane B: the connectivity check passes",
            "lane B: releases the spool",
        ],
    );
    assert_eq!(invariant(&reached), Some(POSSIBLE.to_string()));
}

/// Revision 7, F8A between creating an empty scope and populating it.
#[test]
fn revision_seven_counts_an_empty_scope_before_population_as_finished() {
    let reached = replay(
        initial(Rules::Seven, true),
        steps,
        &[
            "launcher: creates the registry entry with O_EXCL and fsyncs it",
            "launcher: asks the manager to create the scope",
            "lane B: finds the scope empty or absent and closes the entry with an observation",
            "lane B: accepts the request and imports its objects",
            "lane B: publishes the request",
            "lane B: the connectivity check passes",
            "lane B: releases the spool",
        ],
    );
    assert_eq!(invariant(&reached), Some(POSSIBLE.to_string()));
}

/// Revision 7, F8B: without a manager a descendant leaves the process group by `setsid`; the group is empty, lane B
/// closes the entry and releases, and the escaped descendant deletes the objects.
#[test]
fn revision_seven_process_group_fallback_lets_a_descendant_escape() {
    let reached = replay(
        initial(Rules::Seven, false),
        steps,
        &[
            "launcher: creates the registry entry with O_EXCL and fsyncs it",
            "launcher: chooses a new process group as the scope",
            "launcher: launches the destructive process into the scope",
            "prune: takes its reachability view",
            "prune: a descendant calls setsid and leaves the process group",
            "prune: the main process ends",
            "lane B: finds the scope empty or absent and closes the entry with an observation",
            "lane B: accepts the request and imports its objects",
            "lane B: publishes the request",
            "lane B: the connectivity check passes",
            "lane B: releases the spool",
            "prune: deletes the objects its view found unreachable",
        ],
    );
    assert_eq!(invariant(&reached), Some(LOSS.to_string()));
}

/// The exhaustive search finds a violation under each of the two earlier rules.
#[test]
fn the_search_finds_a_violation_under_revisions_six_and_seven() {
    for rules in [Rules::Six, Rules::Seven] {
        let report: Report = check(rules);
        assert!(report.violation.is_some(), "no violation under {:?}", rules);
    }
}

/// The candidate: revoke before treating a launch as impossible, populate the scope before the entry is running,
/// subreaper without a manager. No schedule releases while a destructive process can still delete.
#[test]
fn candidate_never_releases_while_maintenance_can_still_delete() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F8 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// The candidate does not hold the spool for ever: a run that ends releases it (the positive control of liveness).
#[test]
fn candidate_releases_after_a_run_ends() {
    let reached = replay(
        initial(Rules::Candidate, true),
        steps,
        &[
            "launcher: creates the registry entry with O_EXCL and fsyncs it",
            "launcher: asks the manager to create the scope",
            "launcher: moves the entry from reserved to running with one rename, the trampoline in the scope",
            "trampoline: executes the destructive process",
            "prune: the main process ends",
            "launcher: waits for the process, finds no descendant left and writes terminal",
            "lane B: accepts the request and imports its objects",
            "lane B: publishes the request",
            "lane B: the connectivity check passes",
            "lane B: releases the spool",
        ],
    );
    assert_eq!(reached.lane_b, 4);
    assert_eq!(invariant(&reached), None);
}

/// Closing a reserved entry on an empty scope, without revoking, reopens the revision-7 hole.
#[test]
fn candidate_that_observes_an_empty_scope_is_red() {
    let report: Report = check(Rules::CandidateObservesEmpty);
    let violation = report.violation.expect("a violation when an empty scope is counted as finished");
    assert_eq!(violation.message, POSSIBLE);
}
