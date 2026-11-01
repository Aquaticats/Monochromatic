//! The decoded-audio source contract and the `AudioSpec` describing a stream.
//!
//! Stage one keeps decoding in the platform crates:
//!  each flavor opens a file with its
//! own decoder and hands the shared core an implementation of [`TruePeakSource`] over
//! interleaved `f32` PCM.
//!  The desktop and Android `Source` traits are already this
//! shape,
//!  except seeking:
//!  this trait seeks by FRAME,
//!  not seconds,
//!  because reproducible
//! window placement needs the runtime and the bench sidecar to land on the same frame.

/// What:
///      `use crate::error::TruePeakError;`.
///  The crate-owned error the fallible
///           methods return.
///  `crate::` means the root of this crate.
/// Why:
///       `next_chunk` and `seek_to_frame` return `Result<_, TruePeakError>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TruePeakError } from "./error";
/// ```
use crate::error::TruePeakError;

/// What:
///      `#[derive(Clone, Copy, Debug, PartialEq)]` auto-generates value copying,
///           bitwise copy (so passing it never moves it),
///  debug printing,
///  and
///           equality.
///  `pub struct AudioSpec { ... }` is a small record describing a
///           decoded stream:
///  sample rate,
///  channel count,
///  and track length.
/// Why:
///       The meter and window math need the rate and channel count,
///  and the policy
///           branch needs the duration;
///  this is the minimal shared descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AudioSpec = { rate: number; channels: number; durationSecs: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioSpec {
    /// What:
    ///      `pub rate: u32`.
    ///  Samples per second per channel (e.g. 44100,
    ///  48000).
    ///           `u32` (siblings `u16` too narrow at 96000+,
    ///  `u64`/`usize` overkill,
    ///           `i32` never negative).
    /// Why:
    ///       Convert seconds to frames for window placement and report the stream.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rate: number;
    /// ```
    pub rate: u32,
    /// What:
    ///      `pub channels: u16`.
    ///  Channel count (1 = mono,
    ///  2 = stereo).
    ///  `u16`
    ///           (sibling `u8` would cap at 255,
    ///  fine in practice,
    ///  but `u16` matches
    ///           the platform `AudioSpec`;
    ///  `usize` would invite index confusion).
    /// Why:
    ///       The meter demultiplexes interleaved samples by channel count.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// channels: number;
    /// ```
    pub channels: u16,
    /// What:
    ///      `pub duration_secs: f64`.
    ///  Track length in seconds.
    ///  `f64` (sibling
    ///           `f32` loses precision on long tracks) to match the platform spec.
    /// Why:
    ///       Duration drives the policy branch (short exact vs long probe).
    /// Gotcha:
    ///    A non-finite or non-positive value means "unknown duration";
    ///  the
    ///           policy then forces a full exact scan (see `duration_known`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// durationSecs: number;
    /// ```
    pub duration_secs: f64,
}

/// What:
///      `impl AudioSpec { ... }`.
///  Small derived helpers over the three fields.
/// Why:
///       The degenerate-duration rule lives in one place rather than scattered
///           across the policy branch and the bench sidecar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // methods on the AudioSpec shape
/// ```
impl AudioSpec {
    /// What:
    ///      `pub fn duration_known(&self) -> bool`.
    ///  True only when the reported
    ///           duration is finite and strictly positive.
    ///  `&self` borrows read-only.
    /// Why:
    ///       Unknown or non-positive duration cannot pick a policy branch,
    ///  so the
    ///           service treats it as "full scan exact" (a plan amendment).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// durationKnown(): boolean { return Number.isFinite(this.durationSecs) && this.durationSecs > 0; }
    /// ```
    pub fn duration_known(&self) -> bool {
        // What:     `self.duration_secs.is_finite() && self.duration_secs > 0.0`.
        //           `.is_finite()` rejects NaN and the infinities; `> 0.0` rejects
        //           zero and negatives. Tail expression -> return.
        // Why:      Only a real positive length is usable for the short/long branch.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Number.isFinite(this.durationSecs) && this.durationSecs > 0;
        // ```
        return self.duration_secs.is_finite() && self.duration_secs > 0.0
    }
}

/// What:
///      `pub trait TruePeakSource: Send { ... }`.
///  An interface (set of method
///           signatures) any decoder type implements to feed the shared meter.
///  The
///           `: Send` supertrait bound means a value of the type can be moved to
///           another thread (the background warming worker owns its source on a worker
///           thread).
///  Sibling bounds the reader might expect:
///  `Sync` (shared across
///           threads,
///  not needed here),
///  `'static` (no borrowed data,
///  added by the
///           service where required).
/// Why:
///       The shared core measures without knowing which decoder produced the
///           samples;
///  the platform supplies the decoder behind this trait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TruePeakSource {
///   spec(): AudioSpec;
///   nextChunk(): number[];        // throws on decode error
///   seekToFrame(frame: number): void; // throws on seek error
/// }
/// ```
pub trait TruePeakSource: Send {
    /// What:
    ///      `fn spec(&self) -> AudioSpec;`.
    ///  A method signature (no body;
    ///  the
    ///           implementor provides it) returning the stream descriptor.
    ///  `&self`
    ///           borrows the source read-only so querying does not consume it.
    /// Why:
    ///       The meter needs rate and channels;
    ///  the policy needs duration.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec(): AudioSpec;
    /// ```
    fn spec(&self) -> AudioSpec;

    /// What:
    ///      `fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError>;`.
    ///  Decode
    ///           the next block of interleaved `f32` PCM.
    ///  `&mut self` borrows mutably
    ///           (decoding advances the source).
    ///  An EMPTY returned `Vec` signals
    ///           end-of-stream.
    ///  `Vec<f32>` (sibling `&[f32]`) is owned because the
    ///           decoder allocates the block.
    /// Why:
    ///       The meter feeds these blocks one at a time until the stream ends.
    /// Gotcha:
    ///    A chunk MAY end mid-frame;
    ///  the meter's channel cursor handles that,
    ///  so
    ///           an adapter need not align chunks to frame boundaries.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextChunk(): number[]; // empty array means EOF; throws on decode error
    /// ```
    fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError>;

    /// What:
    ///      `fn seek_to_frame(&mut self, frame: u64) -> Result<(), TruePeakError>;`.
    ///           Position the source so the NEXT `next_chunk` begins exactly at
    ///           interleaved frame `frame` (one frame = one sample per channel).
    ///           `frame: u64` (sibling `usize` is platform-width;
    ///  `u64` is explicit and
    ///           wide enough for any track).
    ///  `Result<(), E>` returns nothing on success.
    /// Why:
    ///       Window probing seeks to precise frame offsets;
    ///  seconds-based seeking
    ///           is too loose,
    ///  so the runtime and the bench sidecar agree on the frame.
    /// Gotcha:
    ///    The adapter MUST land exactly on `frame`:
    ///  seek at or before it,
    ///  then
    ///           discard decoded frames up to it.
    ///  Approximate landing would make the
    ///           runtime and the bench measure different windows near the final start.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seekToFrame(frame: number): void; // exact landing; throws on seek error
    /// ```
    fn seek_to_frame(&mut self, frame: u64) -> Result<(), TruePeakError>;
}

/// What:
///      `#[cfg(test)] #[path = "source_tests.rs"] mod tests;`.
///  Test-only submodule
///           in the sibling file `source_tests.rs`,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // source.unit.test.ts
/// ```
#[cfg(test)]
#[path = "source_tests.rs"]
mod tests;
