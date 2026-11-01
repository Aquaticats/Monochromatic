//! One shaped source line and its reading geometry:
//!  caret position,
//!  pointer hit,
//!  and range rectangles.

/// Which side of a soft-wrap offset a reading position stands on.
use crate::document::ReadingAffinity;
/// Range rectangles are as tall as one code row.
use crate::row_map::CODE_ROW;
/// Source/display byte maps keep tabs and Unicode out of hit-test heuristics.
use crate::text_projection::Projection;
/// What:
///  Import the shaping engine's paragraph,
///  caret,
///  and selection types.
/// Why:
///  Caret,
///  hit testing,
///  and selection read the same glyph advances that painting uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Affinity, Cursor, Layout, Selection } from 'parley';
/// ```
use parley::{Affinity, Cursor, Layout, Selection};
/// Half-open ranges of visual rows and source characters.
use std::ops::Range;

/// What:
///  A copyable rectangle record;
///  `f32` is a 32-bit float (sibling `f64`),
///  the unit of logical pixels here.
/// Why:
///  Selection,
///  find matches,
///  and the caret hand the same shape to the native window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadingRect = { x: number; y: number; width: number; height: number };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ReadingRect {
    /// Horizontal source coordinate.
    pub x: f32,
    /// Vertical source coordinate.
    pub y: f32,
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}

/// What:
///  One source line shaped with one common baseline across all font fallback runs;
/// `usize` is an address-sized index (siblings `u32`,
///  `u64`),
///  `Layout<u32>` a paragraph whose brushes are numbers.
/// Why:
///  Rope and string indexing take `usize`;
///  numeric brushes keep syntax roles independent of colors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShapedRow = { row: number; top: number; sourceStart: number; projection: Projection; layout: Layout;
///   baseline: number; baselineShift: number };
/// ```
#[derive(Clone)]
pub struct ShapedRow {
    /// Global logical source line.
    pub row: usize,
    /// Top of the line's code row in logical pixels from the top of the text,
    ///  taken from the frame's
    /// [`crate::row_map::RowMap`];
    ///  zero for a row shaped outside a frame,
    ///  which has no vertical position.
    pub top: f32,
    /// Global source character start.
    pub source_start: usize,
    /// Source/display mappings.
    pub projection: Projection,
    /// Shaped rich text,
    ///  including syntax and selection brush roles.
    pub layout: Layout<u32>,
    /// Common baseline relative to the row in physical pixels.
    pub baseline: f32,
    /// Translation from Parley's natural baseline to the common source baseline.
    pub baseline_shift: f32,
    /// The wrap width in logical pixels this layout is broken at,
    ///  or nothing for an unbroken line.
    pub wrap: Option<f32>,
}

/// Reading geometry of one row,
///  shared by materialized viewport rows and rows shaped for caret movement.
impl ShapedRow {
    /// Number of source characters shown on this row,
    ///  excluding its line terminator.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get sourceLength(): number { return this.projection.sourceToByte.length - 1; }
    /// ```
    pub fn source_len(&self) -> usize {
        return self.projection.source_to_byte.len() - 1;
    }

    /// Number of visual rows this source line occupies:
    ///  one for an unwrapped line,
    /// more when the line was broken at a wrap width.
    /// An empty line the engine leaves after the last row does not count (see [`Self::has_empty_tail`]).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get visualRows(): number { return this.layout.lines().length - (this.hasEmptyTail() ? 1 : 0); }
    /// ```
    pub fn visual_rows(&self) -> usize {
        return self.layout.len() - usize::from(self.has_empty_tail());
    }

    /// What:
    ///  Whether the engine left an empty line after the last row of the text.
    /// Why:
    ///  When blanks end the text and the last of them overflows the width,
    /// the engine hangs the blanks on the last row
    ///  and still emits one more line,
    /// with no characters.
    ///  That line is no row of text:
    /// the caret at the end of the text stands at the end of the row before it,
    ///  and the gutter,
    /// the vertical map,
    /// and the keys must not count it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hasEmptyTail(): boolean { return lines.length > 1 && lines.at(-1).textRange.isEmpty; }
    /// ```
    pub fn has_empty_tail(&self) -> bool {
        let count = self.layout.len();
        if count <= 1 {
            return false;
        }
        if let Some(line) = self.layout.get(count - 1) {
            return line.text_range().is_empty();
        }
        return false;
    }

    /// Logical y of visual row `index` from the top of the line's first row,
    /// at `scale`.
    ///
    /// The step is the first row's line height from the shaping engine's metrics,
    /// so row 0 is exactly zero.
    /// An index past the last row still gives its offset;
    /// callers check `visual_rows` first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// visualRowOffset(index: number, scale: number): number;
    /// ```
    pub fn visual_row_offset(&self, index: usize, scale: f32) -> f32 {
        let height = match self.layout.lines().next() {
            Some(line) => line.metrics().line_height,
            None => CODE_ROW * scale,
        };
        return index as f32 * height / scale;
    }

