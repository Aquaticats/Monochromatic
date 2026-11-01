//! Underrun meter for the audio output.
//!  The realtime callback bumps these counters with relaxed atomics,
//!  and the engine thread reads them once a second and logs what changed.
//! Why:
//!  The callback must not format strings, lock, or log.
//!  Counting there and reporting from the engine thread keeps stutter measurable
//!  without touching the realtime path.
//!
//! Mental model for a TypeScript reader:
//!  this is a pair of counters on a shared object,
//!  `meter.shortCallbacks++` inside the audio callback,
//!  and a timer in the worker that reads them every second.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// What:
///      `pub const REPORT_INTERVAL: Duration = Duration::from_secs(1);`.
///  The shortest gap between two log lines about the meter.
/// Why:
///       Stutter reports must not flood the log; one line per second is enough to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REPORT_INTERVAL_MS = 1000;
/// ```
pub const REPORT_INTERVAL: Duration = Duration::from_secs(1);

/// What:
///      `pub struct AudioMeter { ... }`.
///  Counters the realtime callback writes and the engine thread reads.
/// Why:
///       One shared object lets the callback and the reporter both hold it without a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AudioMeter { shortCallbacks = 0; silentFrames = 0; lastRequestedFrames = 0; lastBufferFrames = 0; }
/// ```
pub struct AudioMeter {
    /// What:
    ///      `short_callbacks: AtomicU64`.
    ///  Callbacks that were playing but got fewer samples than the buffer needed.
    /// Why:
    ///       This is the ring-buffer underrun count the stutter analysis is built on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// shortCallbacks: number;
    /// ```
    short_callbacks: AtomicU64,
    /// What:
    ///      `silent_frames: AtomicU64`.
    ///  Frames of silence written while playing because the ring buffer ran short.
    /// Why:
    ///       Tells a one-sample hiccup from a whole missing buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// silentFrames: number;
    /// ```
    silent_frames: AtomicU64,
    /// What:
    ///      `last_requested_frames: AtomicU64`.
    ///  The frame count PipeWire asked for in the most recent callback.
    /// Why:
    ///       Shows the quantum the graph actually runs at, so latency requests can be checked.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastRequestedFrames: number;
    /// ```
    last_requested_frames: AtomicU64,
    /// What:
    ///      `last_buffer_frames: AtomicU64`.
    ///  The frame count of the buffer the callback filled most recently.
    /// Why:
    ///       Comparing this with `last_requested_frames` shows whether the callback overfills the graph's request.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastBufferFrames: number;
    /// ```
    last_buffer_frames: AtomicU64,
}

/// What:
///      `#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)] pub struct MeterSnapshot { ... }`.
///  A plain copy of the counters at one moment.
/// Why:
///       The reporter subtracts two snapshots to get "what happened in the last second".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MeterSnapshot = { shortCallbacks: number; silentFrames: number; lastRequestedFrames: number; lastBufferFrames: number };
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MeterSnapshot {
    /// What:
    ///      `pub short_callbacks: u64`.
    ///  Copy of the short-callback count.
    /// Why:
    ///       Readers compare it with an earlier snapshot.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// shortCallbacks: number;
    /// ```
    pub short_callbacks: u64,
    /// What:
    ///      `pub silent_frames: u64`.
    ///  Copy of the silent-frame count.
    /// Why:
    ///       Readers compare it with an earlier snapshot.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// silentFrames: number;
    /// ```
    pub silent_frames: u64,
    /// What:
    ///      `pub last_requested_frames: u64`.
    ///  The most recent requested frame count (0 before the first callback).
    /// Why:
    ///       Reported with the change line so the quantum is visible in the log.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastRequestedFrames: number;
    /// ```
    pub last_requested_frames: u64,
    /// What:
    ///      `pub last_buffer_frames: u64`.
    ///  The most recent filled buffer size (0 before the first callback).
    /// Why:
    ///       Reported with the change line so overfilling is visible in the log.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastBufferFrames: number;
    /// ```
    pub last_buffer_frames: u64,
}

impl MeterSnapshot {
    /// What:
    ///      `pub fn since(self, earlier: MeterSnapshot) -> MeterSnapshot`.
    ///  The counter changes between two snapshots; the latest buffer sizes are kept as-is.
    /// Why:
    ///       The reporter logs only what changed in the last interval.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// since(earlier: MeterSnapshot): MeterSnapshot { ... }
    /// ```
    pub fn since(self, earlier: MeterSnapshot) -> MeterSnapshot {
        return MeterSnapshot {
            short_callbacks: self.short_callbacks.saturating_sub(earlier.short_callbacks),
            silent_frames: self.silent_frames.saturating_sub(earlier.silent_frames),
            last_requested_frames: self.last_requested_frames,
            last_buffer_frames: self.last_buffer_frames,
        };
    }
}

impl AudioMeter {
    /// What:
    ///      `pub fn new() -> AudioMeter`.
    ///  Starts every counter at zero.
    /// Why:
    ///       One meter is created per output and shared with the realtime callback.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor() { ... }
    /// ```
    pub fn new() -> AudioMeter {
        return AudioMeter {
            short_callbacks: AtomicU64::new(0),
            silent_frames: AtomicU64::new(0),
            last_requested_frames: AtomicU64::new(0),
            last_buffer_frames: AtomicU64::new(0),
        };
    }

