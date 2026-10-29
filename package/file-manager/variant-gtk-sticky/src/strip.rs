//! Pane-strip controller:
//!  mutates the shared pane model and delegates GTK layout to `layout.rs`.
//!
//! The controller owns the original crate's pure `PaneStripState` (spawn/dedup/close rules reused
//! verbatim),
//!  turns row activations into model mutations,
//!  hands placement snapshots to
//! `StickyLayout`,
//!  and mirrors observable state for the boundary test after every mutation and
//! scroll change.

/// What:
///  imports interior-mutability cells.
/// Why:
///  the model and directory-read generation mutate from GTK closures.
use std::cell::{Cell, RefCell};
/// What:
///  imports the borrowed path type.
/// Why:
///  the root pane and directory reads take paths by reference.
use std::path::Path;
/// What:
///  imports the reference-counted pointer.
/// Why:
///  the controller shares one `StripInner` with activation closures via `Weak` handles.
use std::rc::Rc;

/// What:
///  imports the GTK widget-extension traits.
/// Why:
///  pane builders upcast their concrete widgets to `Widget`.
use gtk4::prelude::*;
/// What:
///  imports concrete GTK fallback widget types.
/// Why:
///  a missing pane or failed directory read degrades to a label widget.
use gtk4::{Label, Widget};

/// What:
///  imports the shared pane-strip state machine from the original app's crate.
/// Why:
///  this variant reuses the original model verbatim;
///  only the layout engine differs.
use file_manager::model::PaneStripState;
/// What:
///  imports entry,
///  id,
///  and location domain types from the original app's crate.
/// Why:
///  activating an entry maps it to a `PaneLocation` and returns a stable `PaneId`.
use file_manager::types::{FileEntry, PaneId, PaneLocation};

/// What:
///  imports the placement snapshot type shared with the pure band math.
/// Why:
///  reconciliation converts model panes into placements for the layout and state output.
use crate::band::Placement;
/// What:
///  imports the sticky layout adapter.
/// Why:
///  this controller delegates GTK reconciliation and reveal to it.
use crate::layout::StickyLayout;
/// What:
///  imports listing and preview pane builders.
/// Why:
///  a directory pane is a listing pane;
///  a preview pane renders a typed icon.
use crate::pane::{build_listing_pane, build_preview_pane};
/// What:
///  imports the observed-state writer and its input bundle.
/// Why:
///  boundary tests poll the mirrored state file after every mutation and scroll.
use crate::state_out::{ObservedInputs, write_observed_state};

/// What:
///  shared controller state captured by GTK closures.
/// Why:
///  row activation needs to mutate the model,
///  reconcile the layout,
///  and scroll to the result;
///      closures hold weak references so they never keep the strip alive accidentally.
pub(crate) struct StripInner {
    /// Sticky GTK layout adapter for pane placement and scrolling.
    pub(crate) layout: Rc<StickyLayout>,
    /// The shared pane-strip state machine.
    pub(crate) state: RefCell<PaneStripState>,
    /// Monotonic directory-read generation.
    generation: Cell<u64>,
    /// Whether the top-level window has mapped;
    ///  mirrored as the observed `ready` fact so boundary
    /// tests never send keys to a surface that does not exist yet.
    mapped: Cell<bool>,
}

/// What:
///  owning handle to a built strip.
/// Why:
///  the activation closures hold only `Weak<StripInner>`,
///  so this strong handle must outlive
///      the window or spawning would silently stop working.
pub struct StripController {
    /// The shared inner state.
    inner: Rc<StripInner>,
}

