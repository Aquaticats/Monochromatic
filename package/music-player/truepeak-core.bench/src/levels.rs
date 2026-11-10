//! The bench's dBTP conversion:
//!  one owner for the silence guard every scoring module needs.
//!
//! What:
//!  `truepeak_core::peak_dbtp` converts a linear peak to decibels,
//!  and a non-positive peak makes its logarithm NaN or negative infinity,
//! so the guard belongs beside the call.
//! Why:
//!  every report compares levels,
//!  and the modules used to carry five hand-copied guards;
//!  a change to the silence convention now lands in one place.

/// Imports the shared dB conversion the guard wraps.
use truepeak_core::peak_dbtp;

/// Convert a linear peak to dBTP,
///  treating silence as a very negative level.
///
/// What:
///  `peak_dbtp(peak)` for a positive peak,
/// `f64::NEG_INFINITY` otherwise.
/// Why:
///  `log10` of a non-positive peak is NaN or negative infinity,
///  and a NaN compared against any bound is false,
///  which would silently drop a silent track from every count the reports make.
pub(crate) fn peak_dbtp_or_silence(peak: f64) -> f64 {
    if peak <= 0.0 {
        return f64::NEG_INFINITY
    } else {
        return peak_dbtp(peak)
    }
}

/// What:
///  `#[cfg(test)] #[path = "levels_tests.rs"] mod tests;`.
///  Test-only submodule in the sibling file,
///  gated to test builds.
/// Why:
///  keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
#[cfg(test)]
#[path = "levels_tests.rs"]
mod tests;
