//! What:
//!  The controls of revision 13's W1 against publication: the twelfth review's schedules (red under revision 12, green under 13),
//!  the surviving transaction process, fallible recovery, E1's three kinds at every death point of the child, PREPARED-HOLD
//!  against a process that never reads, the second closing line, and positive controls showing that each clause of the contract
//!  fails on a state that breaks it.
//! Why:
//!  A green search proves nothing unless the same search is red when the rule is broken; every clause has a hand-built
//!  violating state so that a clause which could never fire is found here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('twelve: a normal exit then a retry lands after the command returned', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_w1_r13::{
    Child, Class, Client, FOREGROUND_BOUND, HOLD_BOUND, Kind, Line, Mark, Owner, Pipe, Rules, State, Tip, check, check_hold, check_with,
    c3_says_nothing_can_land, classify, initial, invariant, invariant_twelve_text_lock_released, steps, violations, Seen,
};

const START: &str = "owner: P1 and P2a start an attempt";
const HOOK_OK: &str = "environment: the person's hook returns and accepts; prepare: ok";
const RECORD: &str = "client: records submitted for the attempt";
const WRITE: &str = "client: writes commit to the pipe";
const COMMITTING_MARK: &str = "COMMITTING: the bound passed with commit submitted; the exit creates its mark";
const COMMITTING_EXIT: &str = "COMMITTING: exits non-zero saying the result is not known";
const PARENT_DEATH: &str = "environment: the parent-death signal ends the transaction process after the owner exited";
const KILL_CHILD: &str = "environment: the transaction process is killed from outside";
const E1: &str = "owner: E1 reads the tip and classifies the attempt";
const E1_UNREAD: &str = "owner: E1 cannot read the tip (store error) and classifies from what it has";
const READS_COMMIT: &str = "environment: update-ref reads commit";
const WRITES_REF: &str = "environment: update-ref writes the ref";
const DIES_BEFORE_REPORT: &str = "environment: update-ref dies after writing the ref and before commit: ok";
const RESET_ERASED: &str = "environment: another writer resets the branch away and the reflog entry is lost";
const RESET_KEPT: &str = "environment: another writer resets the branch away and the reflog keeps the entry";
const OWNER_DIES: &str = "environment: the owner dies";
const REAP: &str = "owner: a new incarnation reaps the previous scope (L2) and takes over";
const HOLD: &str = "owner: PREPARED-HOLD expires and it writes abort";

/// Whether `label` is enabled in `state`.
fn enabled(state: &State, label: &str) -> bool {
    return steps(state).iter().any(|step| { return step.0 == label; });
}

/// The state with the first attempt prepared and the client's `commit` queued (the child has not read it).
fn commit_queued(rules: Rules) -> State {
    return replay(initial(rules), steps, &[START, HOOK_OK, RECORD, WRITE]);
}

/// The twelfth review's first schedule (red under 12): the child stalls, COMMITTING expires, the client exits with no mark, the
/// child ends without landing, E1 infers a failure, and the next attempt starts and can land after the command returned.
#[test]
fn twelve_a_normal_exit_then_a_retry_starts_a_fresh_attempt() {
    let mut reached: State = commit_queued(Rules::Twelve);
    reached = replay(reached, steps, &[COMMITTING_MARK, COMMITTING_EXIT, KILL_CHILD, E1, START]);
    let broken: Vec<String> = violations(&reached);
    assert!(broken.contains(&"a normal exit left no withdrawal mark".to_string()), "{:?}", broken);
    assert!(broken.contains(&"an attempt started after the unresolved exit named an earlier one".to_string()), "{:?}", broken);
    // With no client and no mark the owner decides the second attempt, and it lands after the command returned.
    let landed: State = replay(
        reached,
        steps,
        &[HOOK_OK, "owner: the client lock is gone with no mark; it decides and writes commit", READS_COMMIT, WRITES_REF],
    );
    assert_eq!(landed.landed, 2, "attempt two landed after the exit that named attempt one");
    assert!(violations(&landed).contains(&"an attempt other than the one the unresolved exit names landed".to_string()));
}

