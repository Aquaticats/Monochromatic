/**
 Text erasure for the doodle widget.
 
 Removes entire text input elements when the eraser path touches
 their bounding rect. Tests both eraser endpoint positions **and**
 the full eraser travel segment to avoid missing items during fast
 drags that sweep across a text input without stopping inside it.
 */

import {
  denormalizePoint,
  type NormalizedPoint,
} from './drawing.ts';
import { segmentIntersectsRect, } from './geometry.ts';

/**
 Checks whether a content-space pixel coordinate falls inside an
 input's layout bounds.
 
 Uses `offsetLeft`/`offsetTop`/`offsetWidth`/`offsetHeight` instead
 of `getBoundingClientRect()` so that CSS transforms (zoom/pan) on
 ancestor elements do not affect the comparison.
 
 @param px - test point x in content-space pixels
 
 @param py - test point y in content-space pixels
 
 @param input - text input element to test against
 
 @returns `true` if the point is inside the input's layout bounds
 */
function pointInInputRect(
  {
    px,
    py,
    input,
  }: {
    readonly px: number;
    readonly py: number;
    readonly input: HTMLInputElement;
  },
): boolean {
  return (px >= input
    .offsetLeft)
    && (px <= (input.offsetLeft
      + input
        .offsetWidth))
    && (py >= input
      .offsetTop)
    && (py <= (input.offsetTop
      + input
        .offsetHeight));
}

/**
 Erases text items touched by the eraser path.
 
 Tests the current and previous eraser positions against each text
 input's bounding rect, and uses {@link segmentIntersectsRect} to
 check whether the eraser travel segment crosses any edge of the
 rect to catch fast sweeps.
 
 @param point - current eraser position in normalized [0..1] space
 
 @param previousPoint - previous eraser position; omitted on first event
 
 @param cw - current canvas width in CSS pixels
 
 @param ch - current canvas height in CSS pixels
 
 @param textLayer - text layer container element
 
 @returns `true` if any text was removed
 
 @example
 ```ts
 const removed = eraseTextAt({
   point: [0.3, 0.4], previousPoint: [0.2, 0.3],
   cw: 800, ch: 600, textLayer,
 });
 ```
 */
export function eraseTextAt({
  point,
  previousPoint,
  cw,
  ch,
  textLayer,
}: {
  readonly point: NormalizedPoint;
  readonly previousPoint?: NormalizedPoint;
  readonly cw: number;
  readonly ch: number;
  readonly textLayer: HTMLDivElement;
},): boolean {
  /**
   Current eraser position in content-space pixels
   */
  const {
    px,
    py,
  } = denormalizePoint({
    point,
    cw,
    ch,
  },);

  /**
   Spread once so removals during iteration do not break the NodeList.
   */
  const inputs = [...textLayer.querySelectorAll<HTMLInputElement>('.text-input',),];
  /**
   Flag flipped only when at least one input is removed, so callers can skip redundant work.
   */
  let erased = false;

  /**
   Previous eraser position in content-space pixels when available
   */
  const prev = previousPoint !== undefined
    ? denormalizePoint({
      point: previousPoint,
      cw,
      ch,
    },)
    : undefined;

  for (const input of inputs) {
    /**
     Check current eraser position
     */
    const hitCurrent = pointInInputRect({
      px,
      py,
      input,
    },);

    /**
     Check previous eraser position when available
     */
    const hitPrevious = (prev !== undefined)
      && pointInInputRect({
        px: prev.px,
        py: prev.py,
        input,
      },);

    /**
     Check whether the eraser travel segment crosses the input rect
     */
    const hitSegment = (prev !== undefined)
      && segmentIntersectsRect({
        sx: prev.px,
        sy: prev.py,
        ex: px,
        ey: py,
        left: input.offsetLeft,
        top: input.offsetTop,
        right: input.offsetLeft
          + input
            .offsetWidth,
        bottom: input.offsetTop
          + input
            .offsetHeight,
      },);

    if (hitCurrent || hitPrevious
      || hitSegment) {
      input.remove();
      erased = true;
    }
  }

  return erased;
}
