//! What:
//!  Controls of the observation vocabulary: the world's operations behave as documented, and a claim that rests on visible
//!  evidence is red where the same claim on durable evidence is green.
//! Why:
//!  The models of the commit protocol are to be built from prefixes and evidence; this shows the vocabulary can tell them apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a claim on visible evidence loses the outcome in a power loss', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_observe::{
    Evidence, HOOK_RESULT, NOTHING, RENAME_DONE, ROOT_EXISTS, TERMINAL_OUTCOME, check, holds, initial, invariant, make_visible, power_loss, steps, sync,
};

/// A fact is visible before it is durable, a sync of an absent fact does nothing, and a power loss keeps only the durable facts.
#[test]
fn the_world_separates_visible_from_durable() {
    let appended = make_visible(NOTHING, TERMINAL_OUTCOME | ROOT_EXISTS);
    assert!(holds(appended, TERMINAL_OUTCOME, Evidence::Visible));
    assert!(!holds(appended, TERMINAL_OUTCOME, Evidence::Durable));
    let synced = sync(appended, TERMINAL_OUTCOME);
    assert!(holds(synced, TERMINAL_OUTCOME, Evidence::Durable));
    assert!(!holds(synced, ROOT_EXISTS, Evidence::Durable));
    assert_eq!(sync(NOTHING, HOOK_RESULT), NOTHING);
    let after = power_loss(synced);
    assert!(holds(after, TERMINAL_OUTCOME, Evidence::Visible));
    assert!(!holds(after, ROOT_EXISTS, Evidence::Visible));
    assert!(!holds(after, RENAME_DONE, Evidence::Visible));
}

/// A claim on visible evidence: the lane saw the line, released, and the power loss removed the line.
#[test]
fn a_claim_on_visible_evidence_loses_the_outcome() {
    let reached = replay(
        initial(Evidence::Visible),
        steps,
        &[
            "writer: appends the terminal line to the ledger",
            "lane: observes the terminal line visible and releases the retention",
            "environment: power is lost",
        ],
    );
    assert_eq!(
        invariant(&reached),
        Some("the retention was released and the terminal outcome is not discoverable after the power loss".to_string())
    );
    assert!(check(Evidence::Visible).violation.is_some());
}

/// A claim on durable evidence: every schedule, a reader between the append and the sync included, is green.
#[test]
fn a_claim_on_durable_evidence_keeps_the_outcome() {
    let report: Report = check(Evidence::Durable);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("observations durable: {} states, deepest {}", report.states, report.deepest);
}
