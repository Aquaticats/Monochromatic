//! What:
//!  Runs independent items as blocking jobs of the process's runtime,
//! under the process's bound on blocking jobs,
//!  and reports what they say and what they return in input order,
//! as if they had run one after another.
//! Why:
//!  The items wait on storage (file copies,
//! directory walks,
//! Git children),
//! so they belong on the blocking pool and never on a worker that polls
//!  tasks (`doc/planning/git-policy-cli-full-tokio.md`,
//! "Blocking files,
//! fsync and the runtime's own shutdown").
//! A blocking job of the pool must own what it reads,
//!  which the scoped threads of `parallel_runner.rs` need not:
//! this form therefore takes the items,
//! the lanes and the work as
//!  values the caller shares,
//! and the calling task stays free while the jobs run.
//! The reporting (the input order,
//! the live
//!  lines of the head item) is the one of `parallel_runner.rs`,
//! so the two runners cannot report differently.
//! The jobs inside their work at once are never more than the workers,
//! the lanes and
//!  the process's bound on blocking jobs allow,
//! whichever
//!  is the fewest:
//! the bound is a ceiling every job of the process shares,
//!  never a reservation one run keeps for itself.
//! That bound comes from the CPUs this process may use
//! (`available_parallelism`,
//!  so a container's quota counts),
//! the limits on threads and address space,
//! and any cap the process asked for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runOrderedOnTheRuntime({ items, lanes, workers, work, onEvent });
//! ```

use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::lazy_runtime::{RuntimeError, run_on_runtime};
use crate::line_sink::LineSink;
use crate::parallel_runner::{Cursor, Message, OrderedEvents, OrderedWork, refuse_lanes_that_do_not_cover_every_item};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// What:
///  Everything one ordered run on the runtime needs.
/// Why:
///   The items,
/// the lanes and the work outlive the calling stack that starts the jobs,
/// so the caller shares them instead of lending
///  them;
/// the consumer stays on the calling task and is lent for the length of the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OrderedRunOfOwned<T, R> = { items: T[]; lanes: number[][]; workers: number; work: OrderedWork<T, R>; onEvent: OrderedEvents<R> };
/// ```
pub struct OrderedRunOfOwned<'run, T, R> {
    /// The items,
    /// in the order their output is reported.
    pub items: Arc<[T]>,
    /// The indices of the items,
    /// grouped:
    /// the items of a lane run one after another,
    /// lanes run side by side.
    ///  Every index appears in exactly one lane.
    pub lanes: Arc<[Vec<usize>]>,
    /// How many jobs the run is asked to keep in flight at once,
    /// at least one,
    /// and never more than there are lanes.
    /// The jobs really inside their work at once are never more than the fewest of this,
    /// the lanes and the process's bound on blocking jobs
    /// (`blocking_jobs_within_the_budget.rs`),
    /// because every job takes a place of that bound before it starts.
    pub workers: usize,
    /// What one item does,
    /// given its index,
    /// itself and a sink that says one line.
    pub work: Arc<dyn OrderedWork<T, R> + Send + Sync>,
    /// The consumer,
    /// called on the calling task only.
    pub on_event: &'run mut dyn OrderedEvents<R>,
}

/// Why an ordered run on the runtime did not finish every item.
#[derive(Debug)]
pub enum OrderedRunError {
    /// The system refused a thread for a job,
    ///  so the items after it in its lane were not run,
    /// and the item it held will not run either.
    ///  This is what a job that takes the claim on the item's work before
    /// it answers can say (`blocking_jobs_within_the_budget.rs`).
    NoThreadToStart,
    /// The system refused a thread for a job,
    ///  and the pool then ran it on another thread,
    /// so the items after it in its lane were not run,
    ///  and whether the item it held finished is not known from here.
    StartedAfterItsThreadWasRefused,
    /// The system refused a thread for a job,
    ///  and the pool had already queued it,
    /// so it may still run later;
    /// the items after it in its lane were not run,
    ///  and whether it ran is not known from here.
    MayStillRunAfterItsThreadWasRefused,
    /// A job panicked,
    /// so the items after it in its lane were not run.
    Panicked,
    /// The runtime ended before the run did.
    Cancelled,
    /// There was no runtime to run the jobs on,
    /// or the call came from inside it.
    NoRuntime(RuntimeError),
}

