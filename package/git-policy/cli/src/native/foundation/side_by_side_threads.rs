//! What:
//!  Run jobs on scoped threads beside each other or beside the calling thread,
//! where a job is a function and the state it reads.
//! Why:
//!  The workers borrow the caller's paths,
//! counters and slices for the length of one thread scope,
//! which a plain `'static`
//!  thread cannot do without owned copies.
//! `std::thread::scope` accepts only a callable,
//! so this is the one place that makes
//!  it;
//! the callers name each job as a function and its borrows as the state's type,
//! and no closure carries them.
//! A thread that
//!  cannot start runs its job in turn (`start_or_run_with`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const results = await Promise.all(states.map((state) => job(state)));
//! const [thread, here] = await Promise.all([job(state), caller(callerState)]);
//! ```

/// The start of one job on a scoped thread,
/// or its run here when no thread starts.
use crate::scoped_start::{Started, start_or_run_with};
use std::thread::Scope;

/// What:
///  A piece of work done inside one thread scope:
/// it starts jobs on the scope,
/// does its own work beside them,
/// and returns.
/// Why:
///   The work is a value that holds the state it reads,
/// so the borrows of the caller are named in a type and no closure carries
///  them.
/// The scope ends,
/// joining every thread still running,
/// when `run` returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ScopeBody<Output> { run(scope: Scope): Output }
/// ```
pub trait ScopeBody<'env> {
    /// What the work returns once it is done.
    type Output;

    /// Do the work,
    /// starting jobs on `scope` as needed.
    fn run<'scope>(self, scope: &'scope Scope<'scope, 'env>) -> Self::Output;
}

/// What:
///  Run `body` inside one thread scope and return what it returns.
/// Why:
///   `std::thread::scope` accepts only a callable,
/// so this is the one place that makes it;
/// it carries the body.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = inThreadScope(body);
/// ```
pub fn in_thread_scope<'env, B: ScopeBody<'env>>(body: B) -> B::Output {
    #[cfg_attr(
        dylint_lib = "monochromatic_lint_closure",
        expect(
            avoidable_closure,
            reason = "`std::thread::scope` accepts only a callable, and it must carry the body; see package/git-policy/cli/doc/closure-lint-suppressions.md"
        )
    )]
    let value: B::Output = std::thread::scope(move |scope| { return body.run(scope); });
    return value;
}

/// The jobs of `run_all_side_by_side`:
/// each state with the function that runs it.
struct AllJobs<S, T> {
    /// The state of each job,
    /// in the order the jobs start.
    states: Vec<S>,
    /// The function every job runs.
    job: fn(S) -> T,
}

impl<'env, S: Send + 'env, T: Send + 'env> ScopeBody<'env> for AllJobs<S, T> {
    type Output = Vec<std::thread::Result<T>>;

    /// Start every job,
    /// then wait for each in the order they started.
    fn run<'scope>(self, scope: &'scope Scope<'scope, 'env>) -> Self::Output {
        let mut started: Vec<Started<'scope, T>> = Vec::new();
        for state in self.states {
            started.push(start_or_run_with(scope, state, self.job));
        }
        let mut results: Vec<std::thread::Result<T>> = Vec::new();
        for each in started {
            results.push(each.join());
        }
        return results;
    }
}

/// What:
///  Run `job` once for each state,
/// every one on a scoped thread of its own,
/// and return the results in the order of the states.
/// Why:
///   The slices of a copy,
/// the workers of a removal and the workspaces of a repair are independent,
/// so they run side by side.
///  A job that panicked is an error in its place;
/// what to make of it is the caller's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const results = await Promise.allSettled(states.map((state) => job(state)));
/// ```
pub fn run_all_side_by_side<S: Send, T: Send>(states: Vec<S>, job: fn(S) -> T) -> Vec<std::thread::Result<T>> {
    return in_thread_scope(AllJobs { states, job });
}

/// The work of `run_beside_caller`:
/// a job for a thread and a function for the calling thread.
struct BesideCaller<S, T, C, U> {
    /// What the thread's job reads.
    thread_state: S,
    /// The job that runs on the thread.
    thread_job: fn(S) -> T,
    /// What the calling thread's function reads.
    caller_state: C,
    /// The function that runs on the calling thread.
    caller_job: fn(C) -> U,
}

impl<'env, S: Send + 'env, T: Send + 'env, C, U> ScopeBody<'env> for BesideCaller<S, T, C, U> {
    type Output = (std::thread::Result<T>, U);

    /// Start the thread's job,
    /// run the caller's function here,
    /// then wait for the thread.
    fn run<'scope>(self, scope: &'scope Scope<'scope, 'env>) -> Self::Output {
        let started: Started<'scope, T> = start_or_run_with(scope, self.thread_state, self.thread_job);
        let here: U = (self.caller_job)(self.caller_state);
        return (started.join(), here);
    }
}

/// What:
///  Run `thread_job(thread_state)` on a scoped thread while `caller_job(caller_state)` runs on the calling thread,
/// and return
///  both results,
/// the thread's first.
/// Why:
///   Two independent reads (a manifest walk of the source beside one of the stage,
/// a Git scan beside the index proof) take as long
///  as the slower one.
/// The caller's function runs after the thread's start,
/// so a thread that cannot start leaves the thread's
///  job to run first,
/// in turn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [fromThread, fromHere] = await Promise.all([threadJob(threadState), callerJob(callerState)]);
/// ```
pub fn run_beside_caller<S: Send, T: Send, C, U>(
    (thread_state, thread_job): (S, fn(S) -> T),
    (caller_state, caller_job): (C, fn(C) -> U),
) -> (std::thread::Result<T>, U) {
    return in_thread_scope(BesideCaller { thread_state, thread_job, caller_state, caller_job });
}

/// Side by side controls stay out of the release executable.
#[cfg(test)]
#[path = "side_by_side_threads_tests.rs"]
mod tests;
