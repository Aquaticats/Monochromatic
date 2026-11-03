//! What:
//!  The controls of finding F6 of the sixth review and its seventh-review follow-up: consistent capture.
//! Why:
//!  Revision 6's schedule (a materializer paused before it advances the version) mixes a snapshot; revision 7's
//!  seqlock stops the mix but holds the odd generation across a reference-transaction hook, so acceptance can wait on
//!  a person's program; the candidate keeps `update-ref` outside the odd section.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision six: a paused materializer mixes a capture', ...);
//! ```

// What:     imports the model's entry points and the explorer's helpers.
// Why:      The tests replay schedules and explore the model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, initial, steps, invariant, endState, Rules } from './model_f6_capture';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f6_capture::{Rules, check, end_state, initial, initial_with, invariant, steps};

/// Revision 6, the reviewer's schedule: the lane installed the index entry and paused before it advanced the version;
/// the capture reads an unchanged version and accepts a mixed snapshot.
#[test]
fn revision_six_accepts_a_snapshot_mixed_by_a_paused_materializer() {
    let reached = replay(
        initial(Rules::Six),
        steps,
        &[
            "lane: writes the logical head",
            "lane: installs the index entry",
            "client: reads the generation",
            "client: reads the logical head",
            "client: reads the index entry",
            "client: reads the consumption entry",
            "client: reads the generation again, unchanged, and accepts the capture",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("a capture accepted a snapshot that mixes before and after of one lane mutation".to_string())
    );
}

/// Revision 7: the lane journals, writes the generation odd and invokes `update-ref`, whose hook waits; a capture of a
/// disjoint request cannot begin, so acceptance waits behind a person's program.
#[test]
fn revision_seven_holds_the_odd_generation_across_a_waiting_hook() {
    let reached = replay(
        initial(Rules::Seven),
        steps,
        &[
            "lane: journals the mutation and fsyncs it",
            "lane: writes the generation odd",
            "lane: invokes update-ref, whose reference-transaction hook waits",
        ],
    );
    assert!(steps(&reached).iter().all(|entry| { return entry.0.starts_with("environment:"); }));
    assert!(end_state(&reached).is_some());
}

/// The exhaustive search finds a violation under each of the two earlier rules.
#[test]
fn the_search_finds_a_violation_under_revisions_six_and_seven() {
    for rules in [Rules::Six, Rules::Seven] {
        let report: Report = check(rules);
        assert!(report.violation.is_some(), "no violation under {:?}", rules);
    }
}

/// The candidate: the odd generation brackets only local writes, so a capture is never mixed and never waits on a
/// hook, with a lane that can die in the middle of a mutation and a new owner that completes it.
#[test]
fn candidate_captures_consistently_and_never_waits_on_a_hook() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F6 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// Revision 8 draft, L4 read literally: the lane journaled a mutation and died before it raised the generation; the new
/// owner completes the writes with the generation even, and a capture accepts the mix. A question for the draft (does L4
/// raise the generation first?), not a candidate; the other lane transitions match the candidate.
#[test]
fn eight_resume_without_raising_the_generation_lets_a_capture_mix() {
    let reached = replay(
        initial(Rules::Eight),
        steps,
        &[
            "lane: journals the mutation and fsyncs it",
            "environment: the lane process dies",
            "lane: a new owner resumes the journaled mutation from its script",
            "client: reads the generation",
            "client: reads the logical head",
            "L4 resume: writes the logical head with the generation unchanged",
            "L4 resume: installs the index entry",
            "client: reads the index entry",
            "L4 resume: updates the consumption map",
            "client: reads the consumption entry",
            "client: reads the generation again, unchanged, and accepts the capture",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("a capture accepted a snapshot that mixes before and after of one lane mutation".to_string())
    );
    assert!(check(Rules::Eight).violation.is_some());
}

/// Revision 8 (red): the lane died with the generation odd and no service exists to start a new owner; every capture waits
/// for ever, because nothing but an owner clears the marker.
#[test]
fn eight_dead_odd_generation_with_no_service_blocks_acceptance() {
    let reached = replay(
        initial_with(Rules::Eight, false),
        steps,
        &[
            "lane: journals the mutation and fsyncs it",
            "lane: invokes update-ref, whose reference-transaction hook waits",
            "environment: the reference-transaction hook returns",
            "lane: writes the generation odd",
            "environment: the lane process dies",
        ],
    );
    assert!(steps(&reached).iter().all(|entry| { return entry.0.starts_with("environment:"); }));
    assert!(end_state(&reached).is_some());
}

/// Revision 9 (green): C0 gives the client a route into recovery, L4 writes the generation odd first, and no capture mixes;
/// the schedule that mixed under revision 8 (resume without raising the generation) is closed.
#[test]
fn nine_recovers_through_c0_and_raises_the_generation_first() {
    let reached = replay(
        initial_with(Rules::Nine, false),
        steps,
        &[
            "lane: journals the mutation and fsyncs it",
            "environment: the lane process dies",
            "C0 ensure the lane: the client acts as an inline owner and resumes the journaled mutation",
        ],
    );
    assert!(!reached.lane_dead);
    let report: Report = check(Rules::Nine);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F6 nine: {} states, deepest {}", report.states, report.deepest);
}
