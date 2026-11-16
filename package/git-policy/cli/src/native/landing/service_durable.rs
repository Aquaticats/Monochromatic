//! What:
//!  Where the landing's own syncs wait:
//! not on the path the person waits on,
//! unless the landing publishes nothing.
//! Why:
//!  The owner's rule for the commit protocol,
//! 2026-11-03:
//! "Force at leaving,
//! else background":
//! no forced write while the person
//!  waits,
//! so a power cut loses that attempt exactly as it does with native Git (staged changes and files are intact,
//! the
//!  commit is run again),
//! and whatever the command already promised when it left (an error,
//! a withdrawal,
//! a hand-over) was
//!  forced to disk first.
//! A landing that publishes a commit therefore only queues the syncs of its records (the intent,
//! the
//!  lock marker,
//! the completion record,
//! the done mark and the directory syncs);
//! one background task makes them in batches (each batch one job of the runtime's blocking pool),
//!  each sync of a batch overlapped.
//! A landing that publishes nothing syncs what it wrote before it returns.
//! A queue no background task takes is nobody's wait either:
//! the records reach the disk with the operating system's own write-back,
//!  which a machine crash loses as it loses any write the page cache held,
//! and a forced write on the commit's thread would be the whole
//!  device's wait on the person's path.
//! Git's own object,
//! reference
//!  and index writes follow `core.fsync` as they do for native Git:
//! the in-process index writer flushes only when
//!  `core.fsync` names the index (`ConfigSnapshot::hardens_index`).
//!  Nothing here changes what a killed process leaves:
//! the page cache keeps every write whether or not it was synced.
//! A service on its way out makes the syncs of the landings it answered at once rather than behind the lane of slow work,
//!  and waits for them no longer than `THE_EXITS_SYNCS_ARE_MADE_WITHIN`
//! (`drain_at_the_exit_within`):
//! the records a bound gives up on reach the disk with the operating system's own write-back,
//!  which is what a kill at that instruction leaves, and the commit itself is in the repository either way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landing = beginDeferral({ hardensIndex });
//! // ... flushFile(file, 'intent-file') queues; flushDir(dir, 'intent-dir') queues ...
//! landing.settle(published); // true: hand the queue to the background task; false: sync it now
//! ```

use crate::scoped_start::{Started, start_or_run_with};
use crate::side_by_side_threads::{ScopeBody, in_thread_scope};
use crate::landing_state_carried_by_its_task::{LandingState, on_the_landing_state};
use std::fs::File;
use std::thread::Scope;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::{run_a_slow_job_within_the_budget_or_at_once_when_leaving, run_blocking_job_within_the_budget};
use crate::service_durable_syncs::SyncFigure;
use crate::lazy_runtime::spawn_on_runtime;
use std::sync::mpsc::{Sender, channel};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// One sync waiting for its turn:
/// a handle on the file or directory,
/// and what it is for.
pub(crate) struct Pending {
    /// A handle on the same file or directory the record lives in;
    /// a flush through it flushes the file itself.
    handle: File,
    /// What the sync is for,
    /// as the controls and the log name it.
    what: &'static str,
}

/// What the background task is asked to do.
enum Job {
    /// Sync every handle of a landing,
    /// all at once.
    Batch(Vec<Pending>),
    /// Answer on the channel when every earlier job is done.
    Barrier(Sender<()>),
}

/// The sender of the one background task,
/// started by the first landing that publishes.
static BACKGROUND: OnceLock<UnboundedSender<Job>> = OnceLock::new();

/// Whether the service is on its way out,
/// so the batches of the landings it answered are synced at once rather than behind a place of the lane of slow work.
/// Why:
///   Those places are kept for the work a person waits for,
/// and an exit has none left to keep them for:
/// the loop that lands commits has ended (`service_server.rs`) and no other can begin,
/// so a batch that waited for a place would make the exit as late as the slow work that holds the lane.
static THE_SERVICE_IS_LEAVING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The wake the exit gives a task that is already waiting for a place of the lane of slow work.
/// A permit is stored and not only handed over,
/// so a batch that reaches that wait after the exit announced it stops waiting too.
static THE_LEAVE_ANNOUNCED: tokio::sync::Notify = tokio::sync::Notify::const_new();

