//! What:
//!  The controls of rule B for the foreground wait: W1 against P2, the decision race, crashes between them, and the bound.
//! Why:
//!  Revision 10 either waits without end behind a hook that never returns or returns exit 0 and lets the request land later;
//!  revision 11 withdraws at the bound by one link race that P2 also uses, so a request lands or never lands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision eleven: the client withdraws and the request never lands', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_w1_p2::{Rules, check, initial, invariant, steps};

const P2A: &str = "P2a prepare: starts update-ref --stdin, writes prepare, a person's hook runs inside";
const W1: &str = "W1: the bound is reached, the client links the decision withdraw and exits with an error";
const HOOK: &str = "environment: the person's hook returns and accepts";

/// Revision 10, reading one (red): the hook never returns, so the client waits and nothing of the protocol's own can end it.
#[test]
fn ten_waits_for_ever_behind_a_hook_that_never_returns() {
    let report: Report = check(Rules::TenWaits);
    let violation = report.violation.expect("an unbounded wait");
    assert_eq!(violation.message, "a wait with nothing of the protocol's own left that can end it");
    assert!(violation.contains_in_order(&[P2A]));
}

/// Revision 10, reading two (red): the command returns exit 0 with a pending notice and the request lands when the hook accepts.
#[test]
fn ten_returns_pending_and_the_request_lands_later() {
    let reached = replay(
        initial(Rules::Ten),
        steps,
        &[
            P2A,
            "FOREGROUND: the bound is reached, exits 0 with a pending notice",
            HOOK,
            "P2b: nothing decides, the transaction goes on",
            "P2c commit: writes commit and reads commit ok (P3 published)",
        ],
    );
    assert_eq!(invariant(&reached), Some("the request landed after its command returned with a pending notice".to_string()));
}

/// Revision 11 (green), schedule 1: the hook never returns, the client withdraws at its bound, the hook later accepts, P2b finds
/// `withdraw`, writes abort, and the request never lands.
#[test]
fn eleven_withdraws_at_the_bound_and_the_request_never_lands() {
    let reached = replay(
        initial(Rules::Eleven),
        steps,
        &[P2A, W1, HOOK, "P2b decide: finds withdraw, writes abort and ends as W2"],
    );
    assert!(!reached.landed && reached.withdrawn && reached.client == 2);
    assert_eq!(invariant(&reached), None);
}

/// Revision 11, schedule 2: both orders of the decision race. If P2b links `commit` first the client that lost reports the
/// outcome, and the request lands.
#[test]
fn eleven_decision_race_has_exactly_one_winner_in_both_orders() {
    let commit_first = replay(
        initial(Rules::Eleven),
        steps,
        &[
            P2A,
            HOOK,
            "P2b decide: links the decision commit",
            "COMMITTING: W1 found commit, the bound is reached, exits 0 saying the commit was decided",
            "P2c commit: writes commit and reads commit ok (P3 published)",
        ],
    );
    assert!(commit_first.landed && commit_first.client == 4 && commit_first.decision == 1);
    assert_eq!(invariant(&commit_first), None);
    let withdraw_first = replay(initial(Rules::Eleven), steps, &[P2A, HOOK, W1, "P2b decide: finds withdraw, writes abort and ends as W2"]);
    assert!(!withdraw_first.landed && withdraw_first.decision == 2);
}

/// Revision 11, schedule 3: a crash after W1 and before W2, and a crash between P2a and P2c; the request stays withdrawn after the
/// restart, and a decision `commit` stands across a crash and lands.
#[test]
fn eleven_survives_a_crash_after_the_decision_either_way() {
    let withdrawn = replay(
        initial(Rules::Eleven),
        steps,
        &[
            P2A,
            W1,
            "environment: the owner dies; update-ref aborts what is prepared",
            "L4: the new owner reads the decision files first and withdraws the request",
        ],
    );
    assert!(!withdrawn.landed && withdrawn.withdrawn);
    let landed = replay(
        initial(Rules::Eleven),
        steps,
        &[
            P2A,
            HOOK,
            "P2b decide: links the decision commit",
            "environment: the owner dies; update-ref aborts what is prepared",
            "L4: the decision commit stands, recovery lands the commit",
        ],
    );
    assert!(landed.landed && landed.decision == 1);
}

/// Revision 11, schedule 4 (this wait): in every schedule the client's wait ends within the bound of the protocol's own steps, no
/// wait lacks an own step, and a request either lands or never lands. Exhaustive.
#[test]
fn eleven_bounds_the_foreground_wait_and_decides_once() {
    let report: Report = check(Rules::Eleven);
    if let Some(violation) = &report.violation {
        panic!("{}: {}", violation.message, violation.trace.join(" ; "));
    }
    assert!(report.exhausted);
    println!("W1 and P2 eleven: {} states, deepest {}", report.states, report.deepest);
}
