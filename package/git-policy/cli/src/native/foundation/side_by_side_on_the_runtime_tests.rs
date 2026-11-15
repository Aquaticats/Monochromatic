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

/// The room this control gives the process:
/// one place for each item it runs at once, and the place the process keeps for the answers a client waits for.
/// Why:
///   The room a control states is the whole bound of the process (`parallel_test_support::room_for`), and this branch
///   keeps one of its places out of the jobs such a control's items take (`PLACES_KEPT_TO_ANSWER_A_CLIENT`).
///  A control that stated only its items would have one place fewer than the items it puts in flight, and a run of it
///  would wait for a place for ever instead of proving the overlap (measured at the merge of `0.6.1-alpha.38`: the
///  process took four places, kept one, and the fourth item never arrived).
const THE_ROOM_OF_THE_OVERLAP: usize = ITEMS + crate::blocking_jobs_within_the_budget::PLACES_KEPT_TO_ANSWER_A_CLIENT;

/// The jobs really overlap:
/// they reach one point together,
/// which a run that did them one after another could never pass.
#[test]
fn the_items_run_at_the_same_time() {
    let room: usize = crate::parallel_test_support::room_for("the items run at the same time", THE_ROOM_OF_THE_OVERLAP);
    assert!(
        crate::blocking_jobs_within_the_budget::places_for_the_services_own_work() >= ITEMS,
        "this control needs a place for each of its {ITEMS} items beside the one kept for the answers a client waits for, and the room of {room} leaves {}",
        crate::blocking_jobs_within_the_budget::places_for_the_services_own_work()
    );
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::new(ITEMS));
    let states: Vec<Arc<Rendezvous>> = (0..ITEMS).map(|_index| { return Arc::clone(&point); }).collect();
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

/// What:
///  Where the system starts no thread, every item answers that no place came, and no item's work runs where nothing can
///  interrupt it.
/// Why:
///   A call into storage must never run on the task that awaits it (`blocking_jobs_within_the_budget.rs`,
///   `run_a_job_or_no_place`): a deadline around that task cannot end it, so a fan-out where no thread can start answers
///   that no place came and the caller looks again.
/// The body runs in a child that is refused every thread.
#[test]
fn where_no_thread_can_start_every_item_answers_that_no_place_came() {
    if ran_in_child(module_path!(), "where_no_thread_can_start_every_item_answers_that_no_place_came") {
        return;
    }
    let ran: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let states: Vec<std::sync::Arc<std::sync::atomic::AtomicBool>> = (0..3).map(|_at| { return std::sync::Arc::clone(&ran); }).collect();
    let results: Vec<Result<(std::thread::ThreadId, bool), BlockingJobError>> = on_the_runtime(run_all_side_by_side_on_the_runtime(states, where_the_job_ran_wherever));
    for each in results {
        assert_eq!(
            each,
            Err(BlockingJobError::NoPlaceWithinTheCeiling),
            "an item the system refused a thread for answered something other than that no place came"
        );
    }
    assert!(!ran.load(std::sync::atomic::Ordering::SeqCst), "an item's work ran where no thread could start and no deadline can end it");
}

/// One item's work as the control of a refused thread reads it:
/// it records that it ran, on whichever thread that is.
/// Why:
///   An answer that no place came must mean the work did not run at all, and only the work itself can say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whereTheJobRan = (ran) => { ran.value = true; return [threadId(), true] }
/// ```
fn where_the_job_ran_wherever(ran: std::sync::Arc<std::sync::atomic::AtomicBool>) -> (std::thread::ThreadId, bool) {
    ran.store(true, std::sync::atomic::Ordering::SeqCst);
    return (std::thread::current().id(), true);
}
