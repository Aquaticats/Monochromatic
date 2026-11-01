//! Where a row's ink changes:
//!  the selected part of a row is a second copy of its draw commands in the selection ink,
//!  clipped to the selection,
//!  and the ordinary ink fills the gaps.

/// The items the clips and draw commands are added to.
use super::Item;
/// The order of two intervals.
use std::cmp::Ordering;

/// What:
///  The physical x range `left` to `right` with both edges moved to the nearest whole pixel.
/// Why:
///  A clip edge inside a pixel covers it partly in each of the two copies of a glyph,
///  and two partly covering layers composite to less than the glyph's own coverage:
///  a faint seam through a stem that crosses the edge.
///  On whole pixels each pixel belongs to one copy.
///  The selection fill behind the text has its edges on whole pixels too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapped(left: number, right: number): [number, number];
/// ```
pub fn snapped(left: f32, right: f32) -> (f32, f32) {
    return (left.round(), right.round());
}

/// What:
///  Order two intervals by where they start.
/// Why:
///  `merged` sorts with it,
///  and a named function says what the order is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function byStart(left: [number, number], right: [number, number]): number;
/// ```
fn by_start(left: &(f32, f32), right: &(f32, f32)) -> Ordering {
    return left.0.total_cmp(&right.0);
}

/// What:
///  Sort `intervals` and join the ones that touch or overlap.
/// Why:
///  The selected part of a row is clipped by its intervals,
///  and the rest of the row by what lies between them;
///  both need ordered,
/// disjoint ranges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function merged(intervals: [number, number][]): [number, number][];
/// ```
fn merged(intervals: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut sorted: Vec<(f32, f32)> = Vec::new();
    for interval in intervals {
        if interval.1 > interval.0 {
            sorted.push(*interval);
        }
    }
    sorted.sort_by(by_start);
    let mut joined: Vec<(f32, f32)> = Vec::new();
    for (start, end) in sorted {
        if let Some(last) = joined.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
            continue;
        }
        joined.push((start, end));
    }
    return joined;
}

/// What:
///  Where a drawing lies in a tile and where a clip around it may reach,
///  in physical pixels.
/// Why:
///  `split` clips a drawing to parts of its row,
///  and the parts it clips to are cut by all three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Place = { extent: [number, number]; band: [number, number]; reach: [number, number] };
/// ```
pub struct Place {
    /// The x range that the clips of a drawing together stay within:
    ///  the tile's width and the margin that keeps the glyphs of its edges.
    pub extent: (f32, f32),
    /// The y range of every clip:
    ///  the row the drawing lies on.
    pub band: (f32, f32),
    /// The x range where the drawing can have ink.
    pub reach: (f32, f32),
}

/// Whether the part of a row from `from` to `to` meets the x range `reach` of a drawing's ink.
fn meets(reach: (f32, f32), from: f32, to: f32) -> bool {
    return to > reach.0 && from < reach.1;
}

/// What:
///  Emit `draw` once in `ordinary` ink outside `intervals` and once in `selected` ink inside them,
///  for each part of the row that the drawing's ink can reach.
///  Every clip spans `place.extent` at most in x and `place.band` in y.
///  With no interval,
/// or the same ink on both sides,
/// `draw` runs once and unclipped.
/// Why:
///  A selected glyph keeps its own shape and takes the selection ink exactly where the selection rectangle lies,
///  even when one glyph (a ligature) spans several characters,
///  so the glyph is clipped at the rectangle's edge and never recolored as a whole.
///  The band is the row the drawing lies on:
///  the renderer's cost of a clip is the area it covers,
///  so a clip as tall as the tile costs the tile's area for every selected row.
///  A part of the row beyond the drawing's ink gets no copy,
///  because every copy is flattened again and the renderer keeps the lines of all of them in a buffer of fixed size:
///  a row selected from its first glyph to its last then costs one copy of each run and not three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function split<T>(items: Item[], intervals: [number, number][], inks: [Rgba, Rgba], place: Place, context: T,
///   draw: (items: Item[], context: T, ink: Rgba) => void): void;
/// ```
pub fn split<T>(
    items: &mut Vec<Item>,
    intervals: &[(f32, f32)],
    inks: ([u8; 4], [u8; 4]),
    place: Place,
    context: &T,
    draw: fn(&mut Vec<Item>, &T, [u8; 4]),
) {
    let (ordinary, selected) = inks;
    let Place {
        extent,
        band,
        reach,
    } = place;
    let joined = merged(intervals);
    if joined.is_empty() || ordinary == selected {
        draw(items, context, ordinary);
        return;
    }
    // The ordinary ink fills the gaps, including the two open ends of the row.
    let mut cursor = extent.0;
    for (start, end) in &joined {
        if *start > cursor && meets(reach, cursor, *start) {
            items.push(Item::Clip {
                left: cursor,
                right: *start,
                top: band.0,
                bottom: band.1,
            });
            draw(items, context, ordinary);
            items.push(Item::Unclip);
        }
        let (left, right) = (start.max(extent.0), end.min(extent.1));
        if meets(reach, left, right) {
            items.push(Item::Clip {
                left,
                right,
                top: band.0,
                bottom: band.1,
            });
            draw(items, context, selected);
            items.push(Item::Unclip);
        }
        cursor = cursor.max(*end);
    }
    if cursor < extent.1 && meets(reach, cursor, extent.1) {
        items.push(Item::Clip {
            left: cursor,
            right: extent.1,
            top: band.0,
            bottom: band.1,
        });
        draw(items, context, ordinary);
        items.push(Item::Unclip);
    }
}