    /// What:
    ///      `pub fn record_short(&self, silent_frames: u64)`.
    ///  Counts one callback that ran short, and the frames it filled with silence.
    /// Why:
    ///       Called only while playing, so pause does not count as an underrun.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// recordShort(silentFrames: number): void { ... }
    /// ```
    pub fn record_short(&self, silent_frames: u64) {
        self.short_callbacks.fetch_add(1, Ordering::Relaxed);
        self.silent_frames.fetch_add(silent_frames, Ordering::Relaxed);
    }

    /// What:
    ///      `pub fn record_buffer(&self, requested_frames: u64, buffer_frames: u64)`.
    ///  Stores the sizes of the latest callback.
    /// Why:
    ///       Lets the log show the quantum PipeWire asked for next to the buffer we filled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// recordBuffer(requestedFrames: number, bufferFrames: number): void { ... }
    /// ```
    pub fn record_buffer(&self, requested_frames: u64, buffer_frames: u64) {
        self.last_requested_frames.store(requested_frames, Ordering::Relaxed);
        self.last_buffer_frames.store(buffer_frames, Ordering::Relaxed);
    }

    /// What:
    ///      `pub fn snapshot(&self) -> MeterSnapshot`.
    ///  Reads every counter once.
    /// Why:
    ///       The reporter takes one snapshot per interval, off the realtime thread.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot(): MeterSnapshot { ... }
    /// ```
    pub fn snapshot(&self) -> MeterSnapshot {
        return MeterSnapshot {
            short_callbacks: self.short_callbacks.load(Ordering::Relaxed),
            silent_frames: self.silent_frames.load(Ordering::Relaxed),
            last_requested_frames: self.last_requested_frames.load(Ordering::Relaxed),
            last_buffer_frames: self.last_buffer_frames.load(Ordering::Relaxed),
        };
    }
}

impl Default for AudioMeter {
    /// What:
    ///      `fn default() -> AudioMeter`.
    ///  Same as `AudioMeter::new`.
    /// Why:
    ///       Lets callers that build a meter by default (such as a stub output) use the same type.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static default(): AudioMeter { return new AudioMeter(); }
    /// ```
    fn default() -> AudioMeter {
        return AudioMeter::new();
    }
}

/// What:
///      `pub struct MeterReporter { ... }`.
///  Decides when the engine thread should log the meter, and what to log.
/// Why:
///       Logs at most once per `REPORT_INTERVAL`, and only when something changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MeterReporter { lastReport: number | null; previous: MeterSnapshot; }
/// ```
pub struct MeterReporter {
    /// What:
    ///      `last_report: Option<Instant>`.
    ///  When the meter was last read, or `None` before the first read.
    /// Why:
    ///       Enforces the once-per-second cadence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastReport: number | null;
    /// ```
    last_report: Option<Instant>,
    /// What:
    ///      `previous: MeterSnapshot`.
    ///  The counters at the last read.
    /// Why:
    ///       The next read subtracts it to get the change.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// previous: MeterSnapshot;
    /// ```
    previous: MeterSnapshot,
    /// What:
    ///      `announced: bool`.
    ///  Whether the buffer sizes have been logged once already.
    /// Why:
    ///       The sizes are logged a single time, even when nothing underruns, so the log shows the quantum.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// announced: boolean;
    /// ```
    announced: bool,
}

impl MeterReporter {
    /// What:
    ///      `pub fn new() -> MeterReporter`.
    ///  A reporter that has not read the meter yet.
    /// Why:
    ///       The first poll is always due, so the very first change is reported.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor() { this.lastReport = null; }
    /// ```
    pub fn new() -> MeterReporter {
        return MeterReporter {
            last_report: None,
            previous: MeterSnapshot::default(),
            announced: false,
        };
    }

    /// What:
    ///      `pub fn take_change(&mut self, now: Instant, read: impl FnOnce() -> MeterSnapshot) -> Option<MeterSnapshot>`.
    ///  When a read is due, reads the meter and returns the change since the last read,
    ///  but only if the underrun count moved; otherwise returns `None`.
    /// Why:
    ///       `read` is a closure so the meter is not touched when no report is due.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// takeChange(now: number, read: () => MeterSnapshot): MeterSnapshot | null { ... }
    /// ```
    pub fn take_change(&mut self, now: Instant, read: impl FnOnce() -> MeterSnapshot) -> Option<MeterSnapshot> {
        let due = match self.last_report {
            Some(last) => now.saturating_duration_since(last) >= REPORT_INTERVAL,
            None => true,
        };
        if !due {
            return None;
        }
        let current = read();
        let change = current.since(self.previous);
        self.previous = current;
        self.last_report = Some(now);
        let announce = !self.announced && current.last_buffer_frames > 0;
        if change.short_callbacks == 0 && !announce {
            return None;
        }
        self.announced = true;
        return Some(change);
    }
}

impl Default for MeterReporter {
    /// What:
    ///      `fn default() -> MeterReporter`.
    ///  Same as `MeterReporter::new`.
    /// Why:
    ///       Lets the engine create the reporter with `Default`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static default(): MeterReporter { return new MeterReporter(); }
    /// ```
    fn default() -> MeterReporter {
        return MeterReporter::new();
    }
}

#[cfg(test)]
#[path = "audio_meter_tests.rs"]
mod tests;
