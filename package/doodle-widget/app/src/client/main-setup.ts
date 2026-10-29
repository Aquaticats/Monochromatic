/**
 Initialization and handler wiring for the doodle widget.
 
 Parses page backgrounds, sets up pointer/toolbar/undo handlers,
 and attaches page switching and resize observers.
 */

import {
  initPages,
  switchToPage,
} from './pages.ts';
import type { ToolMode, } from './pointer-handler-deps.ts';
import { setupZoomPointerHandlers, } from './pointer-handlers-zoom.ts';
import { setupPointerHandlers, } from './pointer-handlers.ts';
import { setTextLayer, } from './text.ts';
import { setupToolbarHandlers, } from './toolbar-handlers.ts';
import { setupUndoHandlers, } from './undo-handlers.ts';
import { initHistory, } from './undo-history.ts';
import { resetZoom, } from './zoom.ts';

/**
 Dependencies for {@link setupWidget}.
 */
export type WidgetDeps = {
  readonly backgroundsScript: HTMLScriptElement;
  readonly canvas: HTMLCanvasElement;
  readonly ctx: CanvasRenderingContext2D;
  readonly getToolMode: () => ToolMode;
  readonly getCanvasSize: () => {
    cw: number;
    ch: number;
  };
  readonly textLayer: HTMLDivElement;
  readonly container: HTMLDivElement;
  /**
   Fixed letter-size page element (coordinate reference for drawing and export)
   */
  readonly page: HTMLDivElement;
  readonly zoomLayer: HTMLDivElement;
  readonly svgOverlay: HTMLDivElement;
  readonly pageToggle: HTMLDivElement;
  readonly colorPicker: HTMLInputElement;
  readonly sizeSlider: HTMLInputElement;
  readonly clearBtn: HTMLButtonElement;
  readonly exportBtn: HTMLButtonElement;
  readonly formatSelect: HTMLSelectElement;
  readonly uploadBtn: HTMLButtonElement;
  readonly uploadInput: HTMLInputElement;
  readonly undoBtn: HTMLButtonElement;
  readonly redoBtn: HTMLButtonElement;
  readonly sizeCanvas: () => void;
};

/**
 Initializes page state, handler wiring, and observers.
 
 @param deps - all DOM elements and state accessors, see {@link WidgetDeps}
 
 @mutates deps - `clearBtn.addEventListener`, `colorPicker.addEventListener`, `exportBtn.addEventListener`, `pageToggle.addEventListener`, `redoBtn.addEventListener`, `sizeSlider.addEventListener`, `textLayer.addEventListener`, `undoBtn.addEventListener`, `uploadBtn.addEventListener`, and `uploadInput.addEventListener` change event targets and retain handlers; `new ResizeObserver(sizeCanvas,).observe` retains `sizeCanvas` and observes its target; `uploadInput.click` can dispatch retained listeners.
 
 @example
 ```ts
 setupWidget(deps);
 ```
 */
export function setupWidget(deps: WidgetDeps,): void {
  /**
   Destructured up front so the long list of handles can be passed individually to each setup helper.
   */
  const {
    backgroundsScript,
    canvas,
    ctx,
    getToolMode,
    getCanvasSize,
    textLayer,
    container,
    page,
    zoomLayer,
    svgOverlay,
    pageToggle,
    colorPicker,
    sizeSlider,
    clearBtn,
    exportBtn,
    formatSelect,
    uploadBtn,
    uploadInput,
    undoBtn,
    redoBtn,
    sizeCanvas,
  } = deps;

  /**
   Untyped JSON tree narrowed before assigning to {@link backgrounds}.
   */
  const parsed: unknown = JSON.parse(backgroundsScript.textContent,);
  if (!Array.isArray(parsed,))
    throw new Error('Page backgrounds data is not an array',);
  /**
   Narrowed alias so consumers see a readonly string array without re-asserting.
   */
  const backgrounds: readonly string[] = parsed;

  initPages({
    backgrounds,
    overlay: svgOverlay,
  },);
  initHistory(backgrounds.length,);
  setTextLayer(textLayer,);

  /**
   Handler hooks returned by undo setup so the toolbar can call them later.
   */
  const {
    pushSnapshot,
    updateUndoButtons,
  } = setupUndoHandlers({
    undoBtn,
    redoBtn,
    ctx,
    getCanvasSize,
    textLayer,
  },);
  textLayer.addEventListener(
    'textfinalized',
    pushSnapshot,
  );

  /**
   Shared bag so both pointer setups receive identical handles without re-typing the object.
   */
  const pointerDeps = {
    canvas,
    ctx,
    getToolMode,
    getCanvasSize,
    textLayer,
    pushSnapshot,
    page,
    zoomLayer,
  };
  setupPointerHandlers(pointerDeps,);
  setupZoomPointerHandlers(pointerDeps,);

  setupToolbarHandlers({
    colorPicker,
    sizeSlider,
    clearBtn,
    exportBtn,
    formatSelect,
    uploadBtn,
    uploadInput,
    page,
    svgOverlay,
    drawCanvas: canvas,
    textLayer,
    ctx,
    getCanvasSize,
    sizeCanvas,
    pushSnapshot,
  },);

  pageToggle.addEventListener(
    'change',
    function handlePageChange(event: Event,): void {
      /**
       Event target destructured so the narrowing `instanceof` check can short-circuit.
       */
      const { target, } = event;
      if (!(target instanceof HTMLInputElement))
        return;
      resetZoom(zoomLayer,);
      /**
       Canvas dimensions resolved after the zoom reset so the page switch sees fresh sizing.
       */
      const {
        cw,
        ch,
      } = getCanvasSize();
      switchToPage({
        index: Number(target.value,),
        ctx,
        cw,
        ch,
        overlay: svgOverlay,
        textLayer,
      },);
      updateUndoButtons();
    },
  );

  new ResizeObserver(sizeCanvas,).observe(page,);
  sizeCanvas();
}
