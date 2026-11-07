//! Geometry helpers for lane-owned scrolling.
//!
//! Kept separate so `lane.rs` stays under the Rust max-lines budget while the math remains named.

/// What:
///  imports the hash-map container.
/// Why:
///  lane grouping is keyed by parent `PaneId`.
use std::collections::HashMap;

/// What:
///  imports pane geometry constants.
/// Why:
///  rectangles and lane dimensions share the pane grid.
use crate::constants::{PANE_GAP, PANE_HEIGHT, PANE_WIDTH};
/// What:
///  imports stable pane identity.
/// Why:
///  direct-child groups are keyed by parent pane id.
use crate::types::PaneId;

/// What:
///  imports parent placement type and row-coordinate helper.
/// Why:
///  lane geometry is computed from placement snapshots.
use super::super::{PanePlacement, scroll};

/// What:
///  lane rectangle in strip-overlay coordinates.
/// Why:
///  debug drawing and sticky-offset calculations share one rectangle shape.
#[derive(Clone, Copy)]
pub(super) struct LaneRect {
    /// Left edge in horizontal strip content coordinates.
    pub(super) x: f64,
    /// Top edge in visible viewport coordinates.
    pub(super) y: f64,
    /// Rectangle width in pixels.
    pub(super) width: f64,
    /// Rectangle height in pixels.
    pub(super) height: f64,
}

/// What:
///  compute direct-child groups keyed by their parent pane id.
/// Why:
///  every parent with children owns one lane.
pub(super) fn direct_child_groups(
    placements: &[PanePlacement],
) -> HashMap<PaneId, Vec<PanePlacement>> {
    let mut groups: HashMap<PaneId, Vec<PanePlacement>> = HashMap::new();
    for placement in placements {
        if let Some(parent) = placement.parent {
            groups.entry(parent).or_default().push(*placement);
        }
    }
    return groups
}

/// What:
///  base-coordinate bottom edge for a parent-plus-direct-children lane.
/// Why:
///  the lane rectangle includes the parent pane and the deepest direct child pane.
pub(super) fn lane_base_bottom(parent: PanePlacement, children: &[PanePlacement]) -> f64 {
    let mut bottom = scroll::row_y(parent.row) + f64::from(PANE_HEIGHT);
    for child in children {
        bottom = bottom.max(scroll::row_y(child.row) + f64::from(PANE_HEIGHT));
    }
    return bottom
}

/// What:
///  the larger of `candidate` and the bound so far,
///  or `candidate` when there is no bound yet.
/// Why:
///  the rail floor is the lowest top edge that every containing lane allows.
pub(super) fn raised_bound(current: Option<f64>, candidate: f64) -> f64 {
    match current {
        Some(bound) => return bound.max(candidate),
        None => return candidate,
    }
}

/// What:
///  the smaller of `candidate` and the bound so far,
///  or `candidate` when there is no bound yet.
/// Why:
///  the rail ceiling is the highest bottom edge that every containing lane allows.
pub(super) fn lowered_bound(current: Option<f64>, candidate: f64) -> f64 {
    match current {
        Some(bound) => return bound.min(candidate),
        None => return candidate,
    }
}

/// What:
///  compute width for a lane spanning `column_span` columns.
/// Why:
///  debug lane boxes must include pane columns and the gaps between them.
pub(super) fn lane_width(column_span: usize) -> f64 {
    return f64::from(column_span as i32 * PANE_WIDTH + (column_span - 1) as i32 * PANE_GAP)
}
