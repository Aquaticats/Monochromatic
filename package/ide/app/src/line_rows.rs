//! How many visual rows each source line takes,
//!  kept so that a change of a few lines costs little and a copy costs almost nothing.
//!
//! A line that is not wrapped,
//!  or not yet measured,
//!  takes one row.
//!  The counts live in chunks of [`CHUNK`] lines that are shared between copies,
//!  with the number of extra rows before each chunk summed beside them,
//!  so reading the rows above a line is one lookup and one short sum,
//!  and a copy of the whole table only copies one pointer per chunk.

/// Shared chunks are cloned on write by `Arc::make_mut`.
use std::sync::Arc;

/// What:
///  Lines per chunk;
///  `usize` is the address-sized index type (siblings `u32`,
///  `u64`).
/// Why:
///  A chunk of 256 four-byte counts is one kilobyte,
///  so changing a line copies one kilobyte,
///  and a table for a million lines holds about four thousand pointers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHUNK = 256;
/// ```
const CHUNK: usize = 256;

/// The extra rows (rows beyond the first) of `CHUNK` consecutive lines.
type Chunk = [u32; CHUNK];

/// What:
///  The visual row count of every line of a text.
///  `Vec<Arc<Chunk>>` is a list of shared,
///  immutable-until-changed chunks.
/// Why:
///  The vertical mapping needs the rows above any line at once,
///  and the background pass changes a few lines at a time while copies of the table travel to the worker
///  and into every frame.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LineRows = { lines: number; chunks: Readonly<Uint32Array>[]; before: number[] };
/// ```
#[derive(Clone, Debug)]
pub struct LineRows {
    /// Number of lines described;
    ///  at least one.
    lines: usize,
    /// Extra rows of each line,
    ///  `CHUNK` lines to a chunk.
    chunks: Vec<Arc<Chunk>>,
    /// Extra rows in all chunks before chunk `i`;
    ///  one entry more than there are chunks,
    ///  the last being the total.
    before: Vec<usize>,
}

/// An empty table describes one line of one row.
impl Default for LineRows {
    /// The table of a one-line text.
    fn default() -> Self {
        return Self::plain(1);
    }
}

/// Tables are equal when they describe the same counts;
///  shared chunks are compared by identity first.
impl PartialEq for LineRows {
    /// Same line count,
    ///  same totals,
    ///  and chunk by chunk the same counts.
    fn eq(&self, other: &Self) -> bool {
        if self.lines != other.lines || self.before != other.before {
            return false;
        }
        // Shared chunks are equal by identity; others are compared by their counts.
        for (left, right) in self.chunks.iter().zip(&other.chunks) {
            if !(Arc::ptr_eq(left, right) || **left == **right) {
                return false;
            }
        }
        return true;
    }
}

/// Build and read the table.
impl LineRows {
    /// What:
    ///  A table of `lines` lines that all take one row;
    ///  an empty text still has one line.
    /// Why:
    ///  Nothing is wrapped until a line has been measured,
    ///  and an unmeasured line counts as one row.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static plain(lines: number): LineRows;
    /// ```
    pub fn plain(lines: usize) -> Self {
        let described = lines.max(1);
        let count = described.div_ceil(CHUNK);
        let empty = Arc::new([0_u32; CHUNK]);
        return Self {
            lines: described,
            chunks: vec![empty; count],
            before: vec![0; count + 1],
        };
    }

    /// Number of lines described.
    pub fn lines(&self) -> usize {
        return self.lines;
    }

    /// Extra rows of `line`,
    /// nothing for a line the table does not have.
    fn extra(&self, line: usize) -> u32 {
        if line >= self.lines {
            return 0;
        }
        return self.chunks[line / CHUNK][line % CHUNK];
    }

    /// Visual rows of `line`;
    ///  a line the table does not have counts as one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rows(line: number): number;
    /// ```
    pub fn rows(&self, line: usize) -> usize {
        return 1 + self.extra(line) as usize;
    }

    /// Extra rows of every line before `line`;
    ///  `line` may equal the line count,
    ///  which gives the total.
    pub fn extra_before(&self, line: usize) -> usize {
        let bounded = line.min(self.lines);
        let chunk = bounded / CHUNK;
        let within = bounded % CHUNK;
        let mut sum = self.before[chunk];
        if within > 0 {
            for extra in &self.chunks[chunk][..within] {
                sum += *extra as usize;
            }
        }
        return sum;
    }

    /// Extra rows of the whole text.
    pub fn total_extra(&self) -> usize {
        return self.before[self.chunks.len()];
    }

    /// What:
    ///  Set the row counts of the given `(line, rows)` pairs and answer whether any count changed.
    ///  A count below one is one,
    ///  and a line the table does not have is skipped.
    /// Why:
    ///  Only chunks that changed are copied,
    ///  and the sums are recomputed once from the first chunk that changed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// apply(updates: Iterable<[line: number, rows: number]>): boolean;
    /// ```
    pub fn apply(&mut self, updates: impl IntoIterator<Item = (usize, usize)>) -> bool {
        let mut first_changed: Option<usize> = None;
        for (line, rows) in updates {
            if line >= self.lines {
                continue;
            }
            let wanted = u32::try_from(rows.max(1) - 1).unwrap_or(u32::MAX);
            if self.extra(line) == wanted {
                continue;
            }
            let chunk = line / CHUNK;
            Arc::make_mut(&mut self.chunks[chunk])[line % CHUNK] = wanted;
            first_changed = match first_changed {
                Some(known) => Some(known.min(chunk)),
                None => Some(chunk),
            };
        }
        let Some(from) = first_changed else {
            return false;
        };
        for index in from..self.chunks.len() {
            let mut sum: usize = 0;
            for extra in self.chunks[index].iter() {
                sum += *extra as usize;
            }
            self.before[index + 1] = self.before[index] + sum;
        }
        return true;
    }

    /// Whether `line` has a count of its own,
    /// that is,
    /// takes more than one row.
    pub fn is_wrapped(&self, line: usize) -> bool {
        return self.extra(line) > 0;
    }
}
