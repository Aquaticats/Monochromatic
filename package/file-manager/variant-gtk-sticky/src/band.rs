//! Pure sticky-band math:
//!  the whole replacement for the original's lane engine.
//!
//! The original (`package/file-manager/app/src/layout/lane.rs`) stores per-lane scroll
//! offsets,
//!  accumulates them along the parent chain,
//!  clamps each pane into every rail containing
//! it,
//!  and runs forward/backward relaxation passes so siblings never overlap.
//!  This module replaces
//! all of that with one stateless rule per pane,
//!  the same rule CSS `position: sticky` applies:
//! a pane rides the app scroll inside its own band,
//!  `y = band_top + clamp(scroll - band_top, 0,
//! band_height - PANE_HEIGHT)`.
//!  Non-overlap needs no solver because the tidy tree layout makes
//! bands within a column disjoint (a rail ends at the deepest direct child,
//!  and the next pane in
//! the same column starts below that subtree).

/// What:
///  imports pane geometry constants.
/// Why:
///  every band and position calculation shares the one pane-size source of truth.
use crate::constants::{PANE_GAP, PANE_HEIGHT, PANE_WIDTH};

/// What:
///  imports the stable pane identity type from the original app's crate.
/// Why:
///  placements are keyed by the same `PaneId` the shared model mints.
use file_manager::types::PaneId;

/// What:
///  immutable placement snapshot for one pane,
///  mirrored from the shared model.
/// Why:
///  the band math needs only grid coordinates and parent links,
///  never GTK widgets.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    /// Stable pane identity.
    pub id: PaneId,
    /// Zero-based lineage column.
    pub column: usize,
    /// Zero-based vertical row within the global row coordinate space.
    pub row: usize,
    /// Parent pane identity,
    ///  if this pane was spawned from another pane.
    pub parent: Option<PaneId>,
}

/// What:
///  one pane's sticky band:
///  the fixed content-coordinate interval the pane may ride within.
/// Why:
///  this rectangle is the original's green `Y6L` rail and the Electron prototype's `.rail`
///      wrapper;
///  the pane is clamped inside it,
///  never the band itself moving.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    /// Band top edge in content coordinates.
    pub top: f64,
    /// Band height in pixels:
    ///  from the pane's own row to its deepest direct child's bottom edge.
    pub height: f64,
}

/// What:
///  vertical pixel offset of `row` within the pane grid.
/// Why:
///  panes tile down each column at a fixed stride shared by every column.
pub fn row_y(row: usize) -> f64 {
    return row as f64 * f64::from(PANE_HEIGHT + PANE_GAP)
}

/// What:
///  compute `placement`'s sticky band from the placement snapshot.
/// Why:
///  a leaf's band is one pane tall (no travel);
///  a parent's band stretches to its deepest
///      DIRECT child's bottom edge,
///  so the parent stays visible while any direct child is.
pub fn band_for(placement: Placement, placements: &[Placement]) -> Band {
    let deepest = placements
        .iter()
        .filter(|candidate| return candidate.parent == Some(placement.id))
        .fold(placement.row, |deepest_row, child| return deepest_row.max(child.row));
    return Band {
        top: row_y(placement.row),
        height: row_y(deepest) + f64::from(PANE_HEIGHT) - row_y(placement.row),
    }
}

/// What:
///  the sticky rule itself:
///  where a pane sits for a given app scroll offset.
/// Why:
///  the pane pins to the viewport top while its band passes it and releases when the band's
///      end pushes it off;
///  this one expression replaces the original's offset store,
///  chain
///      accumulation,
///  rail clamping,
///  and relaxation passes.
pub fn sticky_y(band: Band, scroll: f64) -> f64 {
    let travel = (band.height - f64::from(PANE_HEIGHT)).max(0.0);
    return band.top + (scroll - band.top).clamp(0.0, travel)
}

/// What:
///  resolve every pane's `(x, y)` content position for one app scroll offset.
/// Why:
///  positioning is a pure function of the snapshot and the scroll,
///  so the GTK adapter holds
///      no per-lane state and the boundary test can assert geometry without a GUI.
pub fn positions(placements: &[Placement], scroll: f64) -> Vec<(PaneId, f64, f64)> {
    return placements
        .iter()
        .map(|placement| {
            let x = placement.column as f64 * f64::from(PANE_WIDTH + PANE_GAP);
            let y = sticky_y(band_for(*placement, placements), scroll);
            return (placement.id, x, y)
        })
        .collect()
}

/// What:
///  count pane pairs whose resolved boxes intersect at this scroll offset.
/// Why:
///  the observable non-overlap invariant;
///  sticky bands must keep this zero at every scroll,
///      which the unit tests sweep and the boundary test asserts live.
pub fn overlap_count(placements: &[Placement], scroll: f64) -> usize {
    let resolved = positions(placements, scroll);
    let width = f64::from(PANE_WIDTH);
    let height = f64::from(PANE_HEIGHT);
    return resolved
        .iter()
        .enumerate()
        .map(|(index, (_, left_x, left_y))| {
            return resolved[index + 1..]
                .iter()
                .filter(|(_, right_x, right_y)| {
                    return (left_x < &(right_x + width))
                        && (right_x < &(left_x + width))
                        && (left_y < &(right_y + height))
                        && (right_y < &(left_y + height))
                })
                .count()
        })
        .sum()
}

/// What:
///  whether the first root pane (column 0,
///  lowest row) is pinned to the viewport top while
///       the app is scrolled down.
/// Why:
///  the one observable fact that proves sticking is happening;
///  both prototypes report it under
///      the same state key so the two boundary tests share assertions.
pub fn root_pinned(placements: &[Placement], scroll: f64) -> bool {
    if scroll <= 0.0 {
        return false;
    }
    let Some(root) = placements
        .iter()
        .filter(|placement| return placement.column == 0)
        .min_by_key(|placement| return placement.row)
    else {
        return false;
    };
    let y = sticky_y(band_for(*root, placements), scroll);
    return (y - scroll).abs() < 1.0
}
