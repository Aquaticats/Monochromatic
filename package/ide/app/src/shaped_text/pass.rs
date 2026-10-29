//! Whole-file passes over the row cache:
//!  bringing every line of a range into it at a wrap width,
//!  and breaking kept rows again at a new width without shaping them.

/// The shaper whose cache the passes fill,
///  and the request that names one line.
use super::{RowRequest, TextShaper};
/// Canonical source the lines are read from.
use crate::document::Document;
/// The kept row that is broken again.
use crate::shaped_row::ShapedRow;
/// Syntax classifications the rows are shaped with.
use crate::source_style::StyleSpan;
/// Tabs that stay on the first visual row keep their corrections when the row is broken again,
///  and a tabbed line is shaped again for each corrective pass.
use crate::tab_layout::{passes_for, tabs_on_first_row};
/// Breaking a kept layout exactly as a fresh one is broken.
use crate::wrap::{break_wrapped, has_wrap_width};
/// Lines handled by one chunk.
use std::ops::Range;

/// What:
///  How a line came to be in the cache:
///  it was already there,
///  a kept row was broken again,
///  or the line was shaped.
/// Why:
///  The tests and the structural counters tell the three apart;
///  a resize must produce no `Shaped`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ensured = 'kept' | 'rebroken' | 'shaped';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ensured {
    /// The cache already held the line at this width.
    Kept,
    /// A row kept at another width was broken again in place.
    Rebroken,
    /// The line was shaped from its text.
    Shaped,
}

/// What:
///  The visual rows of a range of lines and what it cost to find them.
/// Why:
///  A pass reports the rows to the window and the counts to the structural tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Measured = { lines: [line: number, rows: number][]; shaped: number; rebroken: number };
/// ```
#[derive(Clone, Debug, Default)]
pub struct Measured {
    /// The visual rows of each line of the range,
    ///  in line order.
    pub lines: Vec<(usize, usize)>,
    /// Lines shaped from their text.
    pub shaped: u64,
    /// Kept rows broken again at the new width.
    pub rebroken: u64,
}

/// What:
///  Work charged for each line a budgeted measurement brings in,
/// besides the characters it shapes.
/// Why:
///  A short line still costs a layout,
/// a break,
/// and a cache entry;
///  charging it as if it were this many more characters keeps a count of tiny lines honest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const LINE_WORK = 32;
/// ```
pub const LINE_WORK: usize = 32;

/// What:
///  Longest line,
/// in characters,
/// that `line_work` reads to look for tabs.
/// Why:
///  A longer line costs more than any budget the window thread spends,
/// so reading it all would only add to the cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const WORK_SCAN_LIMIT = 65_536;
/// ```
pub const WORK_SCAN_LIMIT: usize = 65_536;

/// What:
///  The most work,
/// in characters shaped or broken,
/// that bringing `line` into the cache at one wrap width can cost:
///  [`LINE_WORK`] plus its length,
/// times the corrective passes of its tabs when it has any.
///  A line past [`WORK_SCAN_LIMIT`] costs `usize::MAX`,
/// and a line the text does not have costs nothing.
/// Why:
///  A tabbed line that cannot be broken again is shaped once and then once more for each corrective pass,
///  so its length alone would understate the cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineWork(document: Document, line: number): number;
/// ```
pub fn line_work(document: &Document, line: usize) -> usize {
    let text = document.text();
    if line >= text.len_lines() {
        return 0;
    }
    let slice = text.line(line);
    let length = slice.len_chars();
    if length > WORK_SCAN_LIMIT {
        return usize::MAX;
    }
    let mut has_tab = false;
    for character in slice.chars() {
        if character == '\t' {
            has_tab = true;
            break;
        }
    }
    let shapings = if has_tab { 1 + passes_for(length) } else { 1 };
    return LINE_WORK + length * shapings;
}

/// What:
///  What a budgeted measurement may still spend:
/// lines brought in,
/// and work in the units of [`line_work`].
/// Why:
///  The window thread measures the lines near the view when the width changes;
///  the budget is what keeps that step bounded,
/// whatever the lines are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Budget = { lines: number; work: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    /// Lines that may still be brought in.
    pub lines: usize,
    /// Work that may still be spent.
    pub work: usize,
}

/// Spending.
impl Budget {
    /// Take one line of `work` from the budget,
    /// or answer `false` and take nothing when it does not fit.
    pub fn spend(&mut self, work: usize) -> bool {
        if self.lines == 0 || work > self.work {
            return false;
        }
        self.lines -= 1;
        self.work -= work;
        return true;
    }
}

/// Passes over the cache.
impl TextShaper {
    /// How many shaped lines the cache keeps;
    ///  a worker that shapes a whole file raises it above the window thread's few hundred.
    pub fn keep_layouts(&mut self, lines: usize) {
        self.rows.set_capacity(lines);
    }

    /// Count one corrective pass of the tab correction.
    pub(crate) fn count_tab_pass(&mut self) {
        self.tab_passes += 1;
    }

    /// Kept rows broken again at a new wrap width instead of being shaped again,
    /// since this shaper was made.
    pub fn rebroken_rows(&self) -> u64 {
        return self.rebroken_rows;
    }

    /// Evict the cached lines farthest from `centre` beyond the cache's capacity.
    pub fn trim_rows(&mut self, centre: usize) {
        self.rows.trim(centre);
    }

    /// Lines the cache holds,
    /// at any wrap width.
    pub fn kept_rows(&self) -> usize {
        return self.rows.len();
    }

