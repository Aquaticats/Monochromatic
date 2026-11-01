/**
 The pane-strip state machine: a tree of panes laid out on a `(column, row)` grid.
 
 This is a pure TypeScript port of the plain-Rust model in
 `package/file-manager/app/src/model.rs`, so the Niri-style
 spawn/dedup/close rules stay byte-comparable between the GTK original and
 this Electron prototype. Each pane knows its parent; `column` is lineage
 depth and `row` is assigned by a tidy tree layout so a child aligns to its
 parent's row and a sibling starts below the previous sibling's whole subtree.
 
 Types live in `strip-types.ts` and read-only queries in `strip-query.ts`;
 both are re-exported here so consumers import one module.
 
 @example
 ```ts
 const opened = openRoot({ strip: createStrip(), location: directoryLocation({ path: '/home' }) });
 ```
 
 @packageDocumentation
 */

import {
  canonicalPaneFor,
  paneById,
} from './strip-query.js';
import type {
  Pane,
  PaneId,
  PaneLocation,
  Strip,
} from './strip-types.js';

// Re-export the domain types, sentinels, and constructors so consumers and
// tests import one module for the whole model surface.
export type {
  Pane,
  PaneId,
  PaneLocation,
  PaneLocationKind,
  Strip,
} from './strip-types.js';
export {
  createStrip,
  directoryLocation,
  NO_CANONICAL_PANE,
  PANE_NOT_FOUND,
  previewLocation,
} from './strip-types.js';
export {
  canonicalPaneFor,
  columnCount,
  firstPaneInColumn,
  paneById,
} from './strip-query.js';

/**
 Whether a pane lays out as a root: it has no parent link, or its recorded
 parent is no longer live (closing a parent orphans its children).
 
 @param pane - Pane to classify.
 
 @param panes - Live panes to check parenthood against.
 
 @returns Whether the pane is an effective root.
 
 @example
 ```ts
 isEffectiveRoot({ pane: strip.panes[0], panes: strip.panes });
 ```
 */
function isEffectiveRoot(
  {
    pane,
    panes,
  }: {
    readonly pane: Pane;
    readonly panes: readonly Pane[];
  },
): boolean {
  if (pane.parent === undefined)
    return true;

  return !panes.some(function isParent(candidate,): boolean {
    return candidate.id === pane.parent;
  },);
}

/**
 Effective roots in spawn order.
 
 @param panes - Live panes to group.
 
 @returns Ordered effective roots.
 
 @example
 ```ts
 orderedRoots({ panes: [] });
 ```
 */
function orderedRoots({ panes, }: { readonly panes: readonly Pane[]; },): readonly Pane[] {
  return panes
    .filter(function rootLike(pane,): boolean {
      return isEffectiveRoot({
        pane,
        panes,
      },);
    },)
    .toSorted(function bySpawnOrder(
      left,
      right,
    ): number {
      return left.id - right.id;
    },);
}

/**
 Live direct children of one live parent, in spawn order.
 
 @param panes - Live panes to group.
 
 @param parent - Live parent pane id.
 
 @returns Ordered children of the parent.
 
 @example
 ```ts
 orderedChildrenOf({ panes: [], parent: 0 });
 ```
 */
function orderedChildrenOf(
  {
    panes,
    parent,
  }: {
    readonly panes: readonly Pane[];
    readonly parent: PaneId;
  },
): readonly Pane[] {
  return panes
    .filter(function underParent(pane,): boolean {
      return pane.parent === parent;
    },)
    .toSorted(function bySpawnOrder(
      left,
      right,
    ): number {
      return left.id - right.id;
    },);
}

/**
 Rebuilds one pane with its assigned row, keeping the parent link only while
 that parent is live, so orphans become roots.
 
 @param pane - Pane before the rebuild.
 
 @param panes - Live panes to check parenthood against.
 
 @param row - Row assigned by the tidy layout.
 
 @returns Rebuilt pane.
 
 @example
 ```ts
 withLiveParent({ pane: strip.panes[0], panes: strip.panes, row: 0 });
 ```
 */
