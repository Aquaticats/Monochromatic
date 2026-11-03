//! What:
//!  The controls of finding F7 of the sixth review: the same identity with different content from two clients.
//! Why:
//!  The design promises that an accepted request is immutable and that an identity is bound to its content for good.
//!  Under the revision-6 rule the checker finds the review's schedule (the control that proves it can see the
//!  violation); under the revision-7 rule it finds none; two mutations that drop one fsync each stay red.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision six is red', ...); test('revision seven is green', ...);
//! ```

// What:     `use crate::...` imports the model's entry point, its rule enum and the explorer's report type.
// Why:      The tests explore the model and read the report.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, Rules } from './model_f7_identity';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f7_identity::{Rules, State, acceptable_end, check, initial, invariant, steps};

/// The review's schedule under the revision-6 rule: the second client's rename replaces the first client's
/// acknowledged record, before the lane adopts it and after.
#[test]
fn revision_six_replaces_an_acknowledged_record() {
    let report: Report = check(Rules::Six);
    // What:     `.expect("...")` unwraps an `Option`, panicking with the text when it is `None`.
    // Why:      The control fails loudly if the checker no longer sees the violation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (report.violation === undefined) throw new Error('a violation under revision six');
    // ```
    let violation = report.violation.expect("a violation under revision six");
    assert_eq!(violation.message, "an acknowledged request is not the recorded content of its identity");
    assert!(violation.contains_in_order(&[
        "A: answers that the request is handed over",
        "B: renames its record over inbox/<id>",
    ]));
}

/// The adoption half of the review's schedule, replayed literally: both clients check before either publishes,
/// the lane adopts the first record, and the second client's rename then lands beside it.
#[test]
fn revision_six_records_two_contents_after_adoption() {
    let reached: State = replay(
        initial(Rules::Six),
        steps,
        &[
            "A: reads the inbox, the queue and the ledger and finds no record",
            "B: reads the inbox, the queue and the ledger and finds no record",
            "A: writes its record under a temporary name",
            "A: renames its record over inbox/<id>",
            "A: answers that the request is handed over",
            "lane: adopts the record, moving inbox/<id> to queue/<id>",
            "B: writes its record under a temporary name",
            "B: renames its record over inbox/<id>",
        ],
    );
    assert_eq!(invariant(&reached), Some("two different contents are recorded for one identity".to_string()));
}

/// The seventh review's hole A: `O_EXCL` makes the name visible before the hash is written, the creator dies, and a
/// resend finds a binding with no content.
#[test]
fn revision_seven_strands_an_identity_on_a_partial_binding() {
    let report: Report = check(Rules::Seven);
    let violation = report.violation.expect("a violation under revision seven");
    assert_eq!(violation.message, "an identity is reserved with no recoverable request and a client cannot go on");
    let reached: State = replay(
        initial(Rules::Seven),
        steps,
        &[
            "A: creates ids/<xx>/<id> with O_EXCL, still empty",
            "A: the process dies",
            "A: resends the same request",
            "A: finds a binding with no content",
        ],
    );
    assert!(acceptable_end(&reached).is_some());
}

/// The seventh review's hole B: a complete durable binding, the creator dies before the inbox record exists, and the
/// same-content resend has nothing to answer from.
#[test]
fn revision_seven_strands_an_identity_on_a_binding_without_a_record() {
    let reached: State = replay(
        initial(Rules::Seven),
        steps,
        &[
            "A: creates ids/<xx>/<id> with O_EXCL, still empty",
            "A: writes the hash into the binding",
            "A: fsyncs the binding file and its directory",
            "A: the process dies",
            "A: resends the same request",
            "A: finds its own hash in the binding and no record to answer from",
        ],
    );
    assert!(acceptable_end(&reached).is_some());
}

/// The candidate: publish the complete binding atomically and let a same-hash resend continue acceptance. Every
/// schedule, crashes and power losses included, keeps one content per identity and strands nobody.
#[test]
fn candidate_binds_one_content_and_strands_nobody() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F7 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// Without the fsync of the binding a power loss frees the identity for a second content.
#[test]
fn candidate_without_the_binding_sync_is_red() {
    let report: Report = check(Rules::CandidateNoBindingSync);
    let violation = report.violation.expect("a violation without the binding sync");
    assert!(violation.contains_in_order(&["power loss: unsynced files and directory entries are gone"]));
}

/// Without the fsync of the published record an acknowledged request vanishes with the power.
#[test]
fn candidate_without_the_record_sync_is_red() {
    let report: Report = check(Rules::CandidateNoRecordSync);
    let violation = report.violation.expect("a violation without the record sync");
    assert_eq!(violation.message, "an acknowledged request is not the recorded content of its identity");
    assert!(violation.contains_in_order(&["power loss: unsynced files and directory entries are gone"]));
}