/// The same schedule under 13 (green): the exit created `withdraw-unresolved 1`, E1 returns U for an ended child with `commit`
/// submitted, and no step starts a second attempt; the contract holds in every state of the schedule.
#[test]
fn thirteen_the_same_schedule_cannot_start_a_second_attempt() {
    let mut reached: State = commit_queued(Rules::Thirteen);
    reached = replay(reached, steps, &[COMMITTING_MARK, COMMITTING_EXIT, KILL_CHILD, E1]);
    assert_eq!(reached.mark, Mark::Unresolved(1));
    assert_eq!(reached.recorded, Some((1, Class::Unresolved)));
    assert!(!enabled(&reached, START), "no attempt after the named one may start");
    assert!(violations(&reached).is_empty());
}

/// The twelfth review's second schedule: Git writes `new`, another writer resets the branch away, Git dies before `commit: ok`.
/// Revision 12's E1 says the attempt did not land and closes it as failed (red); revision 13 says U, or `rewritten-away` when the
/// reflog shows it (green), and a closed-as-failed attempt that wrote the ref is the clause that fires.
#[test]
fn a_reset_before_the_child_dies_is_unresolved_or_rewritten_never_failed() {
    let after_write = |rules: Rules, reset: &str| -> State {
        return replay(commit_queued(rules), steps, &[READS_COMMIT, WRITES_REF, reset, DIES_BEFORE_REPORT, E1]);
    };
    let twelve: State = after_write(Rules::Twelve, RESET_ERASED);
    assert!(violations(&twelve).contains(&"an attempt that wrote the ref was closed as positively failed".to_string()));
    let erased: State = after_write(Rules::Thirteen, RESET_ERASED);
    assert_eq!(erased.recorded, Some((1, Class::Unresolved)));
    assert!(violations(&erased).is_empty() && !enabled(&erased, START));
    let kept: State = after_write(Rules::Thirteen, RESET_KEPT);
    assert_eq!(kept.recorded, Some((1, Class::RewrittenAway)));
    assert!(violations(&kept).is_empty() && !enabled(&kept, START));
}

/// The search over every schedule: red under 12 and under 13 (13 by the success-then-death schedule below), green and exhausted under 14.
#[test]
fn twelve_and_thirteen_are_red_and_fourteen_is_green_over_every_schedule() {
    assert!(check(Rules::Twelve).violation.is_some(), "revision 12 must be red");
    let thirteen: Report = check(Rules::Thirteen);
    assert!(thirteen.violation.is_some(), "revision 13 must be red once the owner cannot see the client's exit");
    let fourteen: Report = check(Rules::Fifteen);
    assert!(fourteen.violation.is_none() && fourteen.exhausted, "{:?}", fourteen.violation.map(|found| { return (found.message, found.trace); }));
    println!("W1 against publication, revision 14: {} states, deepest {}, client bound {}", fourteen.states, fourteen.deepest, FOREGROUND_BOUND);
}

