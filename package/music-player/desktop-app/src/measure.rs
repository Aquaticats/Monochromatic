//! Parallel background queue true-peak measurement,
//! with progress and per-track reports.
//!
//! On every queue load the controller starts a background sweep over the queue.
//!  The sweep runs in two
//! phases.
//!  First,
//! a peek reads each track's cached decision and header length and reports
//! them at once,
//! so rows show their length before any decode.
//!  Then the measure phase fans out one worker per
//! logical core over the non-current tracks:
//!  each worker claims tracks from a shared atomic cursor,
//!  skips the
//! ones already cached,
//!  and decodes the rest to measure their true peak,
//!  storing each result through the
//! `CacheHandle` and reporting it to the window.
//!  Workers run at idle OS scheduling priority,
//!  so they
//! saturate every core when the machine is otherwise free yet yield to the realtime audio thread
//! and the UI.
//!  Every reply goes through a `SweepSink` tagged with the sweep's generation,
//! so the
//! controller drops reports from a sweep that a newer queue load has replaced.
//!  Sweeps are never
//! cancelled.
//!  The current track is handled by `peak_swap`,
//!  because the track change must not wait for
//! that one visible result.

/// What:
///      `use std::collections::HashSet;`.
///  The set of exact fingerprints the sweep skips.
/// Why:
///       The skip-check is one lookup against a snapshot taken when the sweep starts.
use std::collections::HashSet;

/// What:
///      `use std::path::{Path, PathBuf};`.
///  Track paths,
/// borrowed and owned.
/// Why:
///       The sweep hands owned paths to its reports and borrows them while it decodes.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///      `use std::sync::Arc;`.
///  Shared ownership for the sweep's inputs.
/// Why:
///       Every worker reads the same track list,
/// cursor and snapshot.
use std::sync::{
    Arc,
    Mutex,
};

/// What:
///      `use std::sync::atomic::{AtomicUsize, Ordering};`.
///  The shared cursor and visit counter.
/// Why:
///       Workers claim tracks and count visits without a lock.
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

/// What:
///      `use std::thread;` and `use std::time::{Duration, Instant};`.
///  Spawning workers and throttling progress reports.
/// Why:
///       Progress is reported at most every quarter second so the window is not flooded.
use std::thread;
/// Imports the duration and instant types that time the sweep's progress reports.
use std::time::{
    Duration,
    Instant,
};

/// What:
///      `use crate::measure_peek::peek_measures;` and `use crate::peakcache::{self, CacheHandle};`.
///  The read-only peek and the cache handle the sweep stores through.
/// Why:
///       The sweep peeks before it measures,
/// and its fingerprints come from the peak cache module.
use crate::measure_peek::peek_measures;
/// Imports the peak cache module and its handle that the sweep reads and writes.
use crate::peakcache::{
    self,
    CacheHandle,
};

/// What:
///      `use crate::sweep_gate::{Admission, SweepGate};`.
///  The pause gate and its answer.
/// Why:
///       Each worker passes the gate before it claims a track (D107).
use crate::sweep_gate::{
    Admission,
    SweepGate,
};

/// What:
///      `use crate::sweep_report::{SweepJob, SweepReport, SweepSink};`.
///  The job the controller starts and the reports the sweep sends back.
/// Why:
///       One job description keeps the spawn signature small.
use crate::sweep_report::{
    SweepJob,
    SweepReport,
    SweepSink,
};

/// What:
///      `use crate::track_measure::{TrackMeasure, measure_from_decision};`.
///  The measure a finished decode reports.
/// Why:
///       A decode yields a decision,
/// and the window needs its length and peak.
use crate::duration_probe::probe_duration_secs;
/// Imports the conversions from a decision or a duration to a track measure.
use crate::track_measure::{
    measure_from_decision,
    measure_from_duration,
};

/// What:
///      `use crate::truepeak::resolve_full;`.
///  The full-file decode that produces a decision.
/// Why:
///       The measure phase and the re-analysis both decode through this one entry point.
use crate::truepeak::resolve_full;

