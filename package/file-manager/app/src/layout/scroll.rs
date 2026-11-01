//! Horizontal reveal helpers for `StripLayout`.
//!
//! Vertical movement belongs to lane groups (`layout/lane.rs`).
//!  This child module keeps the small
//! amount of remaining scroll work:
//!  reveal a newly spawned column horizontally,
//!  ask the lane module
//! to reveal the pane vertically,
//!  and expose the shared row-to-pixel helper.

/// What:
///  imports the single-slot cell.
/// Why:
///  reveal retries count attempts in a `Cell` captured by the timer closure.
use std::cell::Cell;
/// What:
///  imports the reference-counted pointer.
/// Why:
///  reveal timers hold weak references to the layout adapter.
use std::rc::Rc;

/// What:
///  imports GTK adjustment and focus extension traits.
/// Why:
///  reveal reads/writes the horizontal scroll adjustment and focuses pane widgets.
use gtk4::prelude::*;
/// What:
///  imports the scroll-adjustment type and GLib timer module.
/// Why:
///  horizontal reveal operates on `Adjustment`;
///  retries use GLib timers.
use gtk4::{Adjustment, glib};

/// What:
///  imports pane geometry constants.
/// Why:
///  reveal extents and row offsets share pane geometry.
use crate::constants::{PANE_GAP, PANE_HEIGHT, PANE_WIDTH};
/// What:
///  imports stable pane identity.
/// Why:
///  reveal targets a pane widget by id.
use crate::types::PaneId;

/// What:
///  imports the parent module's layout type.
/// Why:
///  this module extends `StripLayout` while retaining access to its private fields.
use super::StripLayout;

/// What:
///  how many timed passes to retry revealing a spawned pane before giving up.
/// Why:
///  scroll bounds settle a layout pass after content changes;
///  the retry must terminate even if a
///      pane can never fully fit.
const MAX_REVEAL_ATTEMPTS: u32 = 20;

/// What:
///  milliseconds between reveal retries.
/// Why:
///  a real delay yields to the frame clock and layout between attempts so scroll bounds update.
const REVEAL_INTERVAL_MS: u64 = 8;

/// What:
///  horizontal reveal methods on the deep layout adapter.
/// Why:
///  placement callers get reveal through `StripLayout` without touching GTK pieces.
impl StripLayout {
    /// What:
    ///  reveal pane `id` horizontally and vertically,
    ///  then focus it.
    /// Why:
    ///  a spawn must bring the newly focused pane into view even though GTK updates scroll
    ///      bounds one layout pass after reconciliation.
    pub(crate) fn scroll_to_pane(self: &Rc<Self>, id: PaneId) {
        let Some(placement) = self
            .placements
            .borrow()
            .iter()
            .find(|placement| return placement.id == id)
            .copied()
        else {
            return;
        };
        let outer_h = self.outer.hadjustment();
        let column_x = placement.column as f64 * f64::from(PANE_WIDTH + PANE_GAP);
        let pane = self.widgets.borrow().get(&id).cloned();
        let attempts = Cell::new(0u32);
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(
            std::time::Duration::from_millis(REVEAL_INTERVAL_MS),
            move || {
                let Some(layout) = weak.upgrade() else {
                    return glib::ControlFlow::Break;
                };
                let horizontal = reveal(&outer_h, column_x, f64::from(PANE_WIDTH));
                let vertical = layout.reveal_lane_member(placement);
                attempts.set(attempts.get() + 1);
                if (horizontal && vertical) || attempts.get() >= MAX_REVEAL_ATTEMPTS {
                    if let Some(focus_target) = &pane {
                        focus_target.grab_focus();
                    }
                    return glib::ControlFlow::Break;
                }
                return glib::ControlFlow::Continue
            },
        );
    }
}

/// What:
///  vertical pixel offset of `row` within the pane grid.
/// Why:
///  panes tile down each column at a fixed stride shared by every lane and column.
pub(super) fn row_y(row: usize) -> f64 {
    return row as f64 * f64::from(PANE_HEIGHT + PANE_GAP)
}

/// What:
///  scroll `adj` so `[start, start + extent)` is fully visible,
///  returning whether it is.
/// Why:
///  callers retry until layout settles because scroll bounds update after content changes.
fn reveal(adj: &Adjustment, start: f64, extent: f64) -> bool {
    let page = adj.page_size();
    let value = adj.value();
    if start >= value && start + extent <= value + page {
        return true;
    }
    let max = (adj.upper() - page).max(0.0);
    let target = if start < value {
        start
    } else {
        start + extent - page
    };
    adj.set_value(target.clamp(0.0, max));
    let settled = adj.value();
    return start >= settled && start + extent <= settled + page
}
