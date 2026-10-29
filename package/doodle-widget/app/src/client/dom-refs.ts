/**
 DOM element references for the doodle widget.
 
 All queries execute at module load time and throw if an element
 is missing, ensuring early failure for missing markup.
 */

/**
 Queries a required DOM element by CSS selector and validates its type.
 
 Takes the expected element constructor so the result is narrowed by a
 runtime `instanceof` check rather than an unchecked assertion. Throws
 when the selector matches nothing or matches an element of the wrong
 type.
 
 @param selector - CSS selector string
 
 @param Ctor - constructor of the expected element type, used both to
   infer the return type and to validate the match at runtime
 
 @returns matched element, guaranteed non-null and of type `T`
 
 @throws Error if no element matches the selector or the match is not a `T`
 
 @example
 ```ts
 const btn = requireElement({ selector: '#my-btn', Ctor: HTMLButtonElement });
 ```
 */
export function requireElement<T extends Element,>(
  {
    selector,
    Ctor,
  }: {
    readonly selector: string;
    readonly Ctor: new () => T;
  },
): T {
  /**
   Query result, possibly null if selector has no match
   */
  const element = document.querySelector<T>(selector,);
  if (element === null)
    throw new Error(`Missing required element: ${selector}`,);
  if (!(element instanceof Ctor))
    throw new Error(`Element ${selector} is not the expected type`,);
  return element;
}

/**
 Canvas container element (scrollable centering viewport)
 */
export const container: HTMLDivElement = requireElement({
  selector: '#canvas-container',
  Ctor: HTMLDivElement,
},);

/**
 Fixed letter-size page element (coordinate reference for drawing and export)
 */
export const page: HTMLDivElement = requireElement({
  selector: '#page',
  Ctor: HTMLDivElement,
},);

/**
 Drawing canvas element
 */
export const canvas: HTMLCanvasElement = requireElement({
  selector: '#draw-canvas',
  Ctor: HTMLCanvasElement,
},);

/**
 SVG overlay element for displaying SVG backgrounds with multiply blending
 */
export const svgOverlay: HTMLDivElement = requireElement({
  selector: '#svg-overlay',
  Ctor: HTMLDivElement,
},);

/**
 Draw tool radio input
 */
export const drawRadio: HTMLInputElement = requireElement({
  selector: '#tool-draw',
  Ctor: HTMLInputElement,
},);

/**
 Erase tool radio input
 */
export const eraseRadio: HTMLInputElement = requireElement({
  selector: '#tool-erase',
  Ctor: HTMLInputElement,
},);

/**
 Zoom tool radio input
 */
export const zoomRadio: HTMLInputElement = requireElement({
  selector: '#tool-zoom',
  Ctor: HTMLInputElement,
},);

/**
 Text label overlay layer
 */
export const textLayer: HTMLDivElement = requireElement({
  selector: '#text-layer',
  Ctor: HTMLDivElement,
},);

/**
 Zoom layer wrapper for CSS transform-based zoom and pan
 */
export const zoomLayer: HTMLDivElement = requireElement({
  selector: '#zoom-layer',
  Ctor: HTMLDivElement,
},);

/**
 Tool selection toggle group
 */
export const toolToggle: HTMLDivElement = requireElement({
  selector: '#tool-toggle',
  Ctor: HTMLDivElement,
},);

/**
 Page selection toggle group
 */
export const pageToggle: HTMLDivElement = requireElement({
  selector: '#page-toggle',
  Ctor: HTMLDivElement,
},);

/**
 JSON script element holding page background SVGs
 */
export const backgroundsScript: HTMLScriptElement = requireElement({
  selector: '#page-backgrounds',
  Ctor: HTMLScriptElement,
},);

/**
 Undo button
 */
export const undoBtn: HTMLButtonElement = requireElement({
  selector: '#undo-btn',
  Ctor: HTMLButtonElement,
},);

/**
 Redo button
 */
export const redoBtn: HTMLButtonElement = requireElement({
  selector: '#redo-btn',
  Ctor: HTMLButtonElement,
},);

/**
 Color picker input
 */
export const colorPicker: HTMLInputElement = requireElement({
  selector: '#color-picker',
  Ctor: HTMLInputElement,
},);

/**
 Stroke width slider
 */
export const sizeSlider: HTMLInputElement = requireElement({
  selector: '#size-slider',
  Ctor: HTMLInputElement,
},);

/**
 Clear button
 */
export const clearBtn: HTMLButtonElement = requireElement({
  selector: '#clear-btn',
  Ctor: HTMLButtonElement,
},);

/**
 Export button
 */
export const exportBtn: HTMLButtonElement = requireElement({
  selector: '#export-btn',
  Ctor: HTMLButtonElement,
},);

/**
 Export format dropdown
 */
export const formatSelect: HTMLSelectElement = requireElement({
  selector: '#format-select',
  Ctor: HTMLSelectElement,
},);

/**
 Upload trigger button
 */
export const uploadBtn: HTMLButtonElement = requireElement({
  selector: '#upload-btn',
  Ctor: HTMLButtonElement,
},);

/**
 Hidden file upload input
 */
export const uploadInput: HTMLInputElement = requireElement({
  selector: '#upload-input',
  Ctor: HTMLInputElement,
},);

/**
 Zoom instruction toast popover
 */
export const zoomToast: HTMLDivElement = requireElement({
  selector: '#zoom-toast',
  Ctor: HTMLDivElement,
},);