/// What:
/// `mod thread_idle_priority;` the per-platform request that a sweep thread run at the
/// machine's idle tier (`src/measure/thread_idle_priority.rs`).
/// Why:
/// the sweep is CPU-bound and must yield to the audio thread and the window,
/// and each
/// platform answers that request in its own vocabulary,
/// so the four answers sit together apart
/// from the sweep itself.
mod thread_idle_priority;

/// Imports the idle-tier request that the coordinator and every worker make before decoding.
use self::thread_idle_priority::lower_current_thread_to_idle;

/// The kind of a cached decision,
/// to tell an exact one from an estimate.
use truepeak_core::DecisionKind;

/// What:
///      `const MEASURE_REPORT_CHUNK: usize = 128;`.
///  How many peeked tracks go into one report.
/// Why:
///       Large libraries are announced in batches,
/// so the window is updated a few times,
/// not once per track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MEASURE_REPORT_CHUNK = 128;
/// ```
const MEASURE_REPORT_CHUNK: usize = 128;

/// What:
///      `const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);`.
///  The minimum time between two progress reports from one worker.
/// Why:
///       A quarter second keeps the count moving without a report per skipped track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PROGRESS_INTERVAL_MS = 250;
/// ```
const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);

/// What:
///      `pub(crate) fn spawn_queue_measurement(job: SweepJob)`.
///  Starts one detached sweep for a queue load.
/// Why:
///       The engine never waits on a sweep,
/// and each report carries the job's generation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spawnQueueMeasurement(job: SweepJob): void { startWorker(() => runSweep(job)); }
/// ```
pub(crate) fn spawn_queue_measurement(job: SweepJob) {
    // What:     `thread::spawn(move || run_sweep(job));`. Spawn the detached coordinator; the
    //           closure takes the whole job. The handle is dropped: the sweep runs to completion.
    // Why:      Background work that does not block the engine and needs no join or cancel.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // startWorker(() => runSweep(job));
    // ```
    thread::spawn(move || {
        let sink: SweepSink = Arc::clone(&job.sink);
        let (generation, refresh_id): (u64, Option<u64>) = (job.generation, job.refresh_id);
        // A refresh sweep tells the coalescer it ended even when the coordinator panicked, or the follow-up would never start.
        #[cfg_attr(
            dylint_lib = "monochromatic_lint_closure",
            expect(
                avoidable_closure,
                reason = "std::panic::catch_unwind takes only a callable that passes no argument to its body, so the job the sweep runs must be captured; catch_unwind is the only way to turn a panic of the sweep into a value"
            )
        )]
        let outcome =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| return run_sweep(job)));
        if outcome.is_err() {
            tracing::error!(generation, "the sweep coordinator panicked");
        }
        if let Some(id) = refresh_id {
            sink(SweepReport::RefreshDone {
                generation,
                id,
            });
        }
    });
}

/// What:
///      `struct SweepShared { tracks, cursor, visited, cache, known, sink, generation }`.
///  The state every worker of one sweep reads and updates.
/// Why:
///       One shared value keeps the worker signature to one argument and makes the sweep's inputs explicit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepShared = { tracks: readonly string[]; cursor: AtomicInt; visited: AtomicInt; cache: CacheHandle; known: Set<bigint>; sink: SweepSink; generation: number };
/// ```
struct SweepShared {
    /// Non-current tracks in queue order,
    /// shared read-only.
    tracks: Arc<[PathBuf]>,
    /// Next index a worker will claim.
    cursor: AtomicUsize,
    /// Tracks visited so far,
    /// whether skipped,
    /// measured,
    /// or failed.
    visited: AtomicUsize,
    /// Handle the decisions are stored through.
    cache: CacheHandle,
    /// Exact fingerprints present when the sweep started.
    known: HashSet<u64>,
    /// Where reports go,
    /// tagged with the generation.
    sink: SweepSink,
    /// Generation of the queue load that started this sweep.
    generation: u64,
    /// Pause gate every worker passes before it claims a track (D107).
    gate: Arc<SweepGate>,
    /// Fingerprints taken by any sweep of this generation.
    claims: Arc<Mutex<HashSet<u64>>>,
    /// Files that could not be decoded,
    /// reported together when the sweep ends.
    failed: Mutex<Vec<PathBuf>>,
    /// True for a refresh sweep,
    /// which reports no progress.
    quiet: bool,
    /// True when `tracks` holds only changed files and `known` is empty:
    /// exactness is asked of the cache per file.
    listed: bool,
}

