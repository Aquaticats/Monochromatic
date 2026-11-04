//! What:
//!  The controls of the disposal of saved content: the procedure is green over every schedule of write-back, one power loss and the owner's
//!  steps; each clause has a control.
//! Why:
//!  The green search must reach a removal after a durable replacement root, and each control must be red for its own defect only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('disposal', () => { expect(check(Ordered).violation).toBeUndefined(); expect(check(RemovesBeforeRootDurable).violation).toBeDefined(); });
//! ```

use crate::model_disposal_r23::{GONE, Rules, State, check, initial, invariant, recovery_has_a_root, steps};
use crate::model_explore::{Report, replay};

const AUTHORIZES: &str = "person: disposes of the saved content explicitly";
const APPENDS: &str = "owner: appends the disposed line to the ledger";
const FORCES_LINE: &str = "owner: forces the disposed line";
const CHECKPOINTS: &str = "owner: checkpoints a root that no longer names the item";
const FORCES_ROOT: &str = "owner: forces the root without the item";
const REMOVES: &str = "owner: removes the saved content";
const RETIRES: &str = "owner: retires the request and removes its saved content";
const CHECKS: &str = "owner: reads the current root before the unlink and finds it without the item";
const SEAL_STARTS: &str = "owner: a seal starts capturing the lane";
const RESEALS: &str = "owner: the seal publishes a root that names the item again";
const LOSS: &str = "environment: power is lost";
const PERSISTS_REMOVAL: &str = "environment: write-back persists the removal of the saved content";

/// The first violation, or a panic when the search finds none.
fn red(rules: Rules) -> (String, Vec<String>) {
    let found = check(rules).violation.unwrap_or_else(|| { panic!("{:?} must be red", rules) });
    return (found.message, found.trace);
}