/// How many syncs the background task has made,
/// for a control that waits for them.
pub static BACKGROUND_SYNCED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Sync every handle of a batch at once,
/// one thread each,
/// and log what failed:
/// nobody waits for this,
/// so a failure cannot
///  fail a commit that was already answered,
/// and the next landing or restart handles the record as it would after a power cut.
fn sync_batch(batch: Vec<Pending>) {
    let figures: Vec<(&'static str, Duration)> = sync_all_at_once(batch.as_slice(), true);
    let mut total: Duration = Duration::ZERO;
    for (_what, took) in &figures {
        total += *took;
    }
    // One line per batch, so a landing whose records were flushed after its answer still says what that cost the device.
    crate::service_log!("the background sync of {} records of a landing took {} ms", figures.len(), total.as_millis());
}

/// What:
///  Syncs every handle of a batch at once,
/// one thread each,
/// and says once when a sync had to be made on the calling thread.
///  A background sync is counted for the controls that wait for them and words its failures as the background's.
/// Why:
///  The background task and the landing that must wait both sync a batch this way;
/// only the wording and the count differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.all(batch.map(async (pending) => { try { await pending.handle.sync(); } catch (e) { warn(e); } if (background) synced++; }));
/// ```
fn sync_all_at_once(batch: &[Pending], background: bool) -> Vec<(&'static str, Duration)> {
    return in_thread_scope(SyncAll { batch, background });
}

/// The syncs of one batch,
/// started together inside one thread scope.
struct SyncAll<'batch> {
    /// The handles to sync,
    /// one thread each.
    batch: &'batch [Pending],
    /// Whether the batch is the background's,
    /// which counts each sync and words its failures as the background's.
    background: bool,
}

impl<'batch> ScopeBody<'batch> for SyncAll<'batch> {
    type Output = Vec<(&'static str, Duration)>;

    /// Start one sync per pending handle,
    /// then say once when any of them had to run here,
    /// and answer how long each took.
    fn run<'scope>(self, scope: &'scope Scope<'scope, 'batch>) -> Vec<(&'static str, Duration)> {
        let mut started: Vec<Started<'scope, (&'static str, Duration)>> = Vec::new();
        for pending in self.batch {
            started.push(start_or_run_with(scope, (pending, self.background), sync_one_pending));
        }
        say_once_if_inline(started.as_slice());
        let mut figures: Vec<(&'static str, Duration)> = Vec::new();
        for each in started {
            if let Ok(figure) = each.join() {
                figures.push(figure);
            }
        }
        return figures;
    }
}

/// Sync one handle,
/// log a failure,
/// count a background sync,
/// and answer what it was for and how long it took.
fn sync_one_pending((pending, background): (&Pending, bool)) -> (&'static str, Duration) {
    let began: Instant = Instant::now();
    if let Err(error) = pending.handle.sync_all() {
        if background {
            crate::service_warn!("the background sync of the {} failed: {error}", pending.what);
        } else {
            crate::service_warn!("the sync of the {} failed: {error}", pending.what);
        }
    }
    let took: Duration = began.elapsed();
    if background {
        BACKGROUND_SYNCED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
    return (pending.what, took);
}

/// Logs once,
/// for a batch,
/// that a sync was made on the calling thread because the system could not start a thread for it.
fn say_once_if_inline<T>(started: &[Started<'_, T>]) {
    if started.iter().any(Started::ran_inline) {
        crate::service_warn!("the system could not start a thread for a sync of the landing's records; it was made where it was asked for");
    }
}

/// The background task:
/// takes the jobs in order until the sender is gone,
/// each batch as one job of the blocking pool
/// (a sync can wait on the disk,
/// and the pool is where tokio keeps such calls off the workers that poll tasks).
async fn background(mut jobs: UnboundedReceiver<Job>) {
    while let Some(job) = jobs.recv().await {
        match job {
            Job::Batch(batch) => {
                let holding_and_syncing = move || {
                    crate::service_hold_a_step_for_a_control::hold_blocking(crate::service_hold_a_step_for_a_control::INSIDE_A_SYNC);
                    sync_batch(batch);
                };
                // A batch asks for a place of slow work while the service is serving, because the work a person waits for keeps
                //  its own places free of it; an exit has no such work left, and waiting there would make it as late as the slow
                //  work that holds the lane.
                let answer: Result<(), BlockingJobError> = if THE_SERVICE_IS_LEAVING.load(std::sync::atomic::Ordering::SeqCst) {
                    run_blocking_job_within_the_budget(holding_and_syncing).await
                } else {
                    run_a_slow_job_within_the_budget_or_at_once_when_leaving(&THE_LEAVE_ANNOUNCED, holding_and_syncing).await
                };
                if let Err(error) = answer {
                    // Nobody waits for this: the records are as a power cut leaves them, which the next landing or restart handles.
                    crate::service_warn!("the background sync of a landing's records could not run: {error}");
                }
            }
            Job::Barrier(done) => {
                let _answered = done.send(());
            }
        }
    }
    return;
}

/// The sender of the background task,
/// started when first asked for.
fn sender() -> &'static UnboundedSender<Job> {
    return BACKGROUND.get_or_init(start_background);
}

/// What:
///  Makes the channel to the background task and starts the task on the runtime;
/// when the runtime has no worker to run it,
/// nothing waits for the syncs and they reach the disk with the operating system's write-back.
/// Why:
///  `sender` runs it once,
/// the first time a sync is handed to the background.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startBackground(): Sender<Job> { const { sender, jobs } = channel(); const task = spawn(background(jobs)); if (!task) warn('inline'); return sender; }
/// ```
fn start_background() -> UnboundedSender<Job> {
    let (sender, jobs) = unbounded_channel::<Job>();
    // The receiver goes into the task; when no task starts it is dropped, and every send then fails, which leaves each flush to the
    //  operating system rather than making it on the thread that asked for it.
    if spawn_on_runtime(background(jobs)).is_none() {
        crate::service_warn!("the runtime has no worker for the task that makes the landing's syncs; each flush is left to the operating system's write-back");
    }
    return sender;
}

/// Waits until every sync handed to the background task so far is done.
///  A control uses it to see the queue empty;
/// the service does not wait for it on a commit's path.
/// It waits no longer than the patience of a control (`PHASE_PATIENCE`), because nothing in this program waits without an
///  end:
/// the service's own exits wait through `drain_at_the_exit_within`, which also says the service is leaving.
pub fn drain() {
    let _made: bool = drain_within(crate::wait_ceilings::PHASE_PATIENCE);
}

/// What became of the syncs of the landings a leaving service answered.
/// Why:
///   The two ends are not the same and the log says which one it is:
/// either every record handed to the background task reached the disk,
/// or the ones still waiting reach it with the operating system's own write-back, which is what a kill at that instruction
///  leaves of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheSyncsOfTheLastLandings = 'made' | 'leftToTheWriteBack';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TheSyncsOfTheLastLandings {
    /// Every sync handed to the background task was made before this call returned.
    Made,
    /// The bound ran out with syncs still to make, so what is left reaches the disk with the operating system's write-back.
    LeftToTheWriteBack,
}

