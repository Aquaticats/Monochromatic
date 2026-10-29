/**
 Active drawing configuration for the doodle widget.
 
 Manages the current stroke color and width that are captured
 by new strokes at creation time.
 */

//region Constants

/**
 Default stroke color as hex, approximating oklch(0.6 0.25 27).
 
 Must match the `value` attribute on `#color-picker` in page.ts.
 */
const DEFAULT_STROKE_COLOR = '#c24e2e';

/**
 Default stroke width in CSS pixels
 */
const DEFAULT_STROKE_WIDTH = 10;

//endregion Constants

//region State

/**
 Active drawing configuration container.
 
 Stored as an object property so module-root state stays in a `const`
 container (`no-module-root-let` would otherwise reject a top-level `let`).
 */
const drawingState: {
  color: string;
  width: number;
} = {
  color: DEFAULT_STROKE_COLOR,
  width: DEFAULT_STROKE_WIDTH,
};

//endregion State

/**
 Returns the active stroke color.
 
 @returns CSS color string used for new strokes
 
 @example
 ```ts
 const color = getStrokeColor();
 ctx.strokeStyle = color;
 ```
 */
export function getStrokeColor(): string {
  return drawingState.color;
}

/**
 Sets the active stroke color for subsequent strokes.
 
 @param color - CSS color string (hex for color picker compatibility)
 
 @example
 ```ts
 setStrokeColor('#ff0000');
 ```
 */
export function setStrokeColor(color: string,): void {
  drawingState.color = color;
}

/**
 Returns the active stroke width.
 
 @returns width in CSS pixels used for new strokes
 
 @example
 ```ts
 const width = getStrokeWidth();
 ctx.lineWidth = width;
 ```
 */
export function getStrokeWidth(): number {
  return drawingState.width;
}

/**
 Sets the active stroke width for subsequent strokes.
 
 @param width - width in CSS pixels
 
 @example
 ```ts
 setStrokeWidth(5);
 ```
 */
export function setStrokeWidth(width: number,): void {
  drawingState.width = width;
}
