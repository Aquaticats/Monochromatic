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
//! because both the build of an item's future and its awaited work are guarded.
//! Each answer leaves its task as that item ends,
//! so a task that stopped unexpectedly still leaves every finished item's result behind it.
//! The runner is a file of its own so that `package_repair_cargo.rs` stays under the
//!  file-size limit and the runner needs nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const results = await mapConcurrent(items, workers, label, async (item) => job(item)); // Array<{ ok: R } | { error: string }>
//! ```

/// The guarded await that turns a panic in one item's work into a result,
/// and the words a panic said.
use crate::call_catching_panics_awaiting::{call_catching_panics_awaiting, panic_words};
/// The guarded call that turns a panic of a plain call into a value.
use crate::panic_notice::while_panics_are_expected;
/// The future an item's job hands back.
use std::future::Future;
/// The boxed form a job takes it in.
use std::pin::Pin;
/// The line an unexpected stop is recorded on.
use crate::diagnostic_log::debug;
/// Sharing of the items,
/// the label and the queue with the tasks.
use std::sync::Arc;
/// The queue that hands each task the next item.
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairCargoConcurrent";

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

/// Where an item's answer goes as soon as that item ends.
type Answers<R> = tokio::sync::mpsc::UnboundedSender<(usize, Result<R, String>)>;

/// What an item's job is for one item:
/// the future it awaits.
type AnItemFuture<'of, R> = Pin<Box<dyn Future<Output = R> + Send + 'of>>;

/// What:
///  The future one item's job handed back,
/// as the guarded call takes it.
/// Why:
///  `run` is called before the guard that awaits its future exists,
/// and this gives that call the state and the function the guard needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const buildTheItemsWork = ([queue, item]: [TheQueue, unknown]) => queue.job.run(item);
/// ```
fn build_the_items_work<'of, T, R>(state: (&'of TheQueue<T, R>, &'of T)) -> AnItemFuture<'of, R> {
    return state.0.job.run(state.1);
}

/// What:
///  Take items from the queue until none is left and do each inside the guard of its own panic,
///  handing each answer over as that item ends.
/// Why:
///   Each task runs this loop,
/// and the guard is how a panic becomes an error that names the item instead of ending the run,
///  as the scoped runner's `call_catching_panics` did.
/// The build of an item's future is guarded too:
/// it runs before the await can catch anything,
/// and a panic there ended the task,
/// taking the answers it had finished and every item it had not taken.
/// Each answer travels as it ends rather than with this task's return value,
/// so a task that stopped still leaves every finished item's result behind it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function takeItems(queue, report) { for (let at = queue.next++; queue.items[at]; at = queue.next++) { let answer; try { answer = ok(await queue.job.run(queue.items[at])); } catch (words) { answer = err(words); } report.send([at, answer]); } }
/// ```
async fn take_items<T: Send + Sync + 'static, R: Send + 'static>(queue: TheQueue<T, R>, report: Answers<R>) {
    loop {
        let at: usize = queue.next.fetch_add(1, Ordering::Relaxed);
        let Some(item) = queue.items.get(at) else { break };
        let caught: Result<R, String> = match while_panics_are_expected((&queue, item), build_the_items_work::<T, R>) {
            Ok(work) => call_catching_panics_awaiting(work).await,
            // The job panicked while it built the item's future, so no future of that item ever existed.
            Err(payload) => Err(panic_words(payload.as_ref())),
        };
        let answer: Result<R, String> = match caught {
            Ok(value) => Ok(value),
            Err(words) => Err(format!("{} stopped because its worker panicked: {words}", queue.label.label(item))),
        };
        // A closed channel is the run ending, and nothing is left that could read this answer.
        if report.send((at, answer)).is_err() {
            break;
        }
    }
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
///  and each item's work is guarded on its own:
/// the build of the item's future and every poll of it,
/// so a panic costs that item and never the results its task had already produced or the items it had still to take.
/// Each answer is handed over as the item ends,
/// so nothing a task has finished is kept in a value a panic could drop.
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
    let (report, mut reported): (Answers<R>, tokio::sync::mpsc::UnboundedReceiver<(usize, Result<R, String>)>) = tokio::sync::mpsc::unbounded_channel();
    let mut started: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();
    for _task in 0..at_once {
        let queue: TheQueue<T, R> =
            TheQueue { items: Arc::clone(&items), job: Arc::clone(&job), label: Arc::clone(&label), next: Arc::clone(&next) };
        started.spawn(take_items(queue, report.clone()));
    }
    // The answers come from the tasks, so this channel is the run's own; every task holds a copy and drops it as it ends.
    drop(report);
    let mut answers: Vec<Option<Result<R, String>>> = std::iter::repeat_with(empty_answer).take(count).collect();
    while let Some((at, answer)) = reported.recv().await {
        answers[at] = Some(answer);
    }
    // Every task has ended by now, so this drains at once; a task that panicked outside an item's guard took no further
    //  item and the answers below say which ones it left.
    while let Some(joined) = started.join_next().await {
        if let Err(panicked) = joined {
            let said: String = match panicked.is_panic() {
                true => panic_words(panicked.into_panic().as_ref()),
                false => String::from("the task was cancelled"),
            };
            debug(TAG, format!("a worker of one job per item stopped: {said}").as_str());
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
