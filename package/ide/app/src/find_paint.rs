//! In-file find rectangles for materialized rows only,
//!  from the same geometry path as selection.

/// Match ranges are source character positions,
///  never display columns.
use crate::find::FindRange;
/// Binary search limits work to matches intersecting the materialized rows.
use crate::find_navigation::visible;
/// Shaped rows own the only glyph geometry;
///  rectangles come from them,
///  not from a second layout.
use crate::shaped_text::{ReadingRect, ShapedView};

/// What:
///  `&ShapedView` and `&[FindRange]` borrow the frame and the sorted matches;
///  `Option<FindRange>`
/// is one range or nothing;
///  `f32` is a 32-bit float (sibling `f64`),
///  the unit of logical pixels here.
/// Why:
///  Only matches inside the materialized rows become native rectangles,
/// so painting cost follows the viewport,
///  not the file.
///  The active match is drawn as the selection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rectangles(view: ShapedView, ranges: FindRange[], active: FindRange | undefined): ReadingRect[];
/// ```
pub fn rectangles(
    view: &ShapedView,
    ranges: &[FindRange],
    active: Option<FindRange>,
) -> Vec<ReadingRect> {
    // What: `Vec::new()` creates an empty growable list.
    // Why: The number of visible rectangles is known only after intersecting rows and matches.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result: ReadingRect[] = [];
    // ```
    let mut result = Vec::new();
    // What: `first()` and `last()` return `Some(&row)` or `None` for an empty frame.
    // Why: An empty document tile has no character window to intersect.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first = view.rows[0]; const last = view.rows.at(-1); if (!first || !last) return result;
    // ```
    let Some(first) = view.rows.first() else {
        return result;
    };
    let Some(last) = view.rows.last() else {
        return result;
    };
    let window_start = first.source_start;
    let window_end = last.source_start + last.projection.source_to_byte.len();
    for range in visible(ranges, window_start, window_end) {
        // What: `Some(*range)` wraps a copy of the borrowed range for comparison with the active one.
        // Why: The active match already has selection ink and background; a second fill would hide them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (sameRange(range, active)) continue;
        // ```
        if active == Some(*range) {
            continue;
        }
        for rectangle in view.range(range.start, range.end) {
            result.push(rectangle);
        }
    }
    return result;
}
