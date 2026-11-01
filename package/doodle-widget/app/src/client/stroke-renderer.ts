/**
 Shared stroke rendering for the doodle widget.
 
 Provides a single rendering function used by both the live canvas
 redraw and the export compositors.
 */

import type { StrokeData, } from './drawing.ts';

/**
 Minimum number of points required for a renderable stroke
 */
export const MIN_STROKE_POINTS = 2;

/**
 Canvas context type accepted by the renderer
 */
type RenderContext = CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D;

/**
 Renders stroke data onto a canvas context.
 
 Each stroke is drawn using its captured color and width, with
 normalized [0..1] coordinates denormalized to the given dimensions.
 
 @param ctx - 2D context (on-screen or offscreen) to draw on
 
 @param cw - canvas width in CSS pixels for coordinate denormalization
 
 @param ch - canvas height in CSS pixels for coordinate denormalization
 
 @param strokes - stroke data with normalized [0..1] coordinates
 
 @example
 ```ts
 renderStrokes({ ctx, cw: 800, ch: 600, strokes: getStrokes() });
 ```
 */
export function renderStrokes({
  ctx,
  cw,
  ch,
  strokes,
}: {
  readonly ctx: RenderContext;
  readonly cw: number;
  readonly ch: number;
  readonly strokes: readonly StrokeData[];
},): void {
  for (const stroke of strokes) {
    if (stroke.points
      .length
      < MIN_STROKE_POINTS)
      continue;
    ctx.strokeStyle = stroke.color;
    ctx.lineWidth = stroke.width;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.beginPath();
    for (const [index, point,] of stroke.points
      .entries()) {
      if (index === 0) {
        ctx.moveTo(
          point[0]
            * cw,
          point[1]
            * ch,
        );
      }
      else {
        ctx.lineTo(
          point[0]
            * cw,
          point[1]
            * ch,
        );
      }
    }
    ctx.stroke();
  }
}
