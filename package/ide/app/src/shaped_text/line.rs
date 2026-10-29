//! Shaping one line on its own,
//! for callers that keep shaped rows by what shaped them.

/// The text shaper whose private methods shape a row,
/// the row it makes,
/// and the document,
/// styles,
/// and wrap width it shapes.
use super::{ShapedRow, TextShaper};
/// The text of the line.
use crate::document::Document;
/// Syntax classes by character offset.
use crate::source_style::StyleSpan;

/// Shape one line without the row cache.
impl TextShaper {
    /// What:
    ///  Shape one source line with its syntax roles,
    ///  broken at `wrap` when there is a wrap width,
    ///  without touching the row cache.
    /// Why:
    ///  A caller that keeps shaped rows by what shaped them,
    ///  such as the minimap's row cache,
    ///  shapes through this and decides itself when a row is still valid.
    ///  The row has no vertical position:
    ///  its `top` is zero.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// shapeLine(document: Document, row: number, scale: number, styles: StyleSpan[], wrap?: number): ShapedRow;
    /// ```
    pub fn shape_line(
        &mut self,
        document: &Document,
        row: usize,
        scale: f32,
        styles: &[StyleSpan],
        wrap: Option<f32>,
    ) -> ShapedRow {
        let last = document.text().len_lines().saturating_sub(1);
        let baseline = self.baseline(scale);
        self.shaped_rows += 1;
        return self.shape_row(
            document,
            (row.min(last), 0.0),
            scale,
            styles,
            baseline,
            wrap,
        );
    }
}
