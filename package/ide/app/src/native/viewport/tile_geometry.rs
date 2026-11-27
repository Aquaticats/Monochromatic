//! The lines the materialized tile holds and the edges the view stays clear of:
//!  which lines cover a view at some offset,
//!  where that tile's raster ends,
//!  and the rows such a tile is cut down to when it is taller than one raster.

/// The clip rule the library shares with every frame,
///  re-exported by the module this one belongs to.
use super::band_clip;
/// The source state,
///  the renderer's texture limit,
///  and the bounds of the tile the window holds.
use crate::native::{
    State,
    render,
    tile::TileBounds,
};
/// Height of one code row:
///  the least a line can take,
///  which bounds how many lines a view can show.
use ide_app::row_map::CODE_ROW;
/// The part of a tall tile's lines its raster holds.
use ide_app::shaped_text::Clip;

/// What:
///  The materialized tile spans this many view heights:
/// one above the view,
/// the view,
/// and one below.
/// Why:
///  A view inside the tile's middle band needs no new pixels,
/// so scrolling inside it does no source work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TILE_VIEWS = 3;
/// ```
pub(in crate::native) const TILE_VIEWS: f32 = 3.0;

/// What:
///  How close,
/// in view heights,
/// the view's edge may come to the tile's edge before the tile is re-centered.
/// Why:
///  The view starts a re-center with one view height of room on each side,
/// so it may scroll
///  `1 - RECENTER_EDGE_VIEWS` = 0.75 view heights before the next re-center:
/// about one view height,
///  and the tile edge never reaches the view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RECENTER_EDGE_VIEWS = 0.25;
/// ```
const RECENTER_EDGE_VIEWS: f32 = 0.25;

/// What:
///  Lines materialized past the last line of the tile's bottom edge,
/// as a margin against rounding.
/// Why:
///  A line's block can be taller than one code row when it carries virtual rows,
///  so the line count of the tile is not the tile's pixel height divided by one code row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TILE_SPARE_LINES = 2;
/// ```
const TILE_SPARE_LINES: usize = 2;

/// What:
///  Whether the materialized tile still holds the view:
///  the view's top edge is at least `edge` below the tile's top edge,
///  and its bottom edge is as far above the tile's bottom edge,
///  where `edge` is [`recenter_edge`].
///  A side that touches the first or last line of the text is always held.
/// Why:
///  Edges come from the live vertical mapping,
/// so rows that appear above the tile move them with the rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holds(current: State, bounds: TileBounds, view: number, height: number): boolean;
/// ```
pub(in crate::native) fn holds(
    current: &State,
    bounds: TileBounds,
    view: f32,
    height: f32,
) -> bool {
    let lines = current.document.text().len_lines();
    let edge = recenter_edge(bounds.width, current.scale, height);
    let end = (bounds.first + bounds.count).min(lines);
    let (top, bottom) = tile_edges(current, bounds, end);
    // The top of the text and its bottom are always held; with a clip they can lie inside a line.
    let top_held = top <= 0.0 || view >= top + edge;
    let bottom_held = bottom >= current.row_map.height() || view + height <= bottom - edge;
    return top_held && bottom_held;
}

/// What:
///  How close,
///  in logical pixels,
///  the view's edge may come to the edge of a tile built for a text area `width` wide at display `scale`
///  before the tile is re-centered:
///  `RECENTER_EDGE_VIEWS` view heights,
///  or what a raster of the largest height the limit allows leaves around a view `height` tall,
///  when that is less.
///  Two code rows are kept back for the rounding of the clip's edges to whole rows,
///  and the answer is negative for a view taller than the raster can be.
/// Why:
///  A tile is never taller than its raster limit.
///  Demanding more room than the limit leaves around the view
///  made every tile the worker built for a window of about 10,000 px or more fail the hold test,
///  so the window discarded it and asked for the same one again,
///  for as long as the window stayed that size,
///  and the measurement pass never finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recenterEdge(width: number, scale: number, height: number): number {
///   return Math.min(RECENTER_EDGE_VIEWS * height, (tileCap(width, scale) - height) / 2 - 2 * CODE_ROW);
/// }
/// ```
fn recenter_edge(
    width: f32,
    scale: f32,
    height: f32,
) -> f32 {
    let room = (render::tile_cap(width, scale, height) - height) / 2.0 - 2.0 * CODE_ROW;
    return (RECENTER_EDGE_VIEWS * height).min(room);
}

