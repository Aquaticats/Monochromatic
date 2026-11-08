//! The stop of the service as its waiters see it:
//! a process that never prepared counts nothing,
//! a stop refuses the work that began after it unless that work must finish,
//! the arm of work that began before the stop resolves when the shutdown tells it,
//! and the shutdown waits for the work in flight for at most its bound.
//! Each control runs in a process of its own (nextest), because the stop is process-wide state.

use super::*;
use crate::lazy_runtime::run_on_runtime;

/// The stop signal the controls record.
const SIGNAL: libc::c_int = libc::SIGTERM;

/// A bound far beyond what a control needs,
/// so only a defect reaches it.
const PATIENCE: Duration = Duration::from_secs(30);

/// A bound short enough for a control that expects to reach it.
const SHORT_BOUND: Duration = Duration::from_millis(100);

/// A process that never prepared to notice a stop counts nothing,
/// refuses nothing and hands out no arm,
/// whatever was recorded.
#[test]
fn a_process_that_never_prepared_counts_nothing_and_refuses_nothing() {
    assert!(record_from_a_signal_handler(SIGNAL));
    let mut work: Unfinished = begin_unfinished_work(false, "git status").expect("work is not refused where no stop is noticed");
    assert_eq!(unfinished_work(), 0);
    assert!(work.take_arm().is_none());
}

/// Work is counted from its start to its drop.
#[test]
fn work_is_counted_from_its_start_to_its_drop() {
    prepare_to_notice().expect("the pipe");
    let first: Unfinished = begin_unfinished_work(false, "git status").expect("work");
    let second: Unfinished = begin_unfinished_work(false, "git log").expect("work");
    assert_eq!(unfinished_work(), 2);
    drop(first);
    assert_eq!(unfinished_work(), 1);
    drop(second);
    assert_eq!(unfinished_work(), 0);
}

/// The first stop signal is the one recorded,
/// and a second is told apart from it.
#[test]
fn only_the_first_stop_signal_is_recorded() {
    prepare_to_notice().expect("the pipe");
    assert_eq!(stopping_signal(), None);
    assert!(record_from_a_signal_handler(SIGNAL));
    assert!(!record_from_a_signal_handler(libc::SIGINT));
    assert_eq!(stopping_signal(), Some(SIGNAL));
    assert!(is_stopping());
}

/// After the stop,
/// work that is not allowed is refused and not counted,
/// and work that must finish is counted and has no arm.
#[test]
fn a_stop_refuses_new_work_unless_it_must_finish_and_the_refusal_is_not_counted() {
    prepare_to_notice().expect("the pipe");
    assert!(record_from_a_signal_handler(SIGNAL));
    let refused: RefusedWhileStopping = begin_unfinished_work(false, "git push").expect_err("work after the stop is refused");
    assert!(refused.to_string().contains("git push") && refused.to_string().contains("stopping"), "{refused}");
    assert_eq!(unfinished_work(), 0, "a refusal gives its count back");
    let mut unwinding: Unfinished = begin_unfinished_work(true, "git update-ref").expect("work that must finish is allowed");
    assert_eq!(unfinished_work(), 1);
    assert!(unwinding.take_arm().is_none(), "work that began after the stop is not ended by it");
}

/// The arm of work that began before the stop resolves when the shutdown tells the stop,
/// not when the signal is recorded:
/// the handler's last line is out first.
#[test]
fn the_arm_of_work_begun_before_the_stop_resolves_when_the_shutdown_tells_it() {
    prepare_to_notice().expect("the pipe");
    let mut work: Unfinished = begin_unfinished_work(false, "git fast-import").expect("work");
    let mut arm: StopArm = work.take_arm().expect("work begun before a stop has an arm");
    let outcome = run_on_runtime(async move {
        assert!(record_from_a_signal_handler(SIGNAL));
        let early = tokio::time::timeout(SHORT_BOUND, arm.stop_told()).await;
        assert!(early.is_err(), "a recorded stop is not yet told");
        assert!(wake_the_shutdown_from_a_signal_handler());
        let shutdown = tokio::spawn(tell_the_stop_and_wait_for_the_work(PATIENCE));
        let told = tokio::time::timeout(PATIENCE, arm.stop_told()).await;
        drop(work);
        let settled = shutdown.await.expect("the shutdown task");
        return (told.is_ok(), settled);
    })
    .expect("the runtime");
    assert!(outcome.0, "the arm resolved when the stop was told");
    assert_eq!(outcome.1, Settled::Completely);
}

/// The shutdown waits for work still in flight and gives up at its bound,
/// and says which.
#[test]
fn the_shutdown_gives_up_at_its_bound_with_work_in_flight() {
    prepare_to_notice().expect("the pipe");
    let held: Unfinished = begin_unfinished_work(false, "git gc").expect("work");
    assert!(record_from_a_signal_handler(SIGNAL));
    assert!(wake_the_shutdown_from_a_signal_handler());
    let settled: Settled = run_on_runtime(tell_the_stop_and_wait_for_the_work(SHORT_BOUND)).expect("the runtime");
    assert_eq!(settled, Settled::AtTheBound);
    assert_eq!(unfinished_work(), 1);
    drop(held);
}

/// With nothing in flight the shutdown settles at once.
#[test]
fn the_shutdown_settles_at_once_with_nothing_in_flight() {
    prepare_to_notice().expect("the pipe");
    assert!(record_from_a_signal_handler(SIGNAL));
    assert!(wake_the_shutdown_from_a_signal_handler());
    let settled: Settled = run_on_runtime(tell_the_stop_and_wait_for_the_work(PATIENCE)).expect("the runtime");
    assert_eq!(settled, Settled::Completely);
}

/// The last-resort bound and the settle bound stay below the service manager's default stop timeout,
/// so the manager never kills a unit that is ending in order.
#[test]
fn the_bounds_end_before_the_managers_default_stop_timeout() {
    let manager_default: Duration = Duration::from_secs(90);
    assert!(SETTLE_CEILING < LAST_RESORT);
    assert!(LAST_RESORT < manager_default, "{LAST_RESORT:?}");
}

/// A handler wakes nothing in a process that did not prepare,
/// so it falls back to ending the process.
#[test]
fn without_a_pipe_the_handler_has_nothing_to_wake() {
    assert!(!wake_the_shutdown_from_a_signal_handler());
}
