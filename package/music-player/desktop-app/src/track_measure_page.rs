//! The measure operations the window runs on every sweep chunk and every projection, which must cost the size of a
//! chunk or a page and not the size of the library.
//!
//! What: `apply_measures` writes sweep reports into the measure map in place; `lookup_for_positions` reads the measures
//! of one page's queue positions.
//! Why: at one million tracks a copy of the map per chunk, or a walk of the whole queue per projection, cost more than
//! the sweep or the page itself. `track_measure` re-exports both, so callers keep one import path.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::track_measure::TrackMeasure;

/// What:
///      `pub fn apply_measures(current: &mut HashMap<PathBuf, TrackMeasure>, updates: &[(PathBuf, TrackMeasure)])`.
///  Writes each update into the map in place; an update replaces a measure for the same path.
/// Why:
///       `merge_measures` copies the whole map, and the sweep reports in chunks of 128 tracks, so at one million
///  tracks the copies alone cost more than the sweep; this costs the size of the chunk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const applyMeasures = (current: Map<string, M>, updates: [string, M][]) => { for (const [p, m] of updates) current.set(p, m); };
/// ```
pub fn apply_measures(current: &mut HashMap<PathBuf, TrackMeasure>, updates: &[(PathBuf, TrackMeasure)]) {
    current.reserve(updates.len());
    for (path, measure) in updates {
        current.insert(path.clone(), *measure);
    }
}

/// What:
///      `pub fn lookup_for_positions(positions, names, paths, measures) -> HashMap<String, TrackMeasure>`.
///  Keys the known measures of the given queue positions by display name.
/// Why:
///       The window shows one page, so only that page's measures are needed; `lookup_by_name` walks the whole queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lookupForPositions = (pos: number[], names: string[], paths: string[], m: Map<string, M>) => new Map(pos.flatMap((i) => (m.has(paths[i]) ? [[names[i], m.get(paths[i])]] : [])));
/// ```
pub fn lookup_for_positions(
    positions: &[usize],
    names: &[String],
    paths: &[PathBuf],
    measures: &HashMap<PathBuf, TrackMeasure>,
) -> HashMap<String, TrackMeasure> {
    return positions
        .iter()
        .filter_map(|position| {
            let measure: &TrackMeasure = measures.get(paths.get(*position)?)?;
            return Some((names.get(*position)?.clone(), *measure));
        })
        .collect();
}
