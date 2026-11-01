//! The background pass that measures every line of the displayed text at the wrap width,
//!  from the viewport outward,
//! in chunks,
//! on the tile worker.
//!
//! The window thread never waits for it:
//!  a line that is not yet measured counts as one row (or its last known height),
//!  and each finished chunk corrects the vertical mapping through the keep-view-still machinery in `rows`.

/// Canonical source,
/// snapshotted as a rope clone.
use ide_app::document::Document;
/// The shaper whose cache the pass fills.
use ide_app::shaped_text::TextShaper;
/// Syntax classifications the cached rows are shaped with.
use ide_app::source_style::SourceStyles;
/// Shaping choices the worker's shaper must match.
use ide_app::source_typography::SourceTypography;
/// The slots are shared between the window thread and the worker.
use std::sync::Mutex;

/// What:
///  Lines measured by one chunk,
/// in each direction.
/// Why:
///  A chunk is about two milliseconds of shaping on an idle host,
///  short enough that a waiting tile job is served promptly,
///  long enough that the hand-over per chunk is small against the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHUNK = 128;
/// ```
pub(super) const CHUNK: usize = 128;

/// What:
///  Most lines of the pass's output the window thread takes in one poll.
/// Why:
///  The worker can run far ahead of the poll under load;
///  the window thread must never apply an unbounded amount of work in one callback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TAKE_LIMIT = 4096;
/// ```
pub(super) const TAKE_LIMIT: usize = 4096;

/// What:
///  What a measurement is valid for:
///  the file generation,
///  the text revision,
///  the display scale as exact bits,
///  and the wrap width as exact bits.
/// Why:
///  A result is applied only when all four still describe the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PassKey = { file: number; revision: number; scale: number; wrap: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PassKey {
    /// File-open generation.
    pub(super) file: u64,
    /// Content revision of the text.
    pub(super) revision: u64,
    /// Display scale as `f32::to_bits`.
    pub(super) scale: u32,
    /// Wrap width as `f32::to_bits`.
    pub(super) wrap: u32,
}

/// What:
///  An owned request to measure the whole text from line `focus` outward.
/// Why:
///  The worker must not borrow native state,
/// which lives on the window thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PassJob = { key: PassKey; generation: number; document: Document; styles: Styles; typography: Typography; scale: number; wrap: number; focus: number };
/// ```
pub(super) struct PassJob {
    /// What the measurement is valid for.
    pub(super) key: PassKey,
    /// Which post of a pass this is;
    /// the worker numbers each request it is given,
    /// and every output repeats its request's number.
    /// Two passes can have the same key (a new syntax snapshot posts the same text and width again),
    /// so the key alone cannot tell the output of a replaced pass
    /// from the output of the pass that replaced it.
    pub(super) generation: u64,
    /// Rope snapshot at the revision the key names.
    pub(super) document: Document,
    /// Syntax classifications,
    /// shared by identity.
    pub(super) styles: SourceStyles,
    /// Shaping choices the worker shaper must match.
    pub(super) typography: SourceTypography,
    /// Physical pixels per logical pixel.
    pub(super) scale: f32,
    /// Wrap width in logical pixels.
    pub(super) wrap: f32,
    /// Line the walk starts from.
    pub(super) focus: usize,
}

/// What:
///  Measured lines on their way to the window:
///  each `(line, visual rows)` pair,
/// whether the walk is complete,
/// and what it cost.
/// Why:
///  The window applies these to its table of visual rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PassOutput = { key: PassKey; generation: number; lines: [number, number][]; done: boolean; shaped: number; rebroken: number };
/// ```
pub(super) struct PassOutput {
    /// What the measurements are valid for.
    pub(super) key: PassKey,
    /// The post of the pass that measured them.
    pub(super) generation: u64,
    /// Visual rows of each measured line.
    pub(super) lines: Vec<(usize, usize)>,
    /// The walk has reached both ends of the text.
    pub(super) done: bool,
    /// Lines shaped for this output,
    /// counted for the structural tests.
    pub(super) shaped: u64,
    /// Kept rows broken again for this output,
    /// counted for the structural tests.
    pub(super) rebroken: u64,
}

/// What:
///  The slot the window writes the wanted pass into,
///  with a counter that tells the worker a newer one is there.
/// Why:
///  Only the newest wanted pass matters,
/// so a slot replaces a queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PassControl { generation = 0; job?: PassJob }
/// ```
#[derive(Default)]
pub(super) struct PassControl {
    /// Generation of the job and the job itself.
    slot: Mutex<(u64, Option<PassJob>)>,
}

/// Write and read the wanted pass.
impl PassControl {
    /// Make `job` the wanted pass.
    pub(super) fn set(&self, job: PassJob) {
        if let Ok(mut slot) = self.slot.lock() {
            slot.0 += 1;
            slot.1 = Some(job);
        }
    }

    /// The wanted pass when it is newer than `seen`,
    /// which is then brought up to date.
    pub(super) fn take_newer(&self, seen: &mut u64) -> Option<PassJob> {
        let mut slot = self.slot.lock().ok()?;
        if slot.0 == *seen {
            return None;
        }
        *seen = slot.0;
        return slot.1.take();
    }
}

