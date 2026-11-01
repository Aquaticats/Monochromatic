//! Reading geometry of a prepared frame:
//!  pointer hits,
//!  the caret,
//!  selection and range rectangles,
//!  and moving
//! the frame when only rows above it changed.

/// The frame record these methods belong to and the width of a selected line end.
use super::{ShapedView, TERMINATOR_MARK};
/// Canonical source and the current reading position,
///  and the side of a soft-wrap offset a caret is drawn on.
use crate::document::{Document, ReadingAffinity};
/// The one vertical mapping:
///  where code rows start and how tall rows and the caret are.
use crate::row_map::{CARET_HEIGHT, CARET_INSET, CODE_ROW, RowMap};
/// Rectangles handed to the window for the caret,
///  selections,
///  and ranges.
use crate::shaped_row::ReadingRect;

/// Resolve reading geometry from the exact layouts used to paint source glyphs.
impl ShapedView {
    /// What:
    ///  Move this frame to where `map` puts its first line,
    ///  without reshaping or repainting;
    ///  the answer
    ///       says whether anything moved.
    ///  `&RowMap` lends the window's current mapping.
    /// Why:
    ///  Virtual rows can appear or change above the materialized lines.
    ///  The frame's own pixels are then
    ///      still right,
    ///  only its place in the text is not:
    ///  every row top and every cached rectangle shifts by
    ///      the same amount.
    ///  The caller must have checked that the materialized lines themselves are unchanged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rebase(map: RowMap): boolean;
    /// ```
    pub fn rebase(&mut self, map: &RowMap) -> bool {
        if self.map == *map {
            return false;
        }
        // The clip's top is a distance below the first line's block, which stays fixed when rows above move.
        let mut skip = 0.0;
        if let Some(clip) = self.viewport.clip {
            skip = clip.top;
        }
        let shift = map.block_top(self.viewport.first) + skip - self.origin;
        // `clone` copies the mapping so rows outside the frame are placed by the current one.
        self.map = map.clone();
        if shift == 0.0 {
            return false;
        }
        self.origin += shift;
        for row in &mut self.rows {
            row.top += shift;
        }
        for rectangle in &mut self.selections {
            rectangle.y += shift;
        }
        for rectangle in &mut self.matches {
            rectangle.y += shift;
        }
        return true;
    }

    /// Convert a pointer to a source character using the shaping engine's hit test,
    ///  on the first visual row of the line.
    pub fn hit(&self, document: &Document, row: usize, x: f32) -> usize {
        for shaped in &self.rows {
            if shaped.row == row {
                return shaped.hit(x, self.viewport.scale);
            }
        }
        let bounded_row = row.min(document.text().len_lines().saturating_sub(1));
        return document.text().line_to_char(bounded_row);
    }

    /// What:
    ///  Convert a pointer at logical `x` and `y` below the top of line `row`'s first visual row to a source character.
    ///  A point below the line's last row gets the last row.
    /// Why:
    ///  A click on a continuation row must resolve in that row,
    ///  not in the middle of the whole line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hitAt(document: Document, row: number, x: number, y: number): number;
    /// ```
    pub fn hit_at(&self, document: &Document, row: usize, x: f32, y: f32) -> usize {
        for shaped in &self.rows {
            if shaped.row == row {
                return shaped.hit_at(x, y, self.viewport.scale);
            }
        }
        let bounded_row = row.min(document.text().len_lines().saturating_sub(1));
        return document.text().line_to_char(bounded_row);
    }

    /// What:
    ///  Return the caret bar on the side of a soft-wrap offset that the document's reading position names:
    ///  its x from the glyph advances of the visual row it is drawn on,
    ///  and its y from that row.
    /// Why:
    ///  A caret at a wrap offset is both the end of the upper row and the start of the lower one,
    ///  and the reading position carries which of the two the reader is on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// caret(document: Document): ReadingRect;
    /// ```
    pub fn caret(&self, document: &Document) -> ReadingRect {
        let head = document.position().head;
        let affinity = document.position().affinity;
        let row = document.text().char_to_line(head);
        let mut x = 0.0;
        let mut offset = 0.0;
        for shaped in &self.rows {
            if shaped.row == row {
                let caret = shaped.caret_at(head, affinity, self.viewport.scale);
                x = caret.x;
                offset = shaped.visual_row_offset(caret.row, self.viewport.scale);
            }
        }
        return ReadingRect {
            x,
            y: self.map.code_top(row) + offset + CARET_INSET,
            width: 2.0,
            height: CARET_HEIGHT,
        };
    }

    /// Return per-line selection rectangles without including annotations in source.
    /// A selected line terminator is marked after the line's text,
    ///  so a selected empty line stays visible.
    pub fn selection(&self, document: &Document) -> Vec<ReadingRect> {
        let position = document.position();
        let start = position.anchor.min(position.head);
        let end = position.anchor.max(position.head);
        let mut result = Vec::new();
        for shaped in &self.rows {
            result.extend(shaped.range(start, end, self.viewport.scale));
            // The terminator sits after the row's visible text and has no glyph of its own.
            let row_end = shaped.source_start + shaped.source_len();
            if start <= row_end && end > row_end {
                // The end of the text is on the last visual row of the line.
                let caret =
                    shaped.caret_at(row_end, ReadingAffinity::Upstream, self.viewport.scale);
                result.push(ReadingRect {
                    x: caret.x,
                    y: shaped.top + shaped.visual_row_offset(caret.row, self.viewport.scale),
                    width: TERMINATOR_MARK,
                    height: CODE_ROW,
                });
            }
        }
        return result;
    }

    /// Return per-line rectangles for any source character range in the materialized rows.
    /// Selection and in-file find matches share this path,
    ///  so neither reshapes a ligature.
    pub fn range(&self, start: usize, end: usize) -> Vec<ReadingRect> {
        let mut result = Vec::new();
        for shaped in &self.rows {
            result.extend(shaped.range(start, end, self.viewport.scale));
        }
        return result;
    }
}