impl std::fmt::Display for OrderedRunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            OrderedRunError::NoThreadToStart => write!(
                formatter,
                "the system refused a thread for a step of this run, so the items after it in its lane were not run and the \
                 item it held will not run. Hint: check the process limit (`ulimit -u`) and the control group's `pids.max`, \
                 free some, and run the command again"
            ),
            OrderedRunError::StartedAfterItsThreadWasRefused => write!(
                formatter,
                "the system refused a thread for a step of this run, and the pool then ran that step on another thread: the \
                 items after it in its lane were not run, and whether the item it held finished is not known from here. Hint: \
                 check the process limit (`ulimit -u`) and the control group's `pids.max`, and check what that worktree looks \
                 like before running the command again"
            ),
            OrderedRunError::MayStillRunAfterItsThreadWasRefused => write!(
                formatter,
                "the system refused a thread for a step of this run, and the pool had already queued that step before it asked for one, \
                 so the step may still run later; the items after it in its lane were not run, and whether it ran is not known from \
                 here. Hint: check the process limit (`ulimit -u`) and the control group's `pids.max`, and check what that worktree \
                 looks like before running the command again"
            ),
            OrderedRunError::Panicked => write!(formatter, "a step of this run panicked, so the items after it in its lane were not run"),
            OrderedRunError::Cancelled => write!(formatter, "a step of this run was cancelled because the runtime was ending"),
            OrderedRunError::NoRuntime(reason) => write!(formatter, "{reason}"),
        };
    }
}

impl From<BlockingJobError> for OrderedRunError {
    fn from(error: BlockingJobError) -> Self {
        return match error {
            BlockingJobError::NoThreadToStart => OrderedRunError::NoThreadToStart,
            BlockingJobError::MayStillRunAfterItsThreadWasRefused => OrderedRunError::MayStillRunAfterItsThreadWasRefused,
            BlockingJobError::StartedAfterItsThreadWasRefused => OrderedRunError::StartedAfterItsThreadWasRefused,
            BlockingJobError::Panicked => OrderedRunError::Panicked,
            BlockingJobError::Cancelled => OrderedRunError::Cancelled,
        };
    }
}

/// One item,
/// with what its job owns:
/// the items,
/// the work,
/// where it says its lines,
/// and which item it is.
struct OneItem<T, R> {
    /// The items the job reads its own out of.
    items: Arc<[T]>,
    /// What one item does.
    work: Arc<dyn OrderedWork<T, R> + Send + Sync>,
    /// Where its lines and its result go.
    sender: UnboundedSender<Message<R>>,
    /// Which item this job is.
    index: usize,
}

impl<T: Send + Sync + 'static, R: Send + 'static> OneItem<T, R> {
    /// Do the item and hand its result to the consumer.
    fn run(self) {
        let OneItem { items, work, sender, index } = self;
        let Some(item) = items.get(index) else { return };
        let mut say: LinesToTheConsumer<R> = LinesToTheConsumer::new(&sender, index);
        let result: R = work.run(index, item, &mut say);
        let _sent = sender.send(Message::Done(index, result));
    }
}

/// What:
///  The lines of one item,
/// sent to the consumer of the calling task with the item's index,
/// as a value the item owns.
/// Why:
///   A job of the blocking pool must own what it reads
/// (`run_blocking_job_within_the_budget`),
/// and the items of an awaited ordered run are just such jobs,
///  so an item that says its lines while it works
/// (`parallel_runner_on_the_runtime_awaiting.rs`)
///  carries a clone of the one sending end the run already uses
/// instead of a borrow of it.
/// No lock is taken:
/// the channel is the whole of the sharing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LinesToTheConsumer { constructor(readonly sender: Sender, readonly index: number) {} say(text: string) { this.sender.send({ line: [this.index, text] }) } }
/// ```
pub struct LinesToTheConsumer<R> {
    /// Where the lines go.
    sender: UnboundedSender<Message<R>>,
    /// The item that says them.
    index: usize,
}

impl<R> LinesToTheConsumer<R> {
    /// The lines of the item at `index` of the run whose consumer reads `sender`.
    pub(crate) fn new(sender: &UnboundedSender<Message<R>>, index: usize) -> Self {
        return LinesToTheConsumer { sender: sender.clone(), index };
    }
}

impl<R> LineSink for LinesToTheConsumer<R> {
    fn say(&mut self, text: &str) {
        // The consumer is gone only when the calling task was ended, and that end is the answer of the run.
        let _sent = self.sender.send(Message::Line(self.index, text.to_string()));
    }
}

/// What one job of the pool reads:
/// the items,
/// the lanes,
/// the counter that hands out lanes,
/// the work and the sender of its results.
struct Slot<T, R> {
    /// The items,
    /// in the order their output is reported.
    items: Arc<[T]>,
    /// The lanes the jobs take.
    lanes: Arc<[Vec<usize>]>,
    /// The counter that hands out the lanes.
    next_lane: Arc<AtomicUsize>,
    /// What one item does.
    work: Arc<dyn OrderedWork<T, R> + Send + Sync>,
    /// Where the lines and the results go.
    sender: UnboundedSender<Message<R>>,
}

