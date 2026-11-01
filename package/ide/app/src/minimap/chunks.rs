//! Cutting a slice's paint items into the chunks that are drawn into separate textures.
//!
//! An item belongs to every chunk whose rows its ink can reach,
//!  so a glyph that stands on the edge of two chunks is drawn into both and the cut shows no seam.
//! A clip and the items it brackets stay together.

/// The items to cut.
use crate::tile_scene::Item;
/// The box of a stroke's path.
use vello::kurbo::Shape;

/// A group of items that are cut together,
/// and the vertical extent their ink can reach,
/// in physical pixels of the text.
struct Unit {
    /// Where the group starts in the list.
    first: usize,
    /// One past where the group ends in the list.
    end: usize,
    /// The highest and lowest physical y the ink can reach.
    reach: (f32, f32),
}

/// The vertical extent one item's ink can reach,
/// with `margin` physical pixels on each side for the part of a glyph a layout box does not hold.
fn reach_of(item: &Item, margin: f32) -> Option<(f32, f32)> {
    return match item {
        Item::Glyphs(run) => {
            let rise = match run.glyphs.first() {
                Some(glyph) => glyph.y,
                None => 0.0,
            };
            let baseline = run.origin.1 + rise;
            Some((baseline - margin, baseline + margin))
        }
        Item::Fill { top, bottom, .. } => Some((*top, *bottom)),
        Item::Stroke(stroke) => {
            let bounds = stroke.path.bounding_box();
            Some((bounds.y0 as f32 - margin, bounds.y1 as f32 + margin))
        }
        Item::Clip { .. } | Item::Unclip => None,
    };
}

/// Widen `reach` by the extent of `item`'s ink.
fn widen(reach: &mut Option<(f32, f32)>, item: &Item, margin: f32) {
    if let Some((low, high)) = reach_of(item, margin) {
        *reach = Some(match *reach {
            Some((from, to)) => (from.min(low), to.max(high)),
            None => (low, high),
        });
    }
}

/// The groups of `items`:
/// a clip,
/// what it brackets,
/// and its end are one group,
/// every other item is a group of its own.
fn units_of(items: &[Item], margin: f32) -> Vec<Unit> {
    let mut units = Vec::new();
    let mut at = 0;
    while at < items.len() {
        let first = at;
        let mut reach: Option<(f32, f32)> = None;
        if matches!(items[at], Item::Clip { .. }) {
            at += 1;
            while at < items.len() && !matches!(items[at], Item::Unclip) {
                widen(&mut reach, &items[at], margin);
                at += 1;
            }
            // The end of the clip belongs to the group.
            at = (at + 1).min(items.len());
        } else {
            widen(&mut reach, &items[at], margin);
            at += 1;
        }
        if let Some(found) = reach {
            units.push(Unit {
                first,
                end: at,
                reach: found,
            });
        }
    }
    return units;
}

/// What:
///  The items of `items` whose ink can reach the physical rows `low` to `high` of the text,
///  in drawing order.
///  `margin` is how far past its box an item's ink can reach.
/// Why:
///  Each chunk's scene holds only the paths that can land in its texture,
///  so a tall picture costs the renderer what its rows cost,
///  not the whole slice for every chunk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cut(items: Item[], spans: [number, number][], margin: number): Item[][];
/// ```
pub fn cut(items: &[Item], spans: &[(f32, f32)], margin: f32) -> Vec<Vec<Item>> {
    let units = units_of(items, margin);
    let mut cuts: Vec<Vec<Item>> = Vec::with_capacity(spans.len());
    for (low, high) in spans {
        let mut kept = Vec::new();
        for unit in &units {
            if unit.reach.1 >= *low && unit.reach.0 <= *high {
                kept.extend_from_slice(&items[unit.first..unit.end]);
            }
        }
        cuts.push(kept);
    }
    return cuts;
}
