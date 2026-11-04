//! What:
//!  The controls of revision 18's receipt responsibility: the delayed client after the final sweep (red under revision 17, green under 18),
//!  an enqueue rejected by a running service, the release window of the flusher lock with two flushers representable, a forcing error that
//!  records every unforced receipt (including one that joined after the snapshot), a flusher killed or reaped mid-sync, replacement
//!  startup, a launch refused by a cap, and unrelated dispatch while receipt forcing is paused, with a single-defect control for each.
//! Why:
//!  The acceptance target for P2c, receipt forcing, L2 and L4 and shutdown names these showing controls. Each has a literal schedule where
//!  the property needs one and an exhaustive search over both clients for the property itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('delayed client', () => { expect(check(Seventeen).violation).toBeDefined(); expect(check(Eighteen).violation).toBeUndefined(); });
//! ```

use crate::model_explore::{Report, replay};
use crate::model_receipt_r18::{Client, Dispatch, Flusher, Receipt, Rules, Service, State, check, check_dispatch, initial, initial_with_unrelated_command, invariant, steps, terminal};

const C0_CREATES: &str = "client 0: creates its receipt";
const C1_CREATES: &str = "client 1: creates its receipt";
const C0_READS: &str = "client 0: reads the seal and whether a service holds the lane";
const C1_READS: &str = "client 1: reads the seal and whether a service holds the lane";
const C0_STARTS: &str = "client 0: starts a one-shot flusher";
const C1_STARTS: &str = "client 1: starts a one-shot flusher";
const SHUTDOWN_SEALED: &str = "environment: a planned shutdown creates lane/sealed";
const SHUTDOWN_PLAIN: &str = "environment: a planned shutdown begins its final sweep";
const ENUMERATE: &str = "owner: the final sweep enumerates lane/receipt/";
const FORCE_FINAL: &str = "owner: the final sweep forces what it enumerated";
const EXIT: &str = "owner: the service exits";
const NEXT_OWNER: &str = "environment: a later wrapper command starts a next owner, which removes lane/sealed (L3)";
const STARTUP_SWEEP: &str = "owner: the next owner's syncer sweeps lane/receipt/ at startup";
const F0_ENUM: &str = "owner: flusher 0 enumerates lane/receipt/";
const F0_ERROR_LISTS: &str = "environment: flusher 0 meets a forcing error and lists the receipts it has not forced";
const F0_RECORDS: &str = "owner: flusher 0 records its listing in lane/receipt-errors";
const F0_RELEASES: &str = "owner: flusher 0 releases the lock on its error path";
const F0_RELISTS: &str = "owner: flusher 0 lists the lane once more and records the receipts that joined";

/// Whether the search found nothing wrong and covered every state.
fn green(rules: Rules) -> bool {
    let report: Report = check(rules);
    assert!(report.violation.is_none(), "{:?}: {:?}", rules, report.violation.map(|found| { return (found.message, found.trace); }));
    return report.exhausted;
}

/// The message of the first violation the search finds, or a panic when it finds none.
fn red(rules: Rules) -> String {
    return check(rules).violation.unwrap_or_else(|| { panic!("{:?} must be red", rules) }).message;
}

/// R17a-1, the acceptance target's delayed client after the final sweep: the service enumerates, then the client creates its receipt and
/// reads the lane, then the service exits and the client rings it. Under 17 the receipt has no executor; under 18 the seal precedes the
/// sweep, so the client's read sees the seal and starts a flusher (or the sweep saw the receipt).
#[test]
fn a_delayed_client_after_the_final_sweep_is_red_under_seventeen_and_green_under_eighteen() {
    let lost: State = replay(initial(Rules::Seventeen), steps, &[SHUTDOWN_PLAIN, ENUMERATE, C0_CREATES, C0_READS, FORCE_FINAL, EXIT, "client 0: rings the service"]);
    assert_eq!(lost.receipts[0], Receipt::V);
    assert!(lost.clients[0] == Client::Done && lost.service == Service::Exited);
    assert!(red(Rules::Seventeen).contains("no executor"));
    let sealed: State = replay(initial(Rules::Eighteen), steps, &[SHUTDOWN_SEALED, ENUMERATE, C0_CREATES, C0_READS]);
    assert_eq!(sealed.clients[0], Client::Read(false), "the client sees the seal and does not ring");
    assert!(green(Rules::Eighteen));
}

