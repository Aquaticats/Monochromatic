/**
 Strip-wide keyboard model: Up/Down move the in-pane selection, Enter opens
 it, Left/Right move column focus, Backspace closes the focused pane,
 matching the GTK original's column navigation.
 
 @example
 ```ts
 installKeyboard({ session });
 ```
 
 @packageDocumentation
 */

import {
  openEntryFromEvent,
  renderAndReport,
  type RendererSession,
} from './session.js';
import {
  closePane,
  columnCount,
  firstPaneInColumn,
  focusPane,
  paneById,
} from './strip.js';

/**
 Moves the in-pane selection of the focused pane and reveals it.
 
 @param delta - Selection movement: -1 up, +1 down.
 
 @param session - Renderer session to mutate.
 
 @example
 ```ts
 moveSelection({ session, delta: 1 });
 ```
 */
function moveSelection(
  {
    delta,
    session,
  }: {
    readonly delta: number;
    readonly session: RendererSession;
  },
): void {
  /**
   Focused pane id captured before lookups.
   */
  const {active} = session.strip;

  if (active === undefined)
    return;

  /**
   Listing of the focused pane, absent for previews.
   */
  const listing = session.stores
    .listings
    .get(active,);

  if ((listing === undefined) || (listing.length === 0))
    return;

  /**
   Current selection index of the focused pane.
   */
  const current = session.stores
    .selections
    .get(active,)
    ?? 0;

  /**
   New selection clamped to the listing bounds.
   */
  const next = Math.min(
    listing.length - 1,
    Math.max(
      0,
      current + delta,
    ),
  );

  session.stores
    .selections
    .set(
    active,
    next,
    );
  renderAndReport({ session, },);
  session.stores
    .paneElements
    .get(active,)
    ?.querySelectorAll<HTMLElement>('.entry',)[next]
    ?.scrollIntoView({ block: 'nearest', },);
}

/**
 Column of the focused pane, defaulting to the leftmost when nothing is
 focused or the focus is stale.
 
 @param session - Renderer session to observe.
 
 @returns Focused column index.
 
 @example
 ```ts
 currentFocusColumn({ session });
 ```
 */
function currentFocusColumn({ session, }: { readonly session: RendererSession; },): number {
  /**
   Focused pane id captured before lookups.
   */
  const {active} = session.strip;

  if (active === undefined)
    return 0;

  /**
   Focused pane, or the not-found sentinel for a stale focus.
   */
  const pane = paneById({
    id: active,
    strip: session.strip,
  },);

  if ((typeof pane) === 'symbol')
    return 0;

  return pane.column;
}

/**
 Moves focus to the top pane of an adjacent column, matching the GTK
 original's Left/Right column navigation.
 
 @param delta - Column movement: -1 left, +1 right.
 
 @param session - Renderer session to mutate.
 
 @example
 ```ts
 moveColumnFocus({ session, delta: 1 });
 ```
 */
function moveColumnFocus(
  {
    delta,
    session,
  }: {
    readonly delta: number;
    readonly session: RendererSession;
  },
): void {
  /**
   Column of the focused pane, defaulting to the leftmost.
   */
  const currentColumn = currentFocusColumn({ session, },);

  /**
   Target column clamped to the existing columns.
   */
  const targetColumn = Math.min(
    columnCount({ strip: session.strip, },) - 1,
    Math.max(
      0,
      currentColumn + delta,
    ),
  );

  /**
   Top pane of the target column, or the not-found sentinel.
   */
  const target = firstPaneInColumn({
    column: targetColumn,
    strip: session.strip,
  },);

  if ((typeof target) === 'symbol')
    return;

  session.strip = focusPane({
    id: target.id,
    strip: session.strip,
  },);
  renderAndReport({ session, },);
}

/**
 Opens the focused pane's selected entry.
 
 @param session - Renderer session to mutate.
 
 @example
 ```ts
 openSelectedEntry({ session });
 ```
 */
function openSelectedEntry({ session, }: { readonly session: RendererSession; },): void {
  /**
   Focused pane id captured before lookups.
   */
  const {active} = session.strip;

  if (active === undefined)
    return;

  /**
   Listing of the focused pane, absent for previews.
   */
  const listing = session.stores
    .listings
    .get(active,);

  /**
   Selected entry of the focused pane.
   */
  const entry = listing?.[session.stores
    .selections
    .get(active,)
    ?? 0];

  if (entry === undefined)
    return;

  openEntryFromEvent({
    entry,
    forceDuplicate: false,
    paneId: active,
    session,
  },);
}

/**
 Closes the focused pane.
 
 @param session - Renderer session to mutate.
 
 @example
 ```ts
 closeActivePane({ session });
 ```
 */
function closeActivePane({ session, }: { readonly session: RendererSession; },): void {
  /**
   Focused pane id captured before the close.
   */
  const {active} = session.strip;

  if (active === undefined)
    return;

  session.strip = closePane({
    id: active,
    strip: session.strip,
  },);
  renderAndReport({ session, },);
}

/**
 Installs the strip-wide keyboard model on the global scope.
 
 @param session - Renderer session the keys mutate.
 
 @example
 ```ts
 installKeyboard({ session });
 ```
 */
export function installKeyboard({ session, }: { readonly session: RendererSession; },): void {
  /**
   Key-name to action lookup for the strip.
   */
  const actions: Readonly<Record<string, () => void>> = {
    ArrowDown: function selectionDown(): void {
      moveSelection({
        delta: 1,
        session,
      },);
    },
    ArrowLeft: function columnLeft(): void {
      moveColumnFocus({
        delta: -1,
        session,
      },);
    },
    ArrowRight: function columnRight(): void {
      moveColumnFocus({
        delta: 1,
        session,
      },);
    },
    ArrowUp: function selectionUp(): void {
      moveSelection({
        delta: -1,
        session,
      },);
    },
    Backspace: function closeFocused(): void {
      closeActivePane({ session, },);
    },
    Enter: function openSelected(): void {
      openSelectedEntry({ session, },);
    },
  };

  globalThis.addEventListener(
    'keydown',
    function handleStripKey(event: Event,): void {
      if (!(event instanceof KeyboardEvent))
        return;

      /**
       Action bound to the pressed key, when one exists.
       */
      const action = actions[event.key];

      if (action === undefined)
        return;

      event.preventDefault();
      action();
    },
  );
}
