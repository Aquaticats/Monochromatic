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
///  look up a placement by id.
/// Why:
///  lane maps store parent ids but geometry needs the parent placement snapshot.
pub(super) fn placement_by_id(
    placements: &[PanePlacement],
    id: PaneId,
) -> Option<PanePlacement> {
    return placements
        .iter()
        .find(|placement| return placement.id == id)
        .copied()
}

/// What:
///  base-coordinate bottom edge for a parent-plus-direct-children lane.
/// Why:
///  the lane rectangle includes the parent pane and the deepest direct child pane.
pub(super) fn lane_base_bottom(parent: PanePlacement, children: &[PanePlacement]) -> f64 {
    return children.iter().fold(
        scroll::row_y(parent.row) + f64::from(PANE_HEIGHT),
        |current, child| return current.max(scroll::row_y(child.row) + f64::from(PANE_HEIGHT)),
    )
}

/// What:
///  compute width for a lane spanning `column_span` columns.
/// Why:
///  debug lane boxes must include pane columns and the gaps between them.
pub(super) fn lane_width(column_span: usize) -> f64 {
    return f64::from(column_span as i32 * PANE_WIDTH + (column_span - 1) as i32 * PANE_GAP)
}
