//! Even window placement across a long track,
//!  in interleaved frames.
//!
//! For a track longer than the policy threshold,
//!  the policy probes a few short windows
//! instead of decoding the whole file.
//!  The windows are spread evenly from the very
//! beginning to the final legal start (`total_frames - window_frames`),
//!  so the last
//! window covers the ending:
//!  a placement that misses the final start can under-read a
//! track that peaks near its end.
//!  A single window therefore sits at the final legal start,
//!  not at frame 0:
//!  with one window there is no spacing,
//!  and the ending is the part the contract protects.
//!  Each window is measured with its own meter so the
//! discontinuity between two non-adjacent windows cannot fabricate an inter-sample
//! spike at the seam.
//!  This module is pure frame arithmetic;
//!  driving the source lives
//! in the service.

/// What:
///      `pub fn window_frames(window_seconds: f64, rate: u32) -> u64`.
///  The number
///           of per-channel frames in one probe window:
///  `floor(window_seconds * rate)`,
///           never below 1.
///  `u64` (sibling `usize` is platform-width) is explicit and
///           wide enough for any window.
/// Why:
///       Window placement and the meter both need the window length in frames;
///  the
///           floor-then-clamp keeps a tiny window or a tiny rate from yielding zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function windowFrames(windowSeconds: number, rate: number): number {
///   return Math.max(1, Math.floor(windowSeconds * rate));
/// }
/// ```
pub fn window_frames(window_seconds: f64, rate: u32) -> u64 {
    // What:     `let raw = (window_seconds * f64::from(rate)).floor();`. `f64::from(rate)`
    //           widens the `u32` rate to `f64` (no precision loss); `.floor()` rounds
    //           down to a whole frame count, still as `f64`.
    // Why:      Frames are whole; flooring matches the plan's `floor(...)`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const raw = Math.floor(windowSeconds * rate);
    // ```
    let raw = (window_seconds * f64::from(rate)).floor();
    // What:     `let frames = raw.max(0.0) as u64;`. Clamp away any negative (a bad
    //           window_seconds) before the `as u64` cast, which would otherwise
    //           saturate oddly. `as u64` truncates the already-whole `f64`.
    // Why:      Produce a non-negative integer frame count.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const frames = Math.max(0, raw);
    // ```
    let frames = raw.max(0.0) as u64;
    // What:     `frames.max(1)`. Never return fewer than one frame. Tail -> return.
    // Why:      A zero-length window cannot be measured; the plan clamps to at least 1.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return Math.max(1, frames);
    // ```
    return frames.max(1)
}

/// What:
///      `pub fn window_frame_starts(total_frames: u64, window_count: usize,
///           window_frames: u64) -> Vec<u64>`.
///  The start frame of each probe window,
///           evenly spaced from 0 to the final legal start;
///  a single window sits at the final legal start.
///  `Vec<u64>` (sibling
///           `[u64; N]` would need a const N;
///  `&[u64]` cannot own) returns one start
///           per window.
/// Why:
///       The service seeks to each start and measures one window there;
///  spreading
///           them to include both ends is what makes the probe representative.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function windowFrameStarts(totalFrames: number, windowCount: number, windowFrames: number): number[] { ... }
/// ```
pub fn window_frame_starts(total_frames: u64, window_count: usize, window_frames: u64) -> Vec<u64> {
    // What:     `if window_count == 0 { return Vec::new(); }`. `Vec::new()` is the empty
    //           owned vector (sibling `vec![]` macro form).
    // Why:      No windows requested means no starts; guard the divisor below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (windowCount === 0) return [];
    // ```
    if window_count == 0 {
        return Vec::new();
    }
    // What:     `let last_start = total_frames.saturating_sub(window_frames);`.
    //           `.saturating_sub` subtracts but floors at 0 instead of underflowing
    //           (unsigned wrap would give a huge number).
    // Why:      The final window must start exactly `window_frames` before the end so it
    //           covers the ending; a track shorter than a window starts at 0.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lastStart = Math.max(0, totalFrames - windowFrames);
    // ```
    let last_start = total_frames.saturating_sub(window_frames);
    // What:     `if window_count == 1 { return vec![last_start]; }`. `vec![last_start]` builds a
    //           one-element owned vector holding the final legal start.
    // Why:      A single window has no spacing to compute, and the contract is that the last window
    //           covers the ending: a window at frame 0 would under-read a track that peaks near its
    //           end. For a track no longer than a window `last_start` is 0, so the start is unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (windowCount === 1) return [lastStart];
    // ```
    if window_count == 1 {
        return vec![last_start];
    }
    // What:     `let divisor = (window_count - 1) as f64;`. The number of gaps between
    //           windows, widened to `f64` for the fractional placement below.
    // Why:      Evenly spacing `window_count` points across `[0, last_start]` divides by
    //           the count of gaps, not the count of points.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const divisor = windowCount - 1;
    // ```
    let divisor = (window_count - 1) as f64;
    // What:     `(0..window_count).map(|index| { ... }).collect()`. Build one start per
    //           window with a functional map over the index range, collected into a
    //           `Vec<u64>`. Tail -> return.
    // Why:      Each window's start is its fraction of the way to `last_start`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return Array.from({ length: windowCount }, (_, index) => { ... });
    // ```
    return (0..window_count)
        .map(|index| {
            // What:     `let fraction = index as f64 / divisor;`. The 0..1 position of
            //           this window: 0 at the first, 1 at the last.
            // Why:      Linear spacing across the legal range.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const fraction = index / divisor;
            // ```
            let fraction = index as f64 / divisor;
            // What:     `let start = (fraction * last_start as f64).round() as u64;`.
            //           Scale the fraction by `last_start`, round to the nearest whole
            //           frame, then `as u64` truncates the rounded value.
            // Why:      Land on an integer frame; rounding keeps placement symmetric.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const start = Math.round(fraction * lastStart);
            // ```
            let start = (fraction * last_start as f64).round() as u64;
            // What:     `start.min(last_start)`. Clamp against float rounding nudging
            //           the last start one frame past `last_start`. Tail of the closure.
            // Why:      Never seek beyond the final legal start.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return Math.min(start, lastStart);
            // ```
            return start.min(last_start)
        })
        .collect()
}

