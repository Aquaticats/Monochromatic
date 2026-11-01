/**
 Canvas 2D context acquisition helper for the doodle widget.
 
 Eliminates the repeated null-check boilerplate when obtaining
 a 2D rendering context from canvas elements.
 */

/**
 Obtains a 2D rendering context from an on-screen canvas, throwing
 if the context is unavailable.
 
 @param canvas - HTML canvas element
 
 @returns 2D rendering context
 
 @throws Error if the browser cannot provide a 2D context
 
 @example
 ```ts
 const ctx = requireCanvasContext(document.querySelector('canvas')!);
 ```
 */
export function requireCanvasContext(
  canvas: HTMLCanvasElement,
): CanvasRenderingContext2D {
  /**
   Retained so the null-check pinpoints which canvas refused the context.
   */
  const ctx = canvas.getContext('2d',);
  if (ctx === null)
    throw new Error('Canvas 2D context unavailable',);
  return ctx;
}

/**
 Obtains a 2D rendering context from an offscreen canvas, throwing
 if the context is unavailable.
 
 @param canvas - offscreen canvas instance
 
 @returns offscreen 2D rendering context
 
 @throws Error if the browser cannot provide a 2D context
 
 @example
 ```ts
 const ctx = requireOffscreenContext(new OffscreenCanvas(800, 600));
 ```
 */
export function requireOffscreenContext(
  canvas: OffscreenCanvas,
): OffscreenCanvasRenderingContext2D {
  /**
   Retained so the null-check pinpoints which offscreen canvas refused the context.
   */
  const ctx = canvas.getContext('2d',);
  if (ctx === null)
    throw new Error('Export canvas 2D context unavailable',);
  return ctx;
}
