//! Controls for the door the service's tasks take to reach a file:
//!  it answers what the job gave,
//!  it gives nothing when the job outlasts its ceiling,
//!  and the worker that awaits it is free while the job runs.

use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The door answers a job's value.
#[test]
fn the_door_answers_what_the_job_gave() {
    crate::lazy_runtime::run_on_runtime(async {
        let answered: Option<u32> = off_the_workers(Step { what: "a job that answers", ceiling: Duration::from_secs(5) }, || { return 41 + 1; }).await;
        assert_eq!(answered, Some(42));
    })
    .expect("the runtime this test runs on");
}

/// What:
///  A job that outlasts its ceiling answers nothing, and the caller goes on at once.
/// Why:
///  The bound is what keeps a file that stopped answering from holding the caller,
/// and a bound that is not reached before the answer is no bound.
#[test]
fn a_job_that_outlasts_its_ceiling_answers_nothing() {
    crate::lazy_runtime::run_on_runtime(async {
        let let_go: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let released: Arc<AtomicBool> = Arc::clone(&let_go);
        let began: std::time::Instant = std::time::Instant::now();
        let answered: Option<u32> = off_the_workers(Step { what: "a job that does not answer", ceiling: Duration::from_millis(100) }, move || {
            // The job waits until this control lets it go, which is after the ceiling has passed.
            while !released.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(2));
            }
            return 7;
        })
        .await;
        let took: Duration = began.elapsed();
        let_go.store(true, Ordering::SeqCst);
        assert_eq!(answered, None, "a job past its ceiling answers nothing");
        assert!(took < Duration::from_secs(2), "and the caller went on at its ceiling, not at the job's end: {took:?}");
    })
    .expect("the runtime this test runs on");
}

/// What:
///  The worker that awaits the door is free while the job runs.
/// Why:
///  That is the whole reason for the door: with one worker, the acceptor and the connections need the worker the
///  task was polled on, so a task that waits inside a file starves them
///  (`service_server_greets_before_the_journal_is_restored_tests.rs` holds the same point at the service's start).
#[test]
fn a_worker_is_free_while_the_job_runs() {
    crate::lazy_runtime::cap_workers(1);
    crate::lazy_runtime::run_on_runtime(async {
        let finished: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let other: Arc<AtomicBool> = Arc::clone(&finished);
        // A task of the same runtime that must run while the job waits: with one worker it can only run if the
        // awaiting task gave the worker back.
        let beside: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            other.store(true, Ordering::SeqCst);
        });
        let answered: Option<u32> = off_the_workers(Step { what: "a job with a slow file", ceiling: Duration::from_secs(10) }, || {
            std::thread::sleep(Duration::from_millis(400));
            return 3;
        })
        .await;
        let _joined: Result<(), tokio::task::JoinError> = beside.await;
        assert_eq!(answered, Some(3));
        assert!(finished.load(Ordering::SeqCst), "the other task ran while the door's job waited, so the worker was free");
    })
    .expect("the runtime this test runs on");
}
