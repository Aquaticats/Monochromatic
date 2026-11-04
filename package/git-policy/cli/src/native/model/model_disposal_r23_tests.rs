//! What:
//!  The controls of the disposal of saved content: the design's order is green over every schedule of write-back, one power loss and the
//!  owner's steps; each clause has a single-defect control.
//! Why:
//!  The green search must reach a removal and a dropped root, and each control must be red for its own defect only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('disposal', () => { expect(check(Ordered).violation).toBeUndefined(); expect(check(RemovesBeforeLineDurable).violation).toBeDefined(); });
//! ```

use crate::model_disposal_r23::{LINE, GONE, Rules, State, check, durable_orphan, initial, invariant, steps};
use crate::model_explore::{Report, replay};

const AUTHORIZES: &str = "person: disposes of the saved content explicitly";
const APPENDS: &str = "owner: appends the disposed line to the ledger";
const FORCES_LINE: &str = "owner: forces the disposed line";
const REMOVES: &str = "owner: removes the saved content";
const RETIRES: &str = "owner: retires the request and removes its saved content";
const DROPS: &str = "owner: checkpoints a root that no longer names the item";
const LOSS: &str = "environment: power is lost";
const PERSISTS_REMOVAL: &str = "environment: write-back persists the removal of the saved content";
const PERSISTS_DROP: &str = "environment: write-back persists the root without the item";

/// The first violation, or a panic when the search finds none.
fn red(rules: Rules) -> (String, Vec<String>) {
    let found = check(rules).violation.unwrap_or_else(|| { panic!("{:?} must be red", rules) });
    return (found.message, found.trace);
}

/// The design's order over every schedule is green and exhaustive.
#[test]
fn the_ordered_rule_is_green_over_every_schedule() {
    let report: Report = check(Rules::Ordered);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// Positive control: the green search reaches a removal and a dropped root after a forced line, and a power loss then leaves no orphan.
#[test]
fn the_green_search_reaches_a_removal_and_a_dropped_root_and_the_loss_leaves_no_orphan() {
    let done: State = replay(initial(Rules::Ordered), steps, &[AUTHORIZES, APPENDS, FORCES_LINE, REMOVES, DROPS, PERSISTS_REMOVAL, PERSISTS_DROP]);
    assert!(done.world.durable & GONE != 0 && done.world.durable & LINE != 0);
    assert!(invariant(&done).is_none());
    let lost: State = replay(done, steps, &[LOSS]);
    assert!(!durable_orphan(&lost));
}

/// Without the person's disposal nothing removes the saved content: the control in which retirement removes it is red, and under the
/// design's rule the removal step is not enabled before the authorization.
#[test]
fn retirement_never_removes_saved_content_and_the_control_that_does_is_red() {
    let (message, trace) = red(Rules::RetirementRemoves);
    assert!(message.contains("did not authorize"));
    assert!(trace.iter().any(|label| { return label == RETIRES; }), "{:?}", trace);
    let start: State = initial(Rules::Ordered);
    assert!(steps(&start).iter().all(|(label, _)| { return label != REMOVES && label != RETIRES; }), "a removal is enabled before any disposal");
}

/// The removal is never durable ahead of the line: the control that removes once the line is appended is red, with a power loss that keeps
/// the removal and loses the line, and a root that still names the item.
#[test]
fn a_removal_before_the_line_is_durable_leaves_a_root_that_names_a_lost_item() {
    let (message, trace) = red(Rules::RemovesBeforeLineDurable);
    assert!(message.contains("names a saved item whose content is gone"));
    assert!(trace.iter().any(|label| { return label == REMOVES; }), "{:?}", trace);
    let lost: State = replay(initial(Rules::RemovesBeforeLineDurable), steps, &[AUTHORIZES, APPENDS, REMOVES, PERSISTS_REMOVAL, LOSS]);
    assert!(durable_orphan(&lost));
    assert_eq!(invariant(&lost).map(|text| { return text.contains("names a saved item"); }), Some(true));
}

/// The root does not drop the item ahead of the line either: the control that does is red.
#[test]
fn a_root_that_drops_the_item_before_the_line_is_durable_is_red() {
    let (message, trace) = red(Rules::RootDropsBeforeLineDurable);
    assert!(message.contains("dropped a saved item"));
    assert!(trace.iter().any(|label| { return label == DROPS; }), "{:?}", trace);
}
