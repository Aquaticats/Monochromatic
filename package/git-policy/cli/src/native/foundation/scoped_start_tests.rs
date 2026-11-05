//! Controls for the start of a scoped thread: the job runs on a thread when one starts and on the calling thread when none does.

use super::*;

/// A job that names the thread it runs on.
fn thread_of(_state: ()) -> std::thread::ThreadId {
    return std::thread::current().id();
}

/// A job that counts its runs.
fn count_run(calls: &std::sync::atomic::AtomicU32) -> std::thread::ThreadId {
    calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    return std::thread::current().id();
}

/// A job that panics.
fn fail(_state: ()) -> u32 {
    panic!("the work failed");
}

/// A start that works runs the job on another thread.
#[test]
fn a_thread_that_starts_runs_the_closure_on_another_thread() {
    let here: std::thread::ThreadId = std::thread::current().id();
    let ran_on = std::thread::scope(|scope| {
        let started = start_or_run_with(scope, (), thread_of);
        assert!(!started.ran_inline());
        return started.join().expect("the thread");
    });
    assert_ne!(ran_on, here);
}

/// A start the system refuses runs the job here, once, and returns its value; a panic is handed to `join`.
#[test]
fn a_thread_that_cannot_start_runs_the_closure_on_the_calling_thread_once() {
    STARTS_FAIL_ON_THIS_THREAD.with(|flag| { flag.set(true); });
    let here: std::thread::ThreadId = std::thread::current().id();
    let calls = std::sync::atomic::AtomicU32::new(0);
    let (ran_on, panicked) = std::thread::scope(|scope| {
        let started = start_or_run_with(scope, &calls, count_run);
        assert!(started.ran_inline());
        let panicking = start_or_run_with(scope, (), fail);
        return (started.join().expect("the value"), panicking.join().is_err());
    });
    STARTS_FAIL_ON_THIS_THREAD.with(|flag| { flag.set(false); });
    assert_eq!(ran_on, here, "run on the calling thread");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1, "the job ran once");
    assert!(panicked, "a panic of the work is a join error, as on a thread");
}

/// A thread of the plain kind that the system refuses is an error to the caller, not a panic.
#[test]
fn a_named_thread_that_cannot_start_is_an_error() {
    let _refused = RefusedStarts::here();
    let refused = spawn_named("never", || {}).expect_err("the start is refused");
    assert_eq!(refused.raw_os_error(), Some(libc::EAGAIN));
}
