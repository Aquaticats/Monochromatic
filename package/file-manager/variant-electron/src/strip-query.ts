/**
 Read-only queries over the pane strip.
 
 Lookups that can miss return the exported sentinels from `strip-types.ts`
 (`PANE_NOT_FOUND`, `NO_CANONICAL_PANE`); consumers narrow with
 `typeof value === 'symbol'` first, then identity.
 
 @example
 ```ts
 const count = columnCount({ strip: createStrip() });
 ```
 
 @packageDocumentation
 */

import {
  NO_CANONICAL_PANE,
  PANE_NOT_FOUND,
  type Pane,
  type PaneId,
  type PaneLocation,
  type Strip,
} from './strip-types.js';

/**
 Checks whether two locations are the same dedup key.
 
 @param left - First location.
 
 @param right - Second location.
 
 @returns Whether kind and path both match.
 
 @example
 ```ts
 sameLocation({ left: { kind: 'directory', path: '/a' }, right: { kind: 'preview', path: '/a' } });
 ```
 */
function sameLocation(
  {
    left,
    right,
  }: {
    readonly left: PaneLocation;
    readonly right: PaneLocation;
  },
): boolean {
  return (left.kind === right.kind) && (left.path === right.path);
}

/**
 Finds the canonical dedup pane for a location, if one is registered.
 
 @param location - Location to look up.
 
 @param strip - Strip to search.
 
 @returns Canonical pane id, or {@link NO_CANONICAL_PANE} when no registered pane shows this location.
 
 @example
 ```ts
 canonicalPaneFor({ strip: createStrip(), location: directoryLocation({ path: '/home' }) });
 ```
 */
export function canonicalPaneFor(
  {
    location,
    strip,
  }: {
    readonly location: PaneLocation;
    readonly strip: Strip;
  },
): PaneId | typeof NO_CANONICAL_PANE {
  /**
   Registered pane whose location equals the lookup key, when present.
   */
  const canonical = strip.panes
    .find(function isCanonicalHolder(pane,): boolean {
      return pane.registeredForDedup && sameLocation({
        left: pane.location,
        right: location,
      },);
    },);

  if (canonical === undefined)
    return NO_CANONICAL_PANE;

  return canonical.id;
}

/**
 Looks up a pane by id.
 
 @param id - Pane identity to find.
 
 @param strip - Strip to search.
 
 @returns Matching pane, or {@link PANE_NOT_FOUND} when the id is not live.
 
 @example
 ```ts
 paneById({ strip: createStrip(), id: 0 });
 ```
 */
export function paneById(
  {
    id,
    strip,
  }: {
    readonly id: PaneId;
    readonly strip: Strip;
  },
): Pane | typeof PANE_NOT_FOUND {
  /**
   Live pane carrying the requested id, when present.
   */
  const pane = strip.panes
    .find(function matchesId(candidate,): boolean {
      return candidate.id === id;
    },);

  if (pane === undefined)
    return PANE_NOT_FOUND;

  return pane;
}

/**
 Number of columns spanned (one past the highest column index), or zero when empty.
 
 @param strip - Strip to measure.
 
 @returns Column count.
 
 @example
 ```ts
 columnCount({ strip: createStrip() });
 ```
 */
export function columnCount({ strip, }: { readonly strip: Strip; },): number {
  return strip.panes
    .reduce(
    function widest(
      count,
      pane,
    ): number {
      return Math.max(
        count,
        pane.column + 1,
      );
    },
    0,
    );
}

/**
 The top-most (lowest-row) pane in a column; keyboard Left/Right navigation
 lands on it.
 
 @param column - Column index to search.
 
 @param strip - Strip to search.
 
 @returns Top pane of the column, or {@link PANE_NOT_FOUND} for an empty column.
 
 @example
 ```ts
 firstPaneInColumn({ strip: createStrip(), column: 0 });
 ```
 */
export function firstPaneInColumn(
  {
    column,
    strip,
  }: {
    readonly column: number;
    readonly strip: Strip;
  },
): Pane | typeof PANE_NOT_FOUND {
  /**
   Column members ordered top to bottom.
   */
  const [top,] = strip.panes
    .filter(function inColumn(pane,): boolean {
      return pane.column === column;
    },)
    .toSorted(function byRow(
      left,
      right,
    ): number {
      return left.row - right.row;
    },);

  if (top === undefined)
    return PANE_NOT_FOUND;

  return top;
}
