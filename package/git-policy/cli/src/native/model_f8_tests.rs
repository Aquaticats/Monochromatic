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

/// Revision 8 draft, G1 to G4: the launcher inside the scope is the launch authority and its death with the scope empty is the
/// proof; no schedule releases while a run can still delete, a finished run does release, and an uncontained run (no manager)
/// is retained for good, which is the draft's stated cost.
#[test]
fn eight_closes_a_run_only_on_an_empty_scope_and_a_dead_launcher() {
    // Since T1 is observation then release, revision 8 is red on the race (see the next tests); the G3 closing rule itself holds.
    assert_eq!(check(Rules::Eight).violation.expect("the T1 race").message, POSSIBLE);
    let finished = replay(
        initial(Rules::Eight, true),
        steps,
        &[
            "G1 reserve: the launcher creates the scope and places itself in it",
            "G1 reserve: the launcher writes the entry reserved by link",
            "G2 run: the launcher starts the destructive process from inside the scope",
            "prune: the main process ends",
            "G3 terminal: the launcher waits for the process and records end",
            "G3 terminal: lane B finds the scope empty and the launcher dead and records terminal",
            "lane B: accepts the request and imports its objects",
            "lane B: publishes the request",
            "lane B: the connectivity check passes",
            "T1 pre: lane B observes no open run and no uncompleted entry",
            "T1 retire: lane B releases the spool",
        ],
    );
    assert_eq!(finished.lane_b, 4);
    assert_eq!(invariant(&finished), None);
    let kept = replay(
        initial(Rules::Eight, false),
        steps,
        &[
            "G4 no manager: the launcher starts the run and registers it as uncontained",
            "prune: the main process ends",
            "lane B: accepts the request and imports its objects",
            "lane B: publishes the request",
        ],
    );
    // The final connectivity check waits for the entry to close, and nothing closes an uncontained run.
    assert!(!steps(&kept).iter().any(|entry| { return entry.0.contains("connectivity check passes"); }));
    assert!(!steps(&kept).iter().any(|entry| { return entry.0.contains("releases the spool"); }));
}

/// The race of the changelog's T1 and G1 row: T1 observes no open run, G1 publishes a reservation, G2 starts the destructive
/// process, and T1 releases. The steps before the release are shared by both tests.
const RACE: [&str; 7] = [
    "lane B: accepts the request and imports its objects",
    "lane B: publishes the request",
    "lane B: the connectivity check passes",
    "G1 reserve: the launcher creates the scope and places itself in it",
    "G1 reserve: the launcher writes the entry reserved by link",
    "G2 run: the launcher starts the destructive process from inside the scope",
    "prune: takes its reachability view",
];

/// Revision 8 (red): the observation and the release are two steps with nothing between them to notice the run.
#[test]
fn eight_releases_after_a_run_was_admitted_between_the_observation_and_the_release() {
    let mut schedule: Vec<&str> = vec![
        "lane B: accepts the request and imports its objects",
        "lane B: publishes the request",
        "lane B: the connectivity check passes",
        "T1 pre: lane B observes no open run and no uncompleted entry",
    ];
    schedule.extend_from_slice(&RACE[3..]);
    schedule.push("T1 retire: lane B releases the spool");
    let reached = replay(initial(Rules::Eight, true), steps, &schedule);
    assert_eq!(invariant(&reached), Some(POSSIBLE.to_string()));
}

/// Revision 9 (green): the release intent and the re-read withdraw the release when the reservation appeared first; when it
/// appears after the re-read the released closure is reachable from the tip and the late prune cannot take it. Exhaustive.
#[test]
fn nine_withdraws_a_release_when_a_run_appears_and_loses_nothing_otherwise() {
    let mut schedule: Vec<&str> = RACE[..3].to_vec();
    schedule.push("T1 (1): lane B writes the release intent");
    schedule.extend_from_slice(&RACE[3..6]);
    schedule.push("T1 (2): lane B re-reads the registry, finds a run that is not terminal and withdraws the intent");
    let withdrawn = replay(initial(Rules::Nine, true), steps, &schedule);
    assert_eq!(withdrawn.lane_b, 3);
    let late = replay(
        initial(Rules::Nine, true),
        steps,
        &[
            "lane B: accepts the request and imports its objects",
            "lane B: publishes the request",
            "lane B: the connectivity check passes",
            "T1 (1): lane B writes the release intent",
            "T1 (2): lane B re-reads the registry and finds no run that is not terminal",
            "T1 (3) to (5): lane B re-runs the check, appends the ledger line and releases the spool",
            "G1 reserve: the launcher creates the scope and places itself in it",
            "G1 reserve: the launcher writes the entry reserved by link",
            "G2 run: the launcher starts the destructive process from inside the scope",
            "prune: takes its reachability view",
        ],
    );
    assert_eq!(invariant(&late), None);
    // Exhaustive search: revision 9 is red for the root race (see nine_releases_a_closure_that_an_earlier_view_found_unreferenced);
    // the release intent alone does not close it.
    assert!(check(Rules::Nine).violation.is_some());
}

/// The schedule of the changelog's T1 and C5 row, shared by the next two tests: T1 steps 1 and 2 pass, a run is admitted and takes
/// its view while the branch is away, the branch returns.
const ROOT_RACE: [&str; 11] = [
    "lane B: accepts the request and imports its objects",
    "lane B: publishes the request",
    "lane B: the connectivity check passes",
    "T1 (1): lane B writes the release intent",
    "T1 (2): lane B re-reads the registry and finds no run that is not terminal",
    "G1 reserve: the launcher creates the scope and places itself in it",
    "G1 reserve: the launcher writes the entry reserved by link",
    "G2 run: the launcher starts the destructive process from inside the scope",
    "person: moves the branch away from the request's commit",
    "prune: takes its reachability view",
    "person: returns the branch to the request's commit",
];

/// Revision 9 (red): no root protects the closure; the earlier view found it unreferenced, T1 releases, the run deletes.
#[test]
fn nine_releases_a_closure_that_an_earlier_view_found_unreferenced() {
    let mut schedule: Vec<&str> = ROOT_RACE.to_vec();
    schedule.push("T1 (3) to (5): lane B re-runs the check, appends the ledger line and releases the spool");
    schedule.push("prune: deletes the objects its view found unreachable");
    let reached = replay(initial(Rules::Nine, true), steps, &schedule);
    assert_eq!(
        invariant(&reached),
        Some("a request's recovery copy was released and its objects were then deleted by wrapper-started maintenance".to_string())
    );
    assert!(check(Rules::Nine).violation.is_some());
}

/// Revision 10 (green): the hold root exists until step 6, so the same view includes the closure and the prune has nothing to
/// delete; exhaustive.
#[test]
fn ten_roots_the_closure_until_the_release_is_done() {
    let rooted = replay(initial(Rules::Ten, true), steps, &ROOT_RACE);
    assert_eq!(rooted.view, 2);
    assert!(!steps(&rooted).iter().any(|entry| { return entry.0.contains("deletes the objects"); }));
    let report: Report = check(Rules::Ten);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F8 ten: {} states, deepest {}", report.states, report.deepest);
}
