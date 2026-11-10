//! What:
//!  Start a job (a function and the state it reads) on a scoped thread,
//! and run it on the calling thread when the system cannot start one.
//! Why:
//!  `std::thread::Scope::spawn` panics with "failed to spawn thread" when the thread cannot start (the process limit,
//! the address
//!  space,
//! no memory),
//! and a panic on the service's landing thread takes the service down:
//! the commit cannot land.
//! The owner's rule
//!  is "Commits should never fail.",
//! and a thread is only a way to do the work sooner,
//! so the work is done in turn where a thread
//!  cannot start.
//! The job is handed to the thread through a slot that either side can empty,
//! because
//!  `Builder::spawn_scoped` consumes its callable even when it fails.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = startOrRunWith(scope, state, work); // a worker thread, or work(state) done here
//! const result = started.join();
//! ```

/// The panic-catching call that runs the job here when no thread starts.
use crate::call_catching_panics::call_catching_panics;
use std::sync::{Arc, Mutex};
use std::thread::{Scope, ScopedJoinHandle};

#[cfg(test)]
thread_local! {
    /// A control makes every start from this thread fail,
    /// as the system does when it has no thread to give.
    pub(crate) static STARTS_FAIL_ON_THIS_THREAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// While it lives,
/// every scoped start from the thread that made it is refused,
/// as when the system has no thread to give.
#[cfg(test)]
pub(crate) struct RefusedStarts;

#[cfg(test)]
impl RefusedStarts {
    /// Refuse the starts of the current thread until the value is dropped.
    pub(crate) fn here() -> Self {
        STARTS_FAIL_ON_THIS_THREAD.with(|flag| { flag.set(true); });
        return Self;
    }
}

#[cfg(test)]
impl Drop for RefusedStarts {
    fn drop(&mut self) {
        STARTS_FAIL_ON_THIS_THREAD.with(|flag| { flag.set(false); });
    }
}

/// A job that runs on a thread of its own,
/// or that already ran on the thread that started it.
pub enum Started<'scope, T> {
    /// The thread that runs it.
    Thread(ScopedJoinHandle<'scope, T>),
    /// What the job returned (or the payload of its panic) on the calling thread,
    /// because no thread could start.
    Inline(std::thread::Result<T>),
}

impl<T> Started<'_, T> {
    /// Waits for the job and returns what it returned,
    /// or the payload of its panic.
    pub fn join(self) -> std::thread::Result<T> {
        return match self {
            Started::Thread(handle) => handle.join(),
            Started::Inline(done) => done,
        };
    }

    /// Whether the job ran on the calling thread because no thread could start.
    pub fn ran_inline(&self) -> bool {
        return matches!(self, Started::Inline(_));
    }
}

/// What:
///  Takes the job out of its shared slot,
/// whether or not a panic poisoned the slot's lock.
/// Why:
///  Whichever of the new thread and the starting thread gets here first runs the job;
/// the other finds `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeJob<J>(slot: { value: J | undefined }): J | undefined { const taken = slot.value; slot.value = undefined; return taken; }
/// ```
fn take_job<J>(slot: &Mutex<Option<J>>) -> Option<J> {
    return match slot.lock() {
        Ok(mut guard) => guard.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
}

/// What:
///  Run `work(state)` on a new thread of `scope`,
/// or here when the thread cannot start.
/// Why:
///   See the module comment.
/// The job is a function and the state it reads,
/// never a closure,
/// so the caller's borrows are named in
///  the state's type.
/// A panic of the job run here is caught and handed to `join`,
/// as a panic on a thread would be.
///  `Builder::spawn_scoped` accepts only a callable,
/// so this is the one place that makes the callable;
/// it carries the slot
///  that holds the job.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = startOrRunWith(scope, state, work); // a worker thread, or work(state) done here
/// ```
pub fn start_or_run_with<'scope, 'env, S, T>(
    scope: &'scope Scope<'scope, 'env>,
    state: S,
    work: fn(S) -> T,
) -> Started<'scope, T>
where
    S: Send + 'scope,
    T: Send + 'scope,
{
    let slot: Arc<Mutex<Option<(S, fn(S) -> T)>>> = Arc::new(Mutex::new(Some((state, work))));
    let theirs: Arc<Mutex<Option<(S, fn(S) -> T)>>> = Arc::clone(&slot);
    #[cfg_attr(
        dylint_lib = "monochromatic_lint_closure",
        expect(
            avoidable_closure,
            reason = "`Builder::spawn_scoped` accepts only a callable bounded by the scope, and it must carry the slot that holds the job; see package/git-policy/cli/doc/closure-lint-suppressions.md"
        )
    )]
    let outcome: std::io::Result<ScopedJoinHandle<'scope, T>> = match starts() {
        Ok(()) => std::thread::Builder::new().spawn_scoped(scope, move || {
            let taken: Option<(S, fn(S) -> T)> = take_job(&theirs);
            return match taken {
                Some((their_state, their_work)) => their_work(their_state),
                None => panic!("the job of a started thread was taken by the thread that started it"),
            };
        }),
        Err(error) => Err(error),
    };
    return match outcome {
        Ok(handle) => Started::Thread(handle),
        Err(_refused) => {
            let taken: Option<(S, fn(S) -> T)> = take_job(&slot);
            match taken {
                Some((our_state, our_work)) => Started::Inline(call_catching_panics(our_state, our_work)),
                None => Started::Inline(Err(Box::new("the job of a thread that did not start was already taken"))),
            }
        }
    };
}

/// What:
///  Start a thread that outlives the caller,
/// and report a refusal as an error.
/// Why:
///   `std::thread::spawn` panics where `Builder::spawn` returns the refusal;
/// the callers of this have no way to do the work in turn
///  and say so in a message instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worker = spawnNamed('name', () => work()); // throws when no thread can start
/// ```
pub fn spawn_named<F, T>(name: &str, work: F) -> std::io::Result<std::thread::JoinHandle<T>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    starts()?;
    return std::thread::Builder::new().name(String::from(name)).spawn(work);
}

/// Whether a thread may start now:
/// always,
/// except where a control made the start fail.
fn starts() -> std::io::Result<()> {
    #[cfg(test)]
    if STARTS_FAIL_ON_THIS_THREAD.with(std::cell::Cell::get) {
        return Err(std::io::Error::from_raw_os_error(libc::EAGAIN));
    }
    return Ok(());
}

/// What:
///  One sentence saying that no thread could be started to `what`,
/// which limit refused it,
/// and what frees one.
/// Why:
///  A refusal reaches a person as the system's own error (`Resource temporarily unavailable`),
/// which names
///  neither what was refused nor anything they can change;
/// a thread is refused by the process limit,
/// the
///  address-space limit or the control group's process limit,
/// and those are the three things to free.
/// Every site that
///  answers a refused start says this one sentence,
/// so no site is left naming only the system's error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refusedThreadSentence('wait for the step a write started', error); // 'no thread could be started to wait for ... Hint: check `ulimit -u` ...'
/// ```
pub fn refused_thread_sentence(what: &str, error: &std::io::Error) -> String {
    return format!(
        "no thread could be started to {what}: the system refused the thread ({error}). Hint: check the process limit \
         (`ulimit -u`), the address-space limit (`ulimit -v`) and the control group's `pids.max`, free some, and run the \
         command again"
    );
}

/// Scoped start controls stay out of the release executable.
#[cfg(test)]
#[path = "scoped_start_tests.rs"]
mod tests;
