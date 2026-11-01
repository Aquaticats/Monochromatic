//! Attenuate-only normalization gain and the dB helpers around it.
//!
//! Production playback multiplies every sample by one constant gain that brings the
//! track's measured true peak down to a -1 dBTP ceiling and never amplifies.
//!  That gain
//! is `normalization_gain`,
//!  kept in linear `f32` to match the meter and the audio
//! output.
//!  The dB helpers (`peak_dbtp`,
//!  `probe_estimated_peak`) are `f64` because the
//! classifier and bench reason in decibels,
//!  where the plan's gain-error bounds live.

/// What:
///      `pub const CEILING: f32 = 0.891_250_9;`.
///  The true-peak target as a linear
///           amplitude:
///  10^(-1/20),
///  i.e. -1 dBTP,
///  written at the precision an `f32`
///           actually represents (the `_` digit separators are ignored;
///  sibling `f64`
///           is used by the dB helpers,
///  `f32` here matches the meter's sample type).
///           This is the same value the desktop crate's `CEILING` already uses,
///  so the
///           shared gain is identical to what desktop shipped.
/// Why:
///       The level every track's true peak is normalized down to;
///  -1 dBTP is the
///           EBU R128 / ATSC A/85 ceiling that leaves room for DAC reconstruction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CEILING = 10 ** (-1 / 20); // -1 dBTP approximately 0.8912509
/// ```
pub const CEILING: f32 = 0.891_250_9;

/// What:
///      `const DBTP_SCALE: f64 = 20.0;`.
///  The voltage-decibel factor in
///           `20 * log10(amplitude)`.
///  `f64` (sibling `f32`) for dB precision.
/// Why:
///       Named so the magic `20` does not appear bare in the dB formulas.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DBTP_SCALE = 20;
/// ```
const DBTP_SCALE: f64 = 20.0;

/// What:
///      `const DB_BASE: f64 = 10.0;`.
///  The base for converting decibels back to a
///           linear ratio (`10^(db/20)`).
///  `f64` (sibling `f32`) for dB precision.
/// Why:
///       Named so the magic `10` does not appear bare in the inflation formula.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DB_BASE = 10;
/// ```
const DB_BASE: f64 = 10.0;

/// What:
///      `pub fn normalization_gain(true_peak: f32) -> f32`.
///  Turn a measured true
///           peak into the constant gain that brings it to the ceiling,
///  never
///           amplifying (gain capped at 1.0).
/// Why:
///       Attenuate-only normalization prevents inter-sample overflow without ever
///           boosting a quiet track,
///  which would risk a sudden loud,
///  possibly harmful,
///           level and is outside the clipping-prevention intent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function normalizationGain(truePeak: number): number {
///   if (truePeak <= 0) return 1;
///   return Math.min(CEILING / truePeak, 1);
/// }
/// ```
pub fn normalization_gain(true_peak: f32) -> f32 {
    // What:     `if true_peak <= 0.0 { return 1.0; }`. A silent or invalid measurement
    //           leaves the signal unchanged.
    // Why:      Avoid dividing by zero and avoid amplifying silence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (truePeak <= 0) return 1;
    // ```
    if true_peak <= 0.0 {
        return 1.0;
    }
    // What:     `(CEILING / true_peak).min(1.0)`. The gain that scales the peak to the
    //           ceiling, `.min(1.0)` clamps it so it never exceeds 1.0 (no boost). Tail
    //           -> return.
    // Why:      Louder-than-ceiling tracks are attenuated to the ceiling; quieter
    //           tracks are left as-is (gain 1.0).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return Math.min(CEILING / truePeak, 1);
    // ```
    return (CEILING / true_peak).min(1.0)
}

/// What:
///      `pub fn peak_dbtp(peak: f64) -> f64`.
///  Convert a linear amplitude to
///           decibels true peak relative to full scale (`20 * log10(peak)`),
///  so a peak
///           of 1.0 is 0 dBTP.
///  `f64` (sibling `f32`) for dB precision.
/// Why:
///       The classifier and bench label tracks and check the gain-error bounds in
///           dB;
///  this is the conversion the plan's formulas use.
/// Gotcha:
///    `log10(0)` is negative infinity;
///  the caller treats a non-positive peak as
///           silence (unity gain) before reaching here,
///  so guard upstream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function peakDbtp(peak: number): number { return 20 * Math.log10(peak); }
/// ```
pub fn peak_dbtp(peak: f64) -> f64 {
    // What:     `DBTP_SCALE * peak.log10()`. `.log10()` is base-10 log on `f64`. Tail
    //           expression -> return.
    // Why:      The standard amplitude-to-dB conversion.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return DBTP_SCALE * Math.log10(peak);
    // ```
    return DBTP_SCALE * peak.log10()
}

/// What:
///      `pub fn probe_estimated_peak(sampled_max_peak: f64, margin_db: f64) -> f64`.
///           Inflate a windowed (sampled) maximum peak by `margin_db` decibels and
///           return the estimated linear peak.
///  `f64` (sibling `f32`) for dB precision.
/// Why:
///       A probe never sees the whole track,
///  so the policy adds a fixed safety
///           margin before deciding the gain;
///  this is `sampled_max_dbtp + margin_db`
///           expressed back in linear amplitude,
///  the input to `normalization_gain`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function probeEstimatedPeak(sampledMax: number, marginDb: number): number {
///   return sampledMax * 10 ** (marginDb / 20);
/// }
/// ```
pub fn probe_estimated_peak(sampled_max_peak: f64, margin_db: f64) -> f64 {
    // What:     `sampled_max_peak * DB_BASE.powf(margin_db / DBTP_SCALE)`. `.powf(x)`
    //           raises `DB_BASE` (10.0) to a fractional power. Multiplying scales the
    //           linear peak by the dB margin's linear ratio. Tail -> return.
    // Why:      Adding `margin_db` in the dB domain equals multiplying by
    //           `10^(margin_db/20)` in the linear domain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return sampledMax * DB_BASE ** (marginDb / DBTP_SCALE);
    // ```
    return sampled_max_peak * DB_BASE.powf(margin_db / DBTP_SCALE)
}

/// What:
///      `#[cfg(test)] #[path = "gain_tests.rs"] mod tests;`.
///  Test-only submodule
///           in the sibling file `gain_tests.rs`,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // gain.unit.test.ts
/// ```
#[cfg(test)]
#[path = "gain_tests.rs"]
mod tests;
