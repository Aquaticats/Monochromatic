//! Pane-strip controller:
//!  mutates the pane model and delegates GTK layout to `layout.rs`.
//!
//! The controller owns the pure `PaneStripState`,
//!  directory-read generation,
//!  and thumbnail service.
//! It turns row activations into model mutations,
//!  then hands a placement snapshot to `StripLayout`.
//! Horizontal reveal,
//!  static canvases,
//!  app-owned vertical scrolling,
//!  and lane sticky offsets live
//! behind that layout interface.

/// What:
///  imports the single-slot cell used for the directory-read generation.
/// Why:
///  directory panes get monotonically increasing snapshot generations.
use std::cell::{Cell, RefCell};
/// What:
///  imports the borrowed path type.
/// Why:
///  the root pane and directory reads take paths by reference.
use std::path::Path;
/// What:
///  imports the reference-counted pointer.
/// Why:
///  the controller shares one `StripInner` with row-activation closures via `Weak` handles.
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
///  imports the layout adapter and its placement snapshot type.
/// Why:
///  this controller delegates GTK reconciliation and scrolling to a deep layout module.
use crate::layout::{PanePlacement, StripLayout};
/// What:
///  imports the pane-strip state machine.
/// Why:
///  this module mutates spawn/dedup/close state and reads pane locations.
use crate::model::PaneStripState;
/// What:
///  imports listing and preview pane builders.
/// Why:
///  a directory pane is a listing pane;
///  a preview pane renders a thumbnail or typed icon.
use crate::pane::{build_listing_pane, build_preview_pane};
/// What:
///  imports thumbnail service and image-detection helper.
/// Why:
///  preview panes need the service;
///  autopreview branches on `is_image`.
use crate::thumbs::{Thumbnails, is_image};
/// What:
///  imports entry,
///  id,
///  and location domain types.
/// Why:
///  activating an entry maps it to a `PaneLocation` and returns a stable `PaneId`.
use crate::types::{FileEntry, PaneId, PaneLocation};

/// What:
///  shared controller state captured by GTK closures.
/// Why:
///  row activation needs to mutate the model,
///  reconcile the layout,
///  and scroll to the result;
///      closures hold weak references so they never keep the strip alive accidentally.
pub(crate) struct StripInner {
    /// Deep GTK layout adapter for pane placement and scrolling.
    pub(crate) layout: Rc<StripLayout>,
    /// The pane-strip state machine.
    pub(crate) state: RefCell<PaneStripState>,
    /// Monotonic directory-read generation.
    generation: Cell<u64>,
    /// Off-thread thumbnail decoder plus bounded cache,
    ///  shared by preview panes.
    thumbs: Thumbnails,
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
///  public surface for constructing the strip,
///  exposing its root widget,
///  and driving
///       verification hooks.
/// Why:
///  callers build a strip,
///  place its widget,
///  and keep the handle alive;
///  all interaction flows
///      through the controller.
impl StripController {
    /// What:
    ///  build a strip rooted at `root`,
    ///  render its root pane,
    ///  and return the controller.
    /// Why:
    ///  the root directory opens in column 0;
    ///  reconciliation creates the first column and pane.
    pub fn new(root: &Path) -> Self {
        let layout = StripLayout::new();
        let inner = Rc::new(StripInner {
            layout,
            state: RefCell::new(PaneStripState::new()),
            generation: Cell::new(0),
            thumbs: Thumbnails::start(),
        });
        inner
            .state
            .borrow_mut()
            .open_root(PaneLocation::Directory(root.to_path_buf()));
        reconcile(&inner);
        crate::keys::install_column_nav(&inner);
        return Self { inner }
    }

    /// What:
    ///  clone the root widget to place in the window.
    /// Why:
    ///  the concrete GTK root lives behind `StripLayout` so callers do not depend on it.
    pub fn widget(&self) -> Widget {
        return self.inner.layout.widget()
    }

