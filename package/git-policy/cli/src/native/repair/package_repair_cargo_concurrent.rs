//! What:
//!  Run one job for every item of a list as tasks of the process's runtime,
//! each item once,
//! and give one result per item in item order.
//! Why:
//!  The repairs of different Cargo workspaces share no state,
//! and each of them waits on a tool child
//!  (`package_repair_command.rs`),
//! which an awaited task may await where a blocked thread cannot:
//! a job of the blocking pool is inside the runtime,
//! so a bridge that starts a child refuses there (`RuntimeError::Nested`).
//! The tasks take items from one queue,
//! so no item waits for another and the results keep item order,
//! and a panic costs only the item it happened in,
//! because each item's work is awaited inside
//!  `call_catching_panics_awaiting`.
//! The runner is a file of its own so that `package_repair_cargo.rs` stays under the
//!  file-size limit and the runner needs nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const results = await mapConcurrent(items, workers, label, async (item) => job(item)); // Array<{ ok: R } | { error: string }>
//! ```

/// The guarded await that turns a panic in one item's work into a result.
use crate::call_catching_panics_awaiting::call_catching_panics_awaiting;
/// The future an item's job hands back.
use std::future::Future;
/// The boxed form a job takes it in.
use std::pin::Pin;
/// Sharing of the items,
/// the label and the queue with the tasks.
use std::sync::Arc;
/// The queue that hands each task the next item.
use std::sync::atomic::{AtomicUsize, Ordering};

/// What:
///  The job a runner does for one item,
///  as work that awaits.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   A job that needs the caller's state (the Cargo program,
/// the request) is a small struct that implements this,
/// and
///  a closure that hands back its own future is a job too,
/// so a test can pass one and shipped code writes no closure for a defect path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ConcurrentAwaitingWork<T, R> { run(item: T): Promise<R>; }
/// ```
pub trait ConcurrentAwaitingWork<T, R> {
    /// Do the job for one item.
    fn run<'a>(&'a self, item: &'a T) -> Pin<Box<dyn Future<Output = R> + Send + 'a>>;
}

/// Every function-like value that hands back its own future is a job:
/// `run` just calls it.
impl<T, R, Job> ConcurrentAwaitingWork<T, R> for Job
where
    Job: for<'a> Fn(&'a T) -> Pin<Box<dyn Future<Output = R> + Send + 'a>>,
{
    fn run<'a>(&'a self, item: &'a T) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> {
        return self(item);
    }
}

/// What:
///  The words that name one item when its work panicked.
/// Why:
///   The error of a panicked item says which item it was,
/// and an item that is a plain path or a workspace names itself
///  through a small struct,
/// while a closure with the right shape is a label too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ItemLabel<T> { label(item: T): string; }
/// ```
pub trait ItemLabel<T> {
    /// Name one item.
    fn label(&self, item: &T) -> String;
}

/// Every function-like value with the right shape is a label:
/// `label` just calls it.
impl<T, Namer: Fn(&T) -> String> ItemLabel<T> for Namer {
    fn label(&self, item: &T) -> String {
        return self(item);
    }
}

/// No answer yet:
/// the value a slot of the report holds until its item ends.
/// Why:
///   A `Vec` of these is made without a closure over the items,
/// and `None` of a result that is not `Clone` cannot be copied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const emptyAnswer = () => undefined;
/// ```
fn empty_answer<R>() -> Option<Result<R, String>> {
    return None;
}

/// What one task of the run reads:
/// the items,
/// the job,
/// the words that name an item whose job panicked,
/// and the queue.
/// Why:
///  The runtime owns the tasks,
/// so what they read travels as shares instead of borrows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheQueue<T, R> = { items: T[]; job: ConcurrentAwaitingWork<T, R>; label: ItemLabel<T>; next: AtomicUsize };
/// ```
struct TheQueue<T, R> {
    /// The items,
    /// in the order their results are reported.
    items: Arc<[T]>,
    /// What one item does.
    job: Arc<dyn ConcurrentAwaitingWork<T, R> + Send + Sync>,
    /// The words that name one item.
    label: Arc<dyn ItemLabel<T> + Send + Sync>,
    /// Where the next item's position is taken.
    next: Arc<AtomicUsize>,
}

