//! The shared Catmull-Rom true-peak meter.
//!
//! This is the one measurement unit that makes full-scan peaks and window-probe peaks
//! comparable:
//!  both feed the SAME meter.
//!  It reads decoded interleaved `f32` PCM,
//! keeps a four-sample sliding window per channel,
//!  and once a channel has four real
//! samples it samples the reconstructed curve at one quarter,
//!  one half,
//!  and three
//! quarters between the two middle samples,
//!  tracking the largest magnitude.
//!  The gain
//! math that turns the measured peak into a normalization gain lives in `gain.rs`.

/// What:
///      `const HALF: f32 = 1.0 / 2.0;`.
///  The fraction one-half.
///  Composed from the
///           always-allowed `-2..=2` range rather than written as a bare `0.5`
///           literal.
///  `f32` (sibling `f64`) to match the PCM sample type.
/// Why:
///       Used as the Catmull-Rom 1/2 scale factor and to build the sample offsets
///           below;
///  the repo bans bare fractional literals,
///  so it is composed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HALF = 1 / 2;
/// ```
const HALF: f32 = 1.0 / 2.0;

/// What:
///      `const QUARTER: f32 = HALF / 2.0;`.
///  One-quarter (0.25),
///  built from HALF.
///           `f32` (sibling `f64`) to match the sample type.
/// Why:
///       The first of three interior sample positions between two samples.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const QUARTER = HALF / 2;
/// ```
const QUARTER: f32 = HALF / 2.0;

/// What:
///      `const THREE_QUARTERS: f32 = HALF + QUARTER;`.
///  Three-quarters (0.75).
///           `f32` (sibling `f64`) to match the sample type.
/// Why:
///       The third interior sample position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THREE_QUARTERS = HALF + QUARTER;
/// ```
const THREE_QUARTERS: f32 = HALF + QUARTER;

/// What:
///      `const WINDOW: usize = 4;`.
///  Number of consecutive samples the cubic
///           interpolation needs (two on each side of the interval it fills).
///  `usize`
///           (siblings `u32`/`u64`) because it sizes and indexes arrays.
/// Why:
///       Catmull-Rom evaluates the curve between the 2nd and 3rd of four points.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WINDOW = 4;
/// ```
const WINDOW: usize = 4;

/// What:
///      `fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32`.
///           Evaluate the Catmull-Rom cubic through four equally-spaced points at
///           position `t` (0.0..=1.0) on the segment BETWEEN `p1` and `p2`.
///  Positional
///           params match the surrounding Rust style (Rust has no object params).
/// Why:
///       Estimates the waveform between two samples,
///  where inter-sample peaks live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function catmullRom(p0: number, p1: number, p2: number, p3: number, t: number): number { ... }
/// ```
fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    // What:     `let t2 = t * t;` and `let t3 = t2 * t;`. The square and cube of `t`.
    //           Plain float multiplies (TS-identical).
    // Why:      The cubic polynomial below uses t, t^2, t^3.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const t2 = t * t;
    // const t3 = t2 * t;
    // ```
    let t2 = t * t;
    let t3 = t2 * t;
    // What:     The Catmull-Rom basis evaluated for these four points. The literal
    //           coefficients (2, 3, 4, 5) are the standard spline matrix entries;
    //           `HALF` is the 1/2 normalization. No trailing `;`, so this multi-line
    //           expression is the tail -> return value.
    // Why:      Standard closed form (Catmull & Rom, 1974); reproduces p1 at t=0 and p2
    //           at t=1 with a smooth curve through the neighbours p0/p3.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return 0.5 * (2*p1 + (p2-p0)*t + (2*p0-5*p1+4*p2-p3)*t2 + (3*p1-3*p2+p3-p0)*t3);
    // ```
    return HALF * (2.0 * p1
        + (p2 - p0) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (3.0 * p1 - 3.0 * p2 + p3 - p0) * t3)
}

