//! What:
//!  Runs independent items as tasks of the process's runtime,
//! one lane per task,
//! and reports what they say and what they return in input order,
//! as if they had run one after another.
//! Why:
//!  Some item's work waits on Git,
//! and a Git step must be awaited on the runtime rather than blocked on
//! (`transaction/transaction_git_awaiting.rs`):
//!  a job of the blocking pool cannot await anything,
//! and the synchronous faces that enter the runtime refuse a call
//!  from inside it (`lazy_runtime::run_on_runtime`,
//! `RuntimeError::Nested`),
//! so the round of the landing
//!  (`landing/service_round.rs`) cannot use the blocking form of this runner.
//! An item that also waits on storage (a file copy,
//! a directory walk) puts that
//!  one call on the blocking pool inside its own work
//! (`run_blocking_job_within_the_budget`),
//! so the process's one bound still covers it and the worker
//!  that awaits Git is free.
//! The lanes,
//! the input order,
//! the live lines of the head item and the ending of the consumer are the blocking
//!  form's (`parallel_runner_on_the_runtime.rs`,
//! `finish_the_run`),
//! so the two forms cannot report differently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runOrderedAwaitingOnTheRuntime({ items, lanes, workers, work, onEvent });
//! ```

use crate::blocking_job::BlockingJobError;
use crate::parallel_runner::{Message, OrderedEvents, refuse_lanes_that_do_not_cover_every_item};
use crate::parallel_runner_on_the_runtime::{CountedOut, LinesToTheConsumer, OrderedRunError, SlotsStillRunning, finish_the_run};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

/// What:
///  What one item of an awaited ordered run does:
/// the item's work as a future,
/// so it may await Git and anything else of the runtime.
/// Why:
///   The blocking form's `OrderedWork` returns its result at once,
/// which a step that awaits cannot;
///  this is the same shape with the result handed back through a boxed future,
/// because a trait with an `async` method
///  cannot be made into a `dyn` value behind an `Arc`.
/// The lines an item says are handed to it as a value it owns
///  (`LinesToTheConsumer`),
/// not as a borrow,
/// so the item may move them into a job of the blocking pool,
/// which must be `'static`,
///  and so say them while that job works.
/// Every line an item says is said before the future it returns ends:
/// the run reports an item's end as soon as that future is over,
/// and a line that arrives for an item already ended is held and never shown.
/// A sink moved into a detached task would therefore lose its lines;
/// the job of the pool is the place it belongs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface OrderedAwaitingWork<T, R> { run(index: number, item: T, say: LineSink): Promise<R> }
/// ```
pub trait OrderedAwaitingWork<T, R>: Send + Sync {
    /// Do the item at `index`,
    /// saying its lines through `say`,
    /// and return its result when its waits are over.
    fn run<'a>(&'a self, index: usize, item: &'a T, say: LinesToTheConsumer<R>) -> Pin<Box<dyn Future<Output = R> + Send + 'a>>;
}

/// What:
///  Everything one awaited ordered run needs.
/// Why:
///   The items,
/// the lanes and the work outlive the calling stack that starts the lanes,
/// so the caller shares them instead of
///  lending them;
/// the consumer stays on the calling task and is lent for the length of the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OrderedAwaitingRunOfOwned<T, R> = { items: T[]; lanes: number[][]; workers: number; work: OrderedAwaitingWork<T, R>; onEvent: OrderedEvents<R> };
/// ```
pub struct OrderedRunOfAwaitingOwned<'run, T, R, E: OrderedEvents<R> + ?Sized> {
    /// The items,
    /// in the order their output is reported.
    pub items: Arc<[T]>,
    /// The indices of the items,
    /// grouped:
    /// the items of a lane run one after another in one task,
    /// lanes run side by side.
    ///  Every index appears in exactly one lane.
    pub lanes: Arc<[Vec<usize>]>,
    /// How many lanes run at once,
    /// at least one,
    /// and never more than there are lanes.
    pub workers: usize,
    /// What one item does,
    /// given its index,
    /// itself and a sink that says one line.
    pub work: Arc<dyn OrderedAwaitingWork<T, R>>,
    /// The consumer,
    /// called on the calling task only.
    /// Named as the caller's own type rather than as a `dyn` value,
    /// so a run whose work and consumer are `Send` is a future a task may await:
    /// a consumer that lends a sink which is not `Send` stays what it was.
    pub on_event: &'run mut E,
}

