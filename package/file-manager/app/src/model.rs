//! The pane-strip state machine:
//!  a tree of panes laid out on a `(column, row)` grid.
//!
//! Pure and GTK-free so the Niri spawn/dedup/close rules (doc/planning/file-manager.md) unit-test
//! directly.
//!  Each pane knows its parent;
//!  `column` is lineage depth and `row` is assigned by a tidy
//! tree layout so a child aligns to its parent's row and a sibling starts below the previous
//! sibling's whole subtree.
//!  Any spawn or close re-lays-out,
//!  so existing panes shift down as
//! subtrees grow.
//!  The UI layer renders this state and calls its mutators.

/// What:
///  imports the hash-map container.
/// Why:
///  panes and the dedup index are keyed maps.
use std::collections::HashMap;

/// What:
///  imports the pane identity and location domain types.
/// Why:
///  the state machine keys panes by `PaneId` and deduplicates by `PaneLocation`.
use crate::types::{PaneId, PaneLocation};

/// What:
///  one pane in the strip:
///  its id,
///  what it shows,
///  its `(column, row)` position,
///  and its parent.
/// Why:
///  `column` is lineage depth (set at spawn);
///  `row` is recomputed by the tree layout;
///  `parent`
///      ties the tree together and orders siblings by spawn (id) order.
#[derive(Clone, Debug)]
pub struct Pane {
    /// Stable identity,
    ///  unique for this pane's lifetime;
    ///  also the sibling sort key (spawn order).
    pub id: PaneId,
    /// What the pane shows;
    ///  also the dedup key unless the pane is a forced duplicate.
    pub location: PaneLocation,
    /// Zero-based column index (lineage depth);
    ///  a child sits one column right of its parent.
    pub column: usize,
    /// Zero-based row index (vertical slot),
    ///  assigned by the tree layout.
    pub row: usize,
    /// Parent pane,
    ///  or `None` for a root (or an orphan whose parent was closed).
    pub parent: Option<PaneId>,
}

/// What:
///  the whole strip:
///  every live pane keyed by id,
///  the active pane,
///  and the dedup index.
/// Why:
///  panes die only on explicit close,
///  with no automatic pruning;
///  `Default` gives the empty
///      strip a fresh session starts from.
#[derive(Default)]
pub struct PaneStripState {
    /// Next id to mint;
    ///  increments so ids are never reused and encode spawn order.
    next_id: u64,
    /// Every live pane keyed by id.
    panes: HashMap<PaneId, Pane>,
    /// The focused pane,
    ///  if any;
    ///  cleared when that pane is closed.
    active: Option<PaneId>,
    /// Location -> canonical pane,
    ///  the lookup that makes revisits dedup-and-focus.
    dedup: HashMap<PaneLocation, PaneId>,
}

/// What:
///  constructors,
///  spawn/dedup/focus mutators,
///  and read accessors for the pane strip.
/// Why:
///  the single place the Niri interaction rules live,
///  tested independently of GTK.
impl PaneStripState {
    /// What:
    ///  build an empty strip.
    /// Why:
    ///  a fresh session (or a not-yet-restored one) starts with no panes.
    pub fn new() -> Self {
        return Self::default()
    }

    /// What:
    ///  open a root pane for `location` in column 0,
    ///  deduplicating first.
    /// Why:
    ///  the initial directory (and any OS-detached open handled elsewhere) has no parent;
    ///  if a
    ///      pane already shows this location,
    ///  focus it instead of spawning a second.
    pub fn open_root(&mut self, location: PaneLocation) -> PaneId {
        if let Some(&existing) = self.dedup.get(&location) {
            self.active = Some(existing);
            return existing;
        }
        let id = self.insert_pane(location, 0, None, true);
        self.relayout();
        return id
    }

