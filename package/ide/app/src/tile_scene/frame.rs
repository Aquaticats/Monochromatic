//! The underline runs of an annotation frame as paint items in one texture of a tile.

/// The line style of the run's severity as strokes,
///  and the change of ink at the edges of a selection.
use super::{
    CULL_MARGIN, Item, row_band, snapped,
    split::{Place, split},
    underline::{self, underline_items},
};
/// The runs and their inks.
use crate::annotation_layout::AnnotationFrame;
/// The ink of the selected part of a run.
use crate::code_colors::CodeColors;
/// A code row's height.
use crate::row_map::CODE_ROW;
/// The shaped rows the runs lie on.
use crate::shaped_text::ShapedView;
/// The rows of a tile one texture holds.
use std::ops::Range;

/// What:
///  The paint items of `frame`'s underline runs that can reach the `rows` of the tile,
///  placed from the first of those rows.
/// Why:
///  The runs are already ordered mildest first,
///  so the worst style ends on top of an overlap.
///  A run whose band lies wholly outside the texture paints nothing in it,
///  so a texture of a stack holds only the runs that reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function underlineFrame(items: Item[], view: ShapedView, frame: AnnotationFrame, colors: CodeColors, horizontal: number, rows: [number, number]): void;
/// ```
pub(super) fn underline_frame(
    items: &mut Vec<Item>,
    view: &ShapedView,
    frame: &AnnotationFrame,
    colors: CodeColors,
    horizontal: f32,
    rows: &Range<u32>,
) {
    let Some(top_row) = view.rows.first() else {
        return;
    };
    let scale = view.viewport.scale;
    let width = view.width as f32;
    let shift = rows.start as f32;
    let height = (rows.end - rows.start) as f32;
    for run in &frame.underlines {
        // The run lies on a materialized code row; the same mapping placed that row's selection rectangles.
        let code_top = view.map.code_top(run.row) + run.offset;
        let mut intervals: Vec<(f32, f32)> = Vec::new();
        for rectangle in &view.selections {
            // A rectangle lies on the run's visual row when its top is that row's top, within rounding.
            if (rectangle.y - code_top).abs() < CODE_ROW * 0.5 {
                intervals.push(snapped(
                    (rectangle.x - horizontal) * scale,
                    (rectangle.x + rectangle.width - horizontal) * scale,
                ));
            }
        }
        let geometry = underline::Geometry {
            span: (
                (run.x - horizontal) * scale,
                (run.x + run.width - horizontal) * scale,
            ),
            // Every row shares one baseline, so the first row's is the baseline of all of them.
            line: (code_top - view.origin) * scale
                + top_row.baseline
                + crate::annotation_paint::DROP * scale
                - shift,
            scale,
            severity: run.severity,
        };
        let band = row_band(
            geometry.line - CODE_ROW * scale / 2.0,
            geometry.line + CODE_ROW * scale / 2.0,
        );
        // A band wholly above or below the texture paints nothing in it.
        if band.1 < -CULL_MARGIN || band.0 > height + CULL_MARGIN {
            continue;
        }
        let severity_ink = frame.colors.severity(run.severity);
        split(
            items,
            &intervals,
            (severity_ink, colors.selected),
            Place {
                extent: (-CULL_MARGIN, width + CULL_MARGIN),
                band,
                // The wave's slopes and the dashes' ends reach a little past the span; ink outside the texture is never seen.
                reach: (
                    (geometry.span.0 - crate::annotation_paint::PERIOD * scale).max(0.0),
                    (geometry.span.1 + crate::annotation_paint::PERIOD * scale).min(width),
                ),
            },
            &geometry,
            underline_items,
        );
    }
}