/// What:
///      `fn run_sweep(job: SweepJob)`.
///  The detached coordinator:
/// peek every track,
/// then measure the non-current ones with a fan-out.
/// Why:
///       The peek makes lengths and cached peaks visible at once;
/// the fan-out measures the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runSweep(job: SweepJob): void { peekAll(job); measureAll(job); }
/// ```
fn run_sweep(job: SweepJob) {
    // What:     `lower_current_thread_to_idle();`. Put the sweep coordinator in the idle class.
    // Why:      It probes every track at normal priority before any worker starts. Idle keeps
    //           that probe pass and the workers it starts behind playback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lowerCurrentThreadToIdle();
    // ```
    lower_current_thread_to_idle();
    // What:     `report_peeked_measures(&job);`. Announce every track's cached peak and length.
    // Why:      Rows get their lengths before any decode starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // peekAll(job);
    // ```
    // A refresh sweep (D113) leaves the rows as they are; only what it newly measures or fails is reported.
    if !job.quiet {
        report_peeked_measures(&job.tracks, &job);
    } else if !job.announce.is_empty() {
        // A refresh announces the files that are new to the list by peek, so their rows have a length at once (D113).
        report_peeked_measures(&job.announce, &job);
    }
    // What:     `let measure_tracks = job.tracks minus the current track`. The tracks the fan-out visits.
    // Why:      The current track is measured by `peak_swap`, so the sweep leaves it alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const measureTracks = job.tracks.filter((p) => p !== job.current);
    // ```
    let mut measure_tracks: Vec<PathBuf> = Vec::new();
    for path in &job.tracks {
        if job.current.as_ref() != Some(path) {
            measure_tracks.push(path.clone());
        }
    }
    // What:     `let shared = Arc::new(SweepShared { ... })`. Snapshot the known set and build the shared state.
    // Why:      One snapshot serves every worker; the cursor and visit counter start at zero.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const shared = { tracks, cursor: 0, visited: 0, cache, known: cache.knownFingerprints(), sink, generation };
    // ```
    let total = measure_tracks.len();
    let shared = Arc::new(SweepShared {
        tracks: measure_tracks.into(),
        cursor: AtomicUsize::new(0),
        visited: AtomicUsize::new(0),
        cache: job.cache.clone(),
        known: if job.listed {
            HashSet::new()
        } else {
            job.cache.known_fingerprints()
        },
        sink: Arc::clone(&job.sink),
        generation: job.generation,
        gate: Arc::clone(&job.gate),
        claims: Arc::clone(&job.claims),
        failed: Mutex::new(Vec::new()),
        quiet: job.quiet,
        listed: job.listed,
    });
    // What:     `report_progress(&shared, 0)` when there is work. Show the bar as the analysis starts.
    // Why:      D26: the scan bar appears when analysis starts and leaves when it finishes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (total > 0) sink({ kind: "progress", done: 0, total });
    // ```
    if total > 0 && !job.quiet {
        report_progress(&shared, 0);
    }
    // What:     `let workers = worker_count(total);`. The fan-out width.
    // Why:      One per core, never more than there are tracks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const workers = workerCount(total);
    // ```
    let workers = worker_count(total);
    tracing::info!(
        tracks = total,
        workers,
        generation = job.generation,
        "warming sweep started"
    );
    // What:     The loop below starts `workers` threads, each with its own clone of the shared state, and keeps
    //           their join handles.
    // Why:      Every worker shares the same `SweepShared`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const handles = range(workers).map(() => startWorker(() => runWorker(shared)));
    // ```
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity(workers);
    for _worker in 0..workers {
        let worker_shared = Arc::clone(&shared);
        handles.push(thread::spawn(move || run_worker(worker_shared)));
    }
    // What:     join every worker, logging a panicked one.
    // Why:      One bad thread must not poison the sweep; the final progress report runs after all joins.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const h of handles) await h;
    // ```
    for handle in handles {
        if handle.join().is_err() {
            tracing::error!("a warming worker panicked");
        }
    }
    // What:     `report_progress(&shared, total)` after the joins when there was work.
    // Why:      The final report always reaches the total, so the bar leaves even when a worker panicked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (total > 0) sink({ kind: "progress", done: total, total });
    // ```
    if total > 0 && !job.quiet {
        report_progress(&shared, total);
    }
    report_unreadable(&shared);
    tracing::info!(generation = job.generation, "warming sweep finished");
}

