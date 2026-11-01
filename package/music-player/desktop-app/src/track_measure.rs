//! Per-track measures for the library rows and the deck subtitle: the duration and the true peak.
//!
//! What: `TrackMeasure` holds one track's length and peak, and the functions here turn a cached
//! decision or a file header into that record, format the peak, and fill the template fields.
//! Why: the template engine reads `len` and `peak` from `TrackFields`, and the rows show only exact
//! peaks, so the rule lives in one pure module that the window glue calls.

use std::collections::HashMap;
use std::path::PathBuf;

use truepeak_core::{Decision, PeakKind, peak_dbtp};

use crate::template_model::TrackFields;

/// The in-place writer and the page lookup, kept in a sibling file to keep this one short.
pub use crate::track_measure_page::{apply_measures, lookup_for_positions};

/// What:
///      `const PEAK_UNIT: &str = "dBTP";`.
///  The unit printed after every peak value.
/// Why:
///       True peaks are written in dBTP, the unit the design and the default template use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PEAK_UNIT = "dBTP";
/// ```
pub const PEAK_UNIT: &str = "dBTP";

/// What:
///      `const MINUS_SIGN: &str = "\u{2212}";`.
///  The Unicode minus sign, U+2212, drawn in front of a negative peak.
/// Why:
///       The design writes negative peaks with this sign, not with a hyphen-minus.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MINUS_SIGN = "−";
/// ```
pub const MINUS_SIGN: &str = "\u{2212}";

/// What:
///      `const PLUS_SIGN: &str = "+";`.
///  The sign drawn in front of a positive peak.
/// Why:
///       A true peak can rise above zero dBTP, and the sign makes that direction plain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PLUS_SIGN = "+";
/// ```
pub const PLUS_SIGN: &str = "+";

/// What:
///      `const INFINITY_MARK: &str = "\u{221E}";`.
///  The infinity mark shown for a silent track, whose peak is minus infinity in dB.
/// Why:
///       Silence has no finite peak, and the infinity mark says so instead of a number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INFINITY_MARK = "∞";
/// ```
pub const INFINITY_MARK: &str = "\u{221E}";

/// What:
///      `const TENTHS_PER_DB: f64 = 10.0;`.
///  The number of tenths of a decibel in one decibel, the scale of one decimal place.
/// Why:
///       A peak is shown to one decimal place, so it is rounded in tenths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TENTHS_PER_DB = 10;
/// ```
const TENTHS_PER_DB: f64 = 10.0;

/// What:
///      `pub struct TrackMeasure { duration_secs, peak_dbtp, peak_exact }`.
///  The length and true peak of one track, each `None` while it is not known.
/// Why:
///       The rows need both values, and the exact flag keeps a sampled estimate out of the rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrackMeasure = { durationSecs: number | null; peakDbtp: number | null; peakExact: boolean };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrackMeasure {
    /// Duration in seconds, or `None` when the length is not known.
    pub duration_secs: Option<f64>,
    /// True peak in dBTP, or `None` when no decision is cached.
    pub peak_dbtp: Option<f64>,
    /// Whether the peak came from a full scan rather than a sampled estimate.
    pub peak_exact: bool,
}

/// What:
///      `fn valid_duration(secs: f64) -> Option<f64>`.
///  Keeps a finite, positive duration and drops anything else.
/// Why:
///       A zero or non-finite length is a missing value, not a length to show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const validDuration = (secs: number) => (Number.isFinite(secs) && secs > 0 ? secs : null);
/// ```
fn valid_duration(secs: f64) -> Option<f64> {
    return Some(secs).filter(|value| return value.is_finite() && *value > 0.0);
}

/// What:
///      `pub fn measure_from_decision(decision: &Decision) -> TrackMeasure`.
///  Builds the measure of a track from its cached decision.
/// Why:
///       The decision holds the decoded length and the linear peak, and the exact flag tells a full
///  scan from a sampled estimate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const measureFromDecision = (d: Decision): TrackMeasure => ({ durationSecs: validDuration(d.durationSecs), peakDbtp: dbtp(d.measuredPeak), peakExact: d.kind !== "probeEstimate" });
/// ```
pub fn measure_from_decision(decision: &Decision) -> TrackMeasure {
    let peak = peak_dbtp(f64::from(decision.measured_peak));
    return TrackMeasure {
        duration_secs: valid_duration(decision.duration_secs),
        peak_dbtp: Some(peak).filter(|dbtp| return !dbtp.is_nan()),
        peak_exact: decision.peak_kind() == PeakKind::Exact,
    };
}