/// What:
///  Says the service is leaving,
///  and waits no longer than `ceiling` for the syncs of the landings it answered.
/// Why:
///   Those records are why the exit waits at all,
/// and a batch of them waits for a place of the lane of slow work,
/// so the wait here is bounded and the work it gives up on is left where a kill would have left it:
/// the commit is in the repository and on its branch either way, and its records reach the disk with the operating system's
///  write-back.
///  The service is said to be leaving before the wait begins,
/// so a batch already waiting for a place of that lane stops waiting for it and is synced now
/// (`blocking_jobs_within_the_budget.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leaving = true; leaveAnnounced.notifyOne(); return await drainTheSyncsWithin(ceiling) ? 'made' : 'leftToTheWriteBack';
/// ```
pub fn drain_at_the_exit_within(ceiling: Duration) -> TheSyncsOfTheLastLandings {
    THE_SERVICE_IS_LEAVING.store(true, std::sync::atomic::Ordering::SeqCst);
    THE_LEAVE_ANNOUNCED.notify_one();
    return if drain_within(ceiling) { TheSyncsOfTheLastLandings::Made } else { TheSyncsOfTheLastLandings::LeftToTheWriteBack };
}

/// What:
///  Waits until every sync handed to the background task so far is done,
///  or until `ceiling` has passed.
/// Why:
///   A barrier answers when every job handed over before it has ended,
/// so the wait is the queue's own order and not a poll;
/// a queue no task took is answered back by the send itself, and nothing is waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sent = barriered(); return sent ? answered.recvTimeout(ceiling) : false;
/// ```
fn drain_within(ceiling: Duration) -> bool {
    let (done, answered) = channel::<()>();
    if sender().send(Job::Barrier(done)).is_err() {
        // No task takes the queue, so the records reach the disk with the operating system's own write-back and there is
        //  nothing to wait for.
        return false;
    }
    return match answered.recv_timeout(ceiling) {
        Ok(()) => true,
        // The bound ran out, or the task ended without answering: what is left is the write-back's.
        Err(std::sync::mpsc::RecvTimeoutError::Timeout | std::sync::mpsc::RecvTimeoutError::Disconnected) => false,
    };
}