/// What:
///      `fn report_peeked_measures(job: &SweepJob)`.
///  Sends every track's cached decision and header length in chunks.
/// Why:
///       Rows get lengths and cached peaks before any decode,
/// and chunks keep the window updates few.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportPeekedMeasures(job: SweepJob): void { for (const chunk of chunks(job.tracks)) job.sink(peekMeasures(chunk)); }
/// ```
fn report_peeked_measures(
    tracks: &[PathBuf],
    job: &SweepJob,
) {
    for chunk in tracks.chunks(MEASURE_REPORT_CHUNK) {
        let measures = peek_measures(chunk, &job.cache);
        (job.sink)(SweepReport::Measures {
            generation: job.generation,
            measures,
        });
    }
}

/// What:
///      `fn report_progress(shared: &SweepShared, done: usize)`.
///  Sends one progress report for the sweep's generation.
/// Why:
///       The window shows `done` of `total` while the sweep runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reportProgress = (shared: SweepShared, done: number) => shared.sink({ kind: "progress", done, total: shared.tracks.length });
/// ```
fn report_progress(
    shared: &SweepShared,
    done: usize,
) {
    (shared.sink)(SweepReport::Progress {
        generation: shared.generation,
        done,
        total: shared.tracks.len(),
    });
}

/// What:
///      `fn worker_count(track_count: usize) -> usize`.
///  The number of decode workers to
///           spawn:
///  one per logical core,
///  clamped to the track count,
///  `0` for none.
/// Why:
///       Saturate the CPU without spawning idle threads for a short queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function workerCount(trackCount: number): number {
///   if (trackCount === 0) return 0;
///   return Math.min(navigator.hardwareConcurrency ?? 1, trackCount);
/// }
/// ```
fn worker_count(track_count: usize) -> usize {
    // What:     `if track_count == 0 { return 0; }`. Nothing to sweep.
    // Why:      Spawn no threads for an empty queue.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (trackCount === 0) return 0;
    // ```
    if track_count == 0 {
        return 0;
    }
    // What:     `let cores = match thread::available_parallelism() { Ok(count) => count.get(), Err(_) => 1 };` is
    //           the number of cores the operating system reports, or 1 when it cannot say.
    // Why:      The degree of parallelism for CPU-bound decoding.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cores = navigator.hardwareConcurrency ?? 1;
    // ```
    let cores = match thread::available_parallelism() {
        Ok(count) => count.get(),
        Err(_error) => 1,
    };
    // What:     `cores.min(track_count)`. Never more workers than tracks. Tail -> return.
    // Why:      Extra threads beyond the work would just exit immediately.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return Math.min(cores, trackCount);
    // ```
    return cores.min(track_count);
}

