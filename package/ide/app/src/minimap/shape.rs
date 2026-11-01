//! Shaping the lines of a slice and placing them in a view:
//!  which lines a run of picture rows shows,
//!  shaped or taken from the row cache within the work budget,
//!  and the view that carries the selection,
//!  the matches,
//!  and the annotations of those lines.

/// Shaped lines kept by what shaped them.
use super::line_cache::{LineCache, line_key};
/// The inputs and what a build reports.
use super::scene::{BUILD_WORK, Cancel, Source, Stats};
/// Diagnostic marks.
use crate::annotation::{Mark, Visible};
/// The positioning of hints and messages beside the rows.
use crate::annotation_layout::lay_out;
/// Canonical text,
/// whose reading position is the selection.
use crate::document::Document;
/// The rectangles of the matches in a frame.
use crate::find_paint::rectangles;
/// The vertical mapping the lines are placed by.
use crate::row_map::{CODE_ROW, RowMap};
/// Shaping,
/// and the frame it prepares.
use crate::shaped_text::{ShapedRow, ShapedView, TextShaper, Viewport, line_work, raster_width};
/// Blocks of virtual rows.
use crate::virtual_row::Block;
/// Blocks are shared between the window and the worker;
/// a window of bands is a range of numbers.
use std::{ops::Range, sync::Arc};

/// What:
///  The lines the rows `rows` of the picture show,
///  with one code row of margin on each side.
/// Why:
///  A line whose glyphs reach into the slice from outside it must be drawn,
///  so the slice has no seam where its edge cuts a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linesOf(map: RowMap, rows: [number, number], scale: number, ratio: number): [number, number];
/// ```
pub(super) fn lines_of(map: &RowMap, rows: &Range<usize>, scale: f32, ratio: f64) -> Range<usize> {
    let per_row = 1.0 / (ratio * f64::from(scale));
    let top = (rows.start as f64 * per_row) as f32;
    let bottom = (rows.end as f64 * per_row) as f32;
    let first = map.line_at((top - CODE_ROW).max(0.0));
    let end = (map.line_at(bottom + CODE_ROW) + 1).min(map.lines());
    return first..end.max(first);
}

/// What:
///  The shaped rows of `lines`,
///  each from the cache when its key is unchanged and otherwise shaped,
///  within `work`.
///  A line the budget cannot pay for is a blank row,
///  counted in `stats`.
/// Why:
///  The rows of a view follow one another without a gap,
///  because the annotation layout finds a line's row by its distance from the first.
///  A cancelled build answers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shapeLines(shaper: TextShaper, cache: LineCache, source: Source, lines: [number, number], work: number, cancelled: Cancel): ShapedRow[] | undefined;
/// ```
pub(super) fn shape_lines(
    shaper: &mut TextShaper,
    cache: &mut LineCache,
    source: &Source,
    lines: Range<usize>,
    stats: &mut Stats,
    cancelled: &dyn Cancel,
) -> Option<Vec<ShapedRow>> {
    let text = source.document.text();
    let mut left = BUILD_WORK;
    let mut blank: Option<ShapedRow> = None;
    let mut rows = Vec::with_capacity(lines.len());
    for line in lines {
        if line % 32 == 0 && cancelled.is_cancelled() {
            return None;
        }
        let key = line_key(
            source.document,
            source.styles,
            line,
            source.wrap,
            source.scale,
        );
        let start = text.line_to_char(line);
        let mut row = match cache.take(key) {
            Some(kept) => {
                stats.reused += 1;
                kept
            }
            None => {
                let cost = line_work(source.document, line);
                if cost > left {
                    stats.skipped += 1;
                    tracing::warn!(
                        line,
                        cost,
                        left,
                        "a minimap line was left blank: the work budget cannot pay for it"
                    );
                    match &blank {
                        Some(template) => template.clone(),
                        None => {
                            let empty = Document::new("\n");
                            let template =
                                shaper.shape_line(&empty, 0, source.scale, &[], source.wrap);
                            blank = Some(template.clone());
                            template
                        }
                    }
                } else {
                    left -= cost;
                    stats.work += cost;
                    stats.shaped += 1;
                    let shaped = shaper.shape_line(
                        source.document,
                        line,
                        source.scale,
                        source.styles,
                        source.wrap,
                    );
                    cache.put(key, shaped.clone());
                    shaped
                }
            }
        };
        row.row = line;
        row.source_start = start;
        rows.push(row);
    }
    return Some(rows);
}