    /// What:
    ///  spawn a child of `parent` showing `location` one column right,
    ///  focus it,
    ///  and re-lay-out;
    ///       `force_duplicate` skips dedup to mint an unregistered duplicate.
    /// Why:
    ///  single-click descent spawns children that dedup-and-focus on revisit;
    ///  the tree layout
    ///      then aligns the child to its parent and pushes the parent's later siblings below it.
    pub fn spawn_child(
        &mut self,
        parent: PaneId,
        location: PaneLocation,
        force_duplicate: bool,
    ) -> PaneId {
        if !force_duplicate
            && let Some(&existing) = self.dedup.get(&location)
        {
            self.active = Some(existing);
            return existing;
        }
        let column = self.panes.get(&parent).map_or(0, |pane| return pane.column) + 1;
        let id = self.insert_pane(location, column, Some(parent), !force_duplicate);
        self.relayout();
        return id
    }

    /// What:
    ///  focus `id` when it names a live pane.
    /// Why:
    ///  keyboard and pointer selection move the active pane;
    ///  a stale id is ignored.
    pub fn focus(&mut self, id: PaneId) {
        if self.panes.contains_key(&id) {
            self.active = Some(id);
        }
    }

    /// What:
    ///  close one pane and re-lay-out.
    /// Why:
    ///  explicit close is the only way a pane dies;
    ///  the layout closes the gap and any children
    ///      of the closed pane become roots (no automatic pruning,
    ///  per the plan).
    pub fn close(&mut self, id: PaneId) {
        self.remove_pane(id);
        self.relayout();
    }

    /// What:
    ///  close every pane in `column`,
    ///  then re-lay-out once (the "close column" bulk gesture).
    /// Why:
    ///  spawn-on-descent accumulates panes,
    ///  so bulk-close is required early.
    pub fn close_column(&mut self, column: usize) {
        for id in self.ids_where(|pane| return pane.column == column) {
            self.remove_pane(id);
        }
        self.relayout();
    }

    /// What:
    ///  close every pane right of `column`,
    ///  then re-lay-out once ("close everything right").
    /// Why:
    ///  descending then backing up leaves a tail of panes;
    ///  one gesture clears the tail.
    pub fn close_right_of(&mut self, column: usize) {
        for id in self.ids_where(|pane| return pane.column > column) {
            self.remove_pane(id);
        }
        self.relayout();
    }

    /// What:
    ///  the currently focused pane id,
    ///  if any.
    /// Why:
    ///  the renderer highlights it and routes keyboard input to it.
    pub fn active(&self) -> Option<PaneId> {
        return self.active
    }

