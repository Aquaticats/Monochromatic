//! What:
//!  The controls of finding F9 of the sixth review and its seventh-review follow-ups: the hook-bypass ordering.
//! Why:
//!  The held request with a disjoint and an overlapping successor, refusing and succeeding later, is replayed against
//!  each rule; the seventh review's release-point and superseded-version schedules are red under revision 7.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision seven: the consumption entry is released at publication', ...);
//! ```

// What:     imports the model's entry points and the explorer's helpers.
// Why:      The tests replay schedules and explore the model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, initial, steps, invariant, endState, Rules } from './model_f9_hook_bypass';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f9_hook_bypass::{Rules, State, check, end_state, initial, invariant, steps};

/// Revision 6 builds the disjoint successor on the logical tail, which includes the unapproved held request.
#[test]
fn revision_six_exposes_the_held_request_through_a_successor() {
    let reached = replay(
        initial(Rules::Six, false, false),
        steps,
        &["lane: builds the successor on the logical tail, which includes the held request, and publishes it"],
    );
    assert_eq!(
        invariant(&reached),
        Some("the held request was exposed on the branch before its hook succeeded".to_string())
    );
}

/// Revision 7, the review's four schedules for a disjoint and an overlapping successor.
#[test]
fn revision_seven_orders_the_successor_ahead_and_the_held_request_after() {
    let succeeds = replay(
        initial(Rules::Seven, false, false),
        steps,
        &[
            "lane: publishes the disjoint successor by replaying its own delta on the tip",
            "environment: the hook of version one returns ok",
            "lane: re-enters the held request at the tail and publishes it by replaying its delta on the tip",
        ],
    );
    assert!(succeeds.tip_y);
    assert_eq!(succeeds.tip_x, 1);
    assert_eq!(invariant(&succeeds), None);
    let refuses = replay(
        initial(Rules::Seven, false, false),
        steps,
        &["lane: publishes the disjoint successor by replaying its own delta on the tip", "environment: the hook of version one refuses"],
    );
    assert!(refuses.tip_y);
    assert_eq!(refuses.tip_x, 0);
    assert_eq!(invariant(&refuses), None);
    let overlapping = replay(
        initial(Rules::Seven, true, false),
        steps,
        &[
            "lane: sets the overlapping successor aside until the held request's hook decides",
            "environment: the hook of version one refuses",
            "lane: publishes the overlapping successor now that the held request is decided",
        ],
    );
    assert!(overlapping.tip_y);
    assert_eq!(end_state(&overlapping), None);
}

/// Revision 7, seventh review: the consumption entry is released at publication while a foreign program holds
/// `index.lock`, so a plain commit of another path captures a reversal of the held request.
#[test]
fn revision_seven_releases_the_consumption_entry_at_publication() {
    let reached = replay(
        initial(Rules::Seven, false, false),
        steps,
        &[
            "foreign: takes index.lock",
            "environment: the hook of version one returns ok",
            "lane: re-enters the held request at the tail and publishes it by replaying its delta on the tip",
            "client: a plain commit stages only another path",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("a plain commit captured a reversal of the held request that the person did not stage".to_string())
    );
}

/// Revision 7, seventh review: an amendment supersedes the held version; the first version's hook then completes and
/// is attributed to the request, which publishes the superseded version.
#[test]
fn revision_seven_publishes_a_superseded_version_when_its_hook_completes() {
    let reached = replay(
        initial(Rules::Seven, false, true),
        steps,
        &[
            "person: amends the held request, a new version supersedes it and runs its hook",
            "environment: the hook of version one returns ok",
            "lane: re-enters the held request at the tail and publishes it by replaying its delta on the tip",
        ],
    );
    assert_eq!(invariant(&reached), Some("a superseded version of the held request was published".to_string()));
}

/// The exhaustive search finds a violation under each of the two earlier rules.
#[test]
fn the_search_finds_a_violation_under_revisions_six_and_seven() {
    for rules in [Rules::Six, Rules::Seven] {
        let report: Report = check(rules);
        assert!(report.violation.is_some(), "no violation under {:?}", rules);
    }
}

/// The candidate: the consumption entry is released at materialization, and a hook's completion is tied to its version.
/// No exposure, no superseded publication, no invented reversal, and no independent request waits on a hook.
#[test]
fn candidate_orders_the_bypass_and_releases_the_map_at_materialization() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F9 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// The schedule shared by the next two tests: the hook succeeds, the request publishes, a foreign successor changes the path,
/// M1's entry rule declines (M2), and a plain commit of another path follows.
fn foreign_successor_then_plain_commit(rules: Rules) -> State {
    return replay(
        initial(rules, false, false),
        steps,
        &[
            "environment: the hook of version one returns ok",
            "lane: re-enters the held request at the tail and publishes it by replaying its delta on the tip",
            "foreign: lands a successor on the held request's path",
            "M2 void: the entry rule declined because a foreign successor changed the path",
            "client: a plain commit stages only another path",
        ],
    );
}

/// Revision 8 (red): voiding releases the consumption entries, and the old entry returns as staging in an unrelated commit.
#[test]
fn eight_lets_a_voided_entry_return_as_staging() {
    let reached = foreign_successor_then_plain_commit(Rules::Eight);
    assert_eq!(
        invariant(&reached),
        Some("a plain commit captured a reversal of the held request that the person did not stage".to_string())
    );
}

/// Revision 9 (green): M2 keeps the entry as `suppress`, so the same schedule captures nothing.
#[test]
fn nine_suppresses_a_voided_entry() {
    let reached = foreign_successor_then_plain_commit(Rules::Nine);
    assert_eq!(reached.r3, 1);
    assert_eq!(invariant(&reached), None);
}

/// Revision 8 (red): a held version succeeds, the tip moves, and the replay spawns the hook again.
#[test]
fn eight_runs_the_hook_again_when_the_tip_moves() {
    let reached = replay(
        initial(Rules::Eight, false, false),
        steps,
        &[
            "environment: the hook of version one returns ok",
            "foreign: moves the tip after the hook succeeded",
            "B1: the version is replayed on the moved tip and its hook is spawned again",
        ],
    );
    assert_eq!(invariant(&reached), Some("a hook ran again for a version whose hook had already succeeded".to_string()));
}

/// Revision 8 (red): an amendment supersedes a held version and nothing records the old version's outcome. Revision 9 (green):
/// B3 records it superseded.
#[test]
fn eight_leaves_a_superseded_version_without_a_terminal_outcome_and_nine_records_it() {
    let amend: [&str; 1] = ["person: amends the held request, a new version supersedes it and runs its hook"];
    let missing: Option<String> = Some("a superseded version never reached a durable terminal outcome".to_string());
    assert_eq!(end_state(&replay(initial(Rules::Eight, false, true), steps, &amend)), missing);
    assert_ne!(end_state(&replay(initial(Rules::Nine, false, true), steps, &amend)), missing);
}

/// Revision 9 exhaustive: all variants, no violation.
#[test]
fn nine_orders_the_bypass_suppresses_voids_runs_a_hook_once_and_records_superseded_versions() {
    let report: Report = check(Rules::Nine);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F9 nine: {} states, deepest {}", report.states, report.deepest);
    assert!(check(Rules::Eight).violation.is_some());
}