/// R13-1, control 1: the client sees the landing and returns success, the owner dies before P3 and T0, another writer resets the branch
/// away and the reflog entry is lost, a new incarnation recovers. A released client lock looks the same after a success and after a kill,
/// so revision 13's R5 lands the request again (red: an attempt after the command exited published). Revision 14 writes a receipt before
/// `published`, and recovery reads it (green).
#[test]
fn success_then_owner_death_then_reset_does_not_publish_again_under_fourteen() {
    const PUBLISHED: &str = "client: sees commit: ok or the tip at new, prints published and exits 0";
    const RECEIPT: &str = "client: writes the receipt before it prints published";
    const R5: &str = "owner: R5 lands a request whose client is gone with a fresh attempt";
    let common = [START, HOOK_OK, RECORD, WRITE, READS_COMMIT, WRITES_REF, DIES_BEFORE_REPORT];
    let after = |rules: Rules, exit: &[&str]| -> State {
        let mut reached: State = replay(initial(rules), steps, &common);
        reached = replay(reached, steps, exit);
        return replay(reached, steps, &[OWNER_DIES, RESET_ERASED, REAP, E1]);
    };
    let thirteen: State = after(Rules::Thirteen, &[PUBLISHED]);
    assert!(enabled(&thirteen, R5), "the owner sees no client and an unresolved attempt");
    let landed_again: State = replay(thirteen, steps, &[R5]);
    assert!(violations(&landed_again).contains(&"an attempt started after the command exited published".to_string()));
    let fourteen: State = after(Rules::Fourteen, &[RECEIPT, PUBLISHED]);
    assert_eq!(fourteen.recorded, Some((1, Class::Published)), "recovery reads the receipt");
    assert!(!enabled(&fourteen, R5));
    assert!(violations(&fourteen).is_empty());
}

/// R13-1: the success step needs the receipt first under 14, so no schedule exits published without it.
#[test]
fn fourteen_prints_published_only_after_the_receipt() {
    let reached: State = replay(initial(Rules::Fourteen), steps, &[START, HOOK_OK, RECORD, WRITE, READS_COMMIT, WRITES_REF]);
    assert!(!enabled(&reached, "client: sees commit: ok or the tip at new, prints published and exits 0"));
    assert!(enabled(&reached, "client: writes the receipt before it prints published"));
}

/// Control 14: a late positive failure after the unresolved exit (the owner's `abort` was queued first, so the child executes it and the
/// client's `commit` is the ignored second line). The request can never land; revision 14 records the outcome `withdrawn` and starts no
/// attempt; revision 13 leaves it pending for ever (red).
#[test]
fn a_late_positive_failure_after_the_unresolved_exit_records_withdrawn() {
    let schedule = [
        START,
        HOOK_OK,
        RECORD,
        HOLD,
        WRITE,
        COMMITTING_MARK,
        COMMITTING_EXIT,
        "environment: update-ref reads abort",
        "environment: update-ref prints abort: ok and exits",
        E1,
    ];
    let thirteen: State = replay(initial(Rules::Thirteen), steps, &schedule);
    assert!(violations(&thirteen).contains(&"an unresolved attempt ended in a positive failure and the request stayed pending".to_string()));
    let fourteen: State = replay(initial(Rules::Fourteen), steps, &schedule);
    assert!(fourteen.outcome_withdrawn && violations(&fourteen).is_empty());
    assert!(!enabled(&fourteen, START), "no attempt after the one the mark names");
}

/// The transaction process makes progress with its owner dead (review 3b: child progress must not depend on a live owner): the
/// owner dies after `commit` was queued, the child applies it, the client reaches its COMMITTING bound and leaves unresolved, and
/// the landing is the one attempt the exit names.
#[test]
fn the_child_applies_a_queued_commit_after_its_owner_died() {
    let reached: State = replay(
        commit_queued(Rules::Fourteen),
        steps,
        &[OWNER_DIES, READS_COMMIT, WRITES_REF, COMMITTING_MARK, COMMITTING_EXIT],
    );
    assert_eq!(reached.owner, Owner::Dead);
    assert_eq!(reached.client, Client::UnresolvedExit(1));
    assert_eq!(reached.landed, 1);
    assert!(violations(&reached).is_empty(), "the one named attempt may land: {:?}", violations(&reached));
}

