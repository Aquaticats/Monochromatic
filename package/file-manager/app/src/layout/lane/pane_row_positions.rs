//! Where each pane sits vertically inside its column.
//!
//! A pane's visible row starts from its sticky target,
//!  is clamped by every green rail that contains it,
//!  and is then spaced against its neighbours so no two panes overlap.

/// What:
///  imports the hash-map container.
/// Why:
///  resolved positions are keyed by stable `PaneId`,
///  and each column's panes are collected before they are stacked.
use std::collections::HashMap;

/// What:
///  imports GTK widget traits.
/// Why:
///  moving a pane inside its column's fixed canvas is a widget-trait method.
use gtk4::prelude::*;

/// What:
///  imports pane and viewport geometry constants.
/// Why:
///  the vertical stride between siblings and the rail bound come from one pixel grid.
use crate::constants::{
    PANE_GAP,
    PANE_HEIGHT,
};
/// What:
///  imports the stable pane identity type.
/// Why:
///  resolved positions are keyed by pane id.
use crate::types::PaneId;

/// What:
///  imports the placement queries that replace iterator-adapter closures.
/// Why:
///  lookups,
///  containment and sort keys over placement snapshots are named loop functions.
use super::super::placement_queries::{
    lane_contains,
    parent_of,
    placement_by_id,
    row_then_id,
};
/// What:
///  imports parent layout types and the row-coordinate helper.
/// Why:
///  these methods extend `StripLayout` and read `PanePlacement` snapshots.
use super::super::{
    PanePlacement,
    StripLayout,
    scroll,
};
/// What:
///  imports lane geometry helpers.
/// Why:
///  sibling grouping,
///  the lane's bottom edge and the rail bounds are pure calculations.
use super::geometry::{
    direct_child_groups,
    lane_base_bottom,
    lowered_bound,
    raised_bound,
};

/// What:
///  vertical position resolution for every pane widget.
/// Why:
///  rendering,
///  reveal and rail bounds must all agree on where one pane sits.
impl StripLayout {
    /// What:
    ///  move every pane widget to its resolved non-overlapping visual row.
    /// Why:
    ///  columns are static canvases,
    ///  so lane sticky offsets are implemented by moving panes while
    ///      preserving row order and rail bounds.
    pub(in crate::layout) fn position_all_widgets(&self) {
        let placements = self.placements.borrow().clone();
        let positions = self.resolved_y_positions(&placements);
        for placement in &placements {
            let widget = self.widgets.borrow().get(&placement.id).cloned();
            let Some(present_widget) = widget else {
                continue;
            };
            let y = match positions.get(&placement.id) {
                Some(resolved) => *resolved,
                None => self.desired_y_for_pane(*placement),
            };
            if let Some(view) = self.columns.borrow().get(placement.column) {
                view.fixed.move_(&present_widget, 0.0, y);
            }
        }
    }

    /// What:
    ///  compute `placement`'s resolved content y-coordinate.
    /// Why:
    ///  reveal code needs the same non-overlap and rail-bound position used by rendering.
    pub(in crate::layout::lane) fn visual_y_for_pane(
        &self,
        placement: PanePlacement,
    ) -> f64 {
        let placements = self.placements.borrow().clone();
        match self.resolved_y_positions(&placements).get(&placement.id) {
            Some(resolved) => return *resolved,
            None => return self.desired_y_for_pane(placement),
        }
    }

    /// What:
    ///  resolve all pane y positions without overlap inside every green rail.
    /// Why:
    ///  individual clamping can collapse siblings onto one boundary;
    ///  resolving per column keeps
    ///      the row stack ordered while respecting each pane's rail interval.
    fn resolved_y_positions(
        &self,
        placements: &[PanePlacement],
    ) -> HashMap<PaneId, f64> {
        let groups = direct_child_groups(placements);
        let mut columns: HashMap<usize, Vec<PanePlacement>> = HashMap::new();
        for placement in placements {
            columns
                .entry(placement.column)
                .or_default()
                .push(*placement);
        }
        let mut positions = HashMap::new();
        for (_, mut column) in columns {
            column.sort_by_key(row_then_id);
            let resolved = self.resolve_column_positions(&column, placements, &groups);
            for (id, y) in resolved {
                positions.insert(id, y);
            }
        }
        return positions;
    }