function withLiveParent(
  {
    pane,
    panes,
    row,
  }: {
    readonly pane: Pane;
    readonly panes: readonly Pane[];
    readonly row: number;
  },
): Pane {
  /**
   Rebuilt pane without any parent link.
   */
  const base: Pane = {
    column: pane.column,
    id: pane.id,
    location: pane.location,
    registeredForDedup: pane.registeredForDedup,
    row,
  };

  if (isEffectiveRoot({
    pane,
    panes,
  },) || (pane.parent === undefined))
    return base;

  return {
    ...base,
    parent: pane.parent,
  };
}

/**
 Recomputes every pane's `row` with a tidy tree layout (iterative pre-order
 walk): a node's row is the next free leaf-row when the walk enters it, so it
 aligns with its first child, leaves consume rows in order, and a subtree
 occupies a contiguous row block below the previous sibling. Iterative with a
 work-stack so a deep lineage never recurses over a spine.
 
 @param panes - Live panes before row assignment.
 
 @returns Panes with recomputed rows.
 
 @example
 ```ts
 relayout({ panes: [] });
 ```
 */
function relayout({ panes, }: { readonly panes: readonly Pane[]; },): readonly Pane[] {
  /**
   Rows assigned so far, keyed by pane id.
   */
  const rows = new Map<PaneId, number>();

  /**
   Mutable cursor naming the next free leaf row.
   */
  const cursor = { nextRow: 0, };

  /**
   Explicit pre-order work stack seeded with the roots, top of stack last.
   */
  const stack: Pane[] = [...orderedRoots({ panes, },),]
    .toReversed();

  while (stack.length > 0) {
    /**
     Pane entered by this pre-order step.
     */
    const pane = stack.pop();

    if (pane === undefined)
      break;

    rows.set(
      pane.id,
      cursor.nextRow,
    );

    /**
     Direct children of the entered pane, visited before later siblings.
     */
    const children = orderedChildrenOf({
      panes,
      parent: pane.id,
    },);

    if (children.length === 0)
      cursor.nextRow += 1;

    stack.push(...[...children,].toReversed(),);
  }

  return panes.map(function withAssignedRow(pane,): Pane {
    return withLiveParent({
      pane,
      panes,
      row: rows.get(pane.id,) ?? 0,
    },);
  },);
}

/**
 Inserts a freshly minted pane, focuses it, and re-lays-out.
 
 @param column - Lineage depth of the new pane.
 
 @param location - What the pane shows.
 
 @param parent - Parent id; omitted for a root.
 
 @param registerDedup - Whether the pane becomes the canonical dedup holder.
 
 @param strip - Strip before insertion.
 
 @returns Strip after insertion plus the new pane id.
 
 @example
 ```ts
 insertPane({ strip: createStrip(), location: directoryLocation({ path: '/home' }), column: 0, registerDedup: true });
 ```
 */
function insertPane(
  {
    column,
    location,
    parent,
    registerDedup,
    strip,
  }: {
    readonly column: number;
    readonly location: PaneLocation;
    readonly parent?: PaneId;
    readonly registerDedup: boolean;
    readonly strip: Strip;
  },
): {
  readonly id: PaneId;
  readonly strip: Strip;
} {
  /**
   Fresh never-reused identity encoding spawn order.
   */
  const id: PaneId = strip.nextId;

  /**
   New pane before the tidy layout assigns its row.
   */
  const pane: Pane = {
    column,
    id,
    location,
    registeredForDedup: registerDedup,
    row: 0,
    ...((parent === undefined)
      ? {}
      : { parent, }),
  };

  return {
    id,
    strip: {
      active: id,
      nextId: strip.nextId + 1,
      panes: relayout({
        panes: [
          ...strip.panes,
          pane,
        ],
      },),
    },
  };
}

/**
 Opens a root pane for a location in column 0, deduplicating first: when a
 registered pane already shows this location, it is focused instead.
 
 @param location - What the root pane shows.
 
 @param strip - Strip before the open.
 
 @returns Strip after the open plus the (existing or new) pane id.
 
 @example
 ```ts
 openRoot({ strip: createStrip(), location: directoryLocation({ path: '/home' }) });
 ```
 */