/// What a landing holds while it runs:
/// its syncs are queued,
/// and it settles them when it ends.
pub struct Deferral {
    /// Whether `settle` has run;
    /// a deferral dropped without it (a panic) syncs what it queued.
    settled: bool,
}

/// Starts queueing the syncs of the landing on this thread.
///  `hardens_index` says whether the index file the landing installs is flushed before the rename (`core.fsync` names it).
pub fn begin(hardens_index: bool) -> Deferral {
    on_the_landing_state(hardens_index, start_deferring);
    return Deferral { settled: false };
}

/// Begins queueing the syncs of the landing in `state`,
/// with the index hardened or not.
fn start_deferring(state: &LandingState, hardens_index: bool) {
    state.deferring_syncs.set(true);
    state.index_hardened.set(hardens_index);
    state.queued_syncs.borrow_mut().clear();
    state.syncs.borrow_mut().clear();
}

/// What:
///  Records one sync of the landing this code runs in,
/// as its answer states it.
/// Why:
///   A landing that made more syncs and one that made the same syncs on a stalling device look alike
///  in the phases;
/// the record says how many there were and how long each took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landing.syncs.push({ what, waited, took })
/// ```
fn note_a_sync(figure: SyncFigure) {
    on_the_landing_state(figure, record_a_sync);
}

/// Pushes one sync into the landing's state.
fn record_a_sync(state: &LandingState, figure: SyncFigure) {
    state.syncs.borrow_mut().push(figure);
}

/// What:
///  Takes the syncs the landing this code runs in has recorded so far,
/// in the order it asked for them.
/// Why:
///   The landing's answer states them,
/// and nothing else reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const syncs = landing.syncs.splice(0)
/// ```
pub fn take_syncs() -> Vec<SyncFigure> {
    return on_the_landing_state((), drain_syncs);
}

/// Takes the recorded syncs out of the landing's state.
fn drain_syncs(state: &LandingState, (): ()) -> Vec<SyncFigure> {
    return state.syncs.take();
}

/// Ends the deferral on this thread and returns what it queued.
fn end() -> Vec<Pending> {
    return on_the_landing_state((), stop_deferring);
}

/// Ends the deferral in `state` and takes what it queued.
fn stop_deferring(state: &LandingState, (): ()) -> Vec<Pending> {
    state.deferring_syncs.set(false);
    state.index_hardened.set(true);
    return state.queued_syncs.take();
}

