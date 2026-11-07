//! The blocking-job helper: the value comes back, a panic in the job is an error, and a system that refuses every thread gives an
//! error value, not a panic.

use super::*;
use crate::lazy_runtime::run_on_runtime;

/// The job's value comes back.
#[test]
fn a_job_gives_its_value() {
    let value = run_on_runtime(run_blocking_job(|| { return 41 + 1; })).expect("the runtime").expect("the job");
    assert_eq!(value, 42);
}

/// A panic in the job is an error value and the runtime still serves the next job.
#[test]
fn a_job_that_panics_is_an_error_and_the_next_job_still_runs() {
    let first = run_on_runtime(run_blocking_job(|| -> u8 { panic!("the job panicked") })).expect("the runtime");
    assert_eq!(first, Err(BlockingJobError::Panicked));
    let second = run_on_runtime(run_blocking_job(|| { return 5; })).expect("the runtime").expect("the job");
    assert_eq!(second, 5);
}

/// Where the system refuses every thread, the job does not run, the helper returns the plain error, and nothing panics.
#[test]
fn where_no_thread_can_start_the_job_fails_plainly_and_does_not_run() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_the_job_fails_plainly_and_does_not_run") {
        return;
    }
    let ran = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = std::sync::Arc::clone(&ran);
    let outcome = run_on_runtime(run_blocking_job(move || { flag.store(true, std::sync::atomic::Ordering::SeqCst); }))
        .expect("a current-thread runtime stands in where no thread can start");
    assert_eq!(outcome, Err(BlockingJobError::NoThreadToStart));
    assert!(!ran.load(std::sync::atomic::Ordering::SeqCst), "the job ran inline");
}