/// The ordering of the seal before the final sweep is the clause that makes the delayed client safe.
#[test]
fn sealing_after_the_final_sweep_loses_the_delayed_receipt() {
    assert!(red(Rules::EighteenSealAfterSweep).contains("no executor"));
}

/// Enqueue rejection by a running service: the ring is refused (the queue is draining or full), so nothing handed the receipt over. Under 18
/// the client starts a flusher; the control that takes the refused ring for a handover leaves a V receipt with a running service that was
/// never woken.
#[test]
fn an_enqueue_rejected_by_a_running_service_falls_back_to_a_flusher() {
    let reject = "environment: the running service rejects client 0's enqueue";
    let handled: State = replay(initial(Rules::Eighteen), steps, &[C0_CREATES, C0_READS, reject]);
    assert_eq!(handled.clients[0], Client::Read(false));
    assert!(!handled.bell, "a rejected enqueue woke nobody");
    let ignored: State = replay(initial(Rules::EighteenIgnoresRejection), steps, &[C0_CREATES, C0_READS, reject]);
    assert_eq!(ignored.clients[0], Client::Done);
    assert_eq!(ignored.receipts[0], Receipt::V);
    assert!(red(Rules::EighteenIgnoresRejection).contains("no executor"));
    assert!(green(Rules::Eighteen));
}

/// The release window of the flusher lock, with two flushers representable: flusher 0 has swept nothing and released the lock; client 1
/// creates its receipt, finds the lock free and starts flusher 1, which takes it; flusher 0's one more sweep finds a holder and exits;
/// flusher 1 forces the receipt. The no-lock control runs both flushers at once, and the control that does not sweep again after the release
/// loses the receipt of a client that found the lock held.
#[test]
fn a_launch_in_the_release_window_starts_a_second_flusher_that_takes_the_lock() {
    let window: State = replay(
        initial(Rules::Eighteen),
        steps,
        &[SHUTDOWN_SEALED, ENUMERATE, FORCE_FINAL, EXIT, C0_CREATES, C0_READS, C0_STARTS, F0_ENUM, "owner: flusher 0 forces its snapshot", F0_ENUM, "owner: flusher 0 releases the lock after its last empty sweep"],
    );
    assert_eq!(window.flushers[0], Flusher::Released, "flusher 0 has released the lock and will sweep once more");
    let contending: State = replay(window, steps, &[C1_CREATES, C1_READS, C1_STARTS]);
    assert_eq!(contending.flushers, [Flusher::Released, Flusher::Enum], "a second flusher took the free lock");
    assert_eq!(invariant(&contending), None);
    let settled: State = replay(contending, steps, &["owner: flusher 0 sweeps once more without the lock", "owner: flusher 1 enumerates lane/receipt/", "owner: flusher 1 forces its snapshot"]);
    assert_eq!(settled.flushers[0], Flusher::Off, "flusher 0 saw a holder and exited");
    assert_eq!(settled.receipts[1], Receipt::D);
    assert!(red(Rules::EighteenNoRecheck).contains("no executor"));
    assert!(green(Rules::Eighteen));
}

/// Many outstanding receipts: the lock keeps one flusher; the control without it runs two at once.
#[test]
fn many_outstanding_receipts_keep_one_flusher_and_the_lock_free_control_runs_two() {
    let schedule = [SHUTDOWN_SEALED, C0_CREATES, C1_CREATES, C0_READS, C1_READS, C0_STARTS, C1_STARTS];
    let locked: State = replay(initial(Rules::Eighteen), steps, &schedule);
    assert_eq!(locked.flushers, [Flusher::Enum, Flusher::Off]);
    assert_eq!(invariant(&locked), None);
    let unlocked: State = replay(initial(Rules::EighteenNoFlusherLock), steps, &schedule);
    assert_eq!(unlocked.flushers, [Flusher::Enum, Flusher::Enum]);
    assert!(invariant(&unlocked).is_some_and(|message| { return message.contains("more than one flusher"); }));
    assert!(red(Rules::EighteenNoFlusherLock).contains("more than one flusher"));
}

