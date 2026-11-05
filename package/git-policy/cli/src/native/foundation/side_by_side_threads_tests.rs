//! Controls for jobs on scoped threads: every job runs, results keep the order of the states, a panic is an error in its place,
//! and the calling thread's function runs beside the thread's job.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A job that reads its state and the shared counter.
fn add_to_counter((counter, amount): (&AtomicUsize, usize)) -> usize {
    return counter.fetch_add(amount, Ordering::SeqCst) + amount;
}

/// A job that panics when its state says so.
fn panic_on_zero(state: usize) -> usize {
    assert!(state != 0, "the job was given zero");
    return state;
}

/// A function that names the thread it runs on.
fn thread_of(_state: ()) -> std::thread::ThreadId {
    return std::thread::current().id();
}

/// Every state runs its job once and the results come back in the order of the states.
#[test]
fn every_state_runs_once_and_results_keep_their_order() {
    let counter: AtomicUsize = AtomicUsize::new(0);
    let states: Vec<(&AtomicUsize, usize)> = (1..=4).map(|amount| { return (&counter, amount); }).collect();
    let results: Vec<std::thread::Result<usize>> = run_all_side_by_side(states, add_to_counter);
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(Result::is_ok));
    assert_eq!(counter.load(Ordering::SeqCst), 10, "each state added its amount once");
}

/// A job that panics is an error at its own index and does not take the other results with it.
#[test]
fn a_job_that_panics_is_an_error_in_its_place() {
    let results: Vec<std::thread::Result<usize>> = run_all_side_by_side(vec![1, 0, 3], panic_on_zero);
    assert_eq!(results[0].as_ref().ok(), Some(&1));
    assert!(results[1].is_err());
    assert_eq!(results[2].as_ref().ok(), Some(&3));
}

/// The thread's job runs on another thread and the caller's function on the calling thread.
#[test]
fn the_callers_function_runs_here_and_the_job_on_another_thread() {
    let here: std::thread::ThreadId = std::thread::current().id();
    let (from_thread, from_here) = run_beside_caller(((), thread_of), ((), thread_of));
    assert_ne!(from_thread.expect("the thread"), here);
    assert_eq!(from_here, here);
}

/// Where no thread can start, every job runs on the calling thread in turn and still returns its value.
#[test]
fn jobs_run_in_turn_where_no_thread_starts() {
    let _refused = crate::scoped_start::RefusedStarts::here();
    let here: std::thread::ThreadId = std::thread::current().id();
    let results: Vec<std::thread::Result<std::thread::ThreadId>> = run_all_side_by_side(vec![(), ()], thread_of);
    for each in results {
        assert_eq!(each.expect("the value"), here);
    }
}