    /// What:
    ///  Break a kept row again at `width` logical pixels,
    /// and answer whether that was possible.
    ///  A row whose tabs are not all on the first visual row,
    /// before or after,
    /// is left for the caller to shape again.
    /// Why:
    ///  Breaking a laid-out line again takes about a microsecond,
    ///  shaping it about fifteen;
    ///  a tab on a continuation row was widened to a stop that depends on where the row broke,
    ///  so such a line has to be shaped again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rebreak(row: ShapedRow, scale: number, width: number): boolean;
    /// ```
    pub fn rebreak(&mut self, row: &mut ShapedRow, scale: f32, width: f32) -> bool {
        if !has_wrap_width(width) {
            return false;
        }
        let tabbed = !row.projection.tabs.is_empty();
        if tabbed && !tabs_on_first_row(&row.layout, &row.projection.tabs) {
            return false;
        }
        // The advance is physical, and the indent is built in logical pixels (see `wrapped_line_layout`).
        let space = self.space_advance(scale) / scale;
        break_wrapped(&mut row.layout, &row.projection.text, scale, width, space);
        if tabbed && !tabs_on_first_row(&row.layout, &row.projection.tabs) {
            return false;
        }
        row.wrap = Some(width);
        return true;
    }

    /// What:
    ///  Make the cache hold the line `request` names at the requested wrap width,
    ///  breaking a kept row again when the line is held at another width,
    ///  shaping only when it must.
    /// Why:
    ///  Tile builds and whole-file passes bring lines in the same way,
    ///  so the same line always ends up as the same row.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ensureRow(document: Document, styles: StyleSpan[], request: RowRequest): Ensured;
    /// ```
    pub(super) fn ensure_row(
        &mut self,
        document: &Document,
        styles: &[StyleSpan],
        request: RowRequest,
    ) -> Ensured {
        if self.rows.get(request.row, request.wrap).is_some() {
            return Ensured::Kept;
        }
        if let Some(width) = request.wrap
            && has_wrap_width(width)
            && let Some(mut stale) = self.rows.remove(request.row)
            && self.rebreak(&mut stale, request.scale, width)
        {
            self.rebroken_rows += 1;
            self.rows.insert(stale);
            return Ensured::Rebroken;
        }
        let fresh = self.shape_row(
            document,
            (request.row, 0.0),
            request.scale,
            styles,
            request.baseline,
            request.wrap,
        );
        self.shaped_rows += 1;
        self.rows.insert(fresh);
        return Ensured::Shaped;
    }

    /// What:
    ///  Bring every line of `lines` into the cache at `wrap` and answer the visual rows of each.
    ///  Lines past the text are skipped.
    /// Why:
    ///  The background pass measures the whole file chunk by chunk and keeps the rows for later tiles and resizes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// measureLines(document: Document, lines: Range<number>, scale: number, wrap: number | undefined,
    ///   styles: StyleSpan[]): Measured;
    /// ```
    pub fn measure_lines(
        &mut self,
        document: &Document,
        lines: Range<usize>,
        scale: f32,
        wrap: Option<f32>,
        styles: &[StyleSpan],
    ) -> Measured {
        return self
            .measure_within(document, lines, scale, wrap, styles, None)
            .unwrap_or_default();
    }

    /// What:
    ///  `measure_lines` for a caller that must not spend more than `budget`:
    ///  every line not already held at `wrap` is charged its [`line_work`] before it is shaped,
    ///  and the answer is nothing when a line does not fit what is left.
    /// Why:
    ///  The window thread uses it when the width changes.
    ///  Lines brought in before the budget ran out stay in the cache,
    /// each tagged with its width,
    /// so they are not lost.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// measureLinesWithin(document: Document, lines: Range<number>, scale: number, wrap: number | undefined,
    ///   styles: StyleSpan[], budget: Budget): Measured | undefined;
    /// ```
    pub fn measure_lines_within(
        &mut self,
        document: &Document,
        lines: Range<usize>,
        scale: f32,
        wrap: Option<f32>,
        styles: &[StyleSpan],
        budget: &mut Budget,
    ) -> Option<Measured> {
        return self.measure_within(document, lines, scale, wrap, styles, Some(budget));
    }

    /// The loop of both measurements;
    /// a budget,
    /// when given,
    /// is charged for each line the cache does not hold yet.
    fn measure_within(
        &mut self,
        document: &Document,
        lines: Range<usize>,
        scale: f32,
        wrap: Option<f32>,
        styles: &[StyleSpan],
        mut budget: Option<&mut Budget>,
    ) -> Option<Measured> {
        self.rows.sync(document.revision(), scale);
        let baseline = self.baseline(scale);
        let end = lines.end.min(document.text().len_lines());
        let mut result = Measured::default();
        for row in lines.start..end {
            if let Some(limit) = budget.as_deref_mut()
                && self.rows.get(row, wrap).is_none()
                && !limit.spend(line_work(document, row))
            {
                tracing::debug!(row, "a measurement ran out of its budget");
                return None;
            }
            let request = RowRequest {
                row,
                scale,
                wrap,
                baseline,
            };
            match self.ensure_row(document, styles, request) {
                Ensured::Kept => {}
                Ensured::Rebroken => result.rebroken += 1,
                Ensured::Shaped => result.shaped += 1,
            }
            if let Some(kept) = self.rows.get(row, wrap) {
                result.lines.push((row, kept.visual_rows()));
            }
        }
        return Some(result);
    }
}