/// What:
///      `#[derive(Clone, Debug, PartialEq)] pub struct WindowPlacement { ... }`.
///  A
///           bundled probe plan:
///  the window length in frames and the per-window start
///           frames.
///  The derives give cloning,
///  debug printing,
///  and equality for tests.
/// Why:
///       The service wants both numbers together;
///  computing them in one place keeps
///           the window length and the starts consistent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WindowPlacement = { windowFrames: number; starts: number[] };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct WindowPlacement {
    /// What:
    ///      `pub window_frames: u64`.
    ///  Per-channel frames in each window.
    /// Why:
    ///       The meter reads exactly this many frames per window.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// windowFrames: number;
    /// ```
    pub window_frames: u64,
    /// What:
    ///      `pub starts: Vec<u64>`.
    ///  The start frame of each window.
    ///  `Vec<u64>`
    ///           (sibling `&[u64]` cannot own) holds one entry per window.
    /// Why:
    ///       The service seeks to each start in turn.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// starts: number[];
    /// ```
    pub starts: Vec<u64>,
}

/// What:
///      `impl WindowPlacement { ... }`.
///  The one constructor that derives a plan
///           from a track's frame count and the policy's window knobs.
/// Why:
///       Keep the floor/spacing rules in a single entry point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // static factory on WindowPlacement
/// ```
impl WindowPlacement {
    /// What:
    ///      `pub fn plan(total_frames: u64, window_count: usize, window_seconds:
    ///           f64, rate: u32) -> WindowPlacement`.
    ///  Compute the window length and the
    ///           even starts for a long track.
    /// Why:
    ///       The service calls one function to get the whole probe plan.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static plan(totalFrames, windowCount, windowSeconds, rate): WindowPlacement { ... }
    /// ```
    pub fn plan(
        total_frames: u64,
        window_count: usize,
        window_seconds: f64,
        rate: u32,
    ) -> WindowPlacement {
        // What:     `let frames = window_frames(window_seconds, rate);`. The clamped
        //           per-window frame count.
        // Why:      Both the starts and the stored plan need it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const frames = windowFrames(windowSeconds, rate);
        // ```
        let frames = window_frames(window_seconds, rate);
        // What:     `WindowPlacement { window_frames: frames, starts:
        //           window_frame_starts(total_frames, window_count, frames) }`. The
        //           struct literal, computing the starts from the same frame count.
        //           Tail -> return.
        // Why:      Hand back a consistent length-plus-starts plan.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { windowFrames: frames, starts: windowFrameStarts(totalFrames, windowCount, frames) };
        // ```
        return WindowPlacement {
            window_frames: frames,
            starts: window_frame_starts(total_frames, window_count, frames),
        }
    }
}

/// What:
///      `#[cfg(test)] #[path = "window_tests.rs"] mod tests;`.
///  Test-only submodule
///           in the sibling file `window_tests.rs`,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // window.unit.test.ts
/// ```
#[cfg(test)]
#[path = "window_tests.rs"]
mod tests;