/// What:
///      `pub struct TruePeakMeter { ... }`.
///  Running state for the streaming peak
///           scan:
///  how many channels,
///  a 4-sample sliding window PER channel,
///  how many
///           real samples each channel has seen,
///  the next sample's channel index
///           (so chunk boundaries that fall mid-frame still route correctly),
///  and the
///           largest magnitude so far.
///  `pub` so the shared service and tests can feed
///           it directly.
/// Why:
///       Lets a caller scan a track chunk by chunk without holding the whole track
///           in memory (constant memory:
///  a few floats per channel).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TruePeakMeter { channels: number; win: number[][]; filled: number[]; cursor: number; peak: number; }
/// ```
pub struct TruePeakMeter {
    /// What:
    ///      `channels: usize`.
    ///  Channel count (interleave width).
    ///  `usize`
    ///           (siblings `u16`/`u32`) because it indexes the per-channel vectors.
    /// Why:
    ///       Demultiplex interleaved samples into per-channel windows.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// channels: number;
    /// ```
    channels: usize,
    /// What:
    ///      `win: Vec<[f32; WINDOW]>`.
    ///  One fixed-size array of the last 4 samples
    ///           per channel.
    ///  `[f32; 4]` is a fixed-length array (sibling `Vec<f32>`,
    ///           a growable one);
    ///  fixed because the window never changes size.
    /// Why:
    ///       Cubic interpolation needs the latest four samples of a channel.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// win: number[][]; // each inner array length 4
    /// ```
    win: Vec<[f32; WINDOW]>,
    /// What:
    ///      `filled: Vec<usize>`.
    ///  Per channel,
    ///  how many real samples have arrived
    ///           (capped at WINDOW).
    ///  `usize` counts.
    /// Why:
    ///       Only interpolate once a channel's window holds four real samples.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// filled: number[];
    /// ```
    filled: Vec<usize>,
    /// What:
    ///      `cursor: usize`.
    ///  The channel index the NEXT fed sample belongs to.
    ///           `usize` (sibling `u32`) because it indexes the per-channel vectors.
    /// Why:
    ///       A decoded chunk can end mid-frame,
    ///  so the next chunk's first sample is
    ///           not necessarily channel 0;
    ///  persisting the cursor across `feed` calls
    ///           keeps channel routing correct at chunk seams (part of `meter_id`).
    /// Gotcha:
    ///    The old per-flavor meters recomputed `index % channels` from each
    ///           chunk's local index,
    ///  silently assuming whole-frame chunks;
    ///  this field
    ///           removes that assumption.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cursor: number; // next sample's channel
    /// ```
    cursor: usize,
    /// What:
    ///      `peak: f32`.
    ///  Largest absolute sample or interpolated value seen so far.
    /// Why:
    ///       This is the measured true peak when the scan ends.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// peak: number;
    /// ```
    peak: f32,
}

/// What:
///      `impl TruePeakMeter { ... }`.
///  The meter's behaviour:
///  construction,
///  feeding
///           samples,
///  and reading the accumulated peak.
/// Why:
///       Rust separates a type's fields (the struct) from its methods (the impl).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TruePeakMeter { /* methods */ }
/// ```
impl TruePeakMeter {
    /// What:
    ///      `pub fn new(channels: usize) -> TruePeakMeter`.
    ///  Build a meter sized for
    ///           `channels` channels,
    ///  all windows zeroed,
    ///  cursor at channel 0.
    /// Why:
    ///       Starting state for a scan.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(channels: number) { ... }
    /// ```
    pub fn new(channels: usize) -> TruePeakMeter {
        // What:     `TruePeakMeter { ... }`. Struct literal. `vec![[0.0; WINDOW];
        //           channels]` builds `channels` copies of a zeroed 4-array;
        //           `vec![0; channels]` builds `channels` zero counts. Tail -> return.
        // Why:      One window and one counter per channel, peak and cursor start at 0.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { channels, win: Array.from({length: channels}, () => [0,0,0,0]),
        //          filled: new Array(channels).fill(0), cursor: 0, peak: 0 };
        // ```
        return TruePeakMeter {
            channels,
            win: vec![[0.0; WINDOW]; channels],
            filled: vec![0; channels],
            cursor: 0,
            peak: 0.0,
        }
    }

    /// What:
    ///      `pub fn feed(&mut self, chunk: &[f32])`.
    ///  Push one interleaved chunk of
    ///           samples through the meter.
    ///  `&mut self` borrows the meter mutably;
    ///           `&[f32]` is a borrowed read-only slice.
    /// Why:
    ///       Update the running peak with this block of audio,
    ///  continuing channel
    ///           routing from wherever the previous chunk ended.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// feed(chunk: number[]): void { ... }
    /// ```
    pub fn feed(&mut self, chunk: &[f32]) {
        // What:     `for &sample in chunk { ... }`. Iterate the slice; the `&sample`
        //           pattern COPIES each `f32` out by value (deref in the pattern).
        // Why:      Each sample is routed to the channel named by the running cursor.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const sample of chunk) { ... }
        // ```
        for &sample in chunk {
            // What:     `let channel = self.cursor;`. The channel this sample belongs to.
            // Why:      The cursor persists across feed calls so mid-frame seams route
            //           correctly.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const channel = this.cursor;
            // ```
            let channel = self.cursor;
            // What:     `self.push(channel, sample);`. Feed one sample to one channel.
            // Why:      Per-channel windowing and interpolation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.push(channel, sample);
            // ```
            self.push(channel, sample);
            // What:     `self.cursor = (self.cursor + 1) % self.channels;`. Advance the
            //           cursor to the next channel, wrapping back to 0 after the last.
            //           `%` is remainder.
            // Why:      Interleaved layout cycles channel 0,1,...,channels-1,0,...
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.cursor = (this.cursor + 1) % this.channels;
            // ```
            self.cursor = (self.cursor + 1) % self.channels;
        }
    }

