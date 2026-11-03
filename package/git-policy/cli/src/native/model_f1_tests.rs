//! What:
//!  The controls of finding F1 of the sixth review and its seventh-review follow-up: publication evidence.
//! Why:
//!  Revision 6 reads an intact anchor with no old-value line as "the swap did not complete"; revision 7 lets a recorded
//!  swap failure outrank positive evidence; the candidate orders the evidence so neither happens.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision six: an intact anchor with deleted lines republishes silently', ...);
//! ```

// What:     imports the model's entry points and the explorer's helpers.
// Why:      The tests replay schedules and explore the model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { check, initial, steps, invariant, Rules } from './model_f1_evidence';
// ```
use crate::model_explore::{Report, replay};
use crate::model_f1_evidence::{Rules, check, initial, invariant, steps};

/// Revision 6, the reviewer's schedule: the swap succeeded, another update moved the branch away, a legitimate reflog
/// operation deleted the lines after the anchor, the owner died; the anchor is intact, so recovery says "not completed".
#[test]
fn revision_six_reads_an_intact_anchor_with_deleted_lines_as_not_published() {
    let reached = replay(
        initial(Rules::Six),
        steps,
        &[
            "lane: records publishing with the reflog anchor",
            "lane: the swap succeeds",
            "foreign: resets the branch away from the commit",
            "reflog operation: deletes the lines after the anchor, keeping the anchor",
            "environment: the owner dies without a durable published record",
            "recovery: decides from the evidence",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("a commit the branch had held and lost was published again silently".to_string())
    );
}

/// Revision 7 handles that schedule: the evidence is unclear, so the commit is landed with a message.
#[test]
fn revision_seven_lands_with_a_message_when_the_lines_are_deleted() {
    let reached = replay(
        initial(Rules::Seven),
        steps,
        &[
            "lane: records publishing with the reflog anchor",
            "lane: the swap succeeds",
            "foreign: resets the branch away from the commit",
            "reflog operation: deletes the lines after the anchor, keeping the anchor",
            "environment: the owner dies without a durable published record",
            "recovery: decides from the evidence",
        ],
    );
    assert_eq!(reached.outcome, 3);
    assert_eq!(invariant(&reached), None);
}

/// The identical commit written by another writer: positive old-value evidence, saved and not published again, under
/// revision 7 and the candidate.
#[test]
fn an_identical_commit_written_and_removed_by_another_writer_is_saved() {
    for rules in [Rules::Seven, Rules::Candidate] {
        let reached = replay(
            initial(rules),
            steps,
            &[
                "lane: records publishing with the reflog anchor",
                "foreign: installs exactly the lane's commit",
                "foreign: resets the branch away from the commit",
                "environment: the owner dies without a durable published record",
                "recovery: decides from the evidence",
            ],
        );
        assert_eq!(reached.outcome, 4, "under {:?}", rules);
        assert_eq!(invariant(&reached), None);
    }
}

/// Revision 7, seventh review: the swap failed on a held ref lock and was recorded; the other writer then installed
/// exactly the commit and reset the branch; the owner died; the recorded failure outranks the positive evidence and the
/// commit is published again silently.
#[test]
fn revision_seven_lets_a_recorded_swap_failure_outrank_positive_evidence() {
    let reached = replay(
        initial(Rules::Seven),
        steps,
        &[
            "lane: records publishing with the reflog anchor",
            "foreign: takes the ref lock",
            "lane: the swap fails because another writer holds the ref lock, and records swap-failed",
            "foreign: releases the ref lock",
            "foreign: installs exactly the lane's commit",
            "foreign: resets the branch away from the commit",
            "environment: the owner dies without a durable published record",
            "recovery: decides from the evidence",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("a commit was published again although the reflog shows the branch moved away from it".to_string())
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

/// The candidate: positive evidence first, and nothing after an owner death is called not-published; the lane never
/// republishes silently what the branch held.
#[test]
fn candidate_orders_the_evidence_so_nothing_is_republished_silently() {
    let report: Report = check(Rules::Candidate);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F1 candidate: {} states, deepest {}", report.states, report.deepest);
}

/// Revision 8 draft, R3 before R4: positive evidence beats a recorded failure (the seventh review's schedule is closed).
#[test]
fn eight_checks_positive_evidence_before_a_recorded_failure() {
    let reached = replay(
        initial(Rules::Eight),
        steps,
        &[
            "lane: records publishing with the reflog anchor",
            "foreign: takes the ref lock",
            "lane: the swap fails because another writer holds the ref lock, and records swap-failed",
            "foreign: releases the ref lock",
            "foreign: installs exactly the lane's commit",
            "foreign: resets the branch away from the commit",
            "environment: the owner dies without a durable published record",
            "recovery: decides from the evidence",
        ],
    );
    assert_eq!(reached.outcome, 4);
    assert_eq!(invariant(&reached), None);
}

/// Revision 8 draft, R4: when a legitimate reflog operation has deleted the lines after the anchor, a recorded swap failure
/// replays the commit silently although the branch had held it and lost it. A finding against the draft, not a candidate.
#[test]
fn eight_replays_silently_when_the_evidence_was_erased() {
    let reached = replay(
        initial(Rules::Eight),
        steps,
        &[
            "lane: records publishing with the reflog anchor",
            "foreign: takes the ref lock",
            "lane: the swap fails because another writer holds the ref lock, and records swap-failed",
            "foreign: releases the ref lock",
            "foreign: installs exactly the lane's commit",
            "foreign: resets the branch away from the commit",
            "reflog operation: deletes the lines after the anchor, keeping the anchor",
            "environment: the owner dies without a durable published record",
            "recovery: decides from the evidence",
        ],
    );
    assert_eq!(invariant(&reached), Some("a commit the branch had held and lost was published again silently".to_string()));
    assert!(check(Rules::Eight).violation.is_some());
}

/// Revision 9 (red, a finding of the ninth review): the lane can observe only that the log after the anchor matches the anchor
/// record; erasing every post-anchor line makes it match, so R4 replays silently a commit the branch held and lost.
#[test]
fn nine_replays_silently_when_every_line_after_the_anchor_was_erased() {
    let reached = replay(
        initial(Rules::Nine),
        steps,
        &[
            "lane: records publishing with the reflog anchor",
            "foreign: takes the ref lock",
            "lane: the swap fails because another writer holds the ref lock, and records swap-failed",
            "foreign: releases the ref lock",
            "foreign: installs exactly the lane's commit",
            "foreign: resets the branch away from the commit",
            "reflog operation: deletes the lines after the anchor, keeping the anchor",
            "environment: the owner dies without a durable published record",
            "recovery: decides from the evidence",
        ],
    );
    assert_eq!(invariant(&reached), Some("a commit the branch had held and lost was published again silently".to_string()));
    assert!(check(Rules::Nine).violation.is_some());
}

/// Revision 10 (green): no oracle; the same schedule lands the commit with a notice, and every schedule is green.
#[test]
fn ten_lands_with_a_notice_without_an_oracle() {
    let reached = replay(
        initial(Rules::Ten),
        steps,
        &[
            "lane: records publishing with the reflog anchor",
            "foreign: takes the ref lock",
            "lane: the swap fails because another writer holds the ref lock, and records swap-failed",
            "foreign: releases the ref lock",
            "foreign: installs exactly the lane's commit",
            "foreign: resets the branch away from the commit",
            "reflog operation: deletes the lines after the anchor, keeping the anchor",
            "environment: the owner dies without a durable published record",
            "recovery: decides from the evidence",
        ],
    );
    assert_eq!(reached.outcome, 3);
    let report: Report = check(Rules::Ten);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("F1 ten: {} states, deepest {}", report.states, report.deepest);
}
