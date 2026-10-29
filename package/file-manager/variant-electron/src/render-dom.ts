/**
 DOM reconciliation for the sticky-flow pane strip.
 
 The strip is rendered as normal-flow HTML: one flex row of columns, each
 column a block stack of rail wrappers, each wrapper holding one
 `position: sticky` pane. All pane movement during scrolling is done by the
 browser's sticky positioning; this module only diffs DOM structure against
 the model and writes the flow margins and heights computed by `bands.ts`.
 
 @example
 ```ts
 renderStrip({ handlers, stores, strip, stripElement });
 ```
 
 @packageDocumentation
 */

import { computeColumnLayouts, } from './bands.js';
import type { BridgeFileEntry, } from './bridge-types.js';
import {
  paneById,
  type Pane,
  type PaneId,
  type Strip,
} from './strip.js';

/**
 Mutable renderer-side stores keyed by pane identity.
 
 @example
 ```ts
 const stores = createRendererStores();
 ```
 */
export type RendererStores = {
  /**
   Column container elements in left-to-right order.
   */
  readonly columnElements: HTMLElement[];

  /**
   Fetched directory listings by pane id.
   */
  readonly listings: Map<PaneId, readonly BridgeFileEntry[]>;

  /**
   Pane section elements by pane id.
   */
  readonly paneElements: Map<PaneId, HTMLElement>;

  /**
   Rail wrapper elements by pane id.
   */
  readonly railElements: Map<PaneId, HTMLElement>;

  /**
   Selected entry index per directory pane.
   */
  readonly selections: Map<PaneId, number>;
};

/**
 Renderer callbacks invoked from pane DOM events.
 
 @example
 ```ts
 const handlers: PaneEventHandlers = {
   onEntryOpen: () => {},
   onPaneClose: () => {},
   onPaneFocus: () => {},
 };
 ```
 */
export type PaneEventHandlers = {
  /**
   Opens an entry of a pane (descend into a directory or preview a file).
   
   @param entry - Clicked listing entry.
   
   @param forceDuplicate - Whether a modifier requested a duplicate pane.
   
   @param paneId - Pane the entry belongs to.
   */
  readonly onEntryOpen: (options: {
    readonly entry: BridgeFileEntry;
    readonly forceDuplicate: boolean;
    readonly paneId: PaneId;
  },) => void;

  /**
   Closes one pane.
   
   @param paneId - Pane to close.
   */
  readonly onPaneClose: (options: { readonly paneId: PaneId; },) => void;

  /**
   Focuses one pane.
   
   @param paneId - Pane to focus.
   */
  readonly onPaneFocus: (options: { readonly paneId: PaneId; },) => void;
};

/**
 Builds empty renderer stores for one strip element.
 
 @returns Fresh mutable stores.
 
 @example
 ```ts
 const stores = createRendererStores();
 ```
 */
export function createRendererStores(): RendererStores {
  return {
    columnElements: [],
    listings: new Map(),
    paneElements: new Map(),
    railElements: new Map(),
    selections: new Map(),
  };
}

/**
 Final path segment of a location path, for the pane header.
 
 @param path - Absolute location path.
 
 @returns Basename, or the path itself for a filesystem root.
 
 @example
 ```ts
 basenameOf({ path: '/home/docs' });
 ```
 */
function basenameOf({ path, }: { readonly path: string; },): string {
  /**
   Path segments with empty tails (trailing separators) dropped.
   */
  const segments = path
    .split('/',)
    .filter(function nonEmpty(segment,): boolean {
      return segment.length > 0;
    },);

  return segments.at(-1,) ?? path;
}

/**
 Builds one pane section element with header, close control, and body.
 
 @param handlers - Renderer callbacks for pane events.
 
 @param pane - Model pane the element renders.
 
 @returns Pane section element.
 
 @example
 ```ts
 buildPaneElement({ pane, handlers });
 ```
 */