/// What:
///  Take items from the queue until none is left and do each inside the guard of its own panic.
/// Why:
///   Each task runs this loop,
/// and the guard is how a panic becomes an error that names the item instead of ending the run,
///  as the scoped runner's `call_catching_panics` did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function takeItems(queue) { const done = []; for (let at = queue.next++; queue.items[at]; at = queue.next++) { done.push([at, await callCatchingPanicsAwaiting(queue.job.run(queue.items[at]))]); } return done; }
/// ```
async fn take_items<T: Send + Sync + 'static, R: Send + 'static>(queue: TheQueue<T, R>) -> Vec<(usize, Result<R, String>)> {
    let mut done: Vec<(usize, Result<R, String>)> = Vec::new();
    loop {
        let at: usize = queue.next.fetch_add(1, Ordering::Relaxed);
        let Some(item) = queue.items.get(at) else { break };
        let caught: Result<R, String> = call_catching_panics_awaiting(queue.job.run(item)).await;
        let answer: Result<R, String> = match caught {
            Ok(value) => Ok(value),
            Err(words) => Err(format!("{} stopped because its worker panicked: {words}", queue.label.label(item))),
        };
        done.push((at, answer));
    }
    return done;
}

/// What:
///  Run `job` for every item as the tasks of the runtime,
/// each item once,
/// one result per item in item order;
/// an item whose work
///  panicked has an error that names it and says what the panic said.
/// Why:
///   The repairs of different workspaces share no state,
/// so a queue of items keeps every task busy,
///  and each item's work is guarded on its own,
/// so a panic costs that item and never the results its task had already produced or the items it had still to take.
/// The items inside their work at once are never more than `workers`,
/// because exactly that many tasks take from the queue.
/// These are tasks and not lanes of `parallel_runner_on_the_runtime_awaiting.rs`:
/// that runner takes its consumer as a
///  borrow that is not `Send`,
/// so its run cannot be awaited inside a task of another run,
/// and the Cargo repair is one wait of the
///  worktree repair's own task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const results = await mapConcurrentAwaiting(items, workers, label, job); // Array<{ ok: R } | { error: string }>
/// ```
pub async fn map_concurrent_awaiting<T: Send + Sync + 'static, R: Send + 'static>(
    items: Arc<[T]>,
    workers: usize,
    label: Arc<dyn ItemLabel<T> + Send + Sync>,
    job: Arc<dyn ConcurrentAwaitingWork<T, R> + Send + Sync>,
) -> Vec<Result<R, String>> {
    let count: usize = items.len();
    let at_once: usize = workers.max(1).min(count.max(1));
    let next: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let mut started: tokio::task::JoinSet<Vec<(usize, Result<R, String>)>> = tokio::task::JoinSet::new();
    for _task in 0..at_once {
        let queue: TheQueue<T, R> =
            TheQueue { items: Arc::clone(&items), job: Arc::clone(&job), label: Arc::clone(&label), next: Arc::clone(&next) };
        started.spawn(take_items(queue));
    }
    let mut answers: Vec<Option<Result<R, String>>> = std::iter::repeat_with(empty_answer).take(count).collect();
    while let Some(joined) = started.join_next().await {
        match joined {
            Ok(done) => {
                for (at, answer) in done {
                    answers[at] = Some(answer);
                }
            }
            // A task that panicked outside the guard of an item took no further item, which the answers below say.
            Err(_panicked) => {}
        }
    }
    let mut ordered: Vec<Result<R, String>> = Vec::new();
    for (at, answer) in answers.into_iter().enumerate() {
        ordered.push(match answer {
            Some(given) => given,
            None => Err(format!("{} gave no result", label.label(&items[at]))),
        });
    }
    return ordered;
}

/// Runner controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_cargo_concurrent_tests.rs"]
mod tests;