/// What:
///  public surface for constructing the strip and exposing its root widget.
/// Why:
///  callers build a strip,
///  place its widget,
///  and keep the handle alive.
impl StripController {
    /// What:
    ///  build a strip rooted at `root`,
    ///  render its root pane,
    ///  and return the controller.
    /// Why:
    ///  the root directory opens in column 0;
    ///  reconciliation creates the first column and
    ///      pane,
    ///  keyboard navigation and scroll-driven state output are wired once here.
    pub fn new(root: &Path) -> Self {
        let layout = StickyLayout::new();
        let inner = Rc::new(StripInner {
            layout,
            state: RefCell::new(PaneStripState::new()),
            generation: Cell::new(0),
            mapped: Cell::new(false),
        });
        let root_id = inner
            .state
            .borrow_mut()
            .open_root(PaneLocation::Directory(root.to_path_buf()));
        reconcile(&inner);
        crate::keys::install_strip_keys(&inner);
        let observer = Rc::downgrade(&inner);
        inner.layout.set_on_scroll(Box::new(move || {
            if let Some(upgraded) = observer.upgrade() {
                write_state(&upgraded);
            }
        }));
        inner.layout.scroll_to_pane(root_id);
        return Self { inner }
    }

    /// What:
    ///  clone the root widget to place in the window.
    /// Why:
    ///  the concrete GTK root lives behind `StickyLayout` so callers do not depend on it.
    pub fn widget(&self) -> Widget {
        return self.inner.layout.widget()
    }

    /// What:
    ///  mark the observed state ready once `window` maps,
    ///  and mirror state at that moment.
    /// Why:
    ///  the compositor drops keystrokes sent before the surface exists,
    ///  so boundary tests
    ///      gate their first key on the observed `ready` fact flipping true.
    pub(crate) fn wire_window_map(&self, window: &gtk4::ApplicationWindow) {
        let weak = Rc::downgrade(&self.inner);
        window.connect_map(move |_| {
            if let Some(inner) = weak.upgrade() {
                inner.mapped.set(true);
                write_state(&inner);
            }
        });
    }
}

/// What:
///  close the model-active pane,
///  if any.
/// Why:
///  the Backspace key closes whatever pane the model considers focused,
///  matching the Electron
///      prototype so both boundary tests share their close step.
pub(crate) fn close_active(inner: &Rc<StripInner>) {
    let active = inner.state.borrow().active();
    if let Some(id) = active {
        close_pane(inner, id);
    }
}

/// What:
///  record pane `id` as the model-active pane and mirror state.
/// Why:
///  keyboard column navigation moves both widget focus and the model's focus,
///  so the observed
///      `activePath` follows what the user sees.
pub(crate) fn focus_pane(inner: &Rc<StripInner>, id: PaneId) {
    inner.state.borrow_mut().focus(id);
    write_state(inner);
}

/// What:
///  mint and record the next directory-read generation.
/// Why:
///  newer reads supersede stale snapshots;
///  the counter only increases.
fn next_generation(inner: &Rc<StripInner>) -> u64 {
    let generation = inner.generation.get() + 1;
    inner.generation.set(generation);
    return generation
}

/// What:
///  convert the model's panes into placement snapshots for layout and state output.
/// Why:
///  both consumers need only grid coordinates and parent links.
fn placements_of(inner: &Rc<StripInner>) -> Vec<Placement> {
    return inner
        .state
        .borrow()
        .panes()
        .map(|pane| return Placement {
            id: pane.id,
            column: pane.column,
            row: pane.row,
            parent: pane.parent,
        })
        .collect()
}

/// What:
///  reconcile layout to the current pane-state snapshot,
///  then mirror observable state.
/// Why:
///  after any model mutation,
///  GTK placement updates through one narrow interface and the
///      boundary test sees the new state.
fn reconcile(inner: &Rc<StripInner>) {
    let placements = placements_of(inner);
    inner
        .layout
        .reconcile(placements, |id| return build_pane_widget(inner, id));
    write_state(inner);
}

/// What:
///  mirror the current observable state into the boundary-test file.
/// Why:
///  recomputed from the model plus the pure band math,
///  never from widget geometry,
///  so the
///      reported facts are exactly what the unit-tested formulas say.
fn write_state(inner: &Rc<StripInner>) {
    let state = inner.state.borrow();
    let active_path = state
        .active()
        .and_then(|id| return state.pane(id))
        .map(|pane| match &pane.location {
            PaneLocation::Directory(path) | PaneLocation::Preview(path) => {
                return path.display().to_string()
            }
        })
        .unwrap_or_default();
    let inputs = ObservedInputs {
        active_path,
        column_count: state.column_count(),
        pane_count: state.len(),
        placements: placements_of(inner),
        ready: inner.mapped.get(),
        scroll: inner.layout.vertical_scroll(),
    };
    drop(state);
    write_observed_state(&inputs);
}

