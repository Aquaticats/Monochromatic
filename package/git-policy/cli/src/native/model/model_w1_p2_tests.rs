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
            "P2c send: writes commit to the transaction",
            "update-ref: applies the commit and writes the ref",
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

/// Revision 11, schedule 2, withdrawal first: W1 links `withdraw`, P2b finds it and aborts; the request never lands.
#[test]
fn eleven_withdrawal_first_never_lands() {
    let withdraw_first = replay(initial(Rules::Eleven), steps, &[P2A, HOOK, W1, "P2b decide: finds withdraw, writes abort and ends as W2"]);
    assert!(!withdraw_first.landed && withdraw_first.decision == 2);
    assert_eq!(invariant(&withdraw_first), None);
}

/// Revision 11 as written (RED against the owner's contract, a defect): the decision is linked first, the client that lost the
/// race reaches the COMMITTING bound and exits 0 saying "decided", and the commit is applied later. The owner's rule is that nothing
/// lands after the command returned; revision 11 names this as its one exception. Shortest violating schedule below.
#[test]
fn eleven_returns_decided_and_the_request_lands_after_the_command_returned() {
    let reached = replay(
        initial(Rules::Eleven),
        steps,
        &[
            P2A,
            HOOK,
            "P2b decide: links the decision commit",
            "COMMITTING: W1 found commit, the bound is reached, exits 0 saying the commit was decided",
            "P2c send: writes commit to the transaction",
            "update-ref: applies the commit and writes the ref",
        ],
    );
    assert_eq!(invariant(&reached), Some("the request landed after its command returned exit 0 saying it was decided".to_string()));
}

/// Revision 11, schedule 3: a crash after W1 and before W2 leaves the request withdrawn after the restart; a decision `commit`
/// stands across a crash and recovery lands it; an owner that dies after the child applied the commit but before the acknowledgement
/// is recovered by reading the ref (the status was uncertain, not lost); a commit that fails after the decision is retried.
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
    let decided = [P2A, HOOK, "P2b decide: links the decision commit"];
    let mut lands: Vec<&str> = decided.to_vec();
    lands.push("environment: the owner dies; update-ref aborts what is prepared");
    lands.push("L4: the decision commit stands, recovery lands the commit");
    let landed = replay(initial(Rules::Eleven), steps, &lands);
    assert!(landed.landed && landed.decision == 1);
    let mut uncertain: Vec<&str> = decided.to_vec();
    uncertain.extend_from_slice(&[
        "P2c send: writes commit to the transaction",
        "update-ref: applies the commit and writes the ref",
        "environment: the owner dies; update-ref aborts what is prepared",
        "L4: the tip equals new (R2), recovery records published",
    ]);
    assert!(replay(initial(Rules::Eleven), steps, &uncertain).landed);
    let mut retried: Vec<&str> = decided.to_vec();
    retried.extend_from_slice(&[
        "P2c send: writes commit to the transaction",
        "update-ref: the commit fails after the decision (store failure)",
        "Q1: retries the decided commit on a trigger",
    ]);
    assert_eq!(replay(initial(Rules::Eleven), steps, &retried).stage, 3);
}

/// Revision 11, schedule 4: the shortest violation of the strict contract is the COMMITTING exit; no schedule leaves a wait without
/// the client's own step, and a withdrawn request never lands (that invariant is not the one that fails).
#[test]
fn eleven_strict_contract_fails_only_on_the_committing_exit() {
    let report: Report = check(Rules::Eleven);
    let violation = report.violation.expect("the committing exit");
    assert_eq!(violation.message, "the request landed after its command returned exit 0 saying it was decided");
    println!("W1 and P2 eleven strict: {} states, deepest {}", report.states, report.deepest);
}
