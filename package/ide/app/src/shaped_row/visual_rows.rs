//! The rows a shaped line shows:
//!  how many visual rows it has,
//!  and which source characters and display bytes each row covers.

/// The shaped line whose rows these methods read.
use super::ShapedRow;
/// Range rectangles are as tall as one code row.
use crate::row_map::CODE_ROW;
/// Half-open ranges of visual rows and source characters.
use std::ops::Range;

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
    pub fn visual_row_offset(
        &self,
        index: usize,
        scale: f32,
    ) -> f32 {
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
    pub fn visual_row_holding(
        &self,
        top: f32,
        scale: f32,
    ) -> Option<usize> {
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
    pub fn visual_row_bytes(
        &self,
        index: usize,
    ) -> Option<Range<usize>> {
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
    pub fn visual_row_sources(
        &self,
        index: usize,
    ) -> Option<Range<usize>> {
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
}