/// What one lane reads:
/// the items,
/// the lanes,
/// the counter that hands out lanes,
/// the work and the sender of its results.
struct Lane<T, R> {
    /// The items,
    /// in the order their output is reported.
    items: Arc<[T]>,
    /// The lanes the tasks take.
    lanes: Arc<[Vec<usize>]>,
    /// The counter that hands out the lanes.
    next_lane: Arc<AtomicUsize>,
    /// What one item does.
    work: Arc<dyn OrderedAwaitingWork<T, R>>,
    /// Where the lines and the results go.
    sender: UnboundedSender<Message<R>>,
}

/// What:
///  Take lanes until none is left and do every item of each lane in turn,
/// handing its lines and its result to the consumer.
/// Why:
///   A lane's items must not overlap (they share a worktree or a repository),
/// so one task takes a whole lane and awaits its items one after
///  another;
/// lanes run side by side,
/// which is what `workers` bounds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let lane = nextLane(); lane; lane = nextLane()) for (const index of lane) sender.send(Done(index, await work(index, items[index], say)));
/// ```
async fn run_a_lane<T: Send + Sync + 'static, R: Send + 'static>(lane: Lane<T, R>) -> Result<(), BlockingJobError> {
    loop {
        let lane_index: usize = lane.next_lane.fetch_add(1, Ordering::Relaxed);
        let Some(indices) = lane.lanes.get(lane_index) else { return Ok(()) };
        for &index in indices {
            let Some(item) = lane.items.get(index) else { continue };
            // The sink is a value the item owns, so a `'static` job of the pool can carry it.
            let say: LinesToTheConsumer<R> = LinesToTheConsumer::new(&lane.sender, index);
            let result: R = lane.work.run(index, item, say).await;
            let _sent = lane.sender.send(Message::Done(index, result));
        }
    }
}

/// What:
///  `run_a_lane` with its end counted,
/// so the consumer stops once every lane of the run has returned.
/// Why:
///  Counting the lanes,
/// and not the open sending ends,
/// is what ends the consumer:
/// a task that ended with its sending end still held would otherwise keep it open,
///  and the consumer would wait for it for good
/// (`blocking_jobs_within_the_budget.rs` says how one such case is measured).
///
async fn run_counted_lane<T: Send + Sync + 'static, R: Send + 'static>(lane: Lane<T, R>, counted: Arc<SlotsStillRunning>) -> Result<(), BlockingJobError> {
    let _counted: CountedOut = CountedOut(counted);
    return run_a_lane(lane).await;
}

/// What:
///  Run every lane as a task of the runtime and report every item in input order.
/// Why:
///   See the module description.
/// A panicking task ends the run with an error,
/// naming what was not run;
/// the items already reported
///  stay reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runOrderedAwaitingOnTheRuntime({ items, lanes, workers, work, onEvent });
/// ```
///
/// # Panics
/// When a lane names an index that is not an item,
/// or the lanes do not cover every item exactly once.
pub async fn run_ordered_awaiting_on_the_runtime<T, R, E>(run: OrderedRunOfAwaitingOwned<'_, T, R, E>) -> Result<(), OrderedRunError>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
    E: OrderedEvents<R> + ?Sized,
{
    let OrderedRunOfAwaitingOwned { items, lanes, workers, work, on_event } = run;
    refuse_lanes_that_do_not_cover_every_item(items.len(), lanes.as_ref());
    let next_lane: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = unbounded_channel::<Message<R>>();
    let at_once: usize = workers.max(1).min(lanes.len().max(1));
    let still_running: Arc<SlotsStillRunning> = SlotsStillRunning::counting(at_once);
    let mut slots: tokio::task::JoinSet<Result<(), BlockingJobError>> = tokio::task::JoinSet::new();
    for _slot in 0..at_once {
        let lane: Lane<T, R> = Lane {
            items: Arc::clone(&items),
            lanes: Arc::clone(&lanes),
            next_lane: Arc::clone(&next_lane),
            work: Arc::clone(&work),
            sender: sender.clone(),
        };
        slots.spawn(run_counted_lane(lane, Arc::clone(&still_running)));
    }
    // The consumer's own sending end goes, so a run every lane returned from closes its channel.
    drop(sender);
    return finish_the_run(receiver, &still_running, items.len(), on_event, &mut slots).await;
}

/// Controls for the awaited form stay out of the release executable.
#[cfg(test)]
#[path = "parallel_runner_on_the_runtime_awaiting_tests.rs"]
mod tests;
