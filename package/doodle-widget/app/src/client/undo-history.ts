/**
 Per-page undo/redo history stack for the doodle widget.
 
 Maintains an array of state snapshots per page with a pointer
 to the current position. Supports configurable depth limit.
 */

import type { StrokeData, } from './drawing.ts';
import type { TextEntryData, } from './text-page.ts';

/**
 Maximum undo states retained per page
 */
const MAX_HISTORY_DEPTH = 50;

/**
 Absence marker returned by {@link undo} and {@link redo} when the page has no
 state to move to (already at the start or end); never a {@link Snapshot}.
 
 @example
 ```ts
 const snapshot = undo(0);
 if (snapshot !== NO_SNAPSHOT)
   restoreSnapshot(snapshot);
 ```
 */
export const NO_SNAPSHOT: unique symbol = Symbol('doodle-widget/no-snapshot-at-undo-history-boundary',);

/**
 Complete state snapshot for a single page.
 
 @example
 ```ts
 const snap: Snapshot = { strokes: [...getStrokes()], textEntries: serializeTextEntries(layer) };
 ```
 */
export type Snapshot = {
  /**
   Deep-enough copy of strokes (stroke objects are immutable after creation)
   */
  readonly strokes: readonly StrokeData[];
  /**
   Serialized text input entries
   */
  readonly textEntries: readonly TextEntryData[];
};

/**
 Undo/redo state for a single page
 */
type PageHistory = {
  /**
   Ordered states from oldest to newest
   */
  states: Snapshot[];
  /**
   Index of the current state within `states`
   */
  index: number;
};

/**
 History stacks indexed by page number.
 
 Stored as an object property so module-root state stays in a `const`
 container (`no-module-root-let` would otherwise reject a top-level `let`).
 */
const historiesState: { all: PageHistory[]; } = { all: [], };

/**
 Initializes empty history stacks for all pages.
 
 Each page starts with a single empty snapshot at index 0.
 
 @param pageCount - number of pages to create history for
 
 @example
 ```ts
 initHistory(2);
 ```
 */
export function initHistory(pageCount: number,): void {
  /**
   Shared initial empty snapshot
   */
  const empty: Snapshot = {
    strokes: [],
    textEntries: [],
  };
  historiesState.all = Array.from(
    { length: pageCount, },
    function createPageHistory(): PageHistory {
      return {
        states: [empty,],
        index: 0,
      };
    },
  );
}

/**
 Pushes a new state snapshot, truncating any redo history.
 
 @param pageIndex - page to push state for
 
 @param snapshot - captured state after the completed action
 
 @example
 ```ts
 pushSnapshot({ pageIndex: 0, snapshot: { strokes: [], textEntries: [] } });
 ```
 */
export function pushSnapshot({
  pageIndex,
  snapshot,
}: {
  readonly pageIndex: number;
  readonly snapshot: Snapshot;
},): void {
  /**
   Per-page slot; missing only when the page was never initialized, in which case the push is silently dropped.
   */
  const history = historiesState.all[pageIndex];
  if (history === undefined)
    return;

  history.states = history.states
    .slice(
    0,
    history.index
      + 1,
    );
  history.states
    .push(snapshot,);
  history.index = history.states
    .length
    - 1;

  if (history.states
    .length
    > MAX_HISTORY_DEPTH) {
    /**
     Count of oldest entries to drop so the depth cap holds.
     */
    const excess = history.states
      .length
      - MAX_HISTORY_DEPTH;
    history.states = history.states
      .slice(excess,);
    history.index -= excess;
  }
}

/**
 Moves back one step and returns the snapshot to restore.
 
 @param pageIndex - page to undo on
 
 @returns snapshot to restore, or {@link NO_SNAPSHOT} if at the beginning
 
 @example
 ```ts
 const snapshot = undo(0);
 ```
 */
export function undo(pageIndex: number,): Snapshot | typeof NO_SNAPSHOT {
  /**
   Page slot guarded so an absent or at-start page falls through to the absent sentinel.
   */
  const history = historiesState.all[pageIndex];
  if ((history === undefined) || (history.index
    <= 0))
    return NO_SNAPSHOT;
  history.index -= 1;
  return history.states[history.index]
    ?? NO_SNAPSHOT;
}

/**
 Moves forward one step and returns the snapshot to restore.
 
 @param pageIndex - page to redo on
 
 @returns snapshot to restore, or {@link NO_SNAPSHOT} if at the end
 
 @example
 ```ts
 const snapshot = redo(0);
 ```
 */
export function redo(pageIndex: number,): Snapshot | typeof NO_SNAPSHOT {
  /**
   Page slot guarded so an absent or at-end page falls through to the absent sentinel.
   */
  const history = historiesState.all[pageIndex];
  if ((history === undefined) || (history.index
    >= (history.states
      .length
      - 1)))
    return NO_SNAPSHOT;
  history.index += 1;
  return history.states[history.index]
    ?? NO_SNAPSHOT;
}

/**
 Checks whether undo is available for the given page.
 
 @param pageIndex - page to check
 
 @returns `true` if there is at least one prior state
 
 @example
 ```ts
 if (canUndo(0)) undo(0);
 ```
 */
export function canUndo(pageIndex: number,): boolean {
  /**
   Page slot looked up so the predicate covers both presence and position.
   */
  const history = historiesState.all[pageIndex];
  return (history !== undefined) && (history.index
    > 0);
}

/**
 Checks whether redo is available for the given page.
 
 @param pageIndex - page to check
 
 @returns `true` if there is at least one forward state
 
 @example
 ```ts
 if (canRedo(0)) redo(0);
 ```
 */
export function canRedo(pageIndex: number,): boolean {
  /**
   Page slot looked up so the predicate covers both presence and position.
   */
  const history = historiesState.all[pageIndex];
  return (history !== undefined) && (history.index
    < (history.states
      .length
      - 1));
}