    /// What:
    ///      `pub fn peak(&self) -> f32`.
    ///  Read the largest magnitude seen so far.
    ///           `&self` borrows the meter read-only.
    /// Why:
    ///       A streaming caller (full scan or window probe) reads the peak after
    ///           feeding all chunks without owning the meter's internals.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get peak(): number { return this.peak; }
    /// ```
    pub fn peak(&self) -> f32 {
        // What:     `self.peak`. The accumulated field as the tail expression.
        // Why:      Hand the measured true peak back to the caller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.peak;
        // ```
        return self.peak
    }

    /// What:
    ///      `pub fn take_peak(&mut self) -> f32`.
    ///  Read the largest magnitude seen
    ///           since the last take (or since construction) and RESET that running
    ///           maximum to zero,
    ///  while leaving the per-channel window,
    ///  fill counts,
    ///  and
    ///           channel cursor untouched.
    ///  `&mut self` borrows the meter mutably.
    /// Why:
    ///       Segmented measurement:
    ///  feed one segment (a bin or a window),
    ///  take its
    ///           peak,
    ///  then continue feeding the next segment with no discontinuity in
    ///           the sliding window.
    ///  Resetting only the peak (not the window) keeps the
    ///           inter-sample interpolation continuous across segment boundaries,
    ///  so the
    ///           union of segment peaks equals the continuous full-track peak exactly.
    /// Gotcha:
    ///    This resets the running max;
    ///  `peak()` after a take reflects only samples
    ///           fed since.
    ///  To keep a global maximum,
    ///  fold the returned values yourself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// takePeak(): number { const p = this.peak; this.peak = 0; return p; }
    /// ```
    pub fn take_peak(&mut self) -> f32 {
        // What:     `let peak = self.peak;`. Copy the current running max out (f32 is
        //           Copy) before clearing it.
        // Why:      We return the pre-reset value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const peak = this.peak;
        // ```
        let peak = self.peak;
        // What:     `self.peak = 0.0;`. Clear the running max; the window and cursor stay.
        // Why:      Start the next segment's peak fresh while keeping interpolation
        //           continuous.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.peak = 0;
        // ```
        self.peak = 0.0;
        // What:     `peak`. The pre-reset value as the tail expression.
        // Why:      Hand the segment peak back to the caller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return peak;
        // ```
        return peak
    }

    /// What:
    ///      `fn push(&mut self, channel: usize, sample: f32)`.
    ///  Slide one sample
    ///           into a channel's window,
    ///  update the raw peak,
    ///  and (once the window is
    ///           full) sample the interpolated curve between the two middle points.
    /// Why:
    ///       The core inter-sample peak step.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// push(channel: number, sample: number): void { ... }
    /// ```
    fn push(&mut self, channel: usize, sample: f32) {
        // What:     `let window = self.win[channel];`. COPY the 4-array out (arrays of
        //           `Copy` types are `Copy`), so we can read it without holding a
        //           borrow of `self` while we also write `self.peak` below.
        // Why:      Avoids a borrow-checker conflict between the window and the peak.
        // Gotcha:   this is a VALUE copy of the small array, not a reference; mutating
        //           `window` would not touch `self.win`. TS arrays alias by reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const window = this.win[channel].slice(); // value copy
        // ```
        let window = self.win[channel];
        // What:     `let shifted = [window[1], window[2], window[3], sample];`. The
        //           window with the oldest sample dropped and the new sample appended.
        // Why:      Maintain the last four samples in order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const shifted = [window[1], window[2], window[3], sample];
        // ```
        let shifted = [window[1], window[2], window[3], sample];
        // What:     `self.win[channel] = shifted;`. Store the advanced window.
        // Why:      Next push for this channel builds on it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.win[channel] = shifted;
        // ```
        self.win[channel] = shifted;
        // What:     `self.filled[channel] = (self.filled[channel] + 1).min(WINDOW);`.
        //           Count real samples, capping at 4. `.min(WINDOW)` clamps the count.
        // Why:      Know when four real samples are available to interpolate.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.filled[channel] = Math.min(this.filled[channel] + 1, WINDOW);
        // ```
        self.filled[channel] = (self.filled[channel] + 1).min(WINDOW);
        // What:     `let mut local_peak = sample.abs();`. Start from this sample's
        //           magnitude. `.abs()` is absolute value. `let mut` because the
        //           interior points below may raise it (mutation local to this fn).
        // Why:      The stored sample itself is a peak candidate.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let localPeak = Math.abs(sample);
        // ```
        let mut local_peak = sample.abs();
        // What:     `if self.filled[channel] == WINDOW { ... }`. Only interpolate once
        //           the window holds four real samples.
        // Why:      Cubic interpolation needs all four points to be real audio, not the
        //           initial zero padding. Adding no synthetic end padding is part of
        //           `meter_id`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.filled[channel] === WINDOW) { ... }
        // ```
        if self.filled[channel] == WINDOW {
            // What:     `for t in [QUARTER, HALF, THREE_QUARTERS] { ... }`. Evaluate the
            //           curve at three interior positions between `shifted[1]` and
            //           `shifted[2]`. Iterating an array literal BY VALUE (f32 is Copy).
            // Why:      ~4x oversampling: catch peaks that fall between stored samples.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // for (const t of [QUARTER, HALF, THREE_QUARTERS]) { ... }
            // ```
            for t in [QUARTER, HALF, THREE_QUARTERS] {
                // What:     `let interpolated = catmull_rom(shifted[0], shifted[1],
                //           shifted[2], shifted[3], t).abs();`. Interpolated magnitude
                //           at offset `t` on the middle segment.
                // Why:      A candidate inter-sample peak.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const interpolated = Math.abs(catmullRom(shifted[0], shifted[1], shifted[2], shifted[3], t));
                // ```
                let interpolated =
                    catmull_rom(shifted[0], shifted[1], shifted[2], shifted[3], t).abs();
                // What:     `local_peak = local_peak.max(interpolated);`. Keep larger.
                // Why:      Track the highest interior value.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // localPeak = Math.max(localPeak, interpolated);
                // ```
                local_peak = local_peak.max(interpolated);
            }
        }
        // What:     `self.peak = self.peak.max(local_peak);`. Fold this sample's best
        //           candidate into the running maximum.
        // Why:      The overall true peak is the max across the whole track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.peak = Math.max(this.peak, localPeak);
        // ```
        self.peak = self.peak.max(local_peak);
    }
}

