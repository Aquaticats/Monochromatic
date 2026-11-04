//! What:
//!  The controls of reclamation during capture: the components' rule is green over every schedule of write-back, one power loss, the
//!  departure's capture, force and return, and the reclaimer's candidate and removal; each clause has a single-defect control.
//! Why:
//!  Without the controls a green search could be one that never reaches a removal or a return.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('reclamation during capture', () => { expect(check(Checked).violation).toBeUndefined(); expect(check(ReclaimsAnyCandidate).violation).toBeDefined(); });
//! ```

use crate::model_capture_reclaim_r23::{Departure, Rules, State, check, durable_cover, initial, invariant, steps};
use crate::model_explore::{Report, replay};

const CAPTURES: &str = "departure: captures the acknowledgment of the promised record in the current segment";
const FORCES_SEGMENT: &str = "departure: forces the segment it captured";
const RETURNS: &str = "departure: verifies the log by name and returns success";
const DESCENDS: &str = "owner: writes a checkpoint candidate that descends from the promised record";
const UNRELATED: &str = "owner: writes a checkpoint candidate that changed the document";
const FORCES_CANDIDATE: &str = "owner: forces the candidate";
const FORCES_NAME: &str = "owner: forces the candidate's directory entry";
const REMOVES: &str = "owner: removes the old segment";
const LOSS: &str = "environment: power is lost";
const PERSISTS_REMOVAL: &str = "environment: write-back persists the removal of the old segment";

/// The first violation, or a panic when the search finds none.
fn red(rules: Rules) -> (String, Vec<String>) {
    let found = check(rules).violation.unwrap_or_else(|| { panic!("{:?} must be red", rules) });
    return (found.message, found.trace);
}

/// The components' rule over every schedule is green and exhaustive.
#[test]
fn the_checked_rule_is_green_over_every_schedule() {
    let report: Report = check(Rules::Checked);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// Positive control: the green search reaches a returned departure and a removed segment, in one schedule, so the green is not one that
/// never reaches either.
#[test]
fn the_green_search_reaches_a_return_and_a_removal_in_one_schedule() {
    let done: State = replay(
        initial(Rules::Checked),
        steps,
        &[CAPTURES, FORCES_SEGMENT, RETURNS, DESCENDS, FORCES_CANDIDATE, FORCES_NAME, REMOVES, PERSISTS_REMOVAL],
    );
    assert_eq!(done.departure, Departure::Returned);
    assert!(durable_cover(&done) && invariant(&done).is_none(), "the replacement covers the promise after the removal is durable");
    let lost: State = replay(done, steps, &[LOSS]);
    assert!(durable_cover(&lost), "the covered promise survives the power loss");
}

/// A checkpoint that changed the document and recorded no lineage replaces the segment that holds the promise: the control that removes the
/// segment for any durable candidate leaves a log that does not cover what the departure reported as kept.
#[test]
fn a_reclaimer_that_removes_the_segment_for_an_unrelated_candidate_loses_the_promise() {
    let (message, trace) = red(Rules::ReclaimsAnyCandidate);
    assert!(message.contains("does not cover the record it acknowledged"));
    assert!(trace.iter().any(|label| { return label == UNRELATED; }) && trace.iter().any(|label| { return label == REMOVES; }), "{:?}", trace);
    let lost: State = replay(initial(Rules::ReclaimsAnyCandidate), steps, &[CAPTURES, FORCES_SEGMENT, RETURNS, UNRELATED, FORCES_CANDIDATE, FORCES_NAME, REMOVES, PERSISTS_REMOVAL]);
    assert!(invariant(&lost).is_some());
}

/// The reclaimer removes the segment only after the candidate is durable: the control that removes it at once leaves a log with neither the
/// segment nor the candidate when the removal persists and the candidate does not.
#[test]
fn a_reclaimer_that_removes_the_segment_before_the_candidate_is_durable_loses_the_promise_in_a_power_loss() {
    let (message, trace) = red(Rules::RemovesBeforeCandidateDurable);
    assert!(message.contains("does not cover the record it acknowledged"));
    assert!(trace.iter().any(|label| { return label == REMOVES; }), "{:?}", trace);
    let lost: State = replay(initial(Rules::RemovesBeforeCandidateDurable), steps, &[CAPTURES, FORCES_SEGMENT, RETURNS, DESCENDS, REMOVES, PERSISTS_REMOVAL]);
    assert!(invariant(&lost).is_some(), "the removal is durable and the candidate is not");
}

/// The departure returns on a durable cover: the control that returns on what a reader sees returns with the record still unforced, and the
/// power loss takes it.
#[test]
fn a_departure_that_returns_on_a_visible_cover_loses_the_promise_in_a_power_loss() {
    let (message, trace) = red(Rules::ReturnsOnVisibleCover);
    assert!(message.contains("does not cover the record it acknowledged"));
    assert!(trace.iter().any(|label| { return label == RETURNS; }), "{:?}", trace);
    let returned: State = replay(initial(Rules::ReturnsOnVisibleCover), steps, &[CAPTURES, RETURNS]);
    assert!(invariant(&returned).is_some());
}

/// A departure that forces a segment the reclaimer has already removed has forced bytes no name leads to: with the candidate unrelated and
/// the old name gone, the checked rule never lets the removal happen, so the departure's force is never on an orphan that the log relies on.
#[test]
fn under_the_checked_rule_no_schedule_removes_the_segment_for_an_unrelated_candidate() {
    let blocked: State = replay(initial(Rules::Checked), steps, &[UNRELATED, FORCES_CANDIDATE, FORCES_NAME]);
    assert!(steps(&blocked).iter().all(|(label, _)| { return label != REMOVES; }), "the checked rule removed the segment for a candidate that does not descend");
}