/// What:
///  Take lanes until none is left,
/// run every item of each lane as one job of the blocking pool,
/// and hand its lines and result to the consumer.
/// Why:
///   One place is taken per item and given back when that item ends,
/// so a lane of many items never holds places it is not using,
/// and the
///  work of this run cannot take the places another part of the process needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let lane = nextLane(); lane; lane = nextLane()) for (const index of lane) await runBlockingJobWithinTheBudget(() => work(index, items[index], say));
/// ```
async fn run_slot<T, R>(slot: Slot<T, R>) -> Result<(), BlockingJobError>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    loop {
        let lane_index: usize = slot.next_lane.fetch_add(1, Ordering::Relaxed);
        let Some(lane) = slot.lanes.get(lane_index) else { return Ok(()) };
        for &index in lane {
            let one: OneItem<T, R> = OneItem {
                items: Arc::clone(&slot.items),
                work: Arc::clone(&slot.work),
                sender: slot.sender.clone(),
                index,
            };
            run_blocking_job_within_the_budget(move || { one.run(); }).await?;
        }
    }
}

/// How many jobs of one run are still running,
/// and a wake for the consumer when their number falls.
pub(crate) struct SlotsStillRunning {
    /// The jobs that have not returned.
    pub(crate) running: AtomicUsize,
    /// Woken whenever one returns.
    pub(crate) changed: tokio::sync::Notify,
}

impl SlotsStillRunning {
    /// A count of `count` jobs that have not returned.
    pub(crate) fn counting(count: usize) -> Arc<Self> {
        return Arc::new(SlotsStillRunning { running: AtomicUsize::new(count), changed: tokio::sync::Notify::new() });
    }
}

/// What:
///  Counts one job out when it returns,
/// whether it returned or panicked.
/// Why:
///  The consumer reads until every job of the run has returned,
/// and a job that panicked must end that wait too.
pub(crate) struct CountedOut(pub(crate) Arc<SlotsStillRunning>);

impl Drop for CountedOut {
    fn drop(&mut self) {
        let _left: usize = self.0.running.fetch_sub(1, Ordering::SeqCst);
        self.0.changed.notify_one();
    }
}

/// What:
///  Runs `slot` with its end counted,
/// so the consumer stops once every job of the run has returned.
/// Why:
///  Counting the jobs,
/// and not the open sending ends,
/// is what ends the consumer:
/// a job the system refused a thread
///  for was swallowed by the pool's queue with everything it held,
/// a sending end included (`blocking_jobs_within_the_budget.rs`
/// says how that is
///  measured),
/// so a channel that waits for its sending ends would never close.
///
async fn run_counted<T, R>(slot: Slot<T, R>, counted: Arc<SlotsStillRunning>) -> Result<(), BlockingJobError>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    let _counted: CountedOut = CountedOut(counted);
    return run_slot(slot).await;
}

/// What:
///  Run every item as a job of the blocking pool and report in input order.
/// Why:
///   See the module description.
/// A refused thread or a panicking job ends the run with an error,
/// naming what was not run;
/// the
///  items already reported stay reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runOrderedOnTheRuntime({ items, lanes, workers, work, onEvent });
/// ```
///
/// # Panics
/// When a lane names an index that is not an item,
/// or the lanes do not cover every item exactly once.
pub async fn run_ordered_on_the_runtime<T, R>(run: OrderedRunOfOwned<'_, T, R>) -> Result<(), OrderedRunError>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    let OrderedRunOfOwned { items, lanes, workers, work, on_event } = run;
    refuse_lanes_that_do_not_cover_every_item(items.len(), lanes.as_ref());
    let next_lane: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = unbounded_channel::<Message<R>>();
    let at_once: usize = workers.max(1).min(lanes.len().max(1));
    let still_running: Arc<SlotsStillRunning> = SlotsStillRunning::counting(at_once);
    let mut slots: tokio::task::JoinSet<Result<(), BlockingJobError>> = tokio::task::JoinSet::new();
    for _slot in 0..at_once {
        let slot: Slot<T, R> = Slot {
            items: Arc::clone(&items),
            lanes: Arc::clone(&lanes),
            next_lane: Arc::clone(&next_lane),
            work: Arc::clone(&work),
            sender: sender.clone(),
        };
        slots.spawn(run_counted(slot, Arc::clone(&still_running)));
    }
    // The consumer's own sending end goes, so a run every job returned from closes its channel.
    drop(sender);
    return finish_the_run(receiver, &still_running, items.len(), on_event, &mut slots).await;
}

