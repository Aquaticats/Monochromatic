//! What:
//!  The controls of reclamation during capture: the components' rule is green over every schedule of write-back, one power loss, the
//!  departure's capture, force, return and recapture, and the reclaimer's candidate and removal; each clause has a control.
//! Why:
//!  Without the controls a green search could be one that never reaches a removal, a recapture or a return.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('reclamation during capture', () => { expect(check(Checked).violation).toBeUndefined(); expect(check(NoSubstitution).violation).toBeDefined(); });
//! ```

use crate::model_capture_reclaim_r23::{Departure, Root, Rules, State, check, durable_root, initial, invariant, steps, terminal};
use crate::model_explore::{Report, replay};

const CAPTURES: &str = "departure: captures the acknowledgment of the record the log selects now";
const FORCES: &str = "departure: forces the file it captured";
const RETURNS: &str = "departure: verifies the log by name and returns success";
const GONE: &str = "departure: opening the old segment finds it gone and the log is selected again";
const RECAPTURES: &str = "departure: the selected root is not the one it captured, so it captures again";
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

/// Positive control for the removal: a departure that returned before the removal leaves a log whose root is the candidate that replaced
/// the segment, and the root survives the power loss.
#[test]
fn the_green_search_reaches_a_return_and_a_removal_in_one_schedule() {
    let done: State = replay(initial(Rules::Checked), steps, &[CAPTURES, FORCES, RETURNS, DESCENDS, FORCES_CANDIDATE, FORCES_NAME, REMOVES, PERSISTS_REMOVAL]);
    assert_eq!(done.departure, Departure::Returned);
    assert_eq!(durable_root(&done), Root::Newer);
    assert!(invariant(&done).is_none());
    assert_eq!(durable_root(&replay(done, steps, &[LOSS])), Root::Newer);
}

/// The review's schedule (finding 5): capture, force the old segment, write an unrelated higher-generation candidate, force it and its name.
/// The selected root is now the candidate and does not cover the capture, so the departure may not return on the old file's survival; it
/// captures the new root, forces it, and returns.
#[test]
fn an_unrelated_higher_generation_candidate_becomes_the_root_and_the_departure_captures_again() {
    let selected: State = replay(initial(Rules::Checked), steps, &[CAPTURES, FORCES, UNRELATED, FORCES_CANDIDATE, FORCES_NAME]);
    assert_eq!(durable_root(&selected), Root::Newer);
    assert!(steps(&selected).iter().all(|(label, _)| { return label != RETURNS; }), "a return on the survival of the old file alone");
    let returned: State = replay(selected, steps, &[RECAPTURES, CAPTURES, FORCES, RETURNS]);
    assert_eq!(returned.departure, Departure::Returned);
}

/// The ENOENT schedule: the departure selected the old segment, the reclaimer unlinked it, and the open of the path fails; the departure
/// selects again and captures the candidate.
#[test]
fn a_departure_that_finds_the_old_segment_gone_selects_again() {
    let gone: State = replay(initial(Rules::Checked), steps, &[CAPTURES, DESCENDS, FORCES_CANDIDATE, FORCES_NAME, REMOVES]);
    assert!(steps(&gone).iter().any(|(label, _)| { return label == GONE; }), "the open of the removed path has its own step");
    assert!(steps(&gone).iter().all(|(label, _)| { return label != FORCES; }), "the old file is not forced after its name is gone");
    let returned: State = replay(gone, steps, &[GONE, CAPTURES, FORCES, RETURNS]);
    assert_eq!(returned.departure, Departure::Returned);
}

/// Without the departure's recapture an unrelated candidate leaves it with a capture that no root covers: red, with the candidate in the trace.
#[test]
fn a_departure_that_never_captures_again_is_stuck_when_the_root_changes() {
    let (message, trace) = red(Rules::NoSubstitution);
    assert_eq!(message, "a departure that captured a record can never return");
    assert!(trace.iter().any(|label| { return label == UNRELATED; }), "{:?}", trace);
    let stuck: State = replay(initial(Rules::NoSubstitution), steps, &[CAPTURES, FORCES, UNRELATED, FORCES_CANDIDATE, FORCES_NAME]);
    assert!(steps(&stuck).iter().all(|(label, _)| { return label.starts_with("environment:") || label.starts_with("owner:"); }), "the departure has a step of its own left");
    assert!(terminal(&stuck).is_some());
}

/// With the departure's recapture, removing the old segment for any durable candidate is green: the lineage restriction on removal is not
/// what keeps a departure's promise. It keeps an acknowledgment taken before the removal covered (`state_covers`) without a recapture, which
/// the component tests (`r17_a_checked_document_reclaims_and_covers...`) show.
#[test]
fn with_recapture_removal_for_an_unrelated_candidate_is_green_so_lineage_is_not_what_keeps_the_promise() {
    let report: Report = check(Rules::ReclaimsAnyCandidate);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// The reclaimer removes the segment only after the candidate is durable: the control that removes it at once leaves a log with neither
/// the segment nor the candidate when the removal persists and the candidate does not.
#[test]
fn a_reclaimer_that_removes_the_segment_before_the_candidate_is_durable_leaves_no_root_in_a_power_loss() {
    let (message, trace) = red(Rules::RemovesBeforeCandidateDurable);
    assert!(message.contains("no root"));
    assert!(trace.iter().any(|label| { return label == REMOVES; }), "{:?}", trace);
    let lost: State = replay(initial(Rules::RemovesBeforeCandidateDurable), steps, &[CAPTURES, FORCES, RETURNS, DESCENDS, REMOVES, PERSISTS_REMOVAL]);
    assert!(invariant(&lost).is_some(), "the removal is durable and the candidate is not");
}

/// The visible-cover control is a combined weakening (it allows a return straight from the capture and replaces the durable root by the
/// visible one), not a single-guard mutation: it returns with the record unforced, and the power loss takes it.
#[test]
fn a_departure_that_returns_on_a_visible_cover_returns_with_nothing_durable() {
    let (message, trace) = red(Rules::ReturnsOnVisibleCover);
    assert!(message.contains("no root"));
    assert!(trace.iter().any(|label| { return label == RETURNS; }), "{:?}", trace);
}