/// What:
///      `fn run_worker(shared: Arc<SweepShared>)`.
///  One worker:
/// drop to idle priority,
/// then claim tracks until the cursor runs out.
/// Why:
///       Each claimed track is visited once,
/// and progress is throttled per worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runWorker(shared: SweepShared): void { lowerToIdle(); while (...) { measureOne(next()); } }
/// ```
fn run_worker(shared: Arc<SweepShared>) {
    // What:     `lower_current_thread_to_idle();`. Give the decode the idle scheduling class.
    // Why:      The sweep yields to the realtime audio thread and the UI.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lowerCurrentThreadToIdle();
    // ```
    lower_current_thread_to_idle();
    // What:     `let mut last_report = Instant::now();`. The time of this worker's last progress report.
    // Why:      Each worker throttles its own reports, so no lock is needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let lastReport = performance.now();
    // ```
    let mut last_report = Instant::now();
    loop {
        // What:     ask the pause gate for admission before claiming; leave when it is closed.
        // Why:      D107: a pause stops new claims, a track already claimed finishes, and a resume wakes parked
        //           workers. Closing the gate at shutdown releases them so the sweep's joins cannot hang.
        if shared.gate.admit() == Admission::Stop {
            tracing::info!(
                generation = shared.generation,
                "warming worker stops: the sweep gate is closed"
            );
            break;
        }
        // What:     claim the next index; stop when the cursor is past the end.
        // Why:      `fetch_add` hands each track to exactly one worker.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const path = shared.tracks[shared.cursor++];
        // if (path === undefined) break;
        // ```
        let index = shared.cursor.fetch_add(1, Ordering::Relaxed);
        let Some(path) = shared.tracks.get(index) else {
            break;
        };
        measure_one(path, &shared);
        // What:     count this visit, whatever its outcome, and report when the interval has passed.
        // Why:      Skipped, failed, and measured tracks all move the bar; the throttle keeps reports sparse.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const done = ++shared.visited;
        // ```
        let done = shared.visited.fetch_add(1, Ordering::Relaxed) + 1;
        if !shared.quiet && last_report.elapsed() >= PROGRESS_INTERVAL {
            report_progress(&shared, done);
            last_report = Instant::now();
        }
    }
}

/// What:
///      `fn measure_one(path: &Path, shared: &SweepShared)`.
///  Measures one track unless it is already exact in the cache,
/// storing and reporting the result.
/// Why:
///       A track that cannot be fingerprinted or decoded is logged and left out,
/// and the sweep moves on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function measureOne(path: string, shared: SweepShared): void { ... }
/// ```
fn measure_one(
    path: &Path,
    shared: &SweepShared,
) {
    // What:     `let Some(key) = peakcache::fingerprint(path) else { ... };`. Key the track.
    // Why:      A file without a fingerprint cannot be cached, so there is nothing to store.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const key = fingerprint(path); if (key === null) return;
    // ```
    let Some(key) = peakcache::fingerprint(path) else {
        tracing::debug!(path = %path.display(), "skip: no fingerprint");
        return;
    };
    // What:     skip when the snapshot already holds an exact decision.
    // Why:      The snapshot makes a warm restart a quick pass over the queue.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (shared.known.has(key)) return;
    // ```
    if already_exact(shared, key) {
        tracing::debug!(key, "skip: already exact");
        return;
    }
    // What:     `if !claims.insert(key) { return; }`. Take the fingerprint, or leave the file to the sweep that took it.
    // Why:      A refresh sweep (D113) can run beside the sweep of the load; whoever claims a fingerprint first decodes it,
    //           so no file is decoded twice for the same size and modified time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!shared.claims.add(key)) return;
    // ```
    if !claim(shared, key) {
        tracing::debug!(key, "skip: another sweep took it");
        return;
    }
    // What:     `let decision = match resolve_full(path) { ... };`. Decode the whole file.
    // Why:      A decode failure is logged, and the sweep continues with the next track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let decision; try { decision = resolveFull(path); } catch (e) { return; }
    // ```
    let decision = match resolve_full(path) {
        Ok(decision) => decision,
        Err(error) => {
            tracing::debug!(path = %path.display(), cause = %error, "skip: decode failed");
            // What:     release the claim, remember the failure, and report the row's length without a peak.
            // Why:      A failure is retried by the next sweep (D113); the row must not keep the peak of the file that was
            //           there before; the failures of one sweep are reported together when it ends.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // claims.delete(key); failed.push([path, key]); sink(lengthOnly(path));
            // ```
            release_claim(shared, key);
            if let Ok(mut failed) = shared.failed.lock() {
                failed.push(path.to_path_buf());
            }
            (shared.sink)(SweepReport::Measures {
                generation: shared.generation,
                measures: vec![(
                    path.to_path_buf(),
                    measure_from_duration(probe_duration_secs(path)),
                )],
            });
            return;
        }
    };
    shared.cache.upsert(key, decision);
    tracing::debug!(key, "measured; upsert queued");
    (shared.sink)(SweepReport::Measures {
        generation: shared.generation,
        measures: vec![(path.to_path_buf(), measure_from_decision(&decision))],
    });
}

