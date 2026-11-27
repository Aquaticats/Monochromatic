//! Debug-only green boxes and labels drawn over each sibling-group lane.
//!
//! Nothing here runs unless the debug-tint environment variable is set.
//!  Each overlay is drawn from the same rectangle the sticky offsets use,
//!  so a screenshot names the exact lane a reader means.

/// What:
///  imports concrete GTK alignment used by debug overlay positioning.
/// Why:
///  debug rails need explicit alignment inside the strip overlay.
use gtk4::Align;
/// What:
///  imports GTK widget traits.
/// Why:
///  lane sync moves fixed children and places debug overlays.
use gtk4::prelude::*;

/// What:
///  imports pane geometry constants.
/// Why:
///  lane boxes span whole pane columns and the gaps between them.
use crate::constants::{
    PANE_GAP,
    PANE_WIDTH,
};
/// What:
///  imports debug-tint helpers.
/// Why:
///  debug mode draws rounded green lane boxes with labels.
use crate::debug_tint;

/// What:
///  imports the placement extents that replace iterator-adapter closures.
/// Why:
///  a lane's column and row span are named loop functions.
use super::super::placement_queries::{
    last_column_of,
    last_row_of,
    placement_by_id,
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
///  imports lane geometry helpers and the rectangle type.
/// Why:
///  a debug box is drawn from the same rectangle the sticky offsets use.
use super::geometry::{
    LaneRect,
    direct_child_groups,
    lane_base_bottom,
    lane_width,
};

/// What:
///  debug child-lane overlay drawing for the layout adapter.
/// Why:
///  lane rectangles live over the visible column viewports,
///  so every scroll change moves them even when pane placements have not changed.
impl StripLayout {
    /// What:
    ///  redraw debug child-lane overlays from the latest placement snapshot.
    /// Why:
    ///  lane rectangles live over the visible column viewports,
    ///  so scroll changes require moving
    ///      them even when pane placements have not changed.
    pub(in crate::layout) fn refresh_child_lanes(&self) {
        self.clear_child_lanes();
        if !debug_tint::enabled() {
            return;
        }
        let placements = self.placements.borrow().clone();
        let groups = direct_child_groups(&placements);
        for (parent, children) in groups {
            if let Some(parent_placement) = placement_by_id(&placements, parent) {
                self.add_child_lane(parent_placement, &children);
            }
        }
    }

    /// What:
    ///  clear every debug child-lane overlay.
    /// Why:
    ///  lane geometry depends on placement and current offsets,
    ///  so redraw is simpler and less
    ///      error-prone than incremental geometry updates.
    fn clear_child_lanes(&self) {
        for (_, lane) in self.lanes.borrow_mut().drain() {
            self.strip_overlay.remove_overlay(&lane);
        }
    }

    /// What:
    ///  compute a lane's fixed app-layout rectangle.
    /// Why:
    ///  green boxes are rails in the broader layout;
    ///  panes may stick inside them,
    ///  but the boxes
    ///      themselves must not receive sticky offsets.
    fn lane_rect(
        &self,
        parent: PanePlacement,
        children: &[PanePlacement],
    ) -> LaneRect {
        let end_column = last_column_of(children, parent.column);
        let column_span = end_column - parent.column + 1;
        let x = parent.column as f64 * f64::from(PANE_WIDTH + PANE_GAP);
        let y = scroll::row_y(parent.row);
        let width = lane_width(column_span);
        let height = lane_base_bottom(parent, children) - scroll::row_y(parent.row);
        return LaneRect {
            x,
            y,
            width,
            height,
        };
    }

    /// What:
    ///  draw one immediate-child lane around `parent` and its direct child panes.
    /// Why:
    ///  a lane spans from the parent column through the child column and from the parent row
    ///      down to the deepest direct child's bottom,
    ///  including empty grid cells in that span.
    fn add_child_lane(
        &self,
        parent: PanePlacement,
        children: &[PanePlacement],
    ) {
        let rect = self.lane_rect(parent, children);
        let max_child_row = last_row_of(children, parent.row);
        let end_column = last_column_of(children, parent.column);
        let detail = format!(
            "parent={} columns={}..{} rows={}..{} children={}",
            parent.id.0,
            parent.column,
            end_column,
            parent.row,
            max_child_row,
            children.len()
        );
        let lane = debug_tint::lane(
            debug_tint::Y6L_CHILD_LANE,
            Some(&detail),
            rect.width as i32,
            rect.height as i32,
        );
        lane.set_halign(Align::Start);
        lane.set_valign(Align::Start);
        lane.set_margin_start(rect.x as i32);
        lane.set_margin_top(rect.y as i32);
        self.strip_overlay.add_overlay(&lane);
        self.strip_overlay.set_measure_overlay(&lane, false);
        self.lanes.borrow_mut().insert(parent.id, lane);
    }
}
