/**
 Multi-page state management for the doodle widget.
 
 Each page stores its own strokes, text entries, and SVG background.
 Switching pages saves the current page state and restores the target.
 */

import {
  endStroke,
  getStrokes,
  redraw,
  setStrokes,
  type StrokeData,
} from './drawing.ts';
import {
  replaceTextEntries,
  serializeTextEntries,
  type TextEntryData,
} from './text-page.ts';
import {
  clearTextEntries,
  finalizeActiveInput,
} from './text.ts';

/**
 Saved state for a single page.
 
 @example
 ```ts
 const page: PageState = {
   strokes: [],
   textEntries: [],
   svgBackground: '<svg>...</svg>',
 };
 ```
 */
export type PageState = {
  /**
   Drawing strokes on this page
   */
  strokes: StrokeData[];
  /**
   Serialized text input entries
   */
  textEntries: TextEntryData[];
  /**
   SVG overlay innerHTML for this page's background
   */
  svgBackground: string;
};

/**
 Multi-page state container.
 
 Stored as object properties so module-root state stays in a `const`
 container (`no-module-root-let` would otherwise reject top-level `let`).
 */
const pagesState: {
  /**
   All page states indexed by page number
   */
  pages: PageState[];
  /**
   Zero-based index of the currently active page
   */
  currentIndex: number;
} = {
  pages: [],
  currentIndex: 0,
};

/**
 Initializes page states from default SVG backgrounds.
 
 Sets the first page as active and renders its background
 in the SVG overlay.
 
 @param backgrounds - processed SVG markup strings, one per page
 
 @param overlay - SVG overlay element to set initial background
 
 @example
 ```ts
 initPages({ backgrounds: ['<svg>...</svg>'], overlay: svgOverlay });
 ```
 */
export function initPages({
  backgrounds,
  overlay,
}: {
  readonly backgrounds: readonly string[];
  readonly overlay: HTMLElement;
},): void {
  pagesState.pages = backgrounds.map(function createPage(svg,): PageState {
    return {
      strokes: [],
      textEntries: [],
      svgBackground: svg,
    };
  },);
  pagesState.currentIndex = 0;
  /**
   First page state for initial background rendering
   */
  const [firstPage,] = pagesState.pages;
  if (firstPage === undefined)
    throw new Error('No page backgrounds provided',);
  overlay.innerHTML = firstPage.svgBackground;
}

/**
 Returns the zero-based index of the active page.
 
 @returns current page index
 
 @example
 ```ts
 const index = getCurrentPageIndex();
 ```
 */
export function getCurrentPageIndex(): number {
  return pagesState.currentIndex;
}

/**
 Persists the current page's live state (strokes, text, SVG) into
 the pages array.
 
 Finalizes any active text input via {@link finalizeActiveInput} and
 copies stroke data before saving.
 
 @param overlay - SVG overlay element for reading current background
 
 @param textLayer - text layer element for serializing current text entries
 */
function saveCurrentPage({
  overlay,
  textLayer,
}: {
  readonly overlay: HTMLElement;
  readonly textLayer: HTMLDivElement;
},): void {
  /**
   Live page slot, or `undefined` when state has been wiped mid-switch.
   */
  const page = pagesState.pages[pagesState.currentIndex];
  if (page === undefined)
    return;
  finalizeActiveInput();
  page.strokes = [...getStrokes(),];
  page.textEntries = serializeTextEntries(textLayer,);
  page.svgBackground = overlay.innerHTML;
}

/**
 Switches to a different page, saving current state via
 {@link saveCurrentPage} and restoring target.
 
 Finalizes any in-progress stroke via {@link endStroke} or text input
 before saving. No-op if the target index matches the current page
 or is out of range.
 
 @param index - target page index (zero-based)
 
 @param ctx - canvas rendering context for redraw
 
 @param cw - current canvas width in CSS pixels
 
 @param ch - current canvas height in CSS pixels
 
 @param overlay - SVG overlay element
 
 @param textLayer - text layer element
 
 @example
 ```ts
 switchToPage({ index: 1, ctx, cw, ch, overlay: svgOverlay, textLayer });
 ```
 */
export function switchToPage({
  index,
  ctx,
  cw,
  ch,
  overlay,
  textLayer,
}: {
  readonly index: number;
  readonly ctx: CanvasRenderingContext2D;
  readonly cw: number;
  readonly ch: number;
  readonly overlay: HTMLElement;
  readonly textLayer: HTMLDivElement;
},): void {
  if (index === pagesState
    .currentIndex)
    return;
  if ((index < 0) || (index
    >= pagesState
      .pages
      .length))
    return;
  endStroke();
  saveCurrentPage({
    overlay,
    textLayer,
  },);

  //region Restore target page
  /**
   Destination page state; the surrounding bounds check guarantees presence.
   */
  const targetPage = pagesState.pages[index];
  if (targetPage === undefined)
    throw new Error(`Page state missing for target index ${String(index,)}`,);
  setStrokes(targetPage.strokes,);
  replaceTextEntries({
    entries: targetPage.textEntries,
    layer: textLayer,
    clearFn: clearTextEntries,
  },);
  overlay.innerHTML = targetPage.svgBackground;
  pagesState.currentIndex = index;
  //endregion Restore target page

  redraw({
    ctx,
    cw,
    ch,
  },);
}

/**
 Snapshots all page states, saving the current page's live state
 first via {@link saveCurrentPage}.
 
 The current page's strokes, text entries, and SVG background are read
 from their respective live sources (drawing module, text layer DOM,
 SVG overlay DOM) and persisted into the pages array before returning.
 
 @param overlay - SVG overlay element for reading current background
 
 @param textLayer - text layer element for serializing current text entries
 
 @returns shallow copy of all page states
 
 @example
 ```ts
 const allPages = snapshotAllPages({ overlay, textLayer });
 for (const page of allPages) {
   console.log(page.strokes.length, page.textEntries.length);
 }
 ```
 */
export function snapshotAllPages({
  overlay,
  textLayer,
}: {
  readonly overlay: HTMLElement;
  readonly textLayer: HTMLDivElement;
},): readonly PageState[] {
  endStroke();
  saveCurrentPage({
    overlay,
    textLayer,
  },);
  return [...pagesState.pages,];
}
