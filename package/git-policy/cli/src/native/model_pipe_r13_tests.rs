//! What:
//!  The controls of P2a's pipe discipline and the syncer's separation from the owner's loop.
//! Why:
//!  Control 6 and the second half of control 19 of the twelfth review: a retained write end prevents end of input, a short or split write
//!  interleaves, `EPIPE` and a retained read end hide the failure, a buffered `commit` runs before end of input; and a stalled syncer
//!  must not stop the owner admitting work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('revision 12 leaves the bootstrap unspecified and each mistake is reachable', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_pipe_r13::{Child, Rules, check_end_of_input, check_loop, check_pipe, initials, pipe_invariant, pipe_steps};

/// Revision 12 (red): the unspecified bootstrap reaches a hidden failure and an interleaved line; revision 13 (green) reaches neither.
#[test]
fn the_unspecified_bootstrap_hides_failures_and_interleaves_lines() {
    let twelve: Report = check_pipe(Rules::Twelve);
    assert!(twelve.violation.is_some(), "a retained read end or a split write must be reachable");
    let thirteen: Report = check_pipe(Rules::Thirteen);
    assert!(thirteen.violation.is_none() && thirteen.exhausted);
}

/// A retained read end hides a dead reader: the client writes after the child ended, the write succeeds, and nothing queues.
#[test]
fn a_retained_read_end_hides_epipe() {
    let start = initials(Rules::Twelve).into_iter().find(|state| { return state.read_copy && !state.split_lines && !state.stray_write; }).expect("the start");
    let reached = replay(
        start,
        pipe_steps,
        &["client: writes the line in one write", "git: update-ref reads the queued line, executes it and ends", "owner: writes the line in one write"],
    );
    assert!(reached.hidden_failure);
    assert_eq!(pipe_invariant(&reached), Some("a write to a pipe whose child died succeeded because a retained read end hid the failure".to_string()));
    let disciplined = initials(Rules::Thirteen).remove(0);
    let seen = replay(
        disciplined,
        pipe_steps,
        &["client: writes the line in one write", "git: update-ref reads the queued line, executes it and ends", "owner: the write fails with EPIPE and the writer knows"],
    );
    assert!(!seen.hidden_failure && seen.child == Child::Dead);
}

/// Two writers that each write a line in two parts interleave into garbage; one atomic write per line cannot.
#[test]
fn split_writes_interleave_and_single_writes_do_not() {
    let start = initials(Rules::Twelve).into_iter().find(|state| { return state.split_lines && !state.read_copy && !state.stray_write; }).expect("the start");
    let reached = replay(start, pipe_steps, &["client: writes the first part of the line", "owner: writes its first part into the other writer's half line"]);
    assert!(reached.garbage);
}

/// A retained write end (a hook descendant or an owner copy) prevents end of input: an abandoned prepared transaction never ends (red
/// under 12); the specified bootstrap closes every other end (green under 13).
#[test]
fn a_retained_write_end_prevents_end_of_input_and_the_abandoned_transaction_never_ends() {
    let twelve: Report = check_end_of_input(Rules::Twelve);
    assert!(twelve.violation.is_some());
    let thirteen: Report = check_end_of_input(Rules::Thirteen);
    assert!(thirteen.violation.is_none() && thirteen.exhausted, "{:?}", thirteen.violation.map(|found| { return found.message; }));
}

/// A buffered `commit` runs before end of input: the owner dies and the client exits with `commit` queued, and the child executes it
/// rather than aborting (the owner's death does not cancel a submitted instruction).
#[test]
fn a_buffered_commit_is_read_before_end_of_input() {
    let start = initials(Rules::Thirteen).remove(0);
    let reached = replay(
        start,
        pipe_steps,
        &["client: writes the line in one write", "client: exits and closes its write end", "owner: dies or exits and closes its write end"],
    );
    let labels: Vec<String> = pipe_steps(&reached).into_iter().map(|step| { return step.0; }).collect();
    assert!(labels.contains(&"git: update-ref reads the queued line, executes it and ends".to_string()));
    assert!(!labels.iter().any(|label| { return label.contains("aborts the prepared transaction"); }), "end of input does not cancel the queued line");
}

/// Control 19: when the owner's loop waits for the syncer, a stalled syncer stops admission (red); revision 13 never puts the syncer in
/// the owner's loop, so admission stays reachable (green).
#[test]
fn a_stalled_syncer_must_not_stop_the_owner_from_admitting_work() {
    assert!(check_loop(true).violation.is_some());
    let separate: Report = check_loop(false);
    assert!(separate.violation.is_none() && separate.exhausted);
}