    /// What:
    ///  The visual row of this line whose extent holds the logical `top` of a rectangle,
    ///  or nothing when the rectangle is not on this line.
    ///  A rectangle on a row starts at that row's top,
    /// so a small tolerance covers rounding after a rebase.
    /// Why:
    ///  Selected ink and underlines are matched to the visual row they lie on,
    ///  never by comparing two floats for equality.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// visualRowHolding(top: number, scale: number): number | undefined;
    /// ```
    pub fn visual_row_holding(&self, top: f32, scale: f32) -> Option<usize> {
        let step = self.visual_row_offset(1, scale);
        if step <= 0.0 {
            return None;
        }
        let from = top - self.top;
        let index = ((from + step * 0.5) / step).floor();
        if index < 0.0 || index >= self.visual_rows() as f32 {
            return None;
        }
        return Some(index as usize);
    }

    /// Display bytes of visual row `index`,
    /// or nothing when the row does not exist.
    /// The rows' ranges tile the display text with no gap or overlap.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// visualRowBytes(index: number): [number, number] | undefined;
    /// ```
    pub fn visual_row_bytes(&self, index: usize) -> Option<Range<usize>> {
        // What: The end is clamped to the display text, because an empty line reports one byte past it.
        // Why: The rows must tile the display text exactly, so no range may name a byte the text lacks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const end = Math.min(range.end, text.length);
        // ```
        let length = self.projection.text.len();
        if let Some(line) = self.layout.lines().nth(index) {
            return Some(line.text_range().start..line.text_range().end.min(length));
        }
        return None;
    }

    /// Row-local source characters of visual row `index`,
    /// or nothing when the row does not exist.
    ///
    /// A tab counts as one source character,
    /// so the ranges of the rows tile `0..source_len()`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// visualRowSources(index: number): [number, number] | undefined;
    /// ```
    pub fn visual_row_sources(&self, index: usize) -> Option<Range<usize>> {
        let bytes = self.visual_row_bytes(index)?;
        return Some(
            self.projection.byte_to_source[bytes.start]..self.projection.byte_to_source[bytes.end],
        );
    }

    /// The source characters of every visual row,
    /// as global character ranges that tile the line's visible text.
    /// A line without text has one empty range at its start.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sourceRows(): [number, number][];
    /// ```
    pub fn source_rows(&self) -> Vec<Range<usize>> {
        let mut rows: Vec<Range<usize>> = Vec::new();
        for index in 0..self.visual_rows() {
            if let Some(local) = self.visual_row_sources(index) {
                rows.push(self.source_start + local.start..self.source_start + local.end);
            }
        }
        if rows.is_empty() {
            rows.push(self.source_start..self.source_start);
        }
        return rows;
    }

    /// Physical x of the caret at display byte `byte` on the side `affinity` names.
    fn cursor_x(&self, byte: usize, affinity: Affinity) -> f32 {
        let cursor = Cursor::from_byte_index(&self.layout, byte, affinity);
        // What: `as f32` narrows the engine's 64-bit coordinate to the 32-bit float advances use.
        // Why: Parley's geometry uses `f64`; glyph advances and Slint coordinates use `f32`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return cursor.geometry(this.layout, 1).x0;
        // ```
        return cursor.geometry(&self.layout, 1.0).x0 as f32;
    }

    /// Visual row that holds display byte `byte` on the side `affinity` names.
    /// A byte on a soft-wrap offset is on the lower row,
    /// unless `affinity` is `Upstream`.
    fn visual_row_of(&self, byte: usize, affinity: Affinity) -> usize {
        let mut row = 0;
        for (index, line) in self.layout.lines().enumerate() {
            if line.text_range().start <= byte {
                row = index;
            }
        }
        // What: An upstream caret at the first byte of a lower row belongs to the row above.
        // Why: One offset ends the upper row and starts the lower one; the affinity picks the side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (affinity === 'upstream' && row > 0 && rowStart(row) === byte) row -= 1;
        // ```
        let mut starts_row = false;
        if let Some(line) = self.layout.lines().nth(row) {
            starts_row = line.text_range().start == byte;
        }
        if matches!(affinity, Affinity::Upstream) && row > 0 && starts_row {
            row -= 1;
        }
        // The empty tail is no row: the end of the text is on the row before it.
        return row.min(self.visual_rows() - 1);
    }

