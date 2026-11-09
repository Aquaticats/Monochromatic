//! The blocking-job helper:
//! the value comes back,
//! a panic in the job is an error,
//! and a system that refuses every thread gives an
//! error value,
//! not a panic.

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

/// Where the system refuses every thread,
/// the job does not give its value,
/// the helper returns the plain error,
/// and nothing panics.
#[test]
fn where_no_thread_can_start_the_job_fails_plainly_and_gives_no_value() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_the_job_fails_plainly_and_gives_no_value") {
        return;
    }
    let ran = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = std::sync::Arc::clone(&ran);
    let outcome = run_on_runtime(run_blocking_job(move || { flag.store(true, std::sync::atomic::Ordering::SeqCst); }))
        .expect("a current-thread runtime stands in where no thread can start");
    assert_eq!(outcome, Err(BlockingJobError::MayStillRunAfterItsThreadWasRefused), "nothing here can stop the closure the pool queued");
    assert!(!ran.load(std::sync::atomic::Ordering::SeqCst), "the job had not run at the moment the refusal was answered; the pool still held it queued");
}

/// What:
///  A runner that takes no claim on the job's work says only what it knows:
/// the pool had already queued the step,
/// so it may still run later.
/// Why:
///   The pool pushes the job before it asks for a thread
/// (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`),
/// so "the step changed nothing" is false here.
///  A caller that takes the claim can say it (`blocking_jobs_within_the_budget.rs`);
/// this runner cannot,
/// and
///  its message must not borrow that claim's words.
#[test]
fn a_refusal_from_this_runner_says_the_step_may_still_run_and_claims_no_state_of_it() {
    let said: String = BlockingJobError::MayStillRunAfterItsThreadWasRefused.to_string();
    assert!(said.contains("may still run later"), "the answer keeps the queue the pool made: {said}");
    assert!(said.contains("not known from here"), "the answer says what it cannot know: {said}");
    assert!(!said.contains("changed nothing"), "this runner cannot promise the step changed nothing: {said}");
    assert!(!said.contains("will not run"), "this runner cannot promise the step will not run: {said}");
}

/// What:
///  The answer that does promise the step changed nothing names what makes it sure.
/// Why:
///   "will not run" is true only because a caller took the claim on the step's work first
/// (`the_claim_a_job_takes_before_running_it.rs`),
/// so a reader can tell the two refusals apart.
#[test]
fn the_refusal_that_promises_no_change_names_the_claim_that_makes_it_sure() {
    let said: String = BlockingJobError::NoThreadToStart.to_string();
    assert!(said.contains("changed nothing"), "the answer a claim-taking caller gives: {said}");
    assert!(said.contains("took the claim"), "the answer says why it can promise that: {said}");
}