/// Recovery is fallible (review 3b: recovery was infallible): the new incarnation reaps only after the child ended, its read of the
/// tip can fail, and a failed read classifies as U although the ref was in fact written; a readable tip classifies as published.
#[test]
fn recovery_waits_for_the_scope_to_empty_and_its_tip_read_can_fail() {
    let dead: State = replay(commit_queued(Rules::Thirteen), steps, &[OWNER_DIES, READS_COMMIT, WRITES_REF]);
    assert!(!enabled(&dead, REAP), "the scope still holds the transaction process");
    let ended: State = replay(dead, steps, &[DIES_BEFORE_REPORT, REAP]);
    assert!(ended.recovered && enabled(&ended, E1) && enabled(&ended, E1_UNREAD));
    let unread: State = replay(ended.clone(), steps, &[E1_UNREAD]);
    assert_eq!(unread.recorded, Some((1, Class::Unresolved)));
    let read: State = replay(ended, steps, &[E1]);
    assert_eq!(read.recorded, Some((1, Class::Published)));
}

/// E1's kind at each death point of the child (control 3 of the twelfth review): before reading the closing line with `commit`
/// queued, after reading it, after writing the ref and before reporting; and, for contrast, a child that died with no `commit`.
#[test]
fn the_kind_at_each_point_where_the_child_dies() {
    let before_read: State = replay(commit_queued(Rules::Thirteen), steps, &[KILL_CHILD, E1]);
    assert_eq!(before_read.recorded, Some((1, Class::Unresolved)), "queued commit, child never read it: not proof of nothing");
    let after_read: State = replay(commit_queued(Rules::Thirteen), steps, &[READS_COMMIT, KILL_CHILD, E1]);
    assert_eq!(after_read.recorded, Some((1, Class::Unresolved)));
    let after_write: State = replay(commit_queued(Rules::Thirteen), steps, &[READS_COMMIT, WRITES_REF, DIES_BEFORE_REPORT, E1]);
    assert_eq!(after_write.recorded, Some((1, Class::Published)));
    let never_submitted: State = replay(initial(Rules::Thirteen), steps, &[START, HOOK_OK, KILL_CHILD, E1]);
    assert_eq!(never_submitted.recorded, Some((1, Class::Failed)));
    assert!(enabled(&never_submitted, START), "class F closes the attempt and a mark-free request may start the next");
}

/// A positively failed attempt (a hook refusal) closes, and a later attempt starts and may land while the client still waits.
#[test]
fn only_a_positive_failure_allows_the_next_attempt() {
    let refused: State = replay(
        initial(Rules::Thirteen),
        steps,
        &[START, "environment: the person's hook refuses; update-ref exits non-zero", E1],
    );
    assert_eq!(refused.recorded, Some((1, Class::Failed)));
    assert_eq!(refused.closed, 1);
    assert!(enabled(&refused, START), "class F allows a later attempt");
}

/// PREPARED-HOLD (control 5): the owner enqueues `abort` for a prepared transaction nobody closes, within one of its own steps; a
/// child that never reads again keeps the branch lock, and revision 13 claims only the enqueue.
#[test]
fn prepared_hold_enqueues_abort_and_a_child_that_never_reads_keeps_the_lock() {
    let report: Report = check_hold(Rules::Fifteen);
    assert!(report.violation.is_none() && report.exhausted, "bound {}: {:?}", HOLD_BOUND, report.violation.map(|found| { return found.message; }));
    let wedged: State = replay(initial(Rules::Thirteen), steps, &[START, HOOK_OK, "environment: update-ref never reads again", HOLD]);
    assert_eq!(wedged.pipe, Pipe::Abort);
    assert_eq!(wedged.child, Child::Wedged, "the residue: the lock is still held");
    assert_eq!(invariant(&wedged), None, "revision 13 claims the enqueue only");
}

/// Revision 12's text claimed that `abort` releases the lock whatever else holds the pipe (red): the search finds the wedged child.
#[test]
fn the_revision_twelve_text_that_abort_releases_the_lock_is_false() {
    let report: Report = check_with(Rules::Fifteen, invariant_twelve_text_lock_released);
    let violation = report.violation.expect("a child that never reads keeps the lock");
    assert_eq!(violation.message, "abort was enqueued and the branch lock is still held by a process that never reads");
}

