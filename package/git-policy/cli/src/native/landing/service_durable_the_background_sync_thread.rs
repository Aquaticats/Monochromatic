//! What:
//!  The background task that makes a landing's syncs after the landing has answered:
//!  the one channel every landing hands its batch to,
//!  the task that takes each batch as one job of the blocking pool,
//!  the thread scope that syncs a batch one handle per thread,
//!  and the wait a control makes until the queue is empty.
//! Why:
//!   A sync can wait on the disk,
//!  and nobody waits for these once a commit was answered,
//!  so they run as far from the commit's own thread as the runtime allows,
//!  and a sync that the system could not give a thread still runs where it was asked for.
//!  They live beside the deferral that hands them their batches,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceDurableTheBackgroundSyncThread.ts
//! export const drain = (): void => { sender().send({ barrier: channel() }); };
//! export const handToTheBackground = (batch: Pending[]): Pending[] | undefined => ...;
//! ```

use crate::blocking_job::run_blocking_job;
use crate::lazy_runtime::spawn_on_runtime;
use crate::scoped_start::{
    Started,
    start_or_run_with,
};
use crate::side_by_side_threads::{
    ScopeBody,
    in_thread_scope,
};
use std::fs::File;
use std::sync::OnceLock;
use std::sync::mpsc::{
    Sender,
    channel,
};
use std::thread::Scope;
use std::time::{
    Duration,
    Instant,
};
use tokio::sync::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
    unbounded_channel,
};
/// One sync waiting for its turn:
/// a handle on the file or directory,
/// and what it is for.
pub(crate) struct Pending {
    /// A handle on the same file or directory the record lives in;
    /// a flush through it flushes the file itself.
    pub(super) handle: File,
    /// What the sync is for,
    /// as the controls and the log name it.
    pub(super) what: &'static str,
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
pub(super) fn sync_batch(batch: Vec<Pending>) {
    let figures: Vec<(&'static str, Duration)> = sync_all_at_once(batch.as_slice(), true);
    let mut total: Duration = Duration::ZERO;
    for (_what, took) in &figures {
        total += *took;
    }
    // One line per batch, so a landing whose records were flushed after its answer still says what that cost the device.
    crate::service_log!(
        "the background sync of {} records of a landing took {} ms",
        figures.len(),
        total.as_millis()
    );
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
pub(super) fn sync_all_at_once(
    batch: &[Pending],
    background: bool,
) -> Vec<(&'static str, Duration)> {
    return in_thread_scope(SyncAll {
        batch,
        background,
    });
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
    fn run<'scope>(
        self,
        scope: &'scope Scope<'scope, 'batch>,
    ) -> Vec<(&'static str, Duration)> {
        let mut started: Vec<Started<'scope, (&'static str, Duration)>> = Vec::new();
        for pending in self.batch {
            started.push(start_or_run_with(
                scope,
                (pending, self.background),
                sync_one_pending,
            ));
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
            crate::service_warn!(
                "the background sync of the {} failed: {error}",
                pending.what
            );
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
        crate::service_warn!(
            "the system could not start a thread for a sync of the landing's records; it was made where it was asked for"
        );
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
                if let Err(error) = run_blocking_job(move || {
                    sync_batch(batch);
                })
                .await
                {
                    // Nobody waits for this: the records are as a power cut leaves them, which the next landing or restart handles.
                    crate::service_warn!(
                        "the background sync of a landing's records could not run: {error}"
                    );
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
        crate::service_warn!(
            "the runtime has no worker for the task that makes the landing's syncs; each flush is left to the operating system's write-back"
        );
    }
    return sender;
}

/// Waits until every sync handed to the background task so far is done.
///  A control uses it to see the queue empty;
/// the service does not wait for it on a commit's path.
pub fn drain() {
    let (done, answered) = channel::<()>();
    if sender().send(Job::Barrier(done)).is_ok() {
        let _waited = answered.recv();
    }
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
pub(super) fn hand_to_the_background(batch: Vec<Pending>) -> Option<Vec<Pending>> {
    return match sender().send(Job::Batch(batch)) {
        Ok(()) => None,
        Err(rejected) => match rejected.0 {
            Job::Batch(returned) => Some(returned),
            Job::Barrier(_answered) => None,
        },
    };
}
