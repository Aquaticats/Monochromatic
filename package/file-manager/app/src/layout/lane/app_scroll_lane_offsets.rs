//! Whole-app vertical scrolling turned into each lane's sticky offset.
//!
//! The outer scroller moves first.
//!  Each sibling-group lane then applies its own bounded offset,
//!  which tries to keep its panes visible without preventing that scroll.

/// What:
///  imports the reference-counted pointer.
/// Why:
///  GTK adjustment callbacks hold weak references to `StripLayout`.
use std::rc::Rc;

/// What:
///  imports GTK adjustment and widget traits.
/// Why:
///  lane sync reads the outer vertical adjustment and reacts to its changes.
use gtk4::prelude::*;

/// What:
///  imports pane and viewport geometry constants.
/// Why:
///  lane boxes,
///  sticky offsets,
///  and clamping share the same pixel grid.
use crate::constants::{
    DEFAULT_HEIGHT,
    PANE_HEIGHT,
};
/// What:
///  imports the stable pane identity type.
/// Why:
///  lane offsets are keyed by parent pane id.
use crate::types::PaneId;

/// What:
///  imports the placement lookup by id.
/// Why:
///  a lane's offset starts from its parent pane's row.
use super::super::placement_queries::placement_by_id;
/// What:
///  imports parent layout types and the row-coordinate helper.
/// Why:
///  these methods extend `StripLayout` and use `PanePlacement` snapshots.
use super::super::{
    PanePlacement,
    StripLayout,
    scroll,
};
/// What:
///  imports lane geometry helpers.
/// Why:
///  a lane's maximum offset comes from its deepest bottom edge.
use super::geometry::{
    direct_child_groups,
    lane_base_bottom,
};

/// What:
///  app-scroll sync and vertical reveal for the layout adapter.
/// Why:
///  vertical scroll ownership belongs to the whole app,
///  while lane offsets and reveal react here.
impl StripLayout {
    /// What:
    ///  sync lane offsets whenever the whole-app vertical scroller moves.
    /// Why:
    ///  wheel input should scroll the entire app first;
    ///  lanes then independently try to remain
    ///      visible within their green-box limits.
    pub(in crate::layout) fn install_app_scroll_sync(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.outer.vadjustment().connect_value_changed(move |_| {
            if let Some(layout) = weak.upgrade() {
                layout.sync_lane_offsets_to_app_scroll();
            }
        });
    }

    /// What:
    ///  prune stale lane offsets and clamp live offsets to their current scroll ranges.
    /// Why:
    ///  closing panes or relayout can remove lanes or shrink how far they may scroll.
    pub(in crate::layout) fn prune_lane_offsets(&self) {
        let placements = self.placements.borrow().clone();
        let groups = direct_child_groups(&placements);
        let active: Vec<PaneId> = groups.keys().copied().collect();
        let mut offsets = self.lane_offsets.borrow_mut();
        let mut dropped: Vec<PaneId> = Vec::new();
        for parent in offsets.keys() {
            if !active.contains(parent) {
                dropped.push(*parent);
            }
        }
        for parent in dropped {
            offsets.remove(&parent);
        }
        drop(offsets);
        let mut updates = Vec::new();
        for parent in active {
            if let Some(parent_placement) = placement_by_id(&placements, parent)
                && let Some(children) = groups.get(&parent)
            {
                let max = self.lane_max_offset(parent_placement, children);
                let current = self
                    .lane_offsets
                    .borrow()
                    .get(&parent)
                    .copied()
                    .unwrap_or(0.0);
                updates.push((parent, current.clamp(0.0, max)));
            }
        }
        for (parent, value) in updates {
            self.lane_offsets.borrow_mut().insert(parent, value);
        }
        self.sync_lane_offsets_to_app_scroll();
    }

    /// What:
    ///  reveal `placement` vertically by adjusting the whole-app vertical scroller.
    /// Why:
    ///  spawning a child should bring that child into view while lanes react to app scroll.
    pub(in crate::layout) fn reveal_lane_member(
        &self,
        placement: PanePlacement,
    ) -> bool {
        let adj = self.outer.vadjustment();
        let page = adj.page_size();
        let value = adj.value();
        let start = self.visual_y_for_pane(placement);
        if start >= value && start + f64::from(PANE_HEIGHT) <= value + page {
            return true;
        }
        let max = (adj.upper() - page).max(0.0);
        let target = if start < value {
            start
        } else {
            start + f64::from(PANE_HEIGHT) - page
        };
        adj.set_value(target.clamp(0.0, max));
        self.sync_lane_offsets_to_app_scroll();
        let settled = adj.value();
        // Recomputed after the scroll settled; the name differs from the pre-scroll
        // `start` so the second measurement never shadows the first.
        let final_start = self.visual_y_for_pane(placement);
        return final_start >= settled && final_start + f64::from(PANE_HEIGHT) <= settled + page;
    }

    /// What:
    ///  recompute every sibling-group lane offset from the whole-app vertical scroll.
    /// Why:
    ///  the app scrolls first;
    ///  each green-box lane then applies its own bounded sticky offset so
    ///      it tries to stay visible without preventing the app scroll.
    fn sync_lane_offsets_to_app_scroll(&self) {
        let placements = self.placements.borrow().clone();
        let groups = direct_child_groups(&placements);
        let scroll = self.outer.vadjustment().value();
        let mut updates: Vec<(PaneId, f64)> = Vec::new();
        for (parent, children) in &groups {
            let Some(parent_placement) = placement_by_id(&placements, *parent) else {
                continue;
            };
            let max = self.lane_max_offset(parent_placement, children);
            let top = scroll::row_y(parent_placement.row);
            updates.push((*parent, (scroll - top).clamp(0.0, max)));
        }
        for (parent, next) in updates {
            self.lane_offsets.borrow_mut().insert(parent, next);
        }
        self.position_all_widgets();
        self.refresh_child_lanes();
    }

    /// What:
    ///  choose the maximum scroll offset for a lane.
    /// Why:
    ///  at max offset the lane's deepest direct child bottom reaches the viewport bottom.
    fn lane_max_offset(
        &self,
        parent: PanePlacement,
        children: &[PanePlacement],
    ) -> f64 {
        let bottom = lane_base_bottom(parent, children);
        return (bottom - self.viewport_height()).max(0.0);
    }

    /// What:
    ///  visible viewport height in pixels.
    /// Why:
    ///  lane max offsets and reveal behavior clamp against the actual window,
    ///  with startup
    ///      fallback before GTK has allocated a height.
    fn viewport_height(&self) -> f64 {
        let height = self.outer.height();
        if height > 0 {
            return f64::from(height);
        }
        return f64::from(DEFAULT_HEIGHT);
    }
}
