//! Controls for the count of threads this process started through `spawn_named`,
//! which a fixture removal waits on before it removes a tree they write into.

use super::{begun, running};
use std::sync::mpsc::{Receiver, Sender, channel};

/// What:
///  A started thread is counted from before it runs until its last write,
///  and a thread that ends is counted no more.
/// Why:
///   The count is the whole promise a fixture removal waits on:
///  a removal that saw zero must meet no writer,
///  so the count has to be raised before the job runs and lowered after it wrote.
#[test]
fn a_thread_is_counted_from_its_start_until_its_last_write() {
    let before: usize = running();
    let (reached_sent, reached): (Sender<()>, Receiver<()>) = channel::<()>();
    let (let_go, held): (Sender<()>, Receiver<()>) = channel::<()>();
    let counted = crate::scoped_start::spawn_named("counted-thread-control", move || {
        let _said = reached_sent.send(());
        // Held until the control lets it go, so "while it runs" is a state and not a duration.
        let _held = held.recv();
    })
    .expect("a thread for the control");
    reached.recv().expect("the thread runs");
    assert_eq!(running(), before + 1, "a thread that is running is counted");
    let _let_go = let_go.send(());
    counted.join().expect("the thread ends");
    assert_eq!(running(), before, "a thread that has ended is counted no more");
}

/// What:
///  A count that never carried a thread is given back when it is dropped.
/// Why:
///   A start the system refuses drops the value the thread would have held,
///  so a refused fill does not leave a count that keeps every later fixture removal waiting.
#[test]
fn a_start_that_never_ran_leaves_no_count() {
    let before: usize = running();
    let never_ran = begun();
    assert_eq!(running(), before + 1, "the count is raised before a thread starts");
    drop(never_ran);
    assert_eq!(running(), before, "a count dropped without a thread is given back");
}

/// What:
///  A job that panics is counted no more once its stack has unwound.
/// Why:
///   The value is held by a named binding in the thread's body,
///  which an unwind drops,
///  so a job that dies leaves no count that would keep every later fixture removal waiting for 60 seconds
///  and then fail.
#[test]
fn a_job_that_panics_is_counted_no_more() {
    let before: usize = running();
    let panicking = crate::scoped_start::spawn_named("panicking-thread-control", || { panic!("the job fails, which is what the control measures"); })
        .expect("a thread for the control");
    assert!(panicking.join().is_err(), "the job panics on purpose");
    assert_eq!(running(), before, "a job that unwound is counted no more");
}