/// What:
///  The slot the worker appends measured lines to and the window takes them from.
/// Why:
///  Measurements arrive far more often than the window polls,
///  so they coalesce here instead of queueing one message per chunk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PassOutbox { output?: PassOutput }
/// ```
#[derive(Default)]
pub(super) struct PassOutbox {
    /// Everything measured since the window last took.
    output: Mutex<Option<PassOutput>>,
}

/// Append and take measured lines.
impl PassOutbox {
    /// Add `more` to what waits;
    ///  output of another key replaces it.
    pub(super) fn push(&self, more: PassOutput) {
        let Ok(mut waiting) = self.output.lock() else {
            return;
        };
        match waiting.as_mut() {
            Some(held) if held.key == more.key && held.generation == more.generation => {
                held.lines.extend(more.lines);
                held.done = more.done;
                held.shaped += more.shaped;
                held.rebroken += more.rebroken;
            }
            _ => *waiting = Some(more),
        }
    }

    /// Lines waiting to be taken;
    /// test-only.
    #[cfg(test)]
    pub(super) fn waiting_lines(&self) -> usize {
        return match self.output.lock() {
            Ok(waiting) => match waiting.as_ref() {
                Some(held) => held.lines.len(),
                None => 0,
            },
            Err(_poisoned) => 0,
        };
    }

    /// Take at most `limit` measured lines;
    /// the rest stays for the next take.
    pub(super) fn take(&self, limit: usize) -> Option<PassOutput> {
        let mut waiting = self.output.lock().ok()?;
        let held = waiting.as_mut()?;
        if held.lines.len() <= limit {
            return waiting.take();
        }
        let rest = held.lines.split_off(limit);
        let taken = PassOutput {
            key: held.key,
            generation: held.generation,
            lines: std::mem::replace(&mut held.lines, rest),
            done: false,
            shaped: std::mem::take(&mut held.shaped),
            rebroken: std::mem::take(&mut held.rebroken),
        };
        return Some(taken);
    }
}

/// What:
///  A walk that has started:
///  the lines still to visit lie below `down` and above `up`.
/// Why:
///  The walk spreads from the viewport in both directions,
/// so the lines the reader needs come first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PassRun { job: PassJob; down: number; up: number; lines: number; done: boolean }
/// ```
pub(super) struct PassRun {
    /// The request being carried out.
    job: PassJob,
    /// First line not yet visited below the start.
    down: usize,
    /// First line of the visited band above the start.
    up: usize,
    /// Lines in the text.
    lines: usize,
    /// Where the walk started,
    /// the centre the cache evicts around.
    focus: usize,
}

/// Walk the text a chunk at a time.
impl PassRun {
    /// Start the walk of `job` at its focus.
    pub(super) fn new(job: PassJob) -> Self {
        let lines = job.document.text().len_lines();
        let focus = job.focus.min(lines.saturating_sub(1));
        return Self {
            job,
            down: focus,
            up: focus,
            lines,
            focus,
        };
    }

    /// Whether the walk has reached both ends of the text.
    pub(super) fn done(&self) -> bool {
        return self.down >= self.lines && self.up == 0;
    }

    /// What:
    ///  Start the walk again at `line` when that lies outside the band already visited.
    /// Why:
    ///  A tile built far from the walk means the reader jumped;
    ///  the lines around the new view come first.
    ///  Lines the cache already holds cost only a lookup on the way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refocus(line: number): void;
    /// ```
    pub(super) fn refocus(&mut self, line: usize) {
        let bounded = line.min(self.lines.saturating_sub(1));
        if bounded >= self.up && bounded < self.down {
            return;
        }
        self.down = bounded;
        self.up = bounded;
        self.focus = bounded;
    }

    /// Measure one chunk below and one above the visited band,
    /// and report what was measured.
    pub(super) fn step(&mut self, shaper: &mut TextShaper) -> PassOutput {
        let mut lines = Vec::new();
        let (mut shaped, mut rebroken) = (0, 0);
        let scale = self.job.scale;
        let wrap = Some(self.job.wrap);
        if self.down < self.lines {
            let end = (self.down + CHUNK).min(self.lines);
            let measured = shaper.measure_lines(
                &self.job.document,
                self.down..end,
                scale,
                wrap,
                &self.job.styles,
            );
            lines.extend(measured.lines);
            shaped += measured.shaped;
            rebroken += measured.rebroken;
            self.down = end;
        }
        if self.up > 0 {
            let start = self.up.saturating_sub(CHUNK);
            let measured = shaper.measure_lines(
                &self.job.document,
                start..self.up,
                scale,
                wrap,
                &self.job.styles,
            );
            lines.extend(measured.lines);
            shaped += measured.shaped;
            rebroken += measured.rebroken;
            self.up = start;
        }
        shaper.trim_rows(self.focus);
        return PassOutput {
            key: self.job.key,
            generation: self.job.generation,
            lines,
            done: self.done(),
            shaped,
            rebroken,
        };
    }
}
