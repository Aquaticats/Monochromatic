//! Deep GTK layout adapter for the pane strip.
//!
//! The product model only says that a pane has a `(column, row)` placement and an optional parent.
//! This module hides every GTK detail needed to render that placement graph:
//!  the outer horizontal
//! scroller,
//!  static per-column fixed canvases,
//!  pane-widget map,
//!  app-owned vertical scrolling,
//!  lane
//! sticky offsets,
//!  and reveal retries.
//!  `strip.rs` now only mutates the pane model and asks this
//! adapter to reconcile a placement snapshot.

/// What:
///  imports cells for closure-captured GTK state.
/// Why:
///  focus state and scroll epochs mutate from signal handlers.
use std::cell::{Cell, RefCell};
/// What:
///  imports the hash-map container.
/// Why:
///  pane widgets,
///  lane offsets,
///  and debug lane widgets are keyed by stable `PaneId`.
use std::collections::HashMap;
/// What:
///  imports the reference-counted pointer.
/// Why:
///  GTK signal closures hold weak references to the shared layout adapter.
use std::rc::Rc;

/// What:
///  imports GTK widget-extension traits.
/// Why:
///  layout construction,
///  controller installation,
///  focus,
///  fixed positioning,
///  and overlays use
///      prelude methods.
use gtk4::prelude::*;
/// What:
///  imports concrete GTK layout and event-controller types.
/// Why:
///  the strip is built from an outer scroller,
///  overlay,
///  horizontal box,
///  fixed canvases,
///  widgets,
///      and key controllers.
use gtk4::{Box as GtkBox, EventControllerKey, Fixed, Orientation, Overlay, PolicyType, ScrolledWindow, Widget};

/// What:
///  imports pane geometry constants.
/// Why:
///  every placement and lane calculation uses one pane-size source of truth.
use crate::constants::{PANE_HEIGHT, PANE_WIDTH};
/// What:
///  imports debug-tint helpers.
/// Why:
///  debug mode draws labeled overlays for scroll-region and lane debugging.
use crate::debug_tint;
/// What:
///  imports the stable pane identity type.
/// Why:
///  widget maps and placement snapshots are keyed by `PaneId`.
use crate::types::PaneId;

/// What:
///  app-owned vertical scroll sync and debug-lane implementation for `StripLayout`.
/// Why:
///  keeps the layout interface file under the max-lines budget while making lanes react to the
///      whole-app scroll.
mod lane;
/// What:
///  horizontal reveal and row-coordinate helpers for `StripLayout`.
/// Why:
///  spawn still needs horizontal reveal even after vertical scrolling moved to lanes.
mod scroll;

/// What:
///  immutable placement snapshot for one pane.
/// Why:
///  `StripLayout` needs only geometry and parent links,
///  not the full `PaneStripState` interface.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PanePlacement {
    /// Stable pane identity.
    pub(crate) id: PaneId,
    /// Zero-based lineage column.
    pub(crate) column: usize,
    /// Zero-based vertical row within the global row coordinate space.
    pub(crate) row: usize,
    /// Parent pane identity,
    ///  if this pane was spawned from another pane.
    pub(crate) parent: Option<PaneId>,
}

/// What:
///  one static column's GTK widgets.
/// Why:
///  columns are no longer scrollers;
///  they are fixed canvases that lane offsets move panes across.
struct ColumnView {
    /// Root widget inserted into the horizontal column box.
    root: Widget,
    /// Fixed canvas holding pane widgets at lane-offset-adjusted row positions.
    fixed: Fixed,
}

