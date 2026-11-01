//! Reads each track's known length and peak without decoding anything.
//!
//! What: `peek_measure` looks up a track's cached decision and, when the decision lacks a length, reads the
//! length from the file header; `peek_measures` does the same for a list of tracks.
//! Why: the window shows lengths and cached peaks as soon as the queue loads, and the slow measurement
//! starts only for tracks the cache does not hold.

use std::path::{Path, PathBuf};

use crate::duration_probe::probe_duration_secs;
use crate::peakcache::{self, CacheHandle};
use crate::track_measure::{TrackMeasure, measure_from_decision};

/// What:
///      `pub(crate) fn peek_measure(path: &Path, cache: &CacheHandle) -> TrackMeasure`.
///  One track's measure from the cache and the header, with no decode.
/// Why:
///       A cached decision carries the length and the peak; a track without one gets its length from the header.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function peekMeasure(path: string, cache: CacheHandle): TrackMeasure {
///   const decision = cache.get(fingerprint(path));
///   const base = decision ? measureFromDecision(decision) : emptyMeasure();
///   return base.durationSecs !== null ? base : { ...base, durationSecs: probeDurationSecs(path) };
/// }
/// ```
pub(crate) fn peek_measure(path: &Path, cache: &CacheHandle) -> TrackMeasure {
    // What:     `let decision = fingerprint(path).and_then(|key| cache.get(key));`. Read the cached decision.
    // Why:      A read is a single indexed lookup, so it is cheap on the sweep thread.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const decision = fingerprint(path) === null ? null : cache.get(fingerprint(path));
    // ```
    let decision = peakcache::fingerprint(path).and_then(|key| return cache.get(key));
    let measure = match decision {
        Some(found) => measure_from_decision(&found),
        None => TrackMeasure::default(),
    };
    // What:     return the measure when it has a length, otherwise add the header length.
    // Why:      The header is read only for tracks whose decision gave no length.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (measure.durationSecs !== null) return measure;
    // ```
    if measure.duration_secs.is_some() {
        return measure;
    }
    return TrackMeasure {
        duration_secs: probe_duration_secs(path),
        ..measure
    };
}

/// What:
///      `pub(crate) fn peek_measures(tracks: &[PathBuf], cache: &CacheHandle) -> Vec<(PathBuf, TrackMeasure)>`.
///  The measure of each track, paired with its path, in the same order.
/// Why:
///       The sweep reports the peeked measures in chunks, so the order must match the queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const peekMeasures = (tracks: string[], cache: CacheHandle) => tracks.map((p) => [p, peekMeasure(p, cache)]);
/// ```
pub(crate) fn peek_measures(tracks: &[PathBuf], cache: &CacheHandle) -> Vec<(PathBuf, TrackMeasure)> {
    return tracks
        .iter()
        .map(|path| return (path.clone(), peek_measure(path, cache)))
        .collect();
}

#[cfg(test)]
#[path = "measure_peek_tests.rs"]
mod tests;