function buildPaneElement(
  {
    handlers,
    pane,
  }: {
    readonly handlers: PaneEventHandlers;
    readonly pane: Pane;
  },
): HTMLElement {
  /**
   Sticky pane container.
   */
  const section = document.createElement('section',);
  section.className = (pane.location
    .kind
    === 'directory')
    ? 'pane'
    : 'pane pane-preview';
  section.dataset
    .paneId = String(pane.id,);
  section.addEventListener(
    'mousedown',
    function focusFromPointer(): void {
      handlers.onPaneFocus({ paneId: pane.id, },);
    },
  );

  /**
   Header row with the location basename and a close control.
   */
  const header = document.createElement('header',);
  header.className = 'pane-header';

  /**
   Location basename label.
   */
  const title = document.createElement('span',);
  title.className = 'pane-title';
  title.textContent = basenameOf({ path: pane.location
    .path, },);
  header.append(title,);

  /**
   Close control removing this pane.
   */
  const close = document.createElement('button',);
  close.className = 'pane-close';
  close.type = 'button';
  close.textContent = '×';
  close.setAttribute(
    'aria-label',
    `Close ${title.textContent}`,
  );
  close.addEventListener(
    'click',
    function closeFromPointer(event,): void {
      event.stopPropagation();
      handlers.onPaneClose({ paneId: pane.id, },);
    },
  );
  header.append(close,);
  section.append(header,);

  if (pane.location
    .kind
    === 'directory') {
    /**
     Listing body filled once the directory listing arrives.
     */
    const list = document.createElement('ul',);
    list.className = 'pane-list';
    section.append(list,);
  }
  else {
    /**
     Preview body naming the previewed file.
     */
    const preview = document.createElement('p',);
    preview.className = 'pane-preview-body';
    preview.textContent = pane.location
      .path;
    section.append(preview,);
  }

  return section;
}

/**
 Fills a directory pane's list body from its fetched listing, once.
 
 @param handlers - Renderer callbacks for entry events.
 
 @param pane - Model pane owning the list.
 
 @param paneElement - Pane section element to fill.
 
 @param stores - Renderer stores holding the listing.
 
 @example
 ```ts
 fillPaneListing({ handlers, pane, paneElement, stores });
 ```
 */
function fillPaneListing(
  {
    handlers,
    pane,
    paneElement,
    stores,
  }: {
    readonly handlers: PaneEventHandlers;
    readonly pane: Pane;
    readonly paneElement: HTMLElement;
    readonly stores: RendererStores;
  },
): void {
  /**
   List body element of the pane, absent for preview panes.
   */
  const list = paneElement.querySelector<HTMLUListElement>('.pane-list',);

  if ((!(list instanceof HTMLUListElement)) || (list.dataset
    .filled
    === '1'))
    return;

  /**
   Fetched listing entries for this pane, absent while loading.
   */
  const entries = stores.listings
    .get(pane.id,);

  if (entries === undefined)
    return;

  list.dataset
    .filled = '1';
  entries.forEach(function appendEntry(
    entry,
    index,
  ): void {
    /**
     One listing row.
     */
    const item = document.createElement('li',);
    item.className = (entry.kind === 'directory')
      ? 'entry entry-dir'
      : 'entry';
    item.dataset
      .index = String(index,);
    item.textContent = entry.name;
    item.addEventListener(
      'click',
      function openFromPointer(event,): void {
        handlers.onEntryOpen({
          entry,
          forceDuplicate: event.ctrlKey,
          paneId: pane.id,
        },);
      },
    );
    list.append(item,);
  },);
}

/**
 Updates the selected-entry highlight of one directory pane.
 
 @param pane - Model pane owning the list.
 
 @param paneElement - Pane section element to update.
 
 @param stores - Renderer stores holding the selection.
 
 @example
 ```ts
 updateSelectionHighlight({ pane, paneElement, stores });
 ```
 */
