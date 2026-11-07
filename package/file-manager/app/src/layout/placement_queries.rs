//! Pure queries over pane placement snapshots.
//!
//! Each query is a named function with a loop,
//! so callers state what they ask without passing a closure to an iterator adapter.

/// What:
///  imports the hash-map container.
/// Why:
///  the live-position index is keyed by `PaneId`.
use std::collections::HashMap;

/// What:
///  imports stable pane identity.
/// Why:
///  lookups and the live-position index name panes by id.
use crate::types::PaneId;

/// What:
///  imports the placement snapshot type.
/// Why:
///  every query reads placement snapshots.
use super::PanePlacement;

/// What:
///  look up a placement by id.
/// Why:
///  lane maps store parent ids but geometry needs the parent placement snapshot.
pub(super) fn placement_by_id(placements: &[PanePlacement], id: PaneId) -> Option<PanePlacement> {
    for placement in placements {
        if placement.id == id {
            return Some(*placement)
        }
    }
    return None
}

/// What:
///  number of columns the placements span (one past the highest column index),
///  or zero when there are none.
/// Why:
///  reconcile grows or shrinks the column views to exactly this count.
pub(super) fn column_count_of(placements: &[PanePlacement]) -> usize {
    let mut count = 0;
    for placement in placements {
        count = count.max(placement.column + 1);
    }
    return count
}

/// What:
///  index every placement's `(column, row)` by pane id.
/// Why:
///  stale-widget removal and column height sizing both ask which panes are live and where.
pub(super) fn live_positions(placements: &[PanePlacement]) -> HashMap<PaneId, (usize, usize)> {
    let mut live = HashMap::new();
    for placement in placements {
        live.insert(placement.id, (placement.column, placement.row));
    }
    return live
}

/// What:
///  the deepest row among live panes in `column`,
///  or zero when the column holds none.
/// Why:
///  each static column canvas is sized to its deepest pane row.
pub(super) fn deepest_row_in_column(live: &HashMap<PaneId, (usize, usize)>, column: usize) -> usize {
    let mut deepest = 0;
    for &(pane_column, row) in live.values() {
        if pane_column == column {
            deepest = deepest.max(row);
        }
    }
    return deepest
}

/// What:
///  the sort key that stacks a column's panes by row,
///  then by raw id.
/// Why:
///  a named key function lets `sort_by_key` take it by path instead of through a closure.
pub(super) fn row_then_id(placement: &PanePlacement) -> (usize, u64) {
    return (placement.row, placement.id.0)
}

/// What:
///  the right-most column among `children`,
///  or `fallback` when there are none.
/// Why:
///  a lane spans from the parent column through its right-most direct child.
pub(super) fn last_column_of(children: &[PanePlacement], fallback: usize) -> usize {
    let mut last: Option<usize> = None;
    for child in children {
        last = Some(last.unwrap_or(child.column).max(child.column));
    }
    return last.unwrap_or(fallback)
}

/// What:
///  the deepest row among `children`,
///  or `fallback` when there are none.
/// Why:
///  the lane debug label reports the row span of the direct children.
pub(super) fn last_row_of(children: &[PanePlacement], fallback: usize) -> usize {
    let mut last: Option<usize> = None;
    for child in children {
        last = Some(last.unwrap_or(child.row).max(child.row));
    }
    return last.unwrap_or(fallback)
}

/// What:
///  whether pane `id` is the lane's parent or one of its direct children.
/// Why:
///  a pane is constrained by every green rail that contains it.
pub(super) fn lane_contains(parent: PaneId, children: &[PanePlacement], id: PaneId) -> bool {
    if parent == id {
        return true
    }
    for child in children {
        if child.id == id {
            return true
        }
    }
    return false
}

/// What:
///  the parent id of pane `id`,
///  or nothing when the pane is absent or a root.
/// Why:
///  the sticky offset sums offsets up the ancestor chain.
pub(super) fn parent_of(placements: &[PanePlacement], id: PaneId) -> Option<PaneId> {
    return placement_by_id(placements, id)?.parent
}