    /// Physical x of the caret before the row-local source character `local`,
    /// with the affinity of a plain row.
    /// A position with no soft wrap has one row,
    /// so its caret is the same on either side.
    fn physical_x_on_row(&self, local: usize, row: usize) -> f32 {
        let byte = self.projection.source_to_byte[local.min(self.source_len())];
        for affinity in [Affinity::Downstream, Affinity::Upstream] {
            if self.visual_row_of(byte, affinity) == row {
                return self.cursor_x(byte, affinity);
            }
        }
        return self.cursor_x(byte, Affinity::Downstream);
    }

    /// Visual row whose vertical extent contains physical `y`,
    /// or the last row when `y` is below all of them.
    fn row_at_physical(&self, y: f32) -> usize {
        let mut row = 0;
        for (index, line) in self.layout.lines().enumerate().take(self.visual_rows()) {
            row = index;
            if y < line.metrics().block_max_coord {
                return index;
            }
        }
        return row;
    }

    /// Grapheme boundary nearest to physical `x` for a row-local position the shaping engine proposed.
    ///
    /// The engine divides a multi-character cluster,
    ///  such as a letter with a combining mark or a joined emoji,
    /// into equal parts and can propose a position between them.
    ///  A caret there would split one grapheme,
    /// so the proposal moves to whichever neighboring boundary is nearer to the pointer.
    /// The distances are measured on visual row `row`,
    /// where the pointer is.
    fn snapped(&self, local: usize, x: f32, row: usize) -> usize {
        let mut before = 0;
        let mut after = self.source_len();
        // The stops ascend, so the last one at or before `local` and the first at or after it enclose it.
        for stop in &self.projection.stops {
            if *stop <= local {
                before = *stop;
            }
            if *stop >= local {
                after = *stop;
                break;
            }
        }
        if before == after {
            return local;
        }
        let before_x = self.physical_x_on_row(before, row);
        let after_x = self.physical_x_on_row(after, row);
        if (x - before_x).abs() <= (after_x - x).abs() {
            return before;
        }
        return after;
    }

    /// Source character boundary nearest to physical `(x, y)`,
    /// with `y` measured from the top of this line's first row.
    fn hit_physical(&self, x: f32, y: f32) -> usize {
        // What: `Cursor::from_point` names the glyph under a physical point, in the engine's own units.
        // Why: The point is the pointer in physical pixels, and the row is the one whose extent holds `y`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const cursor = Cursor.fromPoint(this.layout, x, y);
        // ```
        let cursor = Cursor::from_point(&self.layout, x, y);
        let index = cursor.index().min(self.projection.byte_to_source.len() - 1);
        let proposed = self.projection.byte_to_source[index];
        let row = self.row_at_physical(y);
        return self.source_start + self.snapped(proposed, x, row);
    }

    /// Source character boundary nearest to logical `x` on this line's first visual row,
    ///  using the shaping engine's hit test.
    /// An unwrapped line has one row,
    /// so this is its whole hit test.
    /// A point inside a widened tab resolves to the nearer edge of that tab,
    /// and a point inside a multi-character grapheme to the nearer edge of the whole grapheme.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hit(x: number, scale: number): number;
    /// ```
    pub fn hit(&self, x: f32, scale: f32) -> usize {
        // A single row keeps the probe it always had, the middle of the whole layout.
        // A wrapped line has no single middle, so its probe is the middle of its first row.
        // In TS you'd write (pseudocode):
        // ```ts
        // const y = this.visualRows() === 1 ? this.layout.height / 2 : this.firstRowMiddle();
        // ```
        let y = if self.visual_rows() == 1 {
            self.layout.height() / 2.0
        } else {
            self.first_row_middle()
        };
        return self.hit_physical(x * scale, y);
    }

    /// Source character boundary nearest to logical `(x, y)`,
    /// where `y` is measured from the top of this line's first visual row.
    /// Each visual row is one code row tall,
    /// so `y` between `visual_row_offset(n)` and the next offset is on row `n`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hitAt(x: number, y: number, scale: number): number;
    /// ```
    pub fn hit_at(&self, x: f32, y: f32, scale: f32) -> usize {
        return self.hit_physical(x * scale, y * scale);
    }

    /// What:
    ///  The side of a soft-wrap offset a click at logical `y` below the top of this line's first row put the caret on:
    ///  `Upstream` when the click was on the upper row and landed on the offset where the lower row starts,
    ///  otherwise `Downstream`.
    /// Why:
    ///  A click right of the last character of a wrapped row puts the caret at the end of that row,
    ///  not at the start of the next one,
    /// although both are the same offset (IntelliJ keeps the clicked side).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// clickAffinity(position: number, y: number, scale: number): 'upstream' | 'downstream';
    /// ```
    pub fn click_affinity(&self, position: usize, y: f32, scale: f32) -> ReadingAffinity {
        let row = self.row_at_physical(y * scale);
        let local = position
            .saturating_sub(self.source_start)
            .min(self.source_len());
        let byte = self.projection.source_to_byte[local];
        let mut starts_next = false;
        if row + 1 < self.visual_rows()
            && let Some(next) = self.layout.lines().nth(row + 1)
        {
            starts_next = next.text_range().start == byte;
        }
        if starts_next {
            return ReadingAffinity::Upstream;
        }
        return ReadingAffinity::Downstream;
    }

