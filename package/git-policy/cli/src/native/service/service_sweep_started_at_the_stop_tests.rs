//! Controls for the hand-over of a stop's sweep:
//!  a sweep that a thread runs answers within its bound,
//!  and a system that refused the thread is told apart from a sweep that did not answer.

use super::*;

/// A sweep whose thread answers is `Completely` inside the bound:
/// the thread was started at the stop and was waiting for the ask.
#[test]
fn a_sweep_that_is_asked_answers_within_its_bound() {
    let sweep: TheSweepOfMarkersAtTheStop = start_the_sweep_at_the_stop();
    assert_eq!(sweep.ask(Duration::from_secs(30)), Swept::Completely, "the thread was started and answered");
}

/// A stop whose system refused the thread says so at once, rather than waiting out its bound for an answer
/// nobody will send.
#[test]
fn a_sweep_whose_thread_never_started_says_no_thread_ran() {
    let (ask, _asked): (Sender<()>, Receiver<()>) = channel();
    let (_answered, done): (Sender<()>, Receiver<()>) = channel();
    let sweep: TheSweepOfMarkersAtTheStop = TheSweepOfMarkersAtTheStop { ask, done, started: false };
    let began: Duration = Duration::from_millis(0);
    assert_eq!(sweep.ask(began), Swept::NoThread, "no thread ran, and nothing waits for one");
}
