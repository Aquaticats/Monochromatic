//! What:
//!  Run one job for every item of a list on several threads, each item once, and give one result per item in item order.
//! Why:
//!  The repairs of different Cargo workspaces share no state, so a queue of items keeps every thread busy; a panic costs
//!  only the item it happened in. The runner is a file of its own so that `package_repair_cargo.rs` stays under the
//!  file-size limit and the runner needs nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const results = await mapConcurrent(items, workers, label, job); // Array<{ ok: R } | { error: string }>
//! ```

/// The guarded call that turns a panic in one item's job into a result.
use crate::call_catching_panics::call_catching_panics;
/// Threads that borrow the caller's items for the length of one scope, or run in turn when none can start.
use crate::side_by_side_threads::run_all_side_by_side;
/// The shared counter that hands each worker the next item.
use std::sync::atomic::{AtomicUsize, Ordering};

/// What:
///  The job a runner does for one item.
///  A `trait` is a named set of methods that different types can each supply, like a TypeScript `interface`.
/// Why:
///   A job that needs the caller's state (the Cargo program, the request) is a small struct that implements this, and
///  a closure with the right shape is a job too, so a test can pass one and shipped code writes none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ConcurrentWork<T, R> { run(item: T): R; }
/// ```
pub trait ConcurrentWork<T, R> {
    /// Do the job for one item.
    fn run(&self, item: &T) -> R;
}

/// Every function-like value with the right shape is a job: `run` just calls it.
impl<T, R, Job: Fn(&T) -> R> ConcurrentWork<T, R> for Job {
    fn run(&self, item: &T) -> R {
        return self(item);
    }
}

/// What:
///  The words that name one item when its worker panicked.
/// Why:
///   The error of a panicked item says which item it was, and an item that is a plain path or a workspace names itself
///  through a small struct, while a closure with the right shape is a label too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ItemLabel<T> { label(item: T): string; }
/// ```
pub trait ItemLabel<T> {
    /// Name one item.
    fn label(&self, item: &T) -> String;
}

/// Every function-like value with the right shape is a label: `label` just calls it.
impl<T, Namer: Fn(&T) -> String> ItemLabel<T> for Namer {
    fn label(&self, item: &T) -> String {
        return self(item);
    }
}

/// What one worker reads: the items, the counter that hands out the next one, the namer and the job.
type WorkerState<'a, T, R> = (&'a [T], &'a AtomicUsize, &'a (dyn ItemLabel<T> + Sync), &'a (dyn ConcurrentWork<T, R> + Sync));

/// The job for one item, as a function of its two inputs so that the guarded call can name it.
fn run_job_for_item<T, R>((job, item): (&(dyn ConcurrentWork<T, R> + Sync), &T)) -> R {
    return job.run(item);
}

/// What:
///  Take items from the shared counter until none is left and run the job for each inside a catch, so a panic costs
///  the item it happened in and never the results already produced or the items still to take.
/// Why:
///   Each worker thread runs this loop, and the catch is how a panic becomes an error that names the item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let index = next++; items[index] !== undefined; index = next++) { try { done.push([index, job(items[index])]); } catch (panic) { done.push([index, error]); } }
/// ```
fn take_items<T, R>((items, next, label, job): WorkerState<'_, T, R>) -> Vec<(usize, Result<R, String>)> {
    let mut done: Vec<(usize, Result<R, String>)> = Vec::new();
    loop {
        let index: usize = next.fetch_add(1, Ordering::Relaxed);
        let Some(item) = items.get(index) else { break };
        let caught: std::thread::Result<R> = call_catching_panics((job, item), run_job_for_item::<T, R>);
        let outcome: Result<R, String> = match caught {
            Ok(value) => Ok(value),
            Err(payload) => Err(format!("{} stopped because its worker panicked: {}", label.label(item), panic_words(payload.as_ref()))),
        };
        done.push((index, outcome));
    }
    return done;
}

/// The position of an item in a numbered result, which orders the results back into item order.
fn result_index<R>(numbered: &(usize, Result<R, String>)) -> usize {
    return numbered.0;
}

/// What:
///  Run `job` for every item on up to `workers` threads, each item once, one result per item in item order; an item whose job
///  panicked has an error that names it and says what the panic said.
/// Why:
///   The repairs of different workspaces share no state, so a queue of items keeps every thread busy. Each item is run inside its
///  own catch, so a panic costs that item and never the results its thread had already produced or the items it had still to take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const results = await mapConcurrent(items, workers, label, job); // Array<{ ok: R } | { error: string }>
/// ```
pub fn map_concurrent<T: Sync, R: Send>(
    items: &[T],
    workers: usize,
    label: &(dyn ItemLabel<T> + Sync),
    job: &(dyn ConcurrentWork<T, R> + Sync),
) -> Vec<Result<R, String>> {
    let next: AtomicUsize = AtomicUsize::new(0);
    let mut worker_states: Vec<WorkerState<'_, T, R>> = Vec::new();
    for _worker in 0..workers.max(1).min(items.len().max(1)) {
        worker_states.push((items, &next, label, job));
    }
    // A worker cannot panic outside the catch around each item, so a thread that did not return its list is a bug to raise.
    let mut results: Vec<(usize, Result<R, String>)> = Vec::new();
    for joined in run_all_side_by_side(worker_states, take_items::<T, R>) {
        match joined {
            Ok(done) => results.extend(done),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
    results.sort_by_key(result_index);
    let mut ordered: Vec<Result<R, String>> = Vec::new();
    for (_index, result) in results {
        ordered.push(result);
    }
    return ordered;
}

/// What a panic said: its text when it carried a string, else a plain statement that it carried none.
fn panic_words(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    return String::from("no message");
}