/// What:
///      `fn already_exact(shared: &SweepShared, key: u64) -> bool`.
///  True when the cache already holds an exact decision for the fingerprint.
/// Why:
///       A sweep over the whole queue reads one snapshot of the exact fingerprints;
/// a sweep over a few changed files asks
///  the cache about each file,
/// because the snapshot of a library of a million tracks costs more than the whole refresh.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function alreadyExact(shared: SweepShared, key: bigint): boolean { return shared.listed ? cache.get(key)?.kind !== "probe" && cache.get(key) !== null : shared.known.has(key); }
/// ```
fn already_exact(
    shared: &SweepShared,
    key: u64,
) -> bool {
    if shared.listed {
        return match shared.cache.get(key) {
            Some(decision) => decision.kind != DecisionKind::ProbeEstimate,
            None => false,
        };
    }
    return shared.known.contains(&key);
}

/// What:
///      `fn claim(shared: &SweepShared, key: u64) -> bool`.
///  Takes a fingerprint for this generation;
/// true when the caller is the first to take it.
/// Why:
///       Two sweeps of one generation (the load's and a refresh) never decode the same file twice.
/// A poisoned lock counts as
///  taken by the caller,
/// so a worker never skips a file because another thread panicked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function claim(shared: SweepShared, key: bigint): boolean { if (shared.claims.has(key)) return false; shared.claims.add(key); return true; }
/// ```
fn claim(
    shared: &SweepShared,
    key: u64,
) -> bool {
    return match shared.claims.lock() {
        Ok(mut claims) => claims.insert(key),
        Err(_poisoned) => true,
    };
}

/// What:
///      `fn release_claim(shared: &SweepShared, key: u64)`.
///  Gives a fingerprint back after a failed decode.
/// Why:
///       A file that could not be decoded is tried again when the library next changes (D113),
/// so its claim must not stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function releaseClaim(shared: SweepShared, key: bigint): void { shared.claims.delete(key); }
/// ```
fn release_claim(
    shared: &SweepShared,
    key: u64,
) {
    if let Ok(mut claims) = shared.claims.lock() {
        claims.remove(&key);
    }
}

/// What:
///      `fn report_unreadable(shared: &SweepShared)`.
///  Sends the files this sweep could not decode,
/// in one report,
/// when there are any.
/// Why:
///       One report per sweep lets the controller tell the person once,
/// in one message with a count (D83,
/// D9).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportUnreadable(shared: SweepShared): void { if (shared.failed.length) shared.sink({ kind: "unreadable", files: shared.failed }); }
/// ```
fn report_unreadable(shared: &SweepShared) {
    let Ok(mut failed) = shared.failed.lock() else {
        return;
    };
    if failed.is_empty() {
        return;
    }
    let files: Vec<PathBuf> = std::mem::take(&mut *failed);
    tracing::info!(
        files = files.len(),
        generation = shared.generation,
        "the sweep could not decode some files"
    );
    (shared.sink)(SweepReport::Unreadable {
        generation: shared.generation,
        files,
    });
}

/// What:
///      `#[cfg(test)] #[path = "measure_tests.rs"] mod tests;`.
///  Declares the sibling test file.
/// Why:
///       Keep the sweep tests beside the module without inflating its code.
#[cfg(test)]
#[path = "measure_tests.rs"]
mod tests;