/// The second closing line (control 2): the owner's PREPARED-HOLD `abort` is queued first, the client's `commit` arrives second, and
/// `update-ref` executes the abort only; nothing lands, and the attempt closes as positively failed.
#[test]
fn a_second_closing_line_is_not_executed() {
    let reached: State = replay(
        initial(Rules::Thirteen),
        steps,
        &[
            START,
            HOOK_OK,
            RECORD,
            HOLD,
            WRITE,
            "environment: update-ref reads abort",
            "environment: update-ref prints abort: ok and exits",
            E1,
        ],
    );
    assert_eq!(reached.pipe, Pipe::Abort);
    assert_eq!(reached.landed, 0);
    assert_eq!(reached.line, Line::AbortOk);
    assert_eq!(reached.recorded, Some((1, Class::Failed)));
    assert!(violations(&reached).is_empty());
}

/// A client killed after it recorded `submitted` and before it wrote `commit`: the owner, seeing the lock gone with no mark, decides;
/// the request may land later, which the outside-cause rule allows, and no clause of the contract is broken.
#[test]
fn a_client_killed_before_it_wrote_leaves_the_decision_to_the_owner() {
    let reached: State = replay(
        initial(Rules::Thirteen),
        steps,
        &[START, HOOK_OK, RECORD, "environment: the client is killed from outside", "owner: the client lock is gone with no mark; it decides and writes commit"],
    );
    assert_eq!(reached.pipe, Pipe::Commit);
    assert!(violations(&reached).is_empty());
}

/// W1 against a send: the client creates the mark first; the owner that finds the mark after the reader exists writes `abort`; the
/// client that sees a closing line already queued exits without writing one, and no order of the three lets `commit` queue.
#[test]
fn w1_and_the_owner_race_on_the_closing_line_and_commit_never_queues() {
    let marked: State = replay(initial(Rules::Thirteen), steps, &[START, HOOK_OK, "W1: the bound is reached; the client creates the mark withdraw"]);
    assert!(!enabled(&marked, RECORD) && !enabled(&marked, WRITE));
    assert!(!enabled(&marked, "owner: the client lock is gone with no mark; it decides and writes commit"));
    let by_owner: State = replay(marked.clone(), steps, &[ "owner: finds the mark after the reader exists and writes abort", "W1: exits non-zero naming the blocker"]);
    assert_eq!(by_owner.client, Client::Withdrawn);
    assert_eq!(by_owner.pipe, Pipe::Abort);
    let by_client: State = replay(marked, steps, &["W1: writes abort to the open attempt it did not submit", "W1: exits non-zero naming the blocker"]);
    assert_eq!(by_client.client, Client::Withdrawn);
    assert!(violations(&by_client).is_empty());
}

/// E1's classification as a pure function: the revision-12 negative inference and revision 13's U, side by side.
#[test]
fn classify_separates_the_negative_inference_from_unresolved() {
    assert_eq!(classify(Rules::Twelve, true, Line::Nothing, Seen::Absent, true, false), Class::Failed);
    assert_eq!(classify(Rules::Thirteen, true, Line::Nothing, Seen::Absent, true, false), Class::Unresolved);
    assert_eq!(classify(Rules::Thirteen, true, Line::Nothing, Seen::Unread, true, false), Class::Unresolved);
    assert_eq!(classify(Rules::Thirteen, false, Line::Nothing, Seen::Unread, true, false), Class::Failed);
    assert_eq!(classify(Rules::Thirteen, true, Line::AbortOk, Seen::Absent, true, false), Class::Failed);
    assert_eq!(classify(Rules::Thirteen, true, Line::Nothing, Seen::Present, true, false), Class::Published);
    assert_eq!(classify(Rules::Thirteen, true, Line::Nothing, Seen::Rewritten, true, false), Class::RewrittenAway);
}

