//! Controls for the landing's scope:
//! the state belongs to the task that began the landing,
//! never to the thread the code is on,
//! and never to another landing.

use super::*;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

/// A future that is pending once and ready the second time it is polled,
/// so a test chooses where a task stops between two polls.
struct StopsOnce {
    /// Whether the first poll has happened.
    stopped: bool,
}

impl Future for StopsOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.stopped {
            return Poll::Ready(());
        }
        self.stopped = true;
        // Wakes itself so a runtime polls it again at once.
        context.waker().wake_by_ref();
        return Poll::Pending;
    }
}

/// The count of unsettled locks the state of the current landing holds,
/// the sample item of these controls.
fn steps_now() -> usize {
    return with_the_landing_state(|state| return state.unsettled_locks.get());
}

/// Sets the count of unsettled locks of the current landing.
fn set_steps(count: usize) {
    with_the_landing_state(|state| state.unsettled_locks.set(count));
}

/// A scope inside a scope has its own state,
/// and the outer one is as it was when the inner one ends:
/// two operations on one thread never share an item.
#[test]
fn a_scope_inside_a_scope_shadows_it_and_leaves_it_as_it_was() {
    in_a_landing(|| {
        set_steps(11);
        in_a_landing(|| {
            assert_eq!(steps_now(), 0, "the inner scope starts with nothing");
            set_steps(22);
        });
        assert_eq!(steps_now(), 11, "the inner scope's write never reached the outer one");
    });
}

/// Two tasks that interleave on one thread keep each its own state:
/// a thread-local would give the second task's write to the first.
#[test]
fn two_tasks_interleaved_on_one_thread_keep_their_own_state() {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    let (first, second) = runtime.block_on(async {
        let first = in_a_landing_task(async {
            set_steps(1);
            StopsOnce { stopped: false }.await;
            return steps_now();
        });
        let second = in_a_landing_task(async {
            set_steps(2);
            StopsOnce { stopped: false }.await;
            return steps_now();
        });
        return tokio::join!(first, second);
    });
    assert_eq!((first, second), (1, 2), "each task read back what it wrote, not what the other wrote between");
}

/// A task that is polled on one thread and finished on another (what a work-stealing runtime does at an `.await`) reads
/// its own state on the second thread.
#[test]
fn a_task_moved_to_another_thread_between_two_polls_keeps_its_state() {
    let mut task: Pin<Box<dyn Future<Output = usize> + Send>> = Box::pin(in_a_landing_task(async {
        set_steps(33);
        StopsOnce { stopped: false }.await;
        return steps_now();
    }));
    let mut context = Context::from_waker(Waker::noop());
    let first_poll = task.as_mut().poll(&mut context);
    assert!(first_poll.is_pending(), "the task stops after writing its state");
    let first_thread = std::thread::current().id();
    let finished_on = std::thread::spawn(move || {
        let mut second_context = Context::from_waker(Waker::noop());
        let Poll::Ready(read_back) = task.as_mut().poll(&mut second_context) else { panic!("the second poll finishes the task") };
        return (std::thread::current().id(), read_back);
    })
    .join()
    .expect("the second thread");
    assert_ne!(finished_on.0, first_thread, "the control: the task finished on a thread other than the one it began on");
    assert_eq!(finished_on.1, 33, "the state moved with the task");
}

/// A thread that a landing starts is not a landing and reads the default,
/// as a thread-local read on that thread would.
#[test]
fn a_thread_started_inside_a_scope_is_not_inside_a_landing() {
    in_a_landing(|| {
        set_steps(44);
        assert!(inside_a_landing(), "the scope's own thread is inside it");
        let seen = std::thread::spawn(|| return (inside_a_landing(), steps_now())).join().expect("the thread");
        assert_eq!(seen, (false, 0), "the started thread has no landing and no state");
    });
}

/// Outside any scope a write is not kept and a read answers its default.
#[test]
fn outside_any_scope_a_write_is_not_kept() {
    assert!(!inside_a_landing(), "no scope here");
    set_steps(55);
    assert_eq!(steps_now(), 0, "nothing was remembered");
    with_the_landing_state(|state| {
        assert!(state.index_hardened.get(), "the index is hardened by default");
        assert!(!state.deferring_syncs.get(), "syncs are made where they are asked for by default");
    });
}

/// The state ends with its scope.
#[test]
fn the_state_ends_with_its_scope() {
    in_a_landing(|| set_steps(66));
    assert_eq!(steps_now(), 0, "the scope's state is gone");
}
