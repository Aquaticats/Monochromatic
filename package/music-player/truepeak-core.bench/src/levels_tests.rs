//! Tests for the one dBTP conversion the bench's scoring modules call.

use super::*;

/// A peak at full scale,
///  which is the dBTP reference point.
const FULL_SCALE: f64 = 1.0;
/// The loudest level a score treats as positive instead of silent.
const SILENCE: f64 = 0.0;
/// A negative peak,
///  which no meter emits but a subtraction can produce.
const NEGATIVE_PEAK: f64 = -0.25;
/// A peak half of full scale,
///  which is about minus six dBTP.
const HALF_SCALE: f64 = 0.5;
/// A quiet peak a probe window may sample.
const QUIET_PEAK: f64 = 0.05;

/// What:
///  a positive peak converts to the shared dBTP value,
///  so the guard changes nothing for real measurements.
#[test]
fn a_positive_peak_converts_to_the_shared_dbtp() {
    assert_eq!(peak_dbtp_or_silence(FULL_SCALE), 0.0, "full scale is 0 dBTP");
    for peak in [HALF_SCALE, QUIET_PEAK] {
        assert_eq!(peak_dbtp_or_silence(peak), peak_dbtp(peak), "peak {peak}");
    }
}

/// What:
///  silence and negative peaks become a very negative level,
///  never NaN,
///  so every later comparison keeps its answer.
#[test]
fn a_non_positive_peak_is_very_negative_and_never_nan() {
    for peak in [SILENCE, NEGATIVE_PEAK] {
        let level = peak_dbtp_or_silence(peak);
        assert_eq!(level, f64::NEG_INFINITY, "peak {peak}");
        assert!(!level.is_nan(), "peak {peak} must not be NaN");
    }
}

/// What:
///  the conversion keeps the order of the peaks it converts,
///  so a report's maxima and minima stay the signal's.
#[test]
fn the_conversion_keeps_the_order_of_the_peaks_it_converts() {
    let quiet = peak_dbtp_or_silence(QUIET_PEAK);
    let half = peak_dbtp_or_silence(HALF_SCALE);
    let full = peak_dbtp_or_silence(FULL_SCALE);
    assert!(quiet < half && half < full, "{quiet} < {half} < {full}");
    assert!(peak_dbtp_or_silence(SILENCE) < quiet, "silence is the floor");
}
