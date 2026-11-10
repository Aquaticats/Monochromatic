/**
 Resolves absolute X/Y coordinates from SVG path commands.
 Used by bounds computation and stroke expansion during font building.
 
 The command replay itself belongs to {@link ./svg-path-pen-cursor.ts}; this
 module only collects the points its drawn events carry.
 
 @module
 */

import type { Coordinate, } from './font-coordinate.ts';
import type { SVGPathCommand, } from './parse-svg.ts';
import { replayPenCursor, } from './svg-path-pen-cursor.ts';

/**
 Resolves absolute X/Y positions from SVG path commands (expanding H/V to full coords).
 
 @param commands - parsed SVG path commands
 
 @returns array of [x, y] coordinate pairs
 
 @example
 ```ts
 const points = resolveAbsolutePoints(parseSvgPathD("M10 20 H30 V40"));
 // [[10, 20], [30, 20], [30, 40]]
 ```
 */
export function resolveAbsolutePoints(commands: readonly SVGPathCommand[],): Coordinate[] {
  return replayPenCursor(commands,)
    .flatMap(function pointOf(event,): Coordinate[] {
      if (event.kind === 'close')
        return [];
      /**
       Point the drawn event carries.
       */
      const point: Coordinate = [
        event.x,
        event.y,
      ];
      return [point,];
    },);
}