/// Worker error: the schedule of the review. The service has sealed and gone; client 0 starts flusher 0, which enumerates its snapshot; client
/// 1 creates its receipt and joins the lock holder; forcing the snapshot fails. Revision 18 records an error for every receipt the worker has
/// not forced, the joiner's included, so no receipt is left as V behind an exited worker; the control records the snapshot only and leaves
/// client 1's receipt without an executor. The next owner then retries each recorded error, and without that sweep the error stays.
#[test]
fn a_forcing_error_records_every_unforced_receipt_and_the_next_owner_retries_each() {
    let joined = [SHUTDOWN_SEALED, ENUMERATE, FORCE_FINAL, EXIT, C0_CREATES, C0_READS, C0_STARTS, F0_ENUM, C1_CREATES, C1_READS, C1_STARTS, F0_ERROR_LISTS, F0_RECORDS, F0_RELEASES, F0_RELISTS];
    let recorded: State = replay(initial(Rules::Eighteen), steps, &joined);
    assert_eq!(recorded.receipts, [Receipt::Err, Receipt::Err], "both receipts carry the recorded error");
    assert_eq!(terminal(&recorded), None);
    // The control has no second listing, so its schedule ends at the release.
    let control: State = replay(initial(Rules::EighteenErrorIgnoresJoiners), steps, &joined[..joined.len() - 1]);
    assert_eq!(control.receipts, [Receipt::Err, Receipt::V]);
    assert!(terminal(&control).is_some_and(|message| { return message.contains("client 1's receipt is V"); }), "{:?}", terminal(&control));
    assert!(red(Rules::EighteenErrorIgnoresJoiners).contains("no executor"));
    let retried: State = replay(recorded.clone(), steps, &[NEXT_OWNER, STARTUP_SWEEP]);
    assert_eq!(retried.receipts, [Receipt::D, Receipt::D]);
    let unretried: State = replay(replay(initial(Rules::EighteenNoStartupSweep), steps, &joined), steps, &[NEXT_OWNER, STARTUP_SWEEP]);
    assert!(terminal(&unretried).is_some_and(|message| { return message.contains("recorded forcing error was not retried"); }), "{:?}", terminal(&unretried));
    assert!(red(Rules::EighteenNoStartupSweep).contains("not retried") || red(Rules::EighteenNoStartupSweep).contains("no executor"));
    assert!(green(Rules::Eighteen));
}

/// Review 9b item 1 (with 9a section 3D): the error handler is not one atomic transition. The flusher meets the error and lists R; client 1
/// then creates its receipt, finds the lock held and joins; the handler records its listing, releases the lock and exits. Without the
/// second listing after the release, client 1's receipt is V with no executor and no record.
#[test]
fn a_receipt_that_joins_between_the_error_listing_and_the_release_is_recorded_by_the_second_listing() {
    let before_join = [SHUTDOWN_SEALED, ENUMERATE, FORCE_FINAL, EXIT, C0_CREATES, C0_READS, C0_STARTS, F0_ENUM, F0_ERROR_LISTS, C1_CREATES, C1_READS, C1_STARTS];
    let joined: State = replay(initial(Rules::Eighteen), steps, &before_join);
    assert_eq!(joined.receipts, [Receipt::V, Receipt::V], "the joiner joined the holder after the listing: nothing is recorded for it yet");
    assert_eq!(joined.clients[1], Client::Done);
    assert!(joined.flushers.iter().filter(|phase| { return **phase != Flusher::Off; }).count() == 1, "the join started no flusher: the lock was held");
    let covered: State = replay(joined.clone(), steps, &[F0_RECORDS, F0_RELEASES, F0_RELISTS]);
    assert_eq!(covered.receipts, [Receipt::Err, Receipt::Err]);
    assert_eq!(terminal(&covered), None);
    let without: State = replay(replay(initial(Rules::EighteenErrorNoRelist), steps, &before_join), steps, &[F0_RECORDS, F0_RELEASES]);
    assert_eq!(without.receipts, [Receipt::Err, Receipt::V]);
    assert!(terminal(&without).is_some_and(|message| { return message.contains("client 1's receipt is V"); }), "{:?}", terminal(&without));
    assert!(red(Rules::EighteenErrorNoRelist).contains("no executor"));
    assert!(green(Rules::Eighteen));
}