/// What:
///  build the widget for pane `id` from its location.
/// Why:
///  a directory becomes a listing pane wired to spawn;
///  a preview becomes an icon pane;
///  a
///      missing pane degrades to a label.
fn build_pane_widget(inner: &Rc<StripInner>, id: PaneId) -> Widget {
    let location = inner.state.borrow().pane(id).map(|pane| return pane.location.clone());
    match location {
        Some(PaneLocation::Directory(path)) => return build_directory_pane(inner, id, &path),
        Some(PaneLocation::Preview(path)) => {
            let close_weak = Rc::downgrade(inner);
            return build_preview_pane(&path, move || {
                if let Some(upgraded) = close_weak.upgrade() {
                    close_pane(&upgraded, id);
                }
            })
            .upcast::<Widget>()
        }
        None => return Label::new(Some("(missing pane)")).upcast::<Widget>(),
    }
}

/// What:
///  build a directory listing pane for `id` at `path`,
///  wiring row activation to spawn a child.
/// Why:
///  activating a row spawns `path`'s child from this pane via a weak back-reference,
///  so the
///      closure never keeps the strip alive.
fn build_directory_pane(inner: &Rc<StripInner>, id: PaneId, path: &Path) -> Widget {
    let generation = next_generation(inner);
    match file_manager::fs::read_directory(path, generation) {
        Ok(snapshot) => {
            let spawn_weak = Rc::downgrade(inner);
            let close_weak = Rc::downgrade(inner);
            return build_listing_pane(
                &snapshot,
                move |entry, force_dup| {
                    if let Some(upgraded) = spawn_weak.upgrade() {
                        spawn_from(&upgraded, id, entry, force_dup);
                    }
                },
                move || {
                    if let Some(upgraded) = close_weak.upgrade() {
                        close_pane(&upgraded, id);
                    }
                },
            )
            .upcast::<Widget>()
        }
        Err(error) => {
            tracing::error!(%error, path = %path.display(), "failed to read directory for pane");
            return Label::new(Some(&format!("Cannot read {}: {error}", path.display()))).upcast::<Widget>()
        }
    }
}

/// What:
///  spawn a child pane from `source` for the activated `entry`,
///  reconcile,
///  and reveal it;
///       `force_duplicate` skips dedup to mint a fresh pane.
/// Why:
///  a directory (following symlinks) spawns a listing,
///  anything else a preview;
///      dedup-and-focus lives in the shared model,
///  so a revisit focuses the existing pane.
fn spawn_from(inner: &Rc<StripInner>, source: PaneId, entry: &FileEntry, force_duplicate: bool) {
    let location = if entry.path.is_dir() {
        PaneLocation::Directory(entry.path.clone())
    } else {
        PaneLocation::Preview(entry.path.clone())
    };
    let spawned = inner
        .state
        .borrow_mut()
        .spawn_child(source, location, force_duplicate);
    if let Some(column) = inner.state.borrow().pane(spawned).map(|pane| return pane.column) {
        inner.layout.set_focused_column(column);
    }
    reconcile(inner);
    inner.layout.scroll_to_pane(spawned);
    tracing::info!(
        source = source.0,
        spawned = spawned.0,
        entry = %entry.path.display(),
        panes = inner.state.borrow().len(),
        "spawned child pane"
    );
}

/// What:
///  close pane `id` and reconcile so its widget leaves and the tree re-lays-out.
/// Why:
///  explicit close is the only way a pane dies;
///  reconciliation drops the widget and mirrors
///      the new state.
fn close_pane(inner: &Rc<StripInner>, id: PaneId) {
    inner.state.borrow_mut().close(id);
    reconcile(inner);
    tracing::info!(closed = id.0, panes = inner.state.borrow().len(), "closed pane");
}