export function openRoot(
  {
    location,
    strip,
  }: {
    readonly location: PaneLocation;
    readonly strip: Strip;
  },
): {
  readonly id: PaneId;
  readonly strip: Strip;
} {
  /**
   Canonical pane already showing this location, or the not-registered
   sentinel.
   */
  const existing = canonicalPaneFor({
    location,
    strip,
  },);

  if ((typeof existing) !== 'symbol')
    return {
      id: existing,
      strip: {
        ...strip,
        active: existing,
      },
    };

  return insertPane({
    column: 0,
    location,
    registerDedup: true,
    strip,
  },);
}

/**
 Spawns a child of a parent pane one column right, focuses it, and
 re-lays-out; `forceDuplicate` skips dedup to mint an unregistered duplicate.
 
 @param forceDuplicate - Whether to skip dedup and mint a duplicate.
 
 @param location - What the child shows.
 
 @param parent - Pane the child descends from.
 
 @param strip - Strip before the spawn.
 
 @returns Strip after the spawn plus the (existing or new) pane id.
 
 @example
 ```ts
 const opened = openRoot({ strip: createStrip(), location: directoryLocation({ path: '/home' }) });
 spawnChild({ strip: opened.strip, parent: opened.id, location: directoryLocation({ path: '/home/docs' }), forceDuplicate: false });
 ```
 */
export function spawnChild(
  {
    forceDuplicate,
    location,
    parent,
    strip,
  }: {
    readonly forceDuplicate: boolean;
    readonly location: PaneLocation;
    readonly parent: PaneId;
    readonly strip: Strip;
  },
): {
  readonly id: PaneId;
  readonly strip: Strip;
} {
  if (!forceDuplicate) {
    /**
     Canonical pane already showing this location, or the not-registered
     sentinel.
     */
    const existing = canonicalPaneFor({
      location,
      strip,
    },);

    if ((typeof existing) !== 'symbol')
      return {
        id: existing,
        strip: {
          ...strip,
          active: existing,
        },
      };
  }

  /**
   Spawning parent's pane, or the not-found sentinel for a stale id.
   */
  const parentPane = paneById({
    id: parent,
    strip,
  },);

  /**
   Column of the spawning parent, defaulting to a root when the id is stale.
   */
  const parentColumn = ((typeof parentPane) === 'symbol')
    ? -1
    : parentPane.column;

  return insertPane({
    column: parentColumn + 1,
    location,
    parent,
    registerDedup: !forceDuplicate,
    strip,
  },);
}

/**
 Focuses a live pane; a stale id is ignored.
 
 @param id - Pane to focus.
 
 @param strip - Strip before the focus change.
 
 @returns Strip after the focus change.
 
 @example
 ```ts
 const opened = openRoot({ strip: createStrip(), location: directoryLocation({ path: '/home' }) });
 focusPane({ strip: opened.strip, id: opened.id });
 ```
 */
export function focusPane(
  {
    id,
    strip,
  }: {
    readonly id: PaneId;
    readonly strip: Strip;
  },
): Strip {
  if ((typeof paneById({
    id,
    strip,
  },)) === 'symbol')
    return strip;

  return {
    ...strip,
    active: id,
  };
}

/**
 Closes one pane and re-lays-out: the layout closes the gap and any children
 of the closed pane become roots (no automatic pruning). Focus clears when
 the focused pane closes.
 
 @param id - Pane to close.
 
 @param strip - Strip before the close.
 
 @returns Strip after the close.
 
 @example
 ```ts
 const opened = openRoot({ strip: createStrip(), location: directoryLocation({ path: '/home' }) });
 closePane({ strip: opened.strip, id: opened.id });
 ```
 */
export function closePane(
  {
    id,
    strip,
  }: {
    readonly id: PaneId;
    readonly strip: Strip;
  },
): Strip {
  /**
   Live panes surviving the close.
   */
  const survivors = strip.panes
    .filter(function stillLive(pane,): boolean {
      return pane.id !== id;
    },);

  return {
    nextId: strip.nextId,
    panes: relayout({ panes: survivors, },),
    ...(((strip.active === undefined) || (strip.active === id))
      ? {}
      : { active: strip.active, }),
  };
}