/// What:
///  The top and bottom edge of the raster a tile at `bounds` holds,
/// in logical pixels from the top of the text:
///  the clip below the first line's block when there is one,
/// otherwise the blocks of its first and last lines.
///  `end` is the line after the tile's last.
/// Why:
///  The view must stay clear of these edges,
/// and they move with the live vertical mapping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tileEdges(current: State, bounds: TileBounds, end: number): [top: number, bottom: number];
/// ```
fn tile_edges(
    current: &State,
    bounds: TileBounds,
    end: usize,
) -> (f32, f32) {
    let first_top = current.row_map.block_top(bounds.first);
    return match bounds.clip {
        Some(clip) => (first_top + clip.top, first_top + clip.bottom),
        None => (first_top, current.row_map.block_top(end)),
    };
}

/// What:
///  The offsets between which the displayed tile's raster covers the whole view,
///  as the two edges of the raster in logical pixels from the top of the text:
///  a side that is the text's own first or last row is `HOLD_FREE` away,
///  because no offset passes it.
/// Why:
///  The window draws the view at the nearest offset inside this reach while the tile for the real offset has not
///  landed,
///  so a wheel ease that outruns the worker shows real rows of the tile it has and not a blank.
///  `holds` asks for more room (the re-center edge);
///  this is the raster's own edge,
///  which the picture cannot pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reach(current: State): [top: number, bottom: number];
/// ```
pub(in crate::native) fn reach(current: &State) -> (f32, f32) {
    let shown = current.shown;
    let lines = current.document.text().len_lines();
    let end = (shown.first + shown.count).min(lines);
    let (raster_top, raster_bottom) = tile_edges(current, shown, end);
    let top = if raster_top <= 0.0 {
        -HOLD_FREE
    } else {
        raster_top
    };
    let bottom = if raster_bottom >= current.row_map.height() {
        HOLD_FREE
    } else {
        raster_bottom
    };
    return (top, bottom);
}

/// What:
///  The distance,
///  in logical pixels,
///  that stands for "no limit" on a side of the reach.
/// Why:
///  The window holds plain numbers;
///  the text is never this tall.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HOLD_FREE = 1e7;
/// ```
const HOLD_FREE: f32 = 10_000_000.0;

/// What:
///  Choose the window of lines the tile for a view at `view` holds:
///  one view height of overscan above the view and one below,
///  at wrap width `width`,
///  and set it as the requested tile.
/// Why:
///  Re-centering then happens only after the view has traveled about one view height inside the tile.
///  A wheel turn asks for the tile of the place its ease ends at with the same rule,
///  so that tile holds the view when the ease arrives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requestWindow(current: State, view: number, width: number, height: number): void;
/// ```
pub(in crate::native) fn request_window(
    current: &mut State,
    view: f32,
    width: f32,
    height: f32,
) {
    let tile_start = view + height * 0.5 - height * TILE_VIEWS * 0.5;
    let tile_end = view + height * 0.5 + height * TILE_VIEWS * 0.5;
    let first = current.row_map.line_at(tile_start.max(0.0));
    let last = current.row_map.line_at(tile_end);
    current.first = first;
    current.count = last.saturating_sub(first) + 1 + TILE_SPARE_LINES;
    current.width = width;
    current.clip = clip_for(current, (tile_start, tile_end), height);
}

/// What:
///  The part of the lines just chosen for the tile that its raster holds:
///  nothing while the lines fit one raster,
///  otherwise the rows of `band`,
/// the three view heights around the view,
/// cut down to the raster limit.
///  The edges are whole code rows below the first line's block.
/// Why:
///  A tile holds whole lines,
/// and a line wrapped to thousands of rows would pass the raster limit.
///  The band is where the view can go before the next re-center,
/// so the rows outside it are never needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clipFor(current: State, band: [top: number, bottom: number], view: number): Clip | undefined;
/// ```
fn clip_for(
    current: &State,
    band: (f32, f32),
    view: f32,
) -> Option<Clip> {
    let lines = current.document.text().len_lines();
    let end = (current.first + current.count).min(lines);
    let first_top = current.row_map.block_top(current.first);
    let bottom = current.row_map.block_top(end);
    let cap = render::tile_cap(current.width, current.scale, view);
    let clip = band_clip(first_top, bottom, band, cap);
    if let Some(chosen) = clip {
        tracing::debug!(
            first = current.first,
            top = chosen.top,
            bottom = chosen.bottom,
            cap,
            "clipped a tile of lines taller than one raster"
        );
    }
    return clip;
}
