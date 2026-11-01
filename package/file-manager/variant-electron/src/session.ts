/**
 The renderer session: model mutations, DOM re-render, and observed-state
 reporting for boundary tests.
 
 @example
 ```ts
 renderAndReport({ session });
 ```
 
 @packageDocumentation
 */

import type {
  BridgeFileEntry,
  FileManagerBridge,
  ObservedStripState,
} from './bridge-types.js';
import {
  renderStrip,
  type PaneEventHandlers,
  type RendererStores,
} from './render-dom.js';
import { showStatus, } from './shell-dom.js';
import {
  closePane,
  columnCount,
  directoryLocation,
  firstPaneInColumn,
  focusPane,
  paneById,
  previewLocation,
  spawnChild,
  type PaneId,
  type Strip,
} from './strip.js';

/**
 Tolerance in pixels when checking whether the root pane is pinned to the
 scroller's top edge.
 */
const PIN_EPSILON_PX = 1;

/**
 Mutable renderer session: the model, the DOM stores, and the strip element.
 
 @example
 ```ts
 // Created once by the renderer boot module.
 ```
 */
export type RendererSession = {
  readonly bridge: FileManagerBridge;
  readonly stores: RendererStores;

  /**
   Current model snapshot; reassigned by every mutation.
   */
  strip: Strip;
  readonly stripElement: HTMLElement;
};

/**
 Counts pane pairs whose rendered boxes intersect; sticky flow must keep
 this zero, and the boundary test asserts exactly that.
 
 @param session - Renderer session holding the pane elements.
 
 @returns Number of intersecting pane pairs.
 
 @example
 ```ts
 countOverlaps({ session });
 ```
 */
function countOverlaps({ session, }: { readonly session: RendererSession; },): number {
  /**
   Rendered pane boxes in document coordinates.
   */
  const rects = [...session.stores
    .paneElements
    .values(),]
    .map(function boxOf(element,): DOMRect {
      return element.getBoundingClientRect();
    },);

  return rects.reduce(
    function countAgainstLater(
      count,
      rect,
      index,
    ): number {
      return count
        + rects
          .slice(index + 1,)
          .filter(function intersects(other,): boolean {
            return (rect.left < other.right)
              && (other.left < rect.right)
              && (rect.top < other.bottom)
              && (other.top < rect.bottom);
          },)
          .length;
    },
    0,
  );
}

/**
 Focused pane's location path, or empty when nothing is focused.
 
 @param session - Renderer session to observe.
 
 @returns Focused location path or empty string.
 
 @example
 ```ts
 activePathOf({ session });
 ```
 */
function activePathOf({ session, }: { readonly session: RendererSession; },): string {
  /**
   Focused pane id captured before lookups.
   */
  const {active} = session.strip;

  if (active === undefined)
    return '';

  /**
   Focused pane, or the not-found sentinel for a stale focus.
   */
  const pane = paneById({
    id: active,
    strip: session.strip,
  },);

  if ((typeof pane) === 'symbol')
    return '';

  return pane.location
    .path;
}

/**
 Whether the first root pane is pinned to the scroller's top edge while the
 strip is scrolled down: the observable fact of sticky behavior.
 
 @param scrollTop - Current vertical scroll offset.
 
 @param session - Renderer session to observe.
 
 @returns Whether the root pane is pinned.
 
 @example
 ```ts
 isRootPinned({ session, scrollTop: 100 });
 ```
 */
function isRootPinned(
  {
    scrollTop,
    session,
  }: {
    readonly scrollTop: number;
    readonly session: RendererSession;
  },
): boolean {
  if (scrollTop <= 0)
    return false;

  /**
   First root pane, or the not-found sentinel for an empty strip.
   */
  const rootPane = firstPaneInColumn({
    column: 0,
    strip: session.strip,
  },);

  if ((typeof rootPane) === 'symbol')
    return false;

  /**
   Root pane's element, absent before its first render.
   */
  const rootElement = session.stores
    .paneElements
    .get(rootPane.id,);

  if (rootElement === undefined)
    return false;

  /**
   Scroller box in viewport coordinates.
   */
  const stripRect = session.stripElement
    .getBoundingClientRect();

  return Math.abs(rootElement.getBoundingClientRect()
    .top
    - stripRect.top,)
    <= PIN_EPSILON_PX;
}

/**
 Computes the shallow observable state snapshot for boundary tests.
 
 @param session - Renderer session to observe.
 
 @returns Shallow scalar snapshot.
 
 @example
 ```ts
 observeState({ session });
 ```
 */
function observeState({ session, }: { readonly session: RendererSession; },): ObservedStripState {
  /**
   Vertical scroll offset of the strip scroller.
   */
  const { scrollTop, } = session.stripElement;

  return {
    activePath: activePathOf({ session, },),
    columnCount: columnCount({ strip: session.strip, },),
    overlapCount: countOverlaps({ session, },),
    paneCount: session.strip
      .panes
      .length,
    ready: true,
    rootPinned: isRootPinned({
      scrollTop,
      session,
    },),
    scrolledDown: scrollTop > 0,
    scrollTopPx: Math.round(scrollTop,),
  };
}