    /// What:
    ///  borrow a pane by id.
    /// Why:
    ///  the renderer reads a pane's location and position to place and fill it.
    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        return self.panes.get(&id)
    }

    /// What:
    ///  iterate every live pane.
    /// Why:
    ///  the fixed-canvas renderer walks panes to place each at its `(column, row)` slot.
    pub fn panes(&self) -> impl Iterator<Item = &Pane> {
        return self.panes.values()
    }

    /// What:
    ///  number of columns spanned (one past the highest column index),
    ///  or zero when empty.
    /// Why:
    ///  keyboard navigation clamps a Left/Right move to the existing columns.
    pub fn column_count(&self) -> usize {
        return self.panes
            .values()
            .map(|pane| return pane.column + 1)
            .max()
            .unwrap_or(0)
    }

    /// What:
    ///  the id of the top-most pane in `column`,
    ///  if any.
    /// Why:
    ///  Left/Right navigation moves focus to a column's first (lowest-row) pane.
    pub fn first_pane_in_column(&self, column: usize) -> Option<PaneId> {
        return self.panes
            .values()
            .filter(|pane| return pane.column == column)
            .min_by_key(|pane| return pane.row)
            .map(|pane| return pane.id)
    }

    /// What:
    ///  number of live panes.
    /// Why:
    ///  cheap invariant check for tests and instrumentation.
    pub fn len(&self) -> usize {
        return self.panes.len()
    }

    /// What:
    ///  whether the strip holds no panes.
    /// Why:
    ///  pairs with `len` (clippy) and gates first-open behavior.
    pub fn is_empty(&self) -> bool {
        return self.panes.is_empty()
    }

    /// What:
    ///  remove pane `id` from the panes map,
    ///  the dedup index (when canonical),
    ///  and the active
    ///       slot,
    ///  without re-laying-out.
    /// Why:
    ///  the shared removal step for `close`/`close_column`/`close_right_of`,
    ///  which relayout once.
    fn remove_pane(&mut self, id: PaneId) {
        let Some(pane) = self.panes.remove(&id) else {
            return;
        };
        if self.dedup.get(&pane.location) == Some(&id) {
            self.dedup.remove(&pane.location);
        }
        if self.active == Some(id) {
            self.active = None;
        }
    }

    /// What:
    ///  the ids of every pane matching `predicate`,
    ///  snapshotted into a vector.
    /// Why:
    ///  callers mutate the map while closing,
    ///  so the ids are collected first.
    fn ids_where(&self, predicate: impl Fn(&Pane) -> bool) -> Vec<PaneId> {
        return self.panes
            .values()
            .filter(|pane| return predicate(pane))
            .map(|pane| return pane.id)
            .collect()
    }

    /// What:
    ///  sibling ids under `parent` (or the roots when `parent` is `None`),
    ///  in spawn order.
    /// Why:
    ///  the layout visits siblings top-to-bottom by spawn order;
    ///  a pane whose parent was closed
    ///      counts as a root.
    fn ordered_children(&self, parent: Option<PaneId>) -> Vec<PaneId> {
        let mut children: Vec<&Pane> = self
            .panes
            .values()
            .filter(|pane| return self.effective_parent(pane) == parent)
            .collect();
        children.sort_by_key(|pane| return pane.id);
        return children.iter().map(|pane| return pane.id).collect()
    }

    /// What:
    ///  a pane's parent if it is still live,
    ///  else `None` (making the pane a root).
    /// Why:
    ///  closing a parent orphans its children;
    ///  they lay out as roots rather than vanish.
    fn effective_parent(&self, pane: &Pane) -> Option<PaneId> {
        return pane.parent.filter(|id| return self.panes.contains_key(id))
    }

    /// What:
    ///  recompute every pane's `row` with a tidy tree layout (iterative pre-order walk).
    /// Why:
    ///  a node's row is the next free leaf-row at the moment the walk enters it,
    ///  so it aligns
    ///      with its leftmost leaf (its first child),
    ///  leaves consume rows in order,
    ///  and a node's
    ///      whole subtree occupies a contiguous block below the previous sibling.
    ///  Iterative with a
    ///      work-stack so a deep lineage never recurses over a spine.
    fn relayout(&mut self) {
        let mut next_row = 0usize;
        let mut stack: Vec<PaneId> = self.ordered_children(None);
        stack.reverse();
        while let Some(id) = stack.pop() {
            let row = next_row;
            let children = self.ordered_children(Some(id));
            if children.is_empty() {
                next_row += 1;
            }
            if let Some(pane) = self.panes.get_mut(&id) {
                pane.row = row;
            }
            stack.extend(children.into_iter().rev());
        }
    }

    /// What:
    ///  mint a fresh,
    ///  never-reused pane id.
    /// Why:
    ///  identity must survive duplicates and closes and encode spawn order,
    ///  so ids only increase.
    fn mint_id(&mut self) -> PaneId {
        let id = PaneId(self.next_id);
        self.next_id += 1;
        return id
    }

    /// What:
    ///  create a pane in `column` under `parent`,
    ///  optionally registering dedup,
    ///  and focus it.
    /// Why:
    ///  the shared tail of `open_root`/`spawn_child`;
    ///  the row is filled by the caller's relayout.
    fn insert_pane(
        &mut self,
        location: PaneLocation,
        column: usize,
        parent: Option<PaneId>,
        register_dedup: bool,
    ) -> PaneId {
        let id = self.mint_id();
        if register_dedup {
            self.dedup.insert(location.clone(), id);
        }
        self.panes.insert(
            id,
            Pane {
                id,
                location,
                column,
                row: 0,
                parent,
            },
        );
        self.active = Some(id);
        return id
    }
}
