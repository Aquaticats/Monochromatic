/**
 Resolves absolute X/Y coordinates from SVG path commands.
 Used by bounds computation and stroke expansion during font building.
 
 @module
 */

import type { SVGPathCommand, } from './parse-svg.ts';
import type { Coordinate, } from './font-coordinate.ts';

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
export function resolveAbsolutePoints(
  commands: readonly SVGPathCommand[],
): Coordinate[] {
  /**
   Accumulator of resolved absolute coordinates, written in command order.
   */
  const points: Coordinate[] = [];
  /**
   X component of the pen cursor while replaying path commands.
   
   Declared as `let` because M/L set both axes, H rewrites only this one,
   and V leaves it alone; each command updates a different subset.
   */
  let cx = 0;
  /**
   Y component of the pen cursor while replaying path commands.
   
   Declared as `let` for the same reason as `cx`: V rewrites only this axis,
   while H leaves it alone.
   */
  let cy = 0;
  commands.forEach(function resolveCommand(cmd,) {
    if ((cmd.type
      === 'M') || (cmd.type
        === 'L')) {
      cx = cmd.x;
      cy = cmd.y;
      points.push([
        cx,
        cy,
      ],);
    }
    else if (cmd.type
      === 'H') {
      cx = cmd.x;
      points.push([
        cx,
        cy,
      ],);
    }
    else if (cmd.type
      === 'V') {
      cy = cmd.y;
      points.push([
        cx,
        cy,
      ],);
    }
  },);
  return points;
}
