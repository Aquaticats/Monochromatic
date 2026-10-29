/**
 Entry point for the doodle widget client-side application.
 
 Imports DOM references, configures tool mode switching,
 and delegates handler setup to {@link import('./main-setup.ts')}.
 */

import {
  backgroundsScript,
  canvas,
  clearBtn,
  colorPicker,
  container,
  drawRadio,
  eraseRadio,
  exportBtn,
  formatSelect,
  page,
  pageToggle,
  redoBtn,
  sizeSlider,
  svgOverlay,
  textLayer,
  toolToggle,
  undoBtn,
  uploadBtn,
  uploadInput,
  zoomLayer,
  zoomRadio,
  zoomToast,
} from './dom-refs.ts';
import { redraw, } from './drawing.ts';
import { setupWidget, } from './main-setup.ts';
import type { ToolMode, } from './pointer-handlers.ts';
import { requireCanvasContext, } from './require-context.ts';
import { discardActiveInput, } from './text.ts';
import { showZoomToast, } from './zoom-toast.ts';
import { refreshZoomTransform, } from './zoom.ts';

/**
 2D rendering context for the drawing canvas
 */
const ctx = requireCanvasContext(canvas,);

/**
 Current canvas size in CSS pixels.
 
 Stored as object properties so module-root state stays in a `const`
 container (`no-module-root-let` would otherwise reject top-level `let`).
 */
const canvasSize: {
  width: number;
  height: number;
} = {
  width: 0,
  height: 0,
};

/**
 Reads the currently selected tool from the radio group.
 
 @returns active tool mode based on which radio is checked
 */
function getToolMode(): ToolMode {
  if (drawRadio.checked)
    return 'draw';
  if (eraseRadio.checked)
    return 'erase';
  if (zoomRadio.checked)
    return 'zoom';
  return 'text';
}

/**
 Returns current canvas dimensions in CSS pixels.
 
 @returns width and height as `cw` and `ch`
 */
function getCanvasSize(): {
  cw: number;
  ch: number;
} {
  return {
    cw: canvasSize.width,
    ch: canvasSize.height,
  };
}

/**
 Resizes the canvas to match the page element and redraws all strokes.
 
 The page element maintains a fixed letter aspect ratio via CSS,
 scaling to fit the viewport. The canvas always matches the page's
 rendered dimensions so strokes align with the visual frame.
 */
function sizeCanvas(): void {
  canvasSize.width = page.clientWidth;
  canvasSize.height = page.clientHeight;
  canvas.width = canvasSize.width;
  canvas.height = canvasSize.height;
  redraw({
    ctx,
    cw: canvasSize.width,
    ch: canvasSize.height,
  },);
  refreshZoomTransform({
    containerWidth: canvasSize.width,
    containerHeight: canvasSize.height,
    zoomLayer,
  },);
}

/**
 Updates canvas cursor based on the selected tool radio.
 */
function syncCursorToTool(): void {
  /**
   Cached so each branch can compare without re-invoking the getter.
   */
  const mode = getToolMode();
  if ((mode === 'draw') || (mode === 'erase')) {
    canvas.style
      .cursor = 'crosshair';
    discardActiveInput();
  }
  else if (mode === 'zoom') {
    canvas.style
      .cursor = 'zoom-in';
    discardActiveInput();
    showZoomToast(zoomToast,);
  }
  else {
    canvas.style
      .cursor = 'text';
  }
}

toolToggle.addEventListener(
  'change',
  syncCursorToTool,
);
syncCursorToTool();

setupWidget({
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
},);