/// Positive controls: each clause of the contract fires on a state built to break it, so the green search above can fail.
#[test]
fn every_clause_of_the_contract_fires_on_a_state_that_breaks_it() {
    let base: State = initial(Rules::Thirteen);
    let mut no_mark: State = base.clone();
    no_mark.client = Client::Withdrawn;
    assert_eq!(invariant(&no_mark), Some("a normal exit left no withdrawal mark".to_string()));
    let mut marked_lands: State = base.clone();
    marked_lands.mark = Mark::Withdraw;
    marked_lands.landed = 1;
    assert!(invariant(&marked_lands).is_some());
    let mut withdrawn_lands: State = base.clone();
    withdrawn_lands.client = Client::Withdrawn;
    withdrawn_lands.mark = Mark::Withdraw;
    withdrawn_lands.pipe = Pipe::Commit;
    withdrawn_lands.child = Child::Prepared;
    assert!(violations(&withdrawn_lands).contains(&"the command said the request was withdrawn and the request landed or can still land".to_string()));
    let mut later_attempt: State = base.clone();
    later_attempt.client = Client::UnresolvedExit(1);
    later_attempt.mark = Mark::Unresolved(1);
    later_attempt.attempt = 2;
    assert!(violations(&later_attempt).contains(&"an attempt started after the unresolved exit named an earlier one".to_string()));
    let mut other_lands: State = later_attempt.clone();
    other_lands.attempt = 1;
    other_lands.landed = 2;
    assert!(violations(&other_lands).contains(&"an attempt other than the one the unresolved exit names landed".to_string()));
    let mut false_publish: State = base.clone();
    false_publish.client = Client::Published;
    assert_eq!(invariant(&false_publish), Some("the command said published and nothing landed".to_string()));
    let mut false_failure: State = base.clone();
    false_failure.closed = 1;
    false_failure.landed = 1;
    assert_eq!(invariant(&false_failure), Some("an attempt that wrote the ref was closed as positively failed".to_string()));
    let mut leaving: State = base;
    leaving.client = Client::Leaving(Kind::Plain);
    leaving.tip = Tip::Base;
    assert_eq!(invariant(&leaving), Some("a normal exit left no withdrawal mark".to_string()));
}

/// E1 evidence is only as good as its channel (review 3a P2, item E1 evidence): when a hook descendant can write into the file E1 reads, a
/// forged `abort: ok` survives until the attempt ends (here: the ref is written, the report is missing and the tip cannot be read), so E1
/// calls an attempt that wrote the ref positively failed.
/// Revision 13 relies on Git redirecting a hook's standard output away from `out` (a backend gate): without it the search is red, with it
/// green.
#[test]
fn a_hook_that_can_write_the_output_file_forges_positive_failure() {
    let shared: Report = check(Rules::FourteenSharedChannel);
    let violation = shared.violation.expect("a forged line is accepted as evidence");
    assert!(violation.trace.iter().any(|label| { return label.contains("looks like abort: ok"); }), "{:?}", violation.trace);
    assert!(check(Rules::Fifteen).violation.is_none());
}

const SHUTDOWN: &str = "owner: planned shutdown records clean-shutdown and exits";
const KEEPER_EOF: &str = "environment: the keeper write end closed and end of input aborts the prepared transaction";

