//! Shaped source rows kept by line for one text revision and display scale,
//!  each broken at the wrap width it was last broken at.

/// What:
///  `ShapedRow` is the shaped line a cache entry keeps;
///  `BTreeMap` maps a line index to its entry in line order.
/// Why:
///  A row is looked up by line,
///  and the lines farthest from the view are the first and last keys,
///  so eviction needs no sort.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { ShapedRow } from './shaped-row';
/// ```
use crate::shaped_row::ShapedRow;
/// Lines are keyed by their index and compared by their distance from the view's centre.
use std::collections::BTreeMap;

/// What:
///  Most lines a window-thread cache keeps;
///  the lines farthest from the view are evicted first.
/// Why:
///  A tile of three view heights holds about a hundred lines at typical sizes,
///  so a few hundred lines let a re-centered tile reuse the lines it shares with the previous one.
///  The window thread also builds a tile at once when the width changes,
/// for as many lines as it may spend work on
///  (`native::rewrap::REWRAP_LINES`),
/// and a cache smaller than that tile would evict the lines it just broke.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROW_CACHE_LINES = 512;
/// ```
pub const ROW_CACHE_LINES: usize = 512;

/// What:
///  The shaped rows of a text,
/// keyed by line,
///  with the text revision and display scale they were shaped for.
/// Why:
///  Shaping is the expensive part of a tile;
///  a row shaped for the same text and scale is identical to a new one,
///  except for its vertical position,
///  which the caller sets from the current vertical mapping.
///  The wrap width is a property of each row,
///  because a resize re-breaks rows one chunk at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RowCache = { key?: [revision: number, scale: number]; capacity: number; rows: Map<number, ShapedRow> };
/// ```
pub struct RowCache {
    /// Text revision and the scale's exact bits;
    ///  `None` before the first sync.
    key: Option<(u64, u32)>,
    /// Most rows kept.
    capacity: usize,
    /// Shaped rows by line,
    ///  each with its top left at zero.
    rows: BTreeMap<usize, ShapedRow>,
}

/// A cache of the window thread's size.
impl Default for RowCache {
    /// A cache that keeps [`ROW_CACHE_LINES`] lines.
    fn default() -> Self {
        return Self::with_capacity(ROW_CACHE_LINES);
    }
}

/// Wrap widths are compared by their exact bits.
fn same_wrap(left: Option<f32>, right: Option<f32>) -> bool {
    return left.map(f32::to_bits) == right.map(f32::to_bits);
}

/// Cache operations,
///  each judged against the text revision and scale the cache was synced to.
impl RowCache {
    /// A cache that keeps at most `capacity` lines.
    pub fn with_capacity(capacity: usize) -> Self {
        return Self {
            key: None,
            capacity: capacity.max(1),
            rows: BTreeMap::new(),
        };
    }

    /// Change how many lines the cache keeps;
    ///  rows beyond it are evicted by `trim`.
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
    }

    /// Number of rows kept.
    pub fn len(&self) -> usize {
        return self.rows.len();
    }

    /// Whether no row is kept.
    pub fn is_empty(&self) -> bool {
        return self.rows.is_empty();
    }

    /// Empty the cache unless it was shaped for this text revision and scale.
    pub fn sync(&mut self, revision: u64, scale: f32) {
        // What: `to_bits` compares the scale exactly, so two equal scales always produce the same key.
        // Why: A scale change alters every glyph advance, so no row shaped at another scale may be reused.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const key = [revision, scale];
        // ```
        let key = (revision, scale.to_bits());
        if self.key != Some(key) {
            self.rows.clear();
            self.key = Some(key);
        }
    }

    /// The shaped row of `line`,
    /// if it was shaped for the synced text and scale and broken at `wrap`.
    pub fn get(&self, line: usize, wrap: Option<f32>) -> Option<&ShapedRow> {
        let row = self.rows.get(&line)?;
        if same_wrap(row.wrap, wrap) {
            return Some(row);
        }
        return None;
    }

    /// Whether a row is kept for `line` at any width.
    pub fn contains(&self, line: usize) -> bool {
        return self.rows.contains_key(&line);
    }

    /// Keep a freshly shaped row under its line.
    pub fn insert(&mut self, row: ShapedRow) {
        self.rows.insert(row.row, row);
    }

    /// Take the row of `line` out of the cache.
    pub fn remove(&mut self, line: usize) -> Option<ShapedRow> {
        return self.rows.remove(&line);
    }

    /// What:
    ///  Evict the lines farthest from `centre` until no more than the capacity remain.
    /// Why:
    ///  The rows near the view are the ones the next re-center needs;
    ///  far rows would only waste memory.
    ///  The farthest line is always the first or the last key.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function trim(centre: number): void;
    /// ```
    pub fn trim(&mut self, centre: usize) {
        while self.rows.len() > self.capacity {
            let first = self.rows.keys().next().copied();
            let last = self.rows.keys().next_back().copied();
            let (Some(low), Some(high)) = (first, last) else {
                return;
            };
            // `abs_diff` is the distance between two line indices without underflow.
            let farthest = if low.abs_diff(centre) >= high.abs_diff(centre) {
                low
            } else {
                high
            };
            self.rows.remove(&farthest);
        }
    }

    /// Forget every row and the key,
    ///  after a change the key does not describe,
    ///  such as new styles.
    pub fn clear(&mut self) {
        self.rows.clear();
        self.key = None;
    }
}