    /// Logical x of the caret before global source character `position`,
    /// standing on visual row `row`.
    /// A position that is not on that row gets the nearest side of it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// xOnRow(position: number, row: number, scale: number): number;
    /// ```
    pub fn x_on_row(&self, position: usize, row: usize, scale: f32) -> f32 {
        let local = position
            .saturating_sub(self.source_start)
            .min(self.source_len());
        return self.physical_x_on_row(local, row) / scale;
    }

    /// Physical middle of the first visual row,
    /// the probe `hit` uses on a wrapped line.
    fn first_row_middle(&self) -> f32 {
        if let Some(line) = self.layout.lines().next() {
            return line.metrics().block_min_coord + line.metrics().line_height / 2.0;
        }
        return 0.0;
    }

    /// Logical x of the caret standing before global source character `position` on this row,
    ///  on the downstream side of a soft-wrap offset.
    /// Positions outside the row clamp to its start or to the end of its visible text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// caretX(position: number, scale: number): number;
    /// ```
    pub fn caret_x(&self, position: usize, scale: f32) -> f32 {
        return self
            .caret_at(position, ReadingAffinity::Downstream, scale)
            .x;
    }

    /// Logical x and visual row of the caret before global source character `position`,
    ///  on the side of a soft-wrap offset that `affinity` names.
    /// `Upstream` at a wrap offset draws at the end of the upper row,
    /// `Downstream` at the start of the lower one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// caretAt(position: number, affinity: 'upstream' | 'downstream', scale: number): { x: number; row: number };
    /// ```
    pub fn caret_at(&self, position: usize, affinity: ReadingAffinity, scale: f32) -> VisualCaret {
        let local = position
            .saturating_sub(self.source_start)
            .min(self.source_len());
        let byte = self.projection.source_to_byte[local];
        let engine = engine_affinity(affinity);
        return VisualCaret {
            x: self.cursor_x(byte, engine) / scale,
            row: self.visual_row_of(byte, engine),
        };
    }

    /// Rectangles covering the part of the global source range `start..end` that lies on this row,
    ///  one per visual row the range crosses.
    /// Selection and in-file find matches share this path,
    ///  so neither reshapes a ligature.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// range(start: number, end: number, scale: number): ReadingRect[];
    /// ```
    pub fn range(&self, start: usize, end: usize, scale: f32) -> Vec<ReadingRect> {
        // What: `Vec::new()` creates an empty growable list (siblings: fixed `[T; N]`, borrowed `&[T]`).
        // Why: A bidirectional selection can produce several rectangles on one row.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const result: ReadingRect[] = [];
        // ```
        let mut result = Vec::new();
        let len = self.source_len();
        let first = start.saturating_sub(self.source_start).min(len);
        let last = end.saturating_sub(self.source_start).min(len);
        if first == last {
            return result;
        }
        let anchor = Cursor::from_byte_index(
            &self.layout,
            self.projection.source_to_byte[first],
            Affinity::Downstream,
        );
        let focus = Cursor::from_byte_index(
            &self.layout,
            self.projection.source_to_byte[last],
            Affinity::Upstream,
        );
        let selection = Selection::new(anchor, focus);
        // The second element of each pair is the visual row that the rectangle lies on.
        for (rect, row) in selection.geometry(&self.layout) {
            result.push(ReadingRect {
                x: rect.x0 as f32 / scale,
                y: self.top + self.visual_row_offset(row, scale),
                width: (rect.x1 - rect.x0) as f32 / scale,
                height: CODE_ROW,
            });
        }
        return result;
    }
}

/// Engine affinity for a reading affinity.
fn engine_affinity(affinity: ReadingAffinity) -> Affinity {
    return match affinity {
        ReadingAffinity::Upstream => Affinity::Upstream,
        ReadingAffinity::Downstream => Affinity::Downstream,
    };
}

/// What:
///  The caret's logical x and the visual row it is drawn on.
/// Why:
///  A soft-wrap offset is drawn on one row or the other,
/// and the vertical position needs the row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type VisualCaret = { x: number; row: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualCaret {
    /// Logical x of the caret,
    /// measured from the line's start edge.
    pub x: f32,
    /// Index of the visual row the caret is drawn on.
    pub row: usize,
}
