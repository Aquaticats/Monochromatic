//! The messages a background measurement sends back to the engine, and the job that starts one.
//!
//! What: `SweepReport` carries measures and progress tagged with the generation of the queue load
//! that started them, and `SweepJob` describes one sweep over a queue.
//! Why: sweeps are never cancelled, so the controller needs the generation to drop reports from a
//! queue that a newer load has replaced.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::peakcache::CacheHandle;
use crate::sweep_gate::SweepGate;
use crate::track_measure::TrackMeasure;

/// What:
///      `pub enum SweepReport { Measures { .. }, Progress { .. } }`.
///  One report from a sweep or a re-analysis to the engine.
/// Why:
///       Measures update the rows, and progress updates the scan bar; both carry the generation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepReport = { kind: "measures"; generation: number; measures: [string, TrackMeasure][] } | { kind: "progress"; generation: number; done: number; total: number };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum SweepReport {
    /// Measures for some library paths, in the order they were learned.
    Measures {
        /// Generation of the queue load that produced the measures.
        generation: u64,
        /// Each library path with its newest measure.
        measures: Vec<(PathBuf, TrackMeasure)>,
    },
    /// How many of the sweep's tracks have been visited out of its total.
    Progress {
        /// Generation of the queue load that started the sweep.
        generation: u64,
        /// Tracks visited so far, whether measured, skipped, or failed.
        done: usize,
        /// Tracks the sweep will visit.
        total: usize,
    },
    /// Files the sweep could not decode.
    Unreadable {
        /// Generation of the queue load that started the sweep.
        generation: u64,
        /// Each failed file.
        files: Vec<PathBuf>,
    },
    /// A refresh sweep has visited all its tracks and ended.
    RefreshDone {
        /// Generation of the queue load the sweep belongs to.
        generation: u64,
        /// The number the coalescer gave the sweep when it started it.
        id: u64,
    },
}

/// What:
///      `pub type SweepSink = Arc<dyn Fn(SweepReport) + Send + Sync>;`.
///  A cloneable function that delivers a report to the engine from any thread.
/// Why:
///       Sweep threads cannot call the engine's update callback directly, so they call this instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepSink = (report: SweepReport) => void;
/// ```
pub type SweepSink = Arc<dyn Fn(SweepReport) + Send + Sync>;

/// What:
///      `pub fn noop_sink() -> SweepSink`.
///  A sink that drops every report.
/// Why:
///       A controller built without an engine, such as in tests, has nobody to report to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noopSink: SweepSink = () => {};
/// ```
pub fn noop_sink() -> SweepSink {
    return Arc::new(|_report| {});
}

/// What:
///      `pub(crate) struct SweepJob { tracks, current, cache, sink, generation, gate, claims, quiet }`.
///  Everything one sweep needs to start.
/// Why:
///       One value keeps the spawn call small, and the generation travels with the job.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepJob = { tracks: string[]; current: string | null; cache: CacheHandle; sink: SweepSink; generation: number; claims: Set<bigint>; quiet: boolean };
/// ```
pub(crate) struct SweepJob {
    /// Every track of the queue in queue order, the current one included.
    pub(crate) tracks: Vec<PathBuf>,
    /// The track playback is on, which the fan-out leaves to the current-track worker.
    pub(crate) current: Option<PathBuf>,
    /// Handle the decisions are read from and stored through.
    pub(crate) cache: CacheHandle,
    /// Where the reports go.
    pub(crate) sink: SweepSink,
    /// Generation of the queue load that started the job.
    pub(crate) generation: u64,
    /// Pause gate the sweep's workers pass before each claim.
    pub(crate) gate: Arc<SweepGate>,
    /// Fingerprints that some sweep of this generation has taken, measured or in progress. A worker measures a track only
    /// when it is the first to take its fingerprint, so a refresh sweep that overlaps the sweep of the load never decodes
    /// the same file twice.
    pub(crate) claims: Arc<Mutex<HashSet<u64>>>,
    /// True for a refresh sweep (D113): it sends no peeked measures and no progress, only what it newly measures or fails.
    pub(crate) quiet: bool,
    /// True when `tracks` holds only the files that changed: whether a file is already measured is then asked of the cache
    /// one file at a time, because a snapshot of every measured fingerprint costs as much as the library is large.
    pub(crate) listed: bool,
    /// The tracks a quiet sweep announces by peek: new rows get their length and any cached peak at once. A loading sweep
    /// announces all of `tracks` instead.
    pub(crate) announce: Vec<PathBuf>,
    /// The number the coalescer gave a refresh sweep, sent back when it ends so the follow-up can start.
    pub(crate) refresh_id: Option<u64>,
}
