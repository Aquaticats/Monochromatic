//! What:
//!  The controls of revision 18's receipt responsibility: the delayed client after the final sweep (red under revision 17, green under 18),
//!  the single-defect controls (seal after sweep, no recheck sweep, no flusher lock, flusher in the owner's scope, no startup sweep,
//!  foreground forcing before dispatch), enqueue and launch failures, a forcing error, replacement startup and unrelated dispatch while
//!  receipt forcing is paused.
//! Why:
//!  The acceptance target for P2c, receipt forcing, L2 and L4 and shutdown names these showing controls; each is a literal schedule where
//!  the property needs one, and an exhaustive search over both clients for the property itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('delayed client', () => { expect(check(Seventeen).violation).toBeDefined(); expect(check(Eighteen).violation).toBeUndefined(); });
//! ```

use crate::model_explore::{Report, replay};
use crate::model_receipt_r18::{Client, Dispatch, Flusher, Receipt, Rules, Service, State, check, check_dispatch, initial, initial_with_unrelated_command, steps, terminal};

const C0_CREATES: &str = "client 0: creates its receipt";
const C0_READS: &str = "client 0: reads the seal and whether a service holds the lane";
const SHUTDOWN_SEALED: &str = "environment: a planned shutdown creates lane/sealed";
const SHUTDOWN_PLAIN: &str = "environment: a planned shutdown begins its final sweep";
const ENUMERATE: &str = "owner: the final sweep enumerates lane/receipt/";
const FORCE_FINAL: &str = "owner: the final sweep forces what it enumerated";
const EXIT: &str = "owner: the service exits";

/// Whether the search found nothing wrong and covered every state.
fn green(rules: Rules) -> bool {
    let report: Report = check(rules);
    assert!(report.violation.is_none(), "{:?}: {:?}", rules, report.violation.map(|found| { return (found.message, found.trace); }));
    return report.exhausted;
}

/// R17a-1, the acceptance target's delayed client after the final sweep: the service enumerates, then the client creates its receipt and
/// reads the lane, then the service exits and the client rings it. Under 17 the receipt has no executor; under 18 the seal precedes the
/// sweep, so the client's read sees the seal and starts a flusher (or the sweep saw the receipt).
#[test]
fn a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen() {
    let lost: State = replay(initial(Rules::Seventeen), steps, &[SHUTDOWN_PLAIN, ENUMERATE, C0_CREATES, C0_READS, FORCE_FINAL, EXIT, "client 0: rings the service"]);
    assert_eq!(lost.receipts[0], Receipt::V);
    assert!(lost.clients[0] == Client::Done && lost.service == Service::Exited);
    assert!(check(Rules::Seventeen).violation.is_some_and(|found| { return found.message.contains("no executor"); }));
    let sealed: State = replay(initial(Rules::Eighteen), steps, &[SHUTDOWN_SEALED, ENUMERATE, C0_CREATES, C0_READS]);
    assert_eq!(sealed.clients[0], Client::Read(false), "the client sees the seal and does not ring");
    assert!(green(Rules::Eighteen));
}

/// The ordering of the seal before the final sweep is the clause that makes the delayed client safe: the control that seals after the sweep
/// is red with the same schedule.
#[test]
fn sealing_after_the_final_sweep_loses_the_delayed_receipt() {
    assert!(check(Rules::EighteenSealAfterSweep).violation.is_some_and(|found| { return found.message.contains("no executor"); }));
}

/// The flusher releases its lock after the last empty sweep and sweeps once more: the control that does not is red because a client that
/// found the lock held and created its receipt after the empty sweep has no executor.
#[test]
fn a_flusher_that_does_not_sweep_once_more_after_releasing_its_lock_loses_a_wakeup() {
    assert!(check(Rules::EighteenNoRecheck).violation.is_some_and(|found| { return found.message.contains("no executor"); }));
}

/// At most one flusher runs per lane under the lock; without the lock many outstanding receipts start many flushers.
#[test]
fn many_outstanding_receipts_start_one_flusher_and_the_lock_free_control_starts_more() {
    let report: Report = check(Rules::EighteenNoFlusherLock);
    assert!(report.violation.is_some_and(|found| { return found.message.contains("more than one flusher"); }));
    let launches: State = replay(
        initial(Rules::Eighteen),
        steps,
        &[
            SHUTDOWN_SEALED,
            "client 0: creates its receipt",
            "client 1: creates its receipt",
            "client 0: reads the seal and whether a service holds the lane",
            "client 1: reads the seal and whether a service holds the lane",
            "client 0: starts a one-shot flusher",
            "client 1: starts a one-shot flusher",
        ],
    );
    assert_eq!(launches.extra_flushers, 0);
    assert_eq!(launches.flusher, Flusher::Enum);
}