/// Control 20 (the exit audit): a planned service shutdown with a prepared transaction and an attached client. The service never decides
/// for a client that holds the lock, so it writes nothing; the keeper write end closes, end of input aborts the transaction as class F
/// (the client had not written `commit`), and the client is back in FOREGROUND with plain W1 available. A protected execution (the
/// hook of a transaction still preparing) is not cancelled by the shutdown itself.
#[test]
fn a_planned_shutdown_with_an_attached_client_aborts_as_class_f_and_decides_nothing() {
    let reached: State = replay(
        initial(Rules::Fourteen),
        steps,
        &[
            START,
            HOOK_OK,
            SHUTDOWN,
            KEEPER_EOF,
            "environment: update-ref reads abort",
            "environment: update-ref prints abort: ok and exits",
            REAP,
            E1,
        ],
    );
    assert!(reached.clean_shutdown && !reached.decided_by_owner);
    assert_eq!(reached.recorded, Some((1, Class::Failed)));
    assert_eq!(reached.client, Client::Waiting);
    assert!(enabled(&reached, "W1: the bound is reached; the client creates the mark withdraw"), "plain W1 stays available");
    assert!(violations(&reached).is_empty());
    let preparing: State = replay(initial(Rules::Fourteen), steps, &[START, SHUTDOWN]);
    assert_eq!(preparing.child, Child::Preparing, "the shutdown does not cancel the protected execution");
}

/// Control 20, the other half: `commit` was already buffered when the service shut down; the transaction completes.
#[test]
fn a_planned_shutdown_lets_a_buffered_commit_complete() {
    let reached: State = replay(commit_queued(Rules::Fourteen), steps, &[SHUTDOWN, READS_COMMIT, WRITES_REF]);
    assert_eq!(reached.landed, 1);
    assert!(violations(&reached).is_empty());
}

/// Control 20, a request with no client: the shutdown writes `abort` for a prepared transaction nobody will close.
#[test]
fn a_planned_shutdown_aborts_a_prepared_transaction_that_has_no_client() {
    let killed: State = replay(
        initial(Rules::Fourteen),
        steps,
        &[START, HOOK_OK, "environment: the client is killed from outside", SHUTDOWN],
    );
    assert_eq!(killed.pipe, Pipe::Abort);
}

/// Control 22: a query or a resend while the unresolved mark stands. Revision 13 answered `withdrawn` (nothing can land) while the
/// original attempt's `commit` is queued (red); revision 14 says that the original attempt may still complete.
#[test]
fn a_query_during_the_unresolved_mark_never_says_nothing_can_land() {
    let thirteen: State = replay(commit_queued(Rules::Thirteen), steps, &[COMMITTING_MARK]);
    assert!(violations(&thirteen).contains(&"a query was answered withdrawn while the original attempt may still complete".to_string()));
    let fourteen: State = replay(commit_queued(Rules::Fourteen), steps, &[COMMITTING_MARK]);
    assert!(!violations(&fourteen).contains(&"a query was answered withdrawn while the original attempt may still complete".to_string()));
    assert!(c3_says_nothing_can_land(Rules::Fourteen, Mark::Withdraw), "a plain mark can still be answered withdrawn");
}

/// Models review 6a, 4 (P1): a planned exit never causes the parent-death termination of a protected hook. With the hook still running
/// (the transaction process is `Preparing`), the service's planned exit followed by the process ending is the schedule the review reported;
/// the model must reject it as caused by the planned exit.
#[test]
fn p1_a_planned_exit_never_ends_a_protected_hook() {
    let reached: State = replay(
        initial(Rules::Fourteen),
        steps,
        &[START, SHUTDOWN, PARENT_DEATH],
    );
    assert!(
        violations(&reached).contains(&"the planned exit ended a protected execution".to_string()),
        "{:?}",
        violations(&reached)
    );
}