    /// What:
    ///  programmatically spawn the first sub-directory's child pane from the root,
    ///  then close it.
    /// Why:
    ///  exercises the real activation -> spawn -> reconcile -> close path for unattended
    ///      verification;
    ///  gated behind the autospawn env var by the caller.
    pub fn autospawn_first_dir_for_test(&self) {
        let inner = &self.inner;
        let Some((root_id, path)) = root_directory(inner) else {
            return;
        };
        let generation = next_generation(inner);
        let Ok(snapshot) = crate::fs::read_directory(&path, generation) else {
            return;
        };
        if let Some(entry) = snapshot.entries.iter().find(|entry| return entry.path.is_dir()) {
            let child = spawn_from(inner, root_id, entry, false);
            close_pane(inner, child);
        }
    }

    /// What:
    ///  programmatically spawn a preview pane for the first image file in the start directory.
    /// Why:
    ///  exercises the real activation -> preview -> off-thread decode -> cache path for
    ///      unattended verification;
    ///  gated behind the autopreview env var by the caller.
    pub fn autopreview_first_image_for_test(&self) {
        let inner = &self.inner;
        let Some((root_id, path)) = root_directory(inner) else {
            return;
        };
        let generation = next_generation(inner);
        let Ok(snapshot) = crate::fs::read_directory(&path, generation) else {
            return;
        };
        if let Some(entry) = snapshot
            .entries
            .iter()
            .find(|entry| return !entry.path.is_dir() && is_image(&entry.path))
        {
            spawn_from(inner, root_id, entry, false);
        }
    }
}

/// What:
///  the active root pane's id and directory path,
///  if the active pane is a directory.
/// Why:
///  shared setup for the autospawn/autopreview test helpers.
fn root_directory(inner: &Rc<StripInner>) -> Option<(PaneId, std::path::PathBuf)> {
    let state = inner.state.borrow();
    let id = state.active()?;
    match state.pane(id).map(|pane| return pane.location.clone()) {
        Some(PaneLocation::Directory(path)) => return Some((id, path)),
        _ => return None,
    }
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
///  reconcile layout to the current pane-state snapshot.
/// Why:
///  after any model mutation,
///  GTK placement updates through one narrow `StripLayout` interface.
fn reconcile(inner: &Rc<StripInner>) {
    let placements = inner
        .state
        .borrow()
        .panes()
        .map(|pane| return PanePlacement {
            id: pane.id,
            column: pane.column,
            row: pane.row,
            parent: pane.parent,
        })
        .collect();
    inner
        .layout
        .reconcile(placements, |id| return build_pane_widget(inner, id));
}

/// What:
///  build the widget for pane `id` from its location.
/// Why:
///  a directory becomes a listing pane wired to spawn;
///  a preview becomes a thumbnail pane;
///  a
///      missing pane degrades to a label.
fn build_pane_widget(inner: &Rc<StripInner>, id: PaneId) -> Widget {
    let location = inner.state.borrow().pane(id).map(|pane| return pane.location.clone());
    match location {
        Some(PaneLocation::Directory(path)) => return build_directory_pane(inner, id, &path),
        Some(PaneLocation::Preview(path)) => {
            let close_weak = Rc::downgrade(inner);
            return build_preview_pane(&inner.thumbs, &path, move || {
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
    match crate::fs::read_directory(path, generation) {
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
///  reveal it,
///  and
///       return the new pane id;
///  `force_duplicate` skips dedup to mint a fresh pane.
/// Why:
///  a directory (following symlinks) spawns a listing,
///  anything else a preview;
///  dedup-and-focus
///      lives in the model,
///  so a revisit focuses the existing pane unless Ctrl forced a duplicate.
fn spawn_from(
    inner: &Rc<StripInner>,
    source: PaneId,
    entry: &FileEntry,
    force_duplicate: bool,
) -> PaneId {
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
        columns = inner.state.borrow().column_count(),
        "spawned child pane"
    );
    return spawned
}

/// What:
///  close pane `id` and reconcile so its widget leaves and the tree re-lays-out.
/// Why:
///  explicit close is the only way a pane dies;
///  layout reconciliation drops the widget.
fn close_pane(inner: &Rc<StripInner>, id: PaneId) {
    inner.state.borrow_mut().close(id);
    reconcile(inner);
    tracing::info!(closed = id.0, panes = inner.state.borrow().len(), "closed pane");
}
