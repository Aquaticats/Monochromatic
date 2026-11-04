//! What:
//!  Start a closure on a scoped thread, and run it on the calling thread when the system cannot start one.
//! Why:
//!  `std::thread::Scope::spawn` panics with "failed to spawn thread" when the thread cannot start (the process limit, the address
//!  space, no memory), and a panic on the service's landing thread takes the service down: the commit cannot land. The owner's rule
//!  is "Commits should never fail.", and a thread is only a way to do the work sooner, so the work is done in turn where a thread
//!  cannot start. The closure is handed to the thread through a slot that either side can empty, because
//!  `Builder::spawn_scoped` consumes the closure even when it fails.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = startOrRun(() => work()); // a worker thread, or the work done here
//! const result = started.join();
//! ```

use std::sync::{Arc, Mutex};
use std::thread::{Scope, ScopedJoinHandle};

#[cfg(test)]
thread_local! {
    /// A control makes every start from this thread fail, as the system does when it has no thread to give.
    pub(crate) static STARTS_FAIL_ON_THIS_THREAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A closure that runs on a thread of its own, or that already ran on the thread that started it.
pub enum Started<'scope, T> {
    /// The thread that runs it.
    Thread(ScopedJoinHandle<'scope, T>),
    /// What it returned (or the payload of its panic) on the calling thread, because no thread could start.
    Inline(std::thread::Result<T>),
}

impl<T> Started<'_, T> {
    /// Waits for the closure and returns what it returned, or the payload of its panic.
    pub fn join(self) -> std::thread::Result<T> {
        return match self {
            Started::Thread(handle) => handle.join(),
            Started::Inline(done) => done,
        };
    }

    /// Whether the closure ran on the calling thread because no thread could start.
    pub fn ran_inline(&self) -> bool {
        return matches!(self, Started::Inline(_));
    }
}

/// What:
///  Run `work` on a new thread of `scope`, or here when the thread cannot start.
/// Why:
///   See the module comment. A panic of the closure run here is caught and handed to `join`, as a panic on a thread would be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = startOrRun(scope, () => work());
/// ```
pub fn start_or_run<'scope, 'env, T, F>(scope: &'scope Scope<'scope, 'env>, work: F) -> Started<'scope, T>
where
    F: FnOnce() -> T + Send + 'scope,
    T: Send + 'scope,
{
    let slot: Arc<Mutex<Option<F>>> = Arc::new(Mutex::new(Some(work)));
    let theirs: Arc<Mutex<Option<F>>> = Arc::clone(&slot);
    let outcome: std::io::Result<ScopedJoinHandle<'scope, T>> = starts().and_then(|()| {
        return std::thread::Builder::new().spawn_scoped(scope, move || {
            let taken: Option<F> = theirs.lock().map_or_else(|poisoned| { return poisoned.into_inner().take(); }, |mut guard| { return guard.take(); });
            return taken.map(|closure| { return closure(); }).unwrap_or_else(|| { panic!("the closure of a started thread was taken by the thread that started it"); });
        });
    });
    return match outcome {
        Ok(handle) => Started::Thread(handle),
        Err(_refused) => {
            let taken: Option<F> = slot.lock().map_or_else(|poisoned| { return poisoned.into_inner().take(); }, |mut guard| { return guard.take(); });
            match taken {
                Some(closure) => Started::Inline(std::panic::catch_unwind(std::panic::AssertUnwindSafe(closure))),
                None => Started::Inline(Err(Box::new("the closure of a thread that did not start was already taken"))),
            }
        }
    };
}

/// Whether a thread may start now: always, except where a control made the start fail.
fn starts() -> std::io::Result<()> {
    #[cfg(test)]
    if STARTS_FAIL_ON_THIS_THREAD.with(std::cell::Cell::get) {
        return Err(std::io::Error::from_raw_os_error(libc::EAGAIN));
    }
    return Ok(());
}

/// Scoped start controls stay out of the release executable.
#[cfg(test)]
#[path = "scoped_start_tests.rs"]
mod tests;