/// Syncs every handle now,
/// all at once,
/// and names each as waited for.
fn force(batch: Vec<Pending>) {
    for pending in &batch {
        crate::service_fault::note(format!("sync-waited:{}", pending.what).as_str());
    }
    let figures: Vec<(&'static str, Duration)> = sync_all_at_once(batch.as_slice(), false);
    for (what, took) in figures {
        note_a_sync(SyncFigure { what, waited: true, took });
    }
}

impl Deferral {
    /// Ends the deferral.
    /// A landing that published a commit hands its queue to the background task and returns at once;
    /// a
    ///  landing that published nothing (a withdrawal,
    /// an error,
    /// a commit sent back) syncs its records before it returns.
    pub fn settle(self, published: bool) {
        return self.settle_with(published, &hand_to_the_background);
    }

    /// What:
    ///  `settle` with the handing of the queue to the background task given,
    /// so a control proves what a published landing does when no task takes it.
    /// Why:
    ///   The sender is one for the process,
    /// so only the hand can be made to refuse in a control.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// deferral.settleWith(published, hand)
    /// ```
    fn settle_with(mut self, published: bool, hand: &dyn Fn(Vec<Pending>) -> Option<Vec<Pending>>) {
        self.settled = true;
        let batch: Vec<Pending> = end();
        if batch.is_empty() {
            return;
        }
        if published {
            if hand(batch).is_some() {
                // No task takes the queue, and the decision for a landing that published is "Force at leaving, else background":
                //  the records reach the disk with the operating system's own write-back rather than on the commit's thread.
                crate::service_warn!("the runtime has no task for the syncs of a landing that published, so none waits for them here and the records reach the disk with the operating system's own write-back");
            }
        } else {
            force(batch);
        }
    }
}

impl Drop for Deferral {
    fn drop(&mut self) {
        if !self.settled {
            force(end());
        }
    }
}

/// Whether the landing in `state` queues its syncs.
fn is_deferring(state: &LandingState, (): ()) -> bool {
    return state.deferring_syncs.get();
}

/// Queues one sync in the landing's state.
fn queue_a_sync(state: &LandingState, pending: Pending) {
    state.queued_syncs.borrow_mut().push(pending);
}

/// Whether the index the landing in `state` writes is flushed before it is installed.
fn index_hardened_in(state: &LandingState, (): ()) -> bool {
    return state.index_hardened.get();
}

/// Queues a flush in the background task and never waits for it.
///
/// # Errors
///
/// The error of duplicating the handle;
/// a queue that the background task no longer takes is left to the operating system's write-back.
pub fn defer_file(file: &File, what: &'static str) -> std::io::Result<()> {
    return defer_file_with(&hand_to_the_background, file, what);
}

/// What:
///  `defer_file` with the handing of the batch to the background task given,
/// so a control proves what happens when no task takes it.
/// Why:
///   Which task holds the queue is one process-wide sender,
/// so a control cannot make it refuse by way of the runtime;
/// the hand is the one step that decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deferFileWith(hand, file, what)
/// ```
fn defer_file_with(hand: &dyn Fn(Vec<Pending>) -> Option<Vec<Pending>>, file: &File, what: &'static str) -> std::io::Result<()> {
    let handle: File = file.try_clone()?;
    note_a_sync(SyncFigure { what, waited: false, took: Duration::ZERO });
    crate::service_fault::note(format!("sync-queued:{what}").as_str());
    if hand(vec![Pending { handle, what }]).is_some() {
        // No task takes the flush, so it is the operating system's: the marker names the holder and nothing else, which a power
        //  cut loses as it loses any write the page cache held, and the step before a landing removes such a marker at once.
        crate::service_warn!("the runtime has no task for the flush of the {what}, so nothing waits for it here and the write reaches the disk with the operating system's own write-back");
    }
    return Ok(());
}

/// What:
///  Hands one batch to the background task,
/// or answers it back when the task no longer takes it.
/// Why:
///   The receiver is dropped when no task started,
/// so a rejected send is the only sign that nothing will make the syncs.
/// A barrier is never sent through here,
/// so a rejected batch is the only thing the answer holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hand = (batch) => { try { sender().send({ batch }); return undefined; } catch { return batch; } };
/// ```
fn hand_to_the_background(batch: Vec<Pending>) -> Option<Vec<Pending>> {
    return match sender().send(Job::Batch(batch)) {
        Ok(()) => None,
        Err(rejected) => match rejected.0 {
            Job::Batch(returned) => Some(returned),
            Job::Barrier(_answered) => None,
        },
    };
}

/// Flushes a file that is a landing's record:
/// queued while a landing defers,
/// made now otherwise.
///
/// # Errors
///
/// The error of the flush when it is made now.
pub fn flush_file(file: &File, what: &'static str) -> std::io::Result<()> {
    if on_the_landing_state((), is_deferring) {
        crate::service_fault::note(format!("sync-queued:{what}").as_str());
        note_a_sync(SyncFigure { what, waited: false, took: Duration::ZERO });
        let handle: File = file.try_clone()?;
        on_the_landing_state(Pending { handle, what }, queue_a_sync);
        return Ok(());
    }
    let began: Instant = Instant::now();
    let synced: std::io::Result<()> = file.sync_all();
    note_a_sync(SyncFigure { what, waited: true, took: began.elapsed() });
    crate::service_fault::note(format!("sync-waited:{what}").as_str());
    return synced;
}

/// Flushes a directory whose entries are a landing's records,
/// as `flush_file` does for a file.
///
/// # Errors
///
/// The error of opening the directory,
/// or of the flush when it is made now.
pub fn flush_directory(directory: &std::path::Path, what: &'static str) -> std::io::Result<()> {
    return flush_file(&File::open(directory)?, what);
}

/// Flushes the index file the landing writes,
/// when the index is hardened:
/// `core.fsync` names it,
/// as native Git's index write
///  is hardened then.
/// Otherwise the index is written as native Git writes it,
/// unflushed.
/// It is made now,
/// on the landing's path,
///  as Git's own write is.
///
/// # Errors
///
/// The error of the flush.
pub fn flush_index(file: &File, what: &'static str) -> std::io::Result<()> {
    if index_is_hardened() {
        let began: Instant = Instant::now();
        let synced: std::io::Result<()> = file.sync_all();
        note_a_sync(SyncFigure { what, waited: true, took: began.elapsed() });
        crate::service_fault::note(format!("sync-waited:{what}").as_str());
        return synced;
    }
    return Ok(());
}

/// Whether the index file the landing writes is flushed before it is installed.
pub fn index_is_hardened() -> bool {
    return on_the_landing_state((), index_hardened_in);
}

/// Controls for the queue stay out of the release executable.
#[cfg(test)]
#[path = "service_durable_tests.rs"]
mod tests;