/// What:
///  Reports every line and result the jobs of one run sent,
/// in input order,
/// ends when every job has returned,
///  and gives the first failure of the run.
/// Why:
///   Two forms run lanes of items (`run_ordered_on_the_runtime` as blocking jobs,
/// `run_ordered_awaiting_on_the_runtime`
///  as tasks),
/// and their reporting must be one code:
/// the consumer reads until every job of the run has *returned*
///  (a job the system refused a thread for keeps its sending end for the life of the process,
/// so a channel that waits for its
///  sending ends would never close),
/// then drains what the jobs already sent,
/// and only then joins them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (stillRunning()) { await either(message(), oneReturned()); } drain(); return firstFailure(joinAll());
/// ```
pub(crate) async fn finish_the_run<R, E>(
    mut receiver: UnboundedReceiver<Message<R>>,
    still_running: &SlotsStillRunning,
    item_count: usize,
    on_event: &mut E,
    slots: &mut tokio::task::JoinSet<Result<(), BlockingJobError>>,
) -> Result<(), OrderedRunError>
where
    E: OrderedEvents<R> + ?Sized,
{
    let mut cursor: Cursor<R> = Cursor::new(item_count);
    // The first item begins before any job says anything, as in the scoped form.
    cursor.advance(on_event);
    loop {
        if still_running.running.load(Ordering::SeqCst) == 0 {
            break;
        }
        if receiver.is_closed() {
            // No job can say anything more, so the count is the only thing left to wait for.
            still_running.changed.notified().await;
            continue;
        }
        if let Some(said) = wait_for_a_message_or_a_return(&mut receiver, still_running).await {
            cursor.tell(said, on_event);
        }
    }
    // A job that returned said everything it had to say before it returned.
    while let Ok(message) = receiver.try_recv() {
        cursor.tell(message, on_event);
    }
    let mut failure: Option<OrderedRunError> = None;
    while let Some(joined) = slots.join_next().await {
        match joined {
            Ok(Ok(())) => {}
            Ok(Err(refused)) => {
                failure.get_or_insert(OrderedRunError::from(refused));
            }
            Err(join_error) => {
                failure.get_or_insert(if join_error.is_panic() { OrderedRunError::Panicked } else { OrderedRunError::Cancelled });
            }
        }
    }
    return match failure {
        Some(error) => Err(error),
        None => Ok(()),
    };
}

/// What:
///  Waits for one message from the jobs of one run,
///  or for one job of it to return.
/// Why:
///  The consumer of a run has nothing else to do,
/// and the count of the jobs still running is the only thing that ends it
/// (`finish_the_run`),
///  so this is the one place where that wait is written.
/// A wake this call has already missed is not lost:
/// `Notify::notify_one` stores one permit while no task waits
/// (`tokio-1.54.1/src/sync/notify.rs`),
///  and this branch must therefore never be switched off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = await Promise.race([receiver.recv(), oneReturned()]); // undefined: a job returned
/// ```
pub(crate) async fn wait_for_a_message_or_a_return<R>(
    receiver: &mut UnboundedReceiver<Message<R>>,
    still_running: &SlotsStillRunning,
) -> Option<Message<R>> {
    // The wake branch is never switched off: a wake stored while nothing waited is consumed here, and a branch
    //  disabled by the count cannot consume it (`tokio-1.54.1/src/sync/notify.rs`: at most one permit is stored).
    return tokio::select! {
        arrived = receiver.recv() => arrived,
        () = still_running.changed.notified() => None,
    };
}

/// What:
///  `run_ordered_on_the_runtime` for a caller that is not asynchronous:
/// it runs the jobs on the process's runtime and waits here for them,
/// saying the consumer's events on this
///  thread.
/// Why:
///   The callers of the runner today are the commands of the wrapper and the work loop of the service,
/// and each becomes
///  asynchronous on its own (`doc/planning/git-policy-cli-full-tokio.md`,
/// "The order of the move").
/// Until then this face keeps the one
///  entry they can call;
/// it goes when the last caller is asynchronous.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = runOnTheRuntime(async () => runOrderedOnTheRuntime(run)); // or a plain error when already inside
/// ```
///
/// # Panics
/// When a lane names an index that is not an item,
/// or the lanes do not cover every item exactly once.
pub fn run_ordered_and_wait_here<T, R>(run: OrderedRunOfOwned<'_, T, R>) -> Result<(), OrderedRunError>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    return match run_on_runtime(run_ordered_on_the_runtime(run)) {
        Ok(finished) => finished,
        Err(refusal) => Err(OrderedRunError::NoRuntime(refusal)),
    };
}

/// Controls for the runtime form stay out of the release executable.
#[cfg(test)]
#[path = "parallel_runner_on_the_runtime_tests.rs"]
mod tests;
