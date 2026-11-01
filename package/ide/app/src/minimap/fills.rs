//! The fills the window's markup draws behind the source glyphs:
//!  the selection,
//!  the other find matches,
//!  and the boundary of the active match.
//!  The window draws them as rectangles over the toolkit's background;
//!  the minimap has no such rectangles,
//!  so its scene draws the same ones as paint items under the glyphs,
//!  in the markup's order and inks.

/// The rectangles come from the same shaped rows as the glyphs.
use crate::shaped_text::{ReadingRect, ShapedView};
/// The fills are paint items beside the glyph runs.
use crate::tile_scene::Item;

/// What:
///  The inks of the markup's fills,
///  straight RGBA.
/// Why:
///  The selection fill and the foreground are properties of the window's palette,
///  which the minimap worker cannot read;
///  they travel with the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fills = { selection: Rgba; foreground: Rgba; boundary: boolean };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fills {
    /// The selection's background.
    pub selection: [u8; 4],
    /// The foreground,
    /// which the match fill and the boundaries are the translucent forms of.
    pub foreground: [u8; 4],
    /// Whether the selection is a find match,
    /// which the window outlines with a heavier boundary.
    pub boundary: bool,
}

/// Opacity of the fill of a match that is not the selection.
const MATCH_FILL: f32 = 0.14;

/// Opacity of the thin boundary of a match that is not the selection.
const MATCH_BOUNDARY: f32 = 0.7;

/// Width of the boundary of a match that is not the selection,
/// in logical pixels.
const MATCH_BOUNDARY_WIDTH: f32 = 1.0;

/// Width of the boundary of the active match,
/// in logical pixels.
const ACTIVE_BOUNDARY_WIDTH: f32 = 2.0;

/// A rectangle in whole pixels of the text at the view's scale:
/// left,
/// top,
/// right,
/// bottom.
#[derive(Clone, Copy)]
struct Cell {
    /// Left edge.
    left: i64,
    /// Top edge.
    top: i64,
    /// Right edge,
    /// not inside.
    right: i64,
    /// Bottom edge,
    /// not inside.
    bottom: i64,
}

/// A fill of `cell` in the straight RGBA `ink`,
/// as a frame of `inset` pixels along its edges when that is more than zero,
/// or nothing when the ink is clear or the cell is empty.
fn fill(items: &mut Vec<Item>, cell: Cell, inset: i64, ink: [u8; 4]) {
    if ink[3] == 0 || cell.right <= cell.left || cell.bottom <= cell.top {
        return;
    }
    items.push(Item::Fill {
        left: cell.left as f32,
        top: cell.top as f32,
        right: cell.right as f32,
        bottom: cell.bottom as f32,
        inset: inset as f32,
        ink,
    });
}

/// The order cells are joined along a row in:
/// by the rows they cover,
/// then left to right.
fn by_rows(cell: &Cell) -> (i64, i64, i64) {
    return (cell.top, cell.bottom, cell.left);
}

/// The order cells are joined down a column in:
/// by the columns they cover,
/// then top to bottom.
fn by_columns(cell: &Cell) -> (i64, i64, i64) {
    return (cell.left, cell.right, cell.top);
}

/// What:
///  The cells with the ones that touch or overlap joined into one:
///  those on the same rows side by side,
///  then those over the same columns one above the other.
/// Why:
///  The picture scales the cells down,
///  and two cells that share an edge on whole pixels share it on a fraction of a minimap pixel,
///  where each one's partial coverage composites over the other's and leaves a pale seam.
///  One cell has no seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function joined(cells: Cell[]): Cell[];
/// ```
fn joined(mut cells: Vec<Cell>) -> Vec<Cell> {
    cells.sort_by_key(by_rows);
    let mut across: Vec<Cell> = Vec::with_capacity(cells.len());
    for cell in cells {
        if let Some(last) = across.last_mut()
            && last.top == cell.top
            && last.bottom == cell.bottom
            && cell.left <= last.right
        {
            last.right = last.right.max(cell.right);
            continue;
        }
        across.push(cell);
    }
    across.sort_by_key(by_columns);
    let mut down: Vec<Cell> = Vec::with_capacity(across.len());
    for cell in across {
        if let Some(last) = down.last_mut()
            && last.left == cell.left
            && last.right == cell.right
            && cell.top <= last.bottom
        {
            last.bottom = last.bottom.max(cell.bottom);
            continue;
        }
        down.push(cell);
    }
    return down;
}

/// The straight RGBA `color` at `opacity` of its own alpha.
fn faded(color: [u8; 4], opacity: f32) -> [u8; 4] {
    let alpha = (f32::from(color[3]) * opacity).round().clamp(0.0, 255.0) as u8;
    return [color[0], color[1], color[2], alpha];
}

/// The pixels of `rectangle`,
/// in logical pixels of the text,
/// in a frame whose top is `origin` logical pixels down the text,
/// at `scale`:
/// edges rounded to the nearest pixel.
fn cell_of(rectangle: &ReadingRect, origin: f32, scale: f32, grow: (f32, f32, f32)) -> Cell {
    let (side, above, below) = grow;
    let left = (rectangle.x - side) * scale;
    let right = (rectangle.x + rectangle.width + side) * scale;
    let top = (rectangle.y - origin + above) * scale;
    let bottom = (rectangle.y + rectangle.height - origin + below) * scale;
    return Cell {
        left: left.round() as i64,
        top: top.round() as i64,
        right: right.round() as i64,
        bottom: bottom.round() as i64,
    };
}

/// What:
///  The paint items of the selection and the matches of `view`,
///  in physical pixels at the view's scale.
/// Why:
///  The markup draws a selection rectangle in the selection fill,
///  then each other match as a translucent foreground fill with a thin boundary over it,
///  and the glyphs over both;
///  the minimap puts these items before the glyph items,
///  so its marks stand where the window's rectangles do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fillItems(view: ShapedView, fills: Fills): Item[];
/// ```
pub fn fill_items(view: &ShapedView, fills: Fills) -> Vec<Item> {
    let scale = view.viewport.scale;
    let mut items = Vec::new();
    let mut selected = Vec::with_capacity(view.selections.len());
    for rectangle in &view.selections {
        selected.push(cell_of(rectangle, view.origin, scale, (0.0, 0.0, 0.0)));
    }
    for cell in joined(selected) {
        fill(&mut items, cell, 0, fills.selection);
    }
    for rectangle in &view.selections {
        if fills.boundary {
            // The boundary sits just outside the character cells, two pixels to each side.
            let outer = cell_of(
                rectangle,
                view.origin,
                scale,
                (ACTIVE_BOUNDARY_WIDTH, 0.0, 0.0),
            );
            let width = (ACTIVE_BOUNDARY_WIDTH * scale).round() as i64;
            fill(&mut items, outer, width.max(1), fills.foreground);
        }
    }
    for rectangle in &view.matches {
        // A match is drawn one pixel wider on each side and one pixel shorter at the top and bottom.
        let cell = cell_of(rectangle, view.origin, scale, (1.0, 1.0, -1.0));
        fill(&mut items, cell, 0, faded(fills.foreground, MATCH_FILL));
        let width = (MATCH_BOUNDARY_WIDTH * scale).round() as i64;
        fill(
            &mut items,
            cell,
            width.max(1),
            faded(fills.foreground, MATCH_BOUNDARY),
        );
    }
    return items;
}