/**
 Mirrors the current observable state across the bridge.
 
 @param session - Renderer session to observe.
 
 @example
 ```ts
 reportState({ session });
 ```
 */
export function reportState({ session, }: { readonly session: RendererSession; },): void {
  session.bridge
    .reportState(observeState({ session, },),);
}

/**
 Re-renders the strip, reveals the focused pane, and reports state.
 
 @param session - Renderer session to render.
 
 @example
 ```ts
 renderAndReport({ session });
 ```
 */
export function renderAndReport({ session, }: { readonly session: RendererSession; },): void {
  renderStrip({
    handlers: sessionHandlers({ session, },),
    stores: session.stores,
    strip: session.strip,
    stripElement: session.stripElement,
  },);

  if (session.strip
    .active
    !== undefined)
    session.stores
      .paneElements
      .get(session.strip
        .active,)
      ?.scrollIntoView({
        block: 'nearest',
        inline: 'nearest',
      },);

  reportState({ session, },);
}

/**
 Opens one listing entry: descend into a directory (fetching its listing
 first so the new pane renders full) or spawn a preview pane for a file.
 
 @param entry - Entry to open.
 
 @param forceDuplicate - Whether to skip dedup and mint a duplicate pane.
 
 @param paneId - Pane the entry was opened from.
 
 @param session - Renderer session to mutate.
 
 @example
 ```ts
 await openEntry({ session, paneId, entry, forceDuplicate: false });
 ```
 */
async function openEntry(
  {
    entry,
    forceDuplicate,
    paneId,
    session,
  }: {
    readonly entry: BridgeFileEntry;
    readonly forceDuplicate: boolean;
    readonly paneId: PaneId;
    readonly session: RendererSession;
  },
): Promise<void> {
  /**
   Location the new pane shows: a listing for directories, a preview
   otherwise (symlinks stay unresolved, matching the GTK original).
   */
  const location = (entry.kind === 'directory')
    ? directoryLocation({ path: entry.path, },)
    : previewLocation({ path: entry.path, },);

  /**
   Listing fetched before the spawn so a new directory pane renders full;
   previews need no fetch.
   */
  const listing = (entry.kind === 'directory')
    ? await session.bridge
      .listDirectory(entry.path,)
    : undefined;

  /**
   Model after the spawn, plus the (existing or new) pane id.
   */
  const spawned = spawnChild({
    forceDuplicate,
    location,
    parent: paneId,
    strip: session.strip,
  },);

  session.strip = spawned.strip;

  if ((listing !== undefined) && (!session.stores
    .listings
    .has(spawned.id,)))
    session.stores
      .listings
      .set(
      spawned.id,
      listing,
      );

  renderAndReport({ session, },);
}

/**
 Runs an async entry-open and surfaces any failure in the status line,
 because DOM event handlers do not await async listeners.
 
 @param entry - Entry to open.
 
 @param forceDuplicate - Whether to skip dedup and mint a duplicate pane.
 
 @param paneId - Pane the entry was opened from.
 
 @param session - Renderer session to mutate.
 
 @example
 ```ts
 openEntryFromEvent({ session, paneId, entry, forceDuplicate: false });
 ```
 */
export function openEntryFromEvent(
  {
    entry,
    forceDuplicate,
    paneId,
    session,
  }: {
    readonly entry: BridgeFileEntry;
    readonly forceDuplicate: boolean;
    readonly paneId: PaneId;
    readonly session: RendererSession;
  },
): void {
  void (async function openEntryTask(): Promise<void> {
    try {
      await openEntry({
        entry,
        forceDuplicate,
        paneId,
        session,
      },);
    }
    catch (error: unknown) {
      showStatus({ text: `Failed to open ${entry.path}: ${String(error,)}`, },);
    }
  })();
}

/**
 Builds the pane event handlers bound to one session.
 
 @param session - Renderer session the handlers mutate.
 
 @returns Handlers for the DOM reconciler.
 
 @example
 ```ts
 sessionHandlers({ session });
 ```
 */
export function sessionHandlers(
  { session, }: { readonly session: RendererSession; },
): PaneEventHandlers {
  return {
    onEntryOpen: function onEntryOpen(options,): void {
      openEntryFromEvent({
        ...options,
        session,
      },);
    },
    onPaneClose: function onPaneClose({ paneId, },): void {
      session.strip = closePane({
        id: paneId,
        strip: session.strip,
      },);
      renderAndReport({ session, },);
    },
    onPaneFocus: function onPaneFocus({ paneId, },): void {
      session.strip = focusPane({
        id: paneId,
        strip: session.strip,
      },);
      renderAndReport({ session, },);
    },
  };
}