    /// What:
    ///  resolve one column's pane positions with forward/backward spacing passes.
    /// Why:
    ///  preserving `PANE_HEIGHT + PANE_GAP` spacing prevents overlap while clamping to rail
    ///      intervals keeps panes inside their green boxes.
    fn resolve_column_positions(
        &self,
        column: &[PanePlacement],
        placements: &[PanePlacement],
        groups: &HashMap<PaneId, Vec<PanePlacement>>,
    ) -> Vec<(PaneId, f64)> {
        let stride = f64::from(PANE_HEIGHT + PANE_GAP);
        let mut rows: Vec<(PaneId, f64, f64, f64)> = Vec::new();
        for placement in column {
            let (min_y, max_y) = match self.allowed_y_for_pane_in(*placement, placements, groups) {
                Some(bounds) => bounds,
                None => (f64::NEG_INFINITY, f64::INFINITY),
            };
            let y = self.desired_y_for_pane(*placement).clamp(min_y, max_y);
            rows.push((placement.id, y, min_y, max_y));
        }
        for index in 1..rows.len() {
            let floor = rows[index - 1].1 + stride;
            rows[index].1 = rows[index].1.max(floor).min(rows[index].3);
        }
        for index in (0..rows.len().saturating_sub(1)).rev() {
            let ceiling = rows[index + 1].1 - stride;
            rows[index].1 = rows[index].1.min(ceiling).max(rows[index].2);
        }
        let mut resolved: Vec<(PaneId, f64)> = Vec::new();
        for (id, y, _, _) in rows {
            resolved.push((id, y));
        }
        return resolved;
    }

    /// What:
    ///  compute desired sticky content y before rail and overlap constraints.
    /// Why:
    ///  resolved layout starts from the sticky target,
    ///  then constrains it.
    fn desired_y_for_pane(
        &self,
        placement: PanePlacement,
    ) -> f64 {
        return scroll::row_y(placement.row) + self.effective_offset_for_pane(placement.id);
    }

    /// What:
    ///  compute vertical bounds shared by every green rail containing `placement`.
    /// Why:
    ///  a pane can be both a child in one lane and the parent of another;
    ///  both rails constrain it.
    fn allowed_y_for_pane_in(
        &self,
        placement: PanePlacement,
        placements: &[PanePlacement],
        groups: &HashMap<PaneId, Vec<PanePlacement>>,
    ) -> Option<(f64, f64)> {
        let mut min_y: Option<f64> = None;
        let mut max_y: Option<f64> = None;
        for (parent, children) in groups {
            if !lane_contains(*parent, children, placement.id) {
                continue;
            }
            let Some(parent_placement) = placement_by_id(placements, *parent) else {
                continue;
            };
            let top = scroll::row_y(parent_placement.row);
            let bottom = lane_base_bottom(parent_placement, children) - f64::from(PANE_HEIGHT);
            min_y = Some(raised_bound(min_y, top));
            max_y = Some(lowered_bound(max_y, bottom));
        }
        let (top_bound, bottom_bound) = (min_y?, max_y?);
        return Some((top_bound, bottom_bound.max(top_bound)));
    }

    /// What:
    ///  compute lane offsets affecting `id`.
    /// Why:
    ///  non-root panes use their parent lane;
    ///  root panes have no parent lane,
    ///  so they use their
    ///      own root lane.
    fn effective_offset_for_pane(
        &self,
        id: PaneId,
    ) -> f64 {
        let placements = self.placements.borrow();
        let offsets = self.lane_offsets.borrow();
        let Some(placement) = placement_by_id(&placements, id) else {
            return 0.0;
        };
        let mut total = 0.0;
        let mut current = placement.parent.or(Some(id));
        while let Some(pane_id) = current {
            total += offsets.get(&pane_id).copied().unwrap_or(0.0);
            current = parent_of(&placements, pane_id);
        }
        return total;
    }
}
