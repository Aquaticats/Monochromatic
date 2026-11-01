/**
 Sticky rail-band geometry for the pane strip.
 
 This module is the whole replacement for the GTK original's 400-plus-line
 lane engine (`layout/lane.rs` plus its collision solver): it only computes,
 per pane, the flow rectangle (the rail band) that CSS `position: sticky`
 then pins the pane inside. Non-overlap and scroll clamping are not computed
 here at all; they fall out of normal flow and sticky containment in the
 browser's layout engine.
 
 @example
 ```ts
 const layouts = computeColumnLayouts({ panes: [] });
 ```
 
 @packageDocumentation
 */

import {
  PANE_HEIGHT,
  ROW_STRIDE,
} from './constants.js';
import type {
  Pane,
  PaneId,
} from './strip.js';

/**
 One pane's rail wrapper inside its column's normal flow: the margin above it
 and the wrapper height the pane sticks within.
 
 @example
 ```ts
 const rail: RailLayout = { id: 0 as never, marginTopPx: 0, railHeightPx: 520 };
 ```
 */
export type RailLayout = {
  readonly id: PaneId;

  /**
   Gap between the previous wrapper's bottom (or the column top) and this
   wrapper's top, in pixels; places the band at its global grid offset.
   */
  readonly marginTopPx: number;

  /**
   Wrapper height in pixels: from the pane's own row down to its deepest
   direct child's bottom edge, so the pane can stick while any of its direct
   children is on screen.
   */
  readonly railHeightPx: number;
};

/**
 One column's flow stack of rail wrappers, ordered top to bottom.
 
 @example
 ```ts
 const column: ColumnLayout = { column: 0, rails: [] };
 ```
 */
export type ColumnLayout = {
  readonly column: number;
  readonly rails: readonly RailLayout[];
};

/**
 Vertical pixel offset of a grid row; panes tile down each column at a fixed
 stride shared by every column.
 
 @param row - Zero-based grid row.
 
 @returns Content y-coordinate of the row's top edge.
 
 @example
 ```ts
 rowY({ row: 2 });
 ```
 */
export function rowY({ row, }: { readonly row: number; },): number {
  return row * ROW_STRIDE;
}

/**
 Deepest direct-child row of a pane, or its own row when it has no children;
 the rail band extends down to this row's bottom edge.
 
 @param id - Parent pane identity.
 
 @param panes - Live placement snapshot.
 
 @param row - Parent pane's own row.
 
 @returns Deepest row the pane's rail must cover.
 
 @example
 ```ts
 deepestDirectChildRow({ panes: [], id: 0 as never, row: 0 });
 ```
 */
function deepestDirectChildRow(
  {
    id,
    panes,
    row,
  }: {
    readonly id: PaneId;
    readonly panes: readonly Pane[];
    readonly row: number;
  },
): number {
  return panes
    .filter(function isDirectChild(pane,): boolean {
      return pane.parent === id;
    },)
    .reduce(
    function deepest(
      deepestRow,
      child,
    ): number {
      return Math.max(
        deepestRow,
        child.row,
      );
    },
    row,
    );
}

/**
 Rail-band height for one pane: one pane tall for a leaf, and stretching to
 the deepest direct child's bottom edge for a parent.
 
 @param pane - Pane whose band is measured.
 
 @param panes - Live placement snapshot.
 
 @returns Band height in pixels.
 
 @example
 ```ts
 railHeightPx({ panes: [], pane: { id: 0, row: 0 } as never });
 ```
 */
export function railHeightPx(
  {
    pane,
    panes,
  }: {
    readonly pane: Pane;
    readonly panes: readonly Pane[];
  },
): number {
  /**
   Deepest row this pane's rail spans down to.
   */
  const bottomRow = deepestDirectChildRow({
    id: pane.id,
    panes,
    row: pane.row,
  },);

  return (rowY({ row: bottomRow, },)
    + PANE_HEIGHT)
    - rowY({ row: pane.row, },);
}

/**
 Groups the strip into per-column flow stacks of rail wrappers: within each
 column, wrappers are ordered by row and separated by margins that place each
 band at its global grid offset. Rendering these stacks in normal flow with a
 sticky pane inside each wrapper reproduces the approved lane behavior with
 no per-scroll code.
 
 @param panes - Live placement snapshot.
 
 @returns Column layouts ordered left to right.
 
 @example
 ```ts
 computeColumnLayouts({ panes: [] });
 ```
 */
export function computeColumnLayouts(
  { panes, }: { readonly panes: readonly Pane[]; },
): readonly ColumnLayout[] {
  /**
   Highest column index in the snapshot, or -1 for an empty strip.
   */
  const lastColumn = panes.reduce(
    function widest(
      widestColumn,
      pane,
    ): number {
      return Math.max(
        widestColumn,
        pane.column,
      );
    },
    -1,
  );

  return Array.from(
    { length: lastColumn + 1, },
    function buildColumn(
      _unused,
      column,
    ): ColumnLayout {
      /**
       Panes of this column ordered top to bottom.
       */
      const ordered = panes
        .filter(function inColumn(pane,): boolean {
          return pane.column === column;
        },)
        .toSorted(function byRow(
          left,
          right,
        ): number {
          return left.row - right.row;
        },);

      /**
       Mutable flow cursor tracking the previous wrapper's bottom edge.
       */
      const cursor = { bottomPx: 0, };

      /**
       Rail wrappers of this column with flow margins filled in.
       */
      const rails = ordered.map(function toRail(pane,): RailLayout {
        /**
         Global content offset of this pane's band top.
         */
        const topPx = rowY({ row: pane.row, },);

        /**
         Band height this pane sticks within.
         */
        const heightPx = railHeightPx({
          pane,
          panes,
        },);

        /**
         Flow margin placing the band at its global offset.
         */
        const marginTopPx = topPx - cursor.bottomPx;

        cursor.bottomPx = topPx + heightPx;
        return {
          id: pane.id,
          marginTopPx,
          railHeightPx: heightPx,
        };
      },);

      return {
        column,
        rails,
      };
    },
  );
}