/// Reap mid-sync: a flusher in its forcing phase is killed (by the reap of its incarnation scope, in the control where it lives there, or
/// from outside). Its receipts stay V; the next owner's startup sweep forces them. The control whose flusher is reaped with the scope and
/// whose next owner does not sweep loses them.
#[test]
fn a_flusher_killed_or_reaped_mid_sync_leaves_its_receipts_to_the_startup_sweep() {
    let mid_sync = [SHUTDOWN_SEALED, ENUMERATE, FORCE_FINAL, EXIT, C0_CREATES, C0_READS, C0_STARTS, F0_ENUM];
    let killed: State = replay(initial(Rules::Eighteen), steps, &[mid_sync.as_slice(), &["environment: flusher 0 is killed from outside"]].concat());
    assert_eq!(killed.flushers[0], Flusher::Off);
    assert_eq!(killed.receipts[0], Receipt::V);
    assert_eq!(terminal(&killed), None, "an outside kill excuses the receipt until a next owner has swept");
    let swept: State = replay(killed, steps, &[NEXT_OWNER, STARTUP_SWEEP]);
    assert_eq!(swept.receipts[0], Receipt::D);
    let survivor: State = replay(initial(Rules::Eighteen), steps, &[mid_sync.as_slice(), &[NEXT_OWNER]].concat());
    assert!(matches!(survivor.flushers[0], Flusher::Force(_)), "the flusher lives outside the incarnation scope and survives the reap");
    let reaped: State = replay(initial(Rules::EighteenReapedFlusherNoStartupSweep), steps, &[mid_sync.as_slice(), &[NEXT_OWNER]].concat());
    assert_eq!(reaped.flushers[0], Flusher::Off, "the control's flusher was reaped with the scope");
    assert!(red(Rules::EighteenReapedFlusherNoStartupSweep).contains("no executor"));
    assert!(red(Rules::EighteenFlusherInOwnerScope).contains("no executor"));
}

/// A launch refused by a cap set by someone else is the stated outside cause (the command says the receipt is not yet forced and the receipt
/// stays registered); after a next owner has swept it no longer excuses anything.
#[test]
fn a_launch_refused_by_a_cap_is_the_stated_outside_cause_and_nothing_else_is_excused() {
    let refused: State = replay(
        initial(Rules::Eighteen),
        steps,
        &[SHUTDOWN_SEALED, ENUMERATE, FORCE_FINAL, EXIT, C0_CREATES, C0_READS, "environment: a cap set by someone else refuses client 0's flusher"],
    );
    assert_eq!(refused.clients[0], Client::NotYet);
    assert_eq!(refused.receipts[0], Receipt::V);
    assert_eq!(terminal(&refused), None);
    let after: State = replay(refused, steps, &[NEXT_OWNER, STARTUP_SWEEP]);
    assert_eq!(after.receipts[0], Receipt::D);
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

/// Revision 18's search over both clients, the service, both flusher slots, a crash, a kill, a forcing error, a refused launch, a rejected
/// enqueue and a paused disk is green and exhaustive.
#[test]
fn revision_eighteen_is_exhaustively_green() {
    assert!(green(Rules::Eighteen));
}
