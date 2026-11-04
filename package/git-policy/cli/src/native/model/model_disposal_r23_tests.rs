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

/// A replacement root without the ledger proof is not eligible either: the control that removes the content on the durable root alone,
/// whose line was never forced, is red with a lost-content schedule.
#[test]
fn a_replacement_root_without_its_ledger_proof_does_not_make_the_removal_safe() {
    let (message, trace) = red(Rules::RemovesOnRootWithoutLineProof);
    assert!(message.contains("neither the old root nor a durable replacement root"));
    assert!(trace.iter().any(|label| { return label == REMOVES; }), "{:?}", trace);
}