/// What:
///  deep layout module for the pane strip.
/// Why:
///  callers only provide pane placements and widget builders;
///  this implementation owns GTK
///      canvases,
///  focus bookkeeping,
///  app scroll,
///  lane sticky offsets,
///  and reveal behavior.
pub(crate) struct StripLayout {
    /// Outer horizontal scroller holding all columns.
    outer: ScrolledWindow,
    /// Overlay whose main child is the horizontal strip content and whose overlay children are
    /// debug-only cross-column lane rectangles.
    strip_overlay: Overlay,
    /// Horizontal box containing static per-column fixed canvases.
    columns_box: GtkBox,
    /// Static column views by column index.
    columns: RefCell<Vec<ColumnView>>,
    /// Pane widget map by stable pane id.
    widgets: RefCell<HashMap<PaneId, Widget>>,
    /// Vertical scroll offset by lane parent pane id.
    lane_offsets: RefCell<HashMap<PaneId, f64>>,
    /// Debug child-lane overlay widgets by parent pane id.
    lanes: RefCell<HashMap<PaneId, Widget>>,
    /// Latest placement snapshot,
    ///  used by reveal and lane sync without borrowing the model.
    placements: RefCell<Vec<PanePlacement>>,
    /// Column whose pane last received focus,
    ///  used by Left/Right keyboard navigation.
    focused_column: Cell<usize>,
}

/// What:
///  public interface for constructing,
///  reconciling,
///  focusing,
///  and scrolling the strip layout.
/// Why:
///  keeps GTK layout policy behind a narrow seam so `strip.rs` remains a model/controller layer.
impl StripLayout {
    /// What:
    ///  build the GTK layout adapter and return it behind `Rc`.
    /// Why:
    ///  app-scroll sync needs weak references back to the adapter.
    pub(crate) fn new() -> Rc<Self> {
        let columns_box = GtkBox::new(Orientation::Horizontal, crate::constants::PANE_GAP);
        let strip_overlay = Overlay::new();
        strip_overlay.set_child(Some(&columns_box));
        let outer = ScrolledWindow::builder()
            .child(&strip_overlay)
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .hexpand(true)
            .build();
        let layout = Rc::new(Self {
            outer,
            strip_overlay,
            columns_box,
            columns: RefCell::new(Vec::new()),
            widgets: RefCell::new(HashMap::new()),
            lane_offsets: RefCell::new(HashMap::new()),
            lanes: RefCell::new(HashMap::new()),
            placements: RefCell::new(Vec::new()),
            focused_column: Cell::new(0),
        });
        layout.install_app_scroll_sync();
        return layout
    }

    /// What:
    ///  clone the root GTK widget for insertion into the window.
    /// Why:
    ///  callers should not know the root happens to be a `GtkScrolledWindow`.
    pub(crate) fn widget(&self) -> Widget {
        return self.outer.clone().upcast::<Widget>()
    }

    /// What:
    ///  attach a key controller to the layout's root scroller.
    /// Why:
    ///  keyboard navigation is installed by `keys.rs`,
    ///  but the concrete root widget stays hidden.
    pub(crate) fn add_column_key_controller(&self, controller: EventControllerKey) {
        self.outer.add_controller(controller);
    }

    /// What:
    ///  return the column that most recently received pane focus.
    /// Why:
    ///  Left/Right navigation computes its target relative to this value.
    pub(crate) fn focused_column(&self) -> usize {
        return self.focused_column.get()
    }

    /// What:
    ///  record the column that just received pane focus.
    /// Why:
    ///  spawn and keyboard navigation both update the origin for the next Left/Right move.
    pub(crate) fn set_focused_column(&self, column: usize) {
        self.focused_column.set(column);
    }

    /// What:
    ///  focus pane widget `id`,
    ///  returning whether it existed.
    /// Why:
    ///  keyboard navigation can stop event propagation only when focus actually moved.
    pub(crate) fn focus_widget(&self, id: PaneId) -> bool {
        let Some(widget) = self.widgets.borrow().get(&id).cloned() else {
            return false;
        };
        widget.grab_focus();
        return true
    }