/// What:
///  The shaped view of `rows`,
///  placed by the window's vertical mapping from the top of the text,
///  with the selection and match rectangles and the annotation layout of its lines.
/// Why:
///  This is the frame the window's tile is made from,
///  except that its origin is the top of the text and its height the whole text,
///  so its paint items stand where the whole text stands and nothing is culled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function viewOf(shaper: TextShaper, source: Source, rows: ShapedRow[], first: number): ShapedView;
/// ```
pub(super) fn view_of(
    shaper: &mut TextShaper,
    source: &Source,
    rows: Vec<ShapedRow>,
    first: usize,
) -> ShapedView {
    let scale = source.scale;
    let count = rows.len();
    // A wrapped line has the rows it really has, wherever the window's map still counts it as one.
    let placed = if source.wrap.is_some() {
        let mut measured: Vec<(usize, usize)> = Vec::new();
        for shaped in &rows {
            measured.push((shaped.row, shaped.visual_rows()));
        }
        source.map.with_measured(&measured)
    } else {
        source.map.clone()
    };
    let mut placed_rows = Vec::with_capacity(count);
    for shaped in rows {
        let top = placed.code_top(shaped.row);
        placed_rows.push(ShapedRow { top, ..shaped });
    }
    let raster = raster_width(source.wrap);
    let mut view = ShapedView {
        rows: placed_rows,
        viewport: Viewport {
            first,
            count,
            width: raster,
            scale,
            wrap: source.wrap,
            clip: None,
        },
        width: (raster.max(1.0) * scale).ceil() as u32,
        height: (placed.height().max(CODE_ROW) * scale).ceil() as u32,
        origin: 0.0,
        map: placed,
        selections: Vec::new(),
        matches: Vec::new(),
        annotations: None,
    };
    view.selections = view.selection(source.document);
    view.matches = rectangles(&view, source.ranges, source.active);
    let text = source.document.text();
    let end = first + count;
    let start_char = text.line_to_char(first);
    let end_char = if end < text.len_lines() {
        text.line_to_char(end)
    } else {
        text.len_chars()
    };
    let visible = Visible {
        blocks: blocks_between(source.blocks, first, end),
        marks: marks_between(source.marks, start_char, end_char),
    };
    view.annotations = Some(lay_out(&view, &visible, shaper, source.inks));
    return view;
}

/// Index of the first block that belongs to a line at or past `line`,
/// by a halving search.
fn first_block_from(blocks: &[Arc<Block>], line: usize) -> usize {
    let (mut low, mut high) = (0, blocks.len());
    while low < high {
        let middle = low + (high - low) / 2;
        if blocks[middle].line < line {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    return low;
}

/// The blocks of `blocks` that belong to the lines `first` to `end`.
pub fn blocks_between(blocks: &[Arc<Block>], first: usize, end: usize) -> Vec<Arc<Block>> {
    let from = first_block_from(blocks, first);
    let to = first_block_from(blocks, end).max(from);
    return blocks[from..to].to_vec();
}

/// The marks of `marks` that touch the characters `start` to `end`.
pub fn marks_between(marks: &[Mark], start: usize, end: usize) -> Vec<Mark> {
    let mut found = Vec::new();
    for mark in marks {
        if mark.start >= end {
            break;
        }
        if mark.end >= start {
            found.push(*mark);
        }
    }
    return found;
}
