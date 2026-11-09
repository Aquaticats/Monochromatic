//! Controls for jobs of the blocking pool run side by side:
//! every item runs and its answer keeps the order of the states,
//! the items overlap,
//! the job runs inside the process's runtime and the caller's function on the caller's own task,
//! a panicking item is an error in its place,
//! and where no thread can start every item runs in turn.

use super::*;
use crate::no_threads_child_test_support::ran_in_child;
use crate::parallel_test_support::Rendezvous;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many items the controls run.
const ITEMS: usize = 4;

/// A job that reads its state and a shared counter,
/// and answers with its own amount and ten times it,
/// so an answer that came back at another item's place is visible.
fn add_to_counter((counter, amount): (Arc<AtomicUsize>, usize)) -> (usize, usize) {
    let _added: usize = counter.fetch_add(amount, Ordering::SeqCst);
    return (amount, amount * 10);
}

/// A job that panics when its state says so.
fn panic_on_zero(state: usize) -> usize {
    assert!(state != 0, "the job was given zero");
    return state;
}

/// What:
///  Where one job ran:
///  the thread it ran on and whether it ran inside the process's runtime.
/// Why:
///   A job of the blocking pool is inside the runtime and is never the calling thread,
/// so a control can tell it from work done on the caller's task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whereTheJobRan = () => ({ thread: threadId(), inside: insideTheRuntime() });
/// ```
fn where_the_job_ran(_state: ()) -> (std::thread::ThreadId, bool) {
    return (std::thread::current().id(), tokio::runtime::Handle::try_current().is_ok());
}

/// One state of the overlap control:
/// the point every job of it must reach.
fn arrive_at_the_point(point: Arc<Rendezvous>) -> bool {
    return point.arrive();
}

/// What:
///  Run `run` on the process's runtime and give its answer,
/// whether the runtime is the multi-thread one or its stand-in.
/// Why:
///   Every control here is a synchronous test,
/// and the faces under test are awaited ones,
/// so each control enters the runtime once,
/// as the shipped synchronous faces do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = runOnRuntime(run());
/// ```
fn on_the_runtime<F: Future>(run: F) -> F::Output {
    return crate::lazy_runtime::run_on_runtime(run).expect("this process can build its runtime");
}

/// What:
///  The thread the caller's own function runs on.
/// Why:
///   A job of the pool must never be where the caller's work runs,
/// so a control names the caller's thread and compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theCallersThread = () => threadId();
/// ```
fn the_callers_thread(_state: ()) -> std::thread::ThreadId {
    return std::thread::current().id();
}

/// Every state runs its job once and the answers come back in the order of the states.
#[test]
fn every_state_runs_once_and_answers_keep_their_order() {
    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let states: Vec<(Arc<AtomicUsize>, usize)> = (1..=ITEMS).map(|amount| { return (Arc::clone(&counter), amount); }).collect();
    let results: Vec<Result<(usize, usize), BlockingJobError>> = on_the_runtime(run_all_side_by_side_on_the_runtime(states, add_to_counter));
    assert_eq!(results.len(), ITEMS);
    assert_eq!(
        results.into_iter().map(Result::ok).collect::<Vec<Option<(usize, usize)>>>(),
        vec![Some((1, 10)), Some((2, 20)), Some((3, 30)), Some((4, 40))],
        "the answers keep the order of the states"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 1 + 2 + 3 + 4, "each state added its amount once");
}

/// The jobs really overlap:
/// they reach one point together,
/// which a run that did them one after another could never pass.
#[test]
fn the_items_run_at_the_same_time() {
    let room: usize = crate::parallel_test_support::room_for("the items run at the same time", ITEMS);
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::new(ITEMS.min(room)));
    let states: Vec<Arc<Rendezvous>> = (0..ITEMS.min(room)).map(|_index| { return Arc::clone(&point); }).collect();
    let results: Vec<Result<bool, BlockingJobError>> = on_the_runtime(run_all_side_by_side_on_the_runtime(states, arrive_at_the_point));
    for each in results {
        assert!(each.expect("the job ran"), "an item's job did not arrive at the point while the others were there, so the items ran one after another");
    }
}

/// A job of the pool runs inside the process's runtime and never on the calling task,
/// whose own function runs here.
#[test]
fn the_job_runs_inside_the_runtime_and_the_callers_function_here() {
    let here: std::thread::ThreadId = std::thread::current().id();
    let (answer, from_here): (Result<(std::thread::ThreadId, bool), BlockingJobError>, std::thread::ThreadId) = on_the_runtime(run_beside_the_caller_on_the_runtime(
        ((), where_the_job_ran),
        ((), the_callers_thread),
    ));
    let (ran_on, inside): (std::thread::ThreadId, bool) = answer.expect("the job ran");
    assert!(inside, "the job ran outside the process's runtime");
    assert_ne!(ran_on, here, "the job ran on the calling thread, so the caller's function had nothing to run beside");
    assert_eq!(from_here, here, "the caller's function did not run on the calling thread");
}

/// A job that panics is an error at its own index and does not take the other answers with it.
#[test]
fn a_panicking_item_is_an_error_in_its_place() {
    let results: Vec<Result<usize, BlockingJobError>> = on_the_runtime(run_all_side_by_side_on_the_runtime(vec![1, 0, 3], panic_on_zero));
    assert_eq!(results[0].as_ref().ok(), Some(&1));
    assert_eq!(results[1], Err(BlockingJobError::Panicked));
    assert_eq!(results[2].as_ref().ok(), Some(&3));
}

/// Where the system starts no thread,
/// every item's work is done in turn on the awaiting task and still answers.
/// The body runs in a child that is refused every thread.
#[test]
fn where_no_thread_can_start_every_item_runs_in_turn() {
    if ran_in_child(module_path!(), "where_no_thread_can_start_every_item_runs_in_turn") {
        return;
    }
    let here: std::thread::ThreadId = std::thread::current().id();
    let results: Vec<Result<(std::thread::ThreadId, bool), BlockingJobError>> = on_the_runtime(run_all_side_by_side_on_the_runtime(vec![(), (), ()], where_the_job_ran));
    for each in results {
        let (ran_on, _inside): (std::thread::ThreadId, bool) = each.expect("the work of an item the system refused a thread for is done in turn");
        assert_eq!(ran_on, here, "an item's work ran somewhere other than the task that waited for it, where no thread can start");
    }
}