    /// What:
    ///  reconcile GTK widgets to `placements`,
    ///  building missing panes through `build_widget`.
    /// Why:
    ///  one idempotent pass after each model mutation grows/shrinks columns,
    ///  removes stale
    ///      widgets,
    ///  creates missing widgets,
    ///  clamps lane offsets,
    ///  positions live widgets,
    ///  and
    ///      redraws debug lanes.
    pub(crate) fn reconcile(
        self: &Rc<Self>,
        placements: Vec<PanePlacement>,
        mut build_widget: impl FnMut(PaneId) -> Widget,
    ) {
        self.placements.replace(placements.clone());
        let column_count = placements
            .iter()
            .map(|placement| return placement.column + 1)
            .max()
            .unwrap_or(0);
        self.ensure_columns(column_count);
        let live: HashMap<PaneId, (usize, usize)> = placements
            .iter()
            .map(|placement| return (placement.id, (placement.column, placement.row)))
            .collect();
        self.remove_stale(&live);
        for placement in &placements {
            self.ensure_pane_widget(placement, &mut build_widget);
        }
        self.set_content_height(&live);
        self.prune_lane_offsets();
        self.position_all_widgets();
        self.refresh_child_lanes();
    }

    /// What:
    ///  grow or shrink the static column views to exactly `count`.
    /// Why:
    ///  descending adds columns and bulk-close removes them;
    ///  each column owns a fixed pane canvas
    ///      but no vertical scroller.
    fn ensure_columns(self: &Rc<Self>, count: usize) {
        let mut columns = self.columns.borrow_mut();
        while columns.len() < count {
            let index = columns.len();
            let fixed = Fixed::new();
            fixed.set_width_request(PANE_WIDTH);
            fixed.set_vexpand(true);
            fixed.add_css_class("fm-canvas");
            let canvas_detail = format!("column={index}");
            let root = debug_tint::wrap(
                &fixed,
                debug_tint::V6C_COLUMN_CANVAS,
                Some(&canvas_detail),
            );
            root.set_width_request(PANE_WIDTH);
            root.add_css_class("fm-column-root");
            self.columns_box.append(&root);
            columns.push(ColumnView { root, fixed });
        }
        while columns.len() > count {
            if let Some(view) = columns.pop() {
                self.columns_box.remove(&view.root);
            }
        }
    }

    /// What:
    ///  remove pane widgets whose ids no longer appear in `live`.
    /// Why:
    ///  closed panes must leave their column canvas and the widget map.
    fn remove_stale(&self, live: &HashMap<PaneId, (usize, usize)>) {
        let stale: Vec<PaneId> = self
            .widgets
            .borrow()
            .keys()
            .copied()
            .filter(|id| return !live.contains_key(id))
            .collect();
        for id in stale {
            if let Some(widget) = self.widgets.borrow_mut().remove(&id)
                && let Some(fixed) = widget.parent().and_downcast::<Fixed>()
            {
                fixed.remove(&widget);
            }
        }
    }

    /// What:
    ///  ensure `placement.id` has a widget in its target column.
    /// Why:
    ///  missing widgets are built and inserted;
    ///  lane positioning happens in one later pass so
    ///      existing and new widgets use the same offset calculation.
    fn ensure_pane_widget(
        &self,
        placement: &PanePlacement,
        build_widget: &mut impl FnMut(PaneId) -> Widget,
    ) {
        if self.widgets.borrow().contains_key(&placement.id) {
            return;
        }
        let widget = build_widget(placement.id);
        widget.set_size_request(PANE_WIDTH, PANE_HEIGHT);
        if let Some(view) = self.columns.borrow().get(placement.column) {
            view.fixed.put(&widget, 0.0, 0.0);
        }
        self.widgets.borrow_mut().insert(placement.id, widget);
    }

    /// What:
    ///  size each static column canvas to that column's deepest pane row.
    /// Why:
    ///  columns keep the shared row coordinate for alignment without adding vertical scroll
    ///      state of their own.
    fn set_content_height(&self, live: &HashMap<PaneId, (usize, usize)>) {
        for (index, view) in self.columns.borrow().iter().enumerate() {
            let max_row = live
                .values()
                .filter_map(|&(column, row)| return (column == index).then_some(row))
                .max()
                .unwrap_or(0);
            let height = (scroll::row_y(max_row) + f64::from(PANE_HEIGHT)) as i32;
            view.fixed.set_size_request(PANE_WIDTH, height);
        }
    }
}
