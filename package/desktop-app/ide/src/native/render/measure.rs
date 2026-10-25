//! Measuring the lines of a requested tile on the window thread:
//!  the tile that holds only lines whose heights the map has,
//!  and the offset the view moved to while the heights settled.

/// The source state,
///  the rows' map,
///  the tile bounds,
///  and the allowance a new width is measured within.
use crate::native::{
    State,
    rewrap::Allowance,
    rows,
    tile::{self, TileBounds},
};
/// A wrap width must be a positive number of pixels.
use ide_app::wrap::has_wrap_width;

/// What:
///  Most rounds of measuring the lines of one tile.
/// Why:
///  A round measures the lines the tile asked for covers and the map follows the heights it found, so the tile asked
///  for next can reach further down the text; each round brings in the lines that tile gained, so the rounds end
///  when no height changes.
///  The bound keeps a pathological text from holding the window thread,
///  and a tile still changing after it is logged at warn level.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROUNDS = 64;
/// ```
const ROUNDS: usize = 64;

/// What:
///  What measuring the requested tile found:
/// the tile that holds only lines whose heights the map has,
/// the offset the
///  view moved to while the heights settled,
/// and whether the measurement finished.
/// Why:
///  A measurement that spends an allowance can stop short,
/// and the caller then leaves the tile to the worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Measure = { bounds: TileBounds; scrolled?: number; complete: boolean };
/// ```
pub(super) struct Measure {
    /// The tile asked for after the last round.
    pub(super) bounds: TileBounds,
    /// The offset the window must take,
    /// when the heights above the view changed.
    pub(super) scrolled: Option<f32>,
    /// False when the allowance ran out before the tile's lines were all measured.
    pub(super) complete: bool,
}

/// What:
///  Measure the lines of `requested` on the window thread and keep the view still across what that teaches,
///  until the tile asked for holds only lines whose heights the map already has.
///  `offset` is the view's offset,
/// changed when the heights above it changed;
///  the answer is the final offset and the final tile.
///  With an `allowance` every line is charged before it is shaped,
/// broken again,
/// or packed,
///  and the measurement stops at the first that does not fit.
/// Why:
///  A tile built here must agree with the map it is placed by.
///  The lines are shaped once on the way and kept in the shaper's cache,
/// so the build that follows only copies them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function measureRequested(current: State, factor: number, view: [number, number, number],
///   first?: number, allowance?: Allowance): Measure;
/// ```
pub(super) fn measure_requested(
    current: &mut State,
    factor: f32,
    view: (f32, f32, f32),
    first: Option<f32>,
    mut allowance: Option<&mut Allowance>,
) -> Measure {
    let (offset_now, width, height) = view;
    let mut offset = first.unwrap_or(offset_now);
    let mut scrolled = first;
    let mut requested = tile::requested(current);
    if !has_wrap_width(requested.width) {
        return Measure {
            bounds: requested,
            scrolled,
            complete: true,
        };
    }
    // What: Each round measures the lines of the tile asked for, and the map follows what the round taught.
    // Why: Heights the table holds for another width stand inside the tile too, not only above the view. A corrected
    //      height moves the lines after it, so the tile asked for reaches further down the text, and its new last lines
    //      still hold the old heights. The rounds go on until one changes nothing: then the table agrees with the
    //      measurement for every line of the tile that is built next.
    for round in 0..ROUNDS {
        let found = match allowance.as_deref_mut() {
            Some(spending) => spending.measure(current, factor, requested),
            None => {
                let State {
                    shaper,
                    document,
                    styles,
                    ..
                } = &mut *current;
                Some(shaper.measure_lines(
                    document,
                    requested.first..requested.first + requested.count,
                    factor,
                    Some(requested.width),
                    styles,
                ))
            }
        };
        let Some(measured) = found else {
            return Measure {
                bounds: requested,
                scrolled,
                complete: false,
            };
        };
        #[cfg(debug_assertions)]
        {
            current.rows_shaped += measured.shaped;
            current.rows_rebroken += measured.rebroken;
        }
        if !rows::absorb(current, measured.lines) {
            break;
        }
        if let Some(moved) = rows::settle(current, factor, (offset, width, height), true) {
            offset = moved;
            scrolled = Some(moved);
        }
        requested = tile::requested(current);
        if round + 1 == ROUNDS {
            tracing::warn!(
                rounds = ROUNDS,
                first = requested.first,
                count = requested.count,
                "the heights of the tile were still changing after the last round of measuring"
            );
        }
    }
    // The tile the last round asked for is built next, so its lines must be paid for too; those the cache holds
    // at the width cost nothing, and after a round that changed nothing that is every one of them.
    let paid = match allowance.as_deref_mut() {
        Some(spending) => match spending.measure(current, factor, requested) {
            Some(measured) => {
                rows::absorb(current, measured.lines);
                true
            }
            None => false,
        },
        None => true,
    };
    return Measure {
        bounds: requested,
        scrolled,
        complete: paid,
    };
}
