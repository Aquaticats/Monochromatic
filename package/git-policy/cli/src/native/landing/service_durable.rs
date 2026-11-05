//! What:
//!  Where the landing's own syncs wait: not on the path the person waits on, unless the landing publishes nothing.
//! Why:
//!  The owner's rule for the commit protocol, 2026-11-03: "Force at leaving, else background": no forced write while the person
//!  waits, so a power cut loses that attempt exactly as it does with native Git (staged changes and files are intact, the
//!  commit is run again), and whatever the command already promised when it left (an error, a withdrawal, a hand-over) was
//!  forced to disk first. A landing that publishes a commit therefore only queues the syncs of its records (the intent, the
//!  lock marker, the completion record, the done mark and the directory syncs); one background thread makes them in batches,
//!  each batch overlapped. A landing that publishes nothing syncs what it wrote before it returns. Git's own object, reference
//!  and index writes follow `core.fsync` as they do for native Git: the in-process index writer flushes only when
//!  `core.fsync` names the index (`ConfigSnapshot::hardens_index`).
//!  Nothing here changes what a killed process leaves: the page cache keeps every write whether or not it was synced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landing = beginDeferral({ hardensIndex });
//! // ... flushFile(file, 'intent-file') queues; flushDir(dir, 'intent-dir') queues ...
//! landing.settle(published); // true: hand the queue to the background thread; false: sync it now
//! ```

use crate::scoped_start::{Started, start_or_run_with};
use crate::side_by_side_threads::{ScopeBody, in_thread_scope};
use std::cell::{Cell, RefCell};
use std::fs::File;
use std::thread::Scope;
use std::sync::OnceLock;
use std::sync::mpsc::{Receiver, Sender, channel};

/// One sync waiting for its turn: a handle on the file or directory, and what it is for.
struct Pending {
    /// A handle on the same file or directory the record lives in; a flush through it flushes the file itself.
    handle: File,
    /// What the sync is for, as the controls and the log name it.
    what: &'static str,
}

/// What the background thread is asked to do.
enum Job {
    /// Sync every handle of a landing, all at once.
    Batch(Vec<Pending>),
    /// Answer on the channel when every earlier job is done.
    Barrier(Sender<()>),
}

thread_local! {
    /// Whether the landing on this thread queues its syncs (`true`) or makes them where it asks (`false`).
    static DEFERRING: Cell<bool> = const { Cell::new(false) };
    /// Whether the index the landing writes is flushed before it is installed: `core.fsync` names the index, or nothing said.
    static INDEX_HARDENED: Cell<bool> = const { Cell::new(true) };
    /// The syncs this landing has queued so far.
    static QUEUED: RefCell<Vec<Pending>> = const { RefCell::new(Vec::new()) };
}

/// The sender of the one background thread, started by the first landing that publishes.
static BACKGROUND: OnceLock<Sender<Job>> = OnceLock::new();

/// How many syncs the background thread has made, for a control that waits for them.
pub static BACKGROUND_SYNCED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Sync every handle of a batch at once, one thread each, and log what failed: nobody waits for this, so a failure cannot
///  fail a commit that was already answered, and the next landing or restart handles the record as it would after a power cut.
fn sync_batch(batch: Vec<Pending>) {
    sync_all_at_once(batch.as_slice(), true);
}

/// What:
///  Syncs every handle of a batch at once, one thread each, and says once when a sync had to be made on the calling thread.
///  A background sync is counted for the controls that wait for them and words its failures as the background's.
/// Why:
///  The background thread and the landing that must wait both sync a batch this way; only the wording and the count differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.all(batch.map(async (pending) => { try { await pending.handle.sync(); } catch (e) { warn(e); } if (background) synced++; }));
/// ```
fn sync_all_at_once(batch: &[Pending], background: bool) {
    in_thread_scope(SyncAll { batch, background });
}

/// The syncs of one batch, started together inside one thread scope.
struct SyncAll<'batch> {
    /// The handles to sync, one thread each.
    batch: &'batch [Pending],
    /// Whether the batch is the background's, which counts each sync and words its failures as the background's.
    background: bool,
}

impl<'batch> ScopeBody<'batch> for SyncAll<'batch> {
    type Output = ();

    /// Start one sync per pending handle, then say once when any of them had to run here.
    fn run<'scope>(self, scope: &'scope Scope<'scope, 'batch>) {
        let mut started: Vec<Started<'scope, ()>> = Vec::new();
        for pending in self.batch {
            started.push(start_or_run_with(scope, (pending, self.background), sync_one_pending));
        }
        say_once_if_inline(started.as_slice());
    }
}

