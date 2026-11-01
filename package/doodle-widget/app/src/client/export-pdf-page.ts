/**
 Per-page canvas compositing for PDF export.
 
 Renders a single page's layers (white background, strokes, SVG
 with multiply blending) to an offscreen canvas at device pixel
 resolution for sharp PDF embedding.
 */

import type { StrokeData, } from './drawing.ts';
import { renderBaseCanvas, } from './export.ts';

/**
 Renders a single page's layers to an offscreen canvas.
 
 Composites white background, strokes, and SVG overlay (via
 multiply blending) at device pixel resolution. The SVG overlay
 element is temporarily set to the page's background markup for
 CSS layout computation via `getBoundingClientRect`.
 
 @param svgBackground - SVG overlay innerHTML for this page
 
 @param strokes - stroke data for this page
 
 @param container - canvas container for SVG position reference
 
 @param overlay - SVG overlay element (innerHTML is temporarily modified)
 
 @returns composited offscreen canvas at device pixel resolution
 
 @example
 ```ts
 const canvas = await renderPageCanvas({
   svgBackground: '<svg>...</svg>',
   strokes: pageStrokes,
   container, overlay,
 });
 ```
 */
export async function renderPageCanvas({
  svgBackground,
  strokes,
  container,
  overlay,
}: {
  readonly svgBackground: string;
  readonly strokes: readonly StrokeData[];
  readonly container: HTMLDivElement;
  readonly overlay: HTMLDivElement;
},): Promise<OffscreenCanvas> {
  /**
   Set overlay to this page's SVG for layout computation
   */
  overlay.innerHTML = svgBackground;

  /**
   Device pixel ratio for high-DPI rendering
   */
  const dpr = globalThis.devicePixelRatio;

  /**
   Destructured so the surrounding helper layout returned by {@link renderBaseCanvas} can be discarded.
   */
  const { canvas, } = await renderBaseCanvas({
    container,
    overlay,
    strokes,
    imageScale: dpr,
  },);

  return canvas;
}