/// The procedure over every schedule is green and exhaustive.
#[test]
fn the_ordered_procedure_is_green_over_every_schedule() {
    let report: Report = check(Rules::Ordered);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// Positive control: the green search reaches a removal after the durable replacement root, and a power loss then leaves an eligible root.
#[test]
fn the_green_search_reaches_a_removal_after_a_durable_replacement_root() {
    let done: State = replay(initial(Rules::Ordered), steps, &[AUTHORIZES, APPENDS, FORCES_LINE, CHECKPOINTS, FORCES_ROOT, REMOVES, PERSISTS_REMOVAL]);
    assert!(done.world.durable & GONE != 0 && invariant(&done).is_none());
    let lost: State = replay(done, steps, &[LOSS]);
    assert!(recovery_has_a_root(&lost));
}

/// Without the person's disposal nothing removes the saved content: the control in which retirement removes it is red, and under the
/// procedure the removal step is not enabled before the authorization.
#[test]
fn retirement_never_removes_saved_content_and_the_control_that_does_is_red() {
    let (message, trace) = red(Rules::RetirementRemoves);
    assert!(message.contains("did not authorize"));
    assert!(trace.iter().any(|label| { return label == RETIRES; }), "{:?}", trace);
    assert!(steps(&initial(Rules::Ordered)).iter().all(|(label, _)| { return label != REMOVES && label != RETIRES; }), "a removal is enabled before any disposal");
}

/// The review's schedule (finding 6): authorize, a durable line, remove the content, persist the removal, power loss, and no root that drops
/// the item was ever made durable. Recovery's validation fails the old root on its missing file: no eligible root.
#[test]
fn a_removal_on_a_durable_line_alone_leaves_recovery_with_no_eligible_root() {
    let (message, trace) = red(Rules::RemovesBeforeRootDurable);
    assert!(message.contains("neither the old root nor a durable replacement root"));
    assert!(trace.iter().any(|label| { return label == REMOVES; }), "{:?}", trace);
    let lost: State = replay(initial(Rules::RemovesBeforeRootDurable), steps, &[AUTHORIZES, APPENDS, FORCES_LINE, REMOVES, PERSISTS_REMOVAL, LOSS]);
    assert!(!recovery_has_a_root(&lost));
}

/// The review's re-retention schedule (step 2, parts A and B): the replacement root is durable, the item is still on disk, a seal captures
/// the lane and publishes a root that names the item again (reclaiming the replacement), and the unlink then deletes the item.
#[test]
fn a_seal_that_names_the_retired_item_again_leaves_recovery_with_no_eligible_root() {
    let (message, trace) = red(Rules::SealRescansRetired);
    assert!(message.contains("neither the old root nor a durable replacement root"), "{}", message);
    assert!(trace.iter().any(|label| { return label == RESEALS; }), "{:?}", trace);
    let lost: State = replay(
        initial(Rules::SealRescansRetired),
        steps,
        &[AUTHORIZES, APPENDS, FORCES_LINE, CHECKPOINTS, FORCES_ROOT, SEAL_STARTS, RESEALS, "environment: write-back persists the root that names the item again", REMOVES, PERSISTS_REMOVAL, LOSS],
    );
    assert!(!recovery_has_a_root(&lost));
}

/// Under the checked rule no seal captures the item once its line is appended, so a seal that starts after the line publishes a root without
/// it, and one that starts before the root exists has nothing to re-adopt over.
#[test]
fn under_the_checked_rule_a_seal_that_starts_after_the_line_does_not_name_the_item() {
    let after_line: State = replay(initial(Rules::Ordered), steps, &[AUTHORIZES, APPENDS, FORCES_LINE, CHECKPOINTS, SEAL_STARTS]);
    assert!(steps(&after_line).iter().all(|(label, _)| { return label != RESEALS; }), "a seal that started after the line named the item");
    let before_root: State = replay(initial(Rules::Ordered), steps, &[AUTHORIZES, SEAL_STARTS]);
    assert!(steps(&before_root).iter().all(|(label, _)| { return label != RESEALS; }), "the root without the item does not exist yet, so there is nothing to re-adopt over");
}

/// Review of step 2, part B, 1: a seal that began capturing before the line was appended, a disposal that appends its line and publishes its
/// root meanwhile, and the seal's publication of its earlier closure. The serialization (the state writer, held by the seal from its capture
/// to its publication and by the disposal from its line to its root) makes the interleaving unavailable: the line is not appended while a
/// seal is in flight and the seal does not start while a disposal is.
#[test]
fn the_state_writer_keeps_a_seal_and_a_disposal_from_interleaving() {
    let during_seal: State = replay(initial(Rules::Ordered), steps, &[AUTHORIZES, SEAL_STARTS]);
    assert!(steps(&during_seal).iter().all(|(label, _)| { return label != APPENDS; }), "a line was appended while a seal was in flight");
    let during_disposal: State = replay(initial(Rules::Ordered), steps, &[AUTHORIZES, APPENDS]);
    assert!(steps(&during_disposal).iter().all(|(label, _)| { return label != SEAL_STARTS; }), "a seal started while a disposal was in flight");
    let report: Report = check(Rules::Ordered);
    assert!(report.violation.is_none() && report.exhausted);
}

/// Without the serialization the skip rule is not enough: the control has the schedule of the review (the seal captured before the line,
/// published after the replacement root, the unlink in between) and is red.
#[test]
fn without_the_serialization_a_seal_that_began_before_the_line_publishes_over_the_replacement() {
    let (message, trace) = red(Rules::NoSerialization);
    assert!(message.contains("neither the old root nor a durable replacement root"), "{}", message);
    let seal_started: usize = trace.iter().position(|label| { return label == SEAL_STARTS; }).expect("the seal's capture is in the schedule");
    let line: usize = trace.iter().position(|label| { return label == APPENDS; }).expect("the line");
    assert!(seal_started < line, "{:?}", trace);
    assert!(trace.iter().any(|label| { return label == RESEALS; }), "{:?}", trace);
}

/// A recheck of the current root before the unlink is not the exclusion: the seal can publish between the read and the unlink, so the
/// control with only the recheck is red too.
#[test]
fn a_recheck_alone_does_not_close_the_race_with_a_seal() {
    let (message, trace) = red(Rules::RecheckOnly);
    let read: usize = trace.iter().position(|label| { return label == CHECKS; }).expect("the read of the current root is in the schedule");
    let unlink: usize = trace.iter().position(|label| { return label == REMOVES; }).expect("the unlink");
    assert!(read < unlink, "{:?}", trace);
    assert!(message.contains("neither the old root nor a durable replacement root"), "{}", message);
    assert!(trace.iter().any(|label| { return label == RESEALS; }), "{:?}", trace);
}

/// A replacement root without the ledger proof is not eligible either: the control that removes the content on the durable root alone,
/// whose line was never forced, is red with a lost-content schedule. It is a combined weakening (it changes when the root may be made and
/// when the content may be removed); each of those two guards is safety-equivalent to the model only individually, because the other still
/// holds, which is not evidence that either ordering is unnecessary.
#[test]
fn a_replacement_root_without_its_ledger_proof_does_not_make_the_removal_safe() {
    let (message, trace) = red(Rules::RemovesOnRootWithoutLineProof);
    assert!(message.contains("neither the old root nor a durable replacement root"));
    assert!(trace.iter().any(|label| { return label == REMOVES; }), "{:?}", trace);
}

/// Review of step 3 (models): a power loss ends the volatile things the serialization rests on: a seal's capture, the writer ownership and
/// the unlink's read of the root. After a loss a disposal can append its line again, and a seal can start.
#[test]
fn a_power_loss_ends_the_capture_the_writer_ownership_and_the_read() {
    let after_loss: State = replay(initial(Rules::Ordered), steps, &[AUTHORIZES, SEAL_STARTS, LOSS]);
    assert!(!after_loss.seal_in_flight && !after_loss.seal_saw_item && !after_loss.disposal_in_flight && !after_loss.checked);
    assert!(steps(&after_loss).iter().any(|(label, _)| { return label == SEAL_STARTS; }), "a new seal could not start after the loss ended the first");
    let disposing: State = replay(initial(Rules::Ordered), steps, &[AUTHORIZES, APPENDS, LOSS]);
    assert!(!disposing.disposal_in_flight);
}

/// The fifth review of the diffs: the read of the root that the recheck-only control rests on is volatile. The schedule reaches a state in
/// which the read was made (`checked`), and a power loss ends it; with the reset removed the read would survive the loss.
#[test]
fn a_power_loss_ends_a_read_of_the_root_that_was_actually_made() {
    let schedule: [&str; 6] = [AUTHORIZES, APPENDS, FORCES_LINE, CHECKPOINTS, FORCES_ROOT, CHECKS];
    let read_made: State = replay(initial(Rules::RecheckOnly), steps, &schedule);
    assert!(read_made.checked, "the schedule never made the read the test is about");
    let mut with_loss: Vec<&str> = schedule.to_vec();
    with_loss.push(LOSS);
    let after_loss: State = replay(initial(Rules::RecheckOnly), steps, &with_loss);
    assert!(!after_loss.checked, "a read of the root survived a power loss");
}