/// Sync one handle, log a failure, and count a background sync.
fn sync_one_pending((pending, background): (&Pending, bool)) {
    if let Err(error) = pending.handle.sync_all() {
        if background {
            crate::service_warn!("the background sync of the {} failed: {error}", pending.what);
        } else {
            crate::service_warn!("the sync of the {} failed: {error}", pending.what);
        }
    }
    if background {
        BACKGROUND_SYNCED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Logs once, for a batch, that a sync was made on the calling thread because the system could not start a thread for it.
fn say_once_if_inline<T>(started: &[Started<'_, T>]) {
    if started.iter().any(Started::ran_inline) {
        crate::service_warn!("the system could not start a thread for a sync of the landing's records; it was made where it was asked for");
    }
}

/// The background thread: takes the jobs in order until the sender is gone.
fn background(jobs: &Receiver<Job>) {
    while let Ok(job) = jobs.recv() {
        match job {
            Job::Batch(batch) => sync_batch(batch),
            Job::Barrier(done) => {
                let _answered = done.send(());
            }
        }
    }
}

/// The sender of the background thread, started when first asked for.
fn sender() -> &'static Sender<Job> {
    return BACKGROUND.get_or_init(start_background);
}

/// What:
///  Makes the channel to the background thread and starts the thread; when no thread can start, the syncs are made where they are asked for.
/// Why:
///  `sender` runs it once, the first time a sync is handed to the background.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startBackground(): Sender<Job> { const { sender, jobs } = channel(); try { spawn(() => background(jobs)); } catch (e) { warn(e); } return sender; }
/// ```
fn start_background() -> Sender<Job> {
    let (sender, jobs) = channel::<Job>();
    let started = std::thread::Builder::new().name(String::from("cli-git-durable")).spawn(move || { background(&jobs); });
    if let Err(error) = started {
        crate::service_warn!("the thread that makes the landing's syncs could not start ({error}); they are made where they are asked for");
    }
    return sender;
}

/// Waits until every sync handed to the background thread so far is done.
///  A control uses it to see the queue empty; the service does not wait for it on a commit's path.
pub fn drain() {
    let (done, answered) = channel::<()>();
    if sender().send(Job::Barrier(done)).is_ok() {
        let _waited = answered.recv();
    }
}

/// What a landing holds while it runs: its syncs are queued, and it settles them when it ends.
pub struct Deferral {
    /// Whether `settle` has run; a deferral dropped without it (a panic) syncs what it queued.
    settled: bool,
}

/// Starts queueing the syncs of the landing on this thread.
///  `hardens_index` says whether the index file the landing installs is flushed before the rename (`core.fsync` names it).
pub fn begin(hardens_index: bool) -> Deferral {
    DEFERRING.set(true);
    INDEX_HARDENED.set(hardens_index);
    QUEUED.with_borrow_mut(Vec::clear);
    return Deferral { settled: false };
}

/// Ends the deferral on this thread and returns what it queued.
fn end() -> Vec<Pending> {
    DEFERRING.set(false);
    INDEX_HARDENED.set(true);
    return QUEUED.take();
}

/// Syncs every handle now, all at once, and names each as waited for.
fn force(batch: Vec<Pending>) {
    for pending in &batch {
        crate::service_fault::note(format!("sync-waited:{}", pending.what).as_str());
    }
    sync_all_at_once(batch.as_slice(), false);
}

impl Deferral {
    /// Ends the deferral. A landing that published a commit hands its queue to the background thread and returns at once; a
    ///  landing that published nothing (a withdrawal, an error, a commit sent back) syncs its records before it returns.
    pub fn settle(mut self, published: bool) {
        self.settled = true;
        let batch: Vec<Pending> = end();
        if batch.is_empty() {
            return;
        }
        if published {
            if let Err(rejected) = sender().send(Job::Batch(batch)) {
                // The thread is gone, so the queue is made here rather than lost.
                if let Job::Batch(returned) = rejected.0 {
                    force(returned);
                }
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

/// Flushes a file that is a landing's record: queued while a landing defers, made now otherwise.
///
/// # Errors
///
/// The error of the flush when it is made now.
pub fn flush_file(file: &File, what: &'static str) -> std::io::Result<()> {
    if DEFERRING.with(Cell::get) {
        crate::service_fault::note(format!("sync-queued:{what}").as_str());
        let handle: File = file.try_clone()?;
        let mut queued: Vec<Pending> = QUEUED.take();
        queued.push(Pending { handle, what });
        QUEUED.set(queued);
        return Ok(());
    }
    let synced: std::io::Result<()> = file.sync_all();
    crate::service_fault::note(format!("sync-waited:{what}").as_str());
    return synced;
}

/// Flushes a directory whose entries are a landing's records, as `flush_file` does for a file.
///
/// # Errors
///
/// The error of opening the directory, or of the flush when it is made now.
pub fn flush_directory(directory: &std::path::Path, what: &'static str) -> std::io::Result<()> {
    return flush_file(&File::open(directory)?, what);
}

/// Flushes the index file the landing writes, when the index is hardened: `core.fsync` names it, as native Git's index write
///  is hardened then. Otherwise the index is written as native Git writes it, unflushed. It is made now, on the landing's path,
///  as Git's own write is.
///
/// # Errors
///
/// The error of the flush.
pub fn flush_index(file: &File, what: &'static str) -> std::io::Result<()> {
    if INDEX_HARDENED.with(Cell::get) {
        let synced: std::io::Result<()> = file.sync_all();
        crate::service_fault::note(format!("sync-waited:{what}").as_str());
        return synced;
    }
    return Ok(());
}

/// Whether the index file the landing writes is flushed before it is installed.
pub fn index_is_hardened() -> bool {
    return INDEX_HARDENED.with(Cell::get);
}

/// Controls for the queue stay out of the release executable.
#[cfg(test)]
#[path = "service_durable_tests.rs"]
mod tests;