/// Models review 6a, 4 (P1, green side): under revision 17 a planned exit is a drain. With the hook running the service begins to drain and
/// cannot complete while the transaction process lives; no step ends that process except an outside kill, and the parent-death signal
/// exists only after a crash of the owner. The buffered-commit schedule is exhaustive: the commit runs or the process dies with no evidence
/// (class U), and the contract holds in every state.
#[test]
fn a_planned_exit_is_a_drain_that_never_ends_a_protected_hook() {
    let draining: State = replay(initial(Rules::Fifteen), steps, &[START, "owner: planned exit begins a drain"]);
    assert_eq!(draining.child, Child::Preparing);
    let labels: Vec<String> = steps(&draining).into_iter().map(|step| { return step.0; }).collect();
    assert!(!labels.contains(&"owner: the drain completes and the service exits".to_string()), "the drain waits for the hook");
    assert!(!labels.iter().any(|label| { return label.contains("parent-death"); }), "{:?}", labels);
    assert!(!labels.contains(&START.to_string()), "a draining service admits nothing new");
    let crashed: State = replay(initial(Rules::Fifteen), steps, &[START, OWNER_DIES]);
    assert!(enabled(&crashed, "environment: the parent-death signal ends the transaction process after the owner exited"));
    let report: Report = check(Rules::Fifteen);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// Models review 6a, 3 (P1) on the client's side: a mark that cannot be created. The earlier revision's command exits as a normal
/// withdrawal with no mark (red: a normal exit left no mark); revision 17's exit is the outside-fault exit.
#[test]
fn p1_a_mark_that_cannot_be_created_ends_the_command_by_the_outside_fault_exit() {
    let failed = "W1: the mark cannot be created and the command exits";
    let earlier: State = replay(initial(Rules::Fourteen), steps, &[failed]);
    assert!(violations(&earlier).contains(&"a normal exit left no withdrawal mark".to_string()));
    let fifteen: State = replay(initial(Rules::Fifteen), steps, &[failed]);
    assert_eq!(fifteen.client, Client::Faulted);
    assert!(violations(&fifteen).is_empty());
}

/// Models review 7, 2 (P2, red against the previous monitor): the parent-death consequence follows the owner's death, whatever the reason
/// for it; it must not be switched off by an assumption that the exit was clean. A cleanly exited owner with a protected hook still running
/// (a state the drain rule makes unreachable, built here by hand) reaches the parent-death termination, and the monitor then names the
/// planned exit as its cause; an outside kill in the same state does not.
#[test]
fn p7_parent_death_follows_owner_death_whatever_its_reason_and_the_monitor_names_a_planned_exit() {
    let mut premature: State = replay(initial(Rules::Fifteen), steps, &[START]);
    premature.owner = Owner::Dead;
    premature.clean_shutdown = true;
    premature.departed_by_plan = true;
    premature.draining = true;
    assert_eq!(premature.child, Child::Preparing);
    let successors = steps(&premature);
    let parent_death = successors.iter().find(|step| { return step.0.contains("parent-death"); }).expect("the parent-death consequence is physical and independent of the exit reason");
    assert!(violations(&parent_death.1).contains(&"the planned exit ended a protected execution".to_string()), "{:?}", violations(&parent_death.1));
    let kill = successors.iter().find(|step| { return step.0.contains("killed from outside"); }).expect("an outside kill stays possible");
    assert!(!violations(&kill.1).contains(&"the planned exit ended a protected execution".to_string()), "a kill is not the planned exit");
}

/// Models review 7, 2 (mutation): the drain that completes while the hook runs, with the parent-death consequence of the owner's death kept,
/// is red for the premature clean exit itself and then for the planned exit's causal consequence; revision 17's search is green.
#[test]
fn p7_a_premature_drain_completion_is_red_through_the_monitor_and_the_green_is_exhaustive() {
    let premature: State = replay(initial(Rules::FifteenPrematureDrain), steps, &[START, "owner: planned exit begins a drain", "owner: the drain completes and the service exits"]);
    assert!(violations(&premature).contains(&"a cleanly exited owner left a protected execution running".to_string()), "{:?}", violations(&premature));
    let ended: State = replay(premature, steps, &[PARENT_DEATH]);
    assert!(violations(&ended).contains(&"the planned exit ended a protected execution".to_string()), "{:?}", violations(&ended));
    assert!(check(Rules::FifteenPrematureDrain).violation.is_some());
    let report: Report = check(Rules::Fifteen);
    assert!(report.violation.is_none() && report.exhausted);
}