/// What:
///      `pub fn measure_from_duration(duration_secs: Option<f64>) -> TrackMeasure`.
///  Builds a measure that carries only a length, with no peak.
/// Why:
///       A file header gives the length before any peak is measured, so rows can show it at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const measureFromDuration = (secs: number | null): TrackMeasure => ({ durationSecs: secs, peakDbtp: null, peakExact: false });
/// ```
pub fn measure_from_duration(duration_secs: Option<f64>) -> TrackMeasure {
    return TrackMeasure {
        duration_secs: duration_secs.and_then(valid_duration),
        ..TrackMeasure::default()
    };
}

/// What:
///      `pub fn format_peak_dbtp(dbtp: f64) -> String`.
///  Formats a peak with one decimal place, the minus sign for negatives, and the unit.
/// Why:
///       The rows and the deck show the same text, and a silent track reads as an infinity mark.
///  Rounding half up matches the Android formatting this follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const formatPeakDbtp = (dbtp: number) => `${sign}${abs / 10}.${abs % 10} dBTP`;
/// ```
pub fn format_peak_dbtp(dbtp: f64) -> String {
    if dbtp.is_infinite() {
        let sign = if dbtp < 0.0 { MINUS_SIGN } else { PLUS_SIGN };
        return format!("{sign}{INFINITY_MARK} {PEAK_UNIT}");
    }
    let tenths = (dbtp * TENTHS_PER_DB + 0.5).floor() as i64;
    let sign = match tenths {
        value if value < 0 => MINUS_SIGN,
        value if value > 0 => PLUS_SIGN,
        _ => "",
    };
    let magnitude = tenths.unsigned_abs();
    return format!("{sign}{}.{} {PEAK_UNIT}", magnitude / 10, magnitude % 10);
}

/// What:
///      `pub fn exact_peak_text(measure: Option<&TrackMeasure>) -> Option<String>`.
///  The formatted peak when the measure holds an exact one, or `None`.
/// Why:
///       A sampled estimate sits at or below the true peak, so the rows and the deck leave it out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exactPeakText = (m: TrackMeasure | null) => (m?.peakExact && m.peakDbtp !== null ? formatPeakDbtp(m.peakDbtp) : null);
/// ```
pub fn exact_peak_text(measure: Option<&TrackMeasure>) -> Option<String> {
    let found = measure?;
    if !found.peak_exact {
        return None;
    }
    let dbtp = found.peak_dbtp?;
    if dbtp.is_nan() {
        return None;
    }
    return Some(format_peak_dbtp(dbtp));
}

/// What:
///      `pub fn fields_with_measure(fields: TrackFields, measure: Option<&TrackMeasure>) -> TrackFields`.
///  Copies the length and the exact peak text into the template fields of one track.
/// Why:
///       The template reads `len` and `peak` from `TrackFields`, so the measure reaches the rows here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldsWithMeasure = (f: TrackFields, m: TrackMeasure | null) => ({ ...f, len: m?.durationSecs ?? null, peak: exactPeakText(m) });
/// ```
pub fn fields_with_measure(fields: TrackFields, measure: Option<&TrackMeasure>) -> TrackFields {
    return TrackFields {
        len: measure.and_then(|found| return found.duration_secs),
        peak: exact_peak_text(measure),
        ..fields
    };
}

/// What:
///      `pub fn merge_measures(current, updates) -> HashMap<PathBuf, TrackMeasure>`.
///  Returns a new map where each updated path holds its newest measure.
/// Why:
///       Updates arrive in batches and replace the earlier value of the same path, and the caller
///  keeps the previous map unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mergeMeasures = (current: Map<string, M>, updates: [string, M][]) => new Map([...current, ...updates]);
/// ```
pub fn merge_measures(
    current: &HashMap<PathBuf, TrackMeasure>,
    updates: &[(PathBuf, TrackMeasure)],
) -> HashMap<PathBuf, TrackMeasure> {
    return current
        .iter()
        .map(|(path, measure)| return (path.clone(), *measure))
        .chain(updates.iter().cloned())
        .collect();
}

/// What:
///      `pub fn lookup_by_name(names, paths, measures) -> HashMap<String, TrackMeasure>`.
///  Keys each known measure by the display name of its queue entry.
/// Why:
///       The rows and the template fields are built from display names, and the queue pairs each name
///  with its absolute path at the same position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lookupByName = (names: string[], paths: string[], m: Map<string, M>) => new Map(names.flatMap((n, i) => (m.has(paths[i]) ? [[n, m.get(paths[i])]] : [])));
/// ```
pub fn lookup_by_name(
    names: &[String],
    paths: &[PathBuf],
    measures: &HashMap<PathBuf, TrackMeasure>,
) -> HashMap<String, TrackMeasure> {
    return names
        .iter()
        .zip(paths.iter())
        .filter_map(|(name, path)| return measures.get(path).map(|measure| return (name.clone(), *measure)))
        .collect();
}

#[cfg(test)]
#[path = "track_measure_tests.rs"]
mod tests;