/// What:
///      `pub fn true_peak_interleaved(samples: &[f32], channels: usize) -> f32`.
///           Run the streaming meter over an ALREADY-decoded slice of interleaved
///           `f32` PCM and return the measured true peak.
///  `samples: &[f32]` is a
///           borrowed,
///  read-only view;
///  `channels` is the interleave width.
/// Why:
///       A one-shot convenience for tests and synthetic on-device checks:
///  feed a
///           known signal straight into the production meter and assert the result,
///           with no decoder.
///  It shares the exact meter path the streaming scan uses.
/// Gotcha:
///    Feeds the whole slice as ONE chunk;
///  that is equivalent to many chunks,
///           because the meter's per-channel window,
///  `filled`,
///  and `cursor` state
///           persist across samples,
///  so chunk boundaries never move the result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function truePeakInterleaved(samples: Float32Array, channels: number): number { ... }
/// ```
pub fn true_peak_interleaved(samples: &[f32], channels: usize) -> f32 {
    // What:     `if channels == 0 { return 0.0; }`. Guard a zero-channel request,
    //           returning a `0.0` (silence) peak.
    // Why:      Avoids the divide-by-zero `% channels` routing in `feed`; a zero peak
    //           maps to unity gain in the gain math.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (channels === 0) return 0;
    // ```
    if channels == 0 {
        return 0.0;
    }
    // What:     `let mut meter = TruePeakMeter::new(channels);`. The running scanner;
    //           `mut` because `feed` mutates it.
    // Why:      The same accumulator the streaming scan uses.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const meter = new TruePeakMeter(channels);
    // ```
    let mut meter = TruePeakMeter::new(channels);
    // What:     `meter.feed(samples);`. Push the whole interleaved slice through the
    //           meter in one call (`feed` borrows it read-only).
    // Why:      Update the running peak across every sample.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // meter.feed(samples);
    // ```
    meter.feed(samples);
    // What:     `meter.peak()`. The accumulated peak as the tail expression.
    // Why:      Hand the measured true peak back to the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return meter.peak;
    // ```
    return meter.peak()
}

/// What:
///      `#[cfg(test)] #[path = "meter_tests.rs"] mod tests;` declares a test-only
///           submodule whose code lives in the sibling file `meter_tests.rs`.
///           `#[cfg(test)]` gates it to test builds;
///  `#[path = "..."]` aims the module
///           at a flat sibling file.
///  The file stays the `tests` child of meter,
///  so its
///           `use super::*` reaches the module's private items unchanged.
/// Why:
///       Keep `meter.rs` to production code;
///  tests live beside it without inflating
///           this file or its max-lines budget (sibling `*_tests.rs` files are exempt).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // meter.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "meter_tests.rs"]
mod tests;