/// A flusher that ends with the inline owner's scope leaves a receipt with no executor until some later command: red. The green flusher
/// outlives the owner (a transient unit, or a double-forked child in its own session).
#[test]
fn a_flusher_in_the_owners_scope_is_ended_by_the_owner_exit() {
    assert!(check(Rules::EighteenFlusherInOwnerScope).violation.is_some_and(|found| { return found.message.contains("no executor"); }));
}

/// A next owner sweeps `lane/receipt/` at startup: without it a receipt whose forcing failed (or whose flusher could not start) stays
/// registered and unforced after replacement.
#[test]
fn the_next_owner_sweeps_at_startup_and_retries_a_recorded_error() {
    let errored: State = replay(
        initial(Rules::Eighteen),
        steps,
        &[
            SHUTDOWN_SEALED,
            ENUMERATE,
            FORCE_FINAL,
            EXIT,
            C0_CREATES,
            C0_READS,
            "client 0: starts a one-shot flusher",
            "owner: the flusher enumerates lane/receipt/",
            "environment: a forcing error is recorded in lane/receipt-errors and the flusher exits",
        ],
    );
    assert_eq!(errored.receipts[0], Receipt::Err);
    assert!(terminal(&errored).is_none(), "an error that is recorded is not a lost receipt");
    let retried: State = replay(
        errored,
        steps,
        &["environment: a later wrapper command starts a next owner, which removes lane/sealed (L3)", "owner: the next owner's syncer sweeps lane/receipt/ at startup"],
    );
    assert_eq!(retried.receipts[0], Receipt::D);
    assert!(!retried.seal && retried.service == Service::Running);
    assert!(check(Rules::EighteenNoStartupSweep).violation.is_some());
}

/// Enqueue rejection and launch failure: a flusher refused by someone else's cap is the stated outside cause (the command says the receipt
/// is not yet forced and the receipt stays registered); under 18 that is the only quiescent state with a V receipt.
#[test]
fn a_launch_refused_by_a_cap_is_the_stated_outside_cause_and_nothing_else_is_excused() {
    let refused: State = replay(
        initial(Rules::Eighteen),
        steps,
        &[SHUTDOWN_SEALED, ENUMERATE, FORCE_FINAL, EXIT, C0_CREATES, C0_READS, "environment: a cap set by someone else refuses client 0's flusher"],
    );
    assert_eq!(refused.clients[0], Client::NotYet);
    assert_eq!(refused.receipts[0], Receipt::V);
    assert!(green(Rules::Eighteen));
}

/// Pause receipt forcing indefinitely and verify unrelated dispatch remains bounded: under 18 L4 queues the receipts and dispatches; the
/// controls that force first (revision 17's L4 and the foreground-forcing control) never dispatch while the disk is paused.
#[test]
fn unrelated_dispatch_does_not_wait_for_receipt_forcing() {
    let paused: State = replay(
        initial_with_unrelated_command(Rules::Eighteen),
        steps,
        &[C0_CREATES, "environment: the disk stops answering forcing calls", "environment: an unrelated command reaches L4", "owner: L4 queues the outstanding receipts and dispatches"],
    );
    assert_eq!(paused.dispatch, Dispatch::Done);
    assert_eq!(paused.receipts[0], Receipt::V);
    for rules in [Rules::EighteenForegroundForcing, Rules::Seventeen] {
        let found = check_dispatch(rules).violation.expect("red");
        assert!(found.message.contains("dispatch is still waiting"), "{:?}: {:?}", rules, (found.message, found.trace));
    }
}

/// With the unrelated command in the configuration, revision 18 stays green and exhaustive.
#[test]
fn revision_eighteen_is_green_with_an_unrelated_command_at_any_point() {
    let report: Report = check_dispatch(Rules::Eighteen);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// Revision 18's search over both clients, the service, the flusher, a crash, a forcing error, a refused launch and a paused disk is green
/// and exhaustive.
#[test]
fn revision_eighteen_is_exhaustively_green() {
    assert!(green(Rules::Eighteen));
}