function updateSelectionHighlight(
  {
    pane,
    paneElement,
    stores,
  }: {
    readonly pane: Pane;
    readonly paneElement: HTMLElement;
    readonly stores: RendererStores;
  },
): void {
  /**
   Selected entry index of this pane, defaulting to the first entry.
   */
  const selected = stores.selections
    .get(pane.id,)
    ?? 0;

  paneElement.querySelectorAll<HTMLElement>('.entry',)
    .forEach(function highlightEntry(
      entry,
      index,
    ): void {
      entry.classList
        .toggle(
        'selected',
        index === selected,
        );
    },);
}

/**
 Reconciles the strip DOM to the model: grows/shrinks columns, creates
 missing rails and panes, writes flow margins and rail heights, removes stale
 elements, fills arrived listings, and refreshes highlights.
 
 @param handlers - Renderer callbacks for pane events.
 
 @param stores - Mutable renderer stores.
 
 @param strip - Model snapshot to render.
 
 @param stripElement - Scroller element hosting the columns.
 
 @example
 ```ts
 renderStrip({ handlers, stores, strip, stripElement });
 ```
 */
export function renderStrip(
  {
    handlers,
    stores,
    strip,
    stripElement,
  }: {
    readonly handlers: PaneEventHandlers;
    readonly stores: RendererStores;
    readonly strip: Strip;
    readonly stripElement: HTMLElement;
  },
): void {
  /**
   Per-column flow stacks computed from the placement snapshot.
   */
  const layouts = computeColumnLayouts({ panes: strip.panes, },);

  while (stores.columnElements
    .length
    < layouts.length) {
    /**
     New column container appended at the right edge.
     */
    const column = document.createElement('div',);
    column.className = 'column';
    stripElement.append(column,);
    stores.columnElements
      .push(column,);
  }

  while (stores.columnElements
    .length
    > layouts.length)
    stores.columnElements
      .pop()
      ?.remove();

  /**
   Live pane ids, for stale-element removal.
   */
  const live = new Set(strip.panes
    .map(function idOf(pane,): PaneId {
      return pane.id;
    },),);

  [...stores.railElements
    .keys(),]
    .filter(function isStale(id,): boolean {
      return !live.has(id,);
    },)
    .forEach(function removeStale(id,): void {
      stores.railElements
        .get(id,)
        ?.remove();
      stores.railElements
        .delete(id,);
      stores.paneElements
        .delete(id,);
      stores.listings
        .delete(id,);
      stores.selections
        .delete(id,);
    },);

  layouts.forEach(function renderColumn(layout,): void {
    /**
     Column container this layout renders into.
     */
    const column = stores.columnElements[layout.column];

    if (column === undefined)
      return;

    layout.rails
      .forEach(function renderRail(rail,): void {
        /**
         Model pane behind this rail.
         */
        const pane = paneById({
          id: rail.id,
          strip,
        },);

        if ((typeof pane) === 'symbol')
          return;

        /**
         Existing rail wrapper, or a fresh one holding a new pane element.
         */
        const railElement = stores.railElements
          .get(rail.id,)
          ?? (function createRail(): HTMLElement {
          /**
           New rail wrapper the pane sticks within.
           */
            const wrapper = document.createElement('div',);
            wrapper.className = 'rail';

          /**
           New sticky pane element.
           */
            const paneElement = buildPaneElement({
              handlers,
              pane,
            },);
            wrapper.append(paneElement,);
            stores.railElements
              .set(
              rail.id,
              wrapper,
              );
            stores.paneElements
              .set(
              rail.id,
              paneElement,
              );
            return wrapper;
          })();

        railElement.style
          .marginTop = `${rail.marginTopPx}px`;
        railElement.style
          .height = `${rail.railHeightPx}px`;
        column.append(railElement,);

        /**
         Pane element inside the rail wrapper.
         */
        const paneElement = stores.paneElements
          .get(rail.id,);

        if (paneElement === undefined)
          return;

        paneElement.classList
          .toggle(
          'active',
          strip.active === rail.id,
          );
        fillPaneListing({
          handlers,
          pane,
          paneElement,
          stores,
        },);
        updateSelectionHighlight({
          pane,
          paneElement,
          stores,
        },);
      },);
  },);
}
