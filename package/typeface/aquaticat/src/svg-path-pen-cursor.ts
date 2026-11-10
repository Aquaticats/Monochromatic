/**
 The pen cursor that replays SVG path commands.

 Glyph bounds resolution and OpenType path emission both need the same M/L/H/V/Z
 state machine: M and L set both axes, H rewrites X, V rewrites Y, and Z closes
 the current contour without moving the pen. This module is the one owner of that
 replay. {@link ./build-font-resolve-points.ts} collects the points of its drawn
 events, and {@link ./build-font-paths.ts} maps every event onto an OpenType path
 command, so bounds and contours cannot disagree about a command's meaning.

 @module
 */

import type { SVGPathCommand, } from './parse-svg.ts';

/**
 One step of a replayed path: a pen position to move to or draw to, or the close
 of the contour being drawn.
 */
export type PenCursorEvent =
  | {
    readonly kind: 'moveTo';
    readonly x: number;
    readonly y: number;
  }
    | {
      readonly kind: 'lineTo';
      readonly x: number;
      readonly y: number;
    }
    | { readonly kind: 'close'; };

/**
 Replays SVG path commands through the pen cursor.
 
 @param commands - parsed SVG path commands
 
 @returns one event per drawn or closed step, in path order
 
 @example
 ```ts
 replayPenCursor(parseSvgPathD('M10 20 H30 V40 Z'));
 // [{ kind: 'moveTo', x: 10, y: 20 }, { kind: 'lineTo', x: 30, y: 20 },
 //  { kind: 'lineTo', x: 30, y: 40 }, { kind: 'close' }]
 ```
 */
export function replayPenCursor(commands: readonly SVGPathCommand[],): PenCursorEvent[] {
  /**
   Accumulator of replayed events, written in path order.
   */
  const events: PenCursorEvent[] = [];
  /**
   X component of the pen cursor while replaying path commands.
   
   Declared as `let` because M/L set both axes, H rewrites only this one, and V
   leaves it alone; each command updates a different subset.
   */
  let cx = 0;
  /**
   Y component of the pen cursor.
   
   Declared as `let` for the same reason as `cx`: V rewrites only this axis, and
   H leaves it alone.
   */
  let cy = 0;

  commands.forEach(function replayCommand(cmd,) {
    if ((cmd.type
      === 'M') || (cmd.type
        === 'L')) {
      cx = cmd.x;
      cy = cmd.y;
      events.push({
        kind: cmd.type
          === 'M'
          ? 'moveTo'
          : 'lineTo',
        x: cx,
        y: cy,
      },);
    }
    else if (cmd.type
      === 'H') {
      cx = cmd.x;
      events.push({
        kind: 'lineTo',
        x: cx,
        y: cy,
      },);
    }
    else if (cmd.type
      === 'V') {
      cy = cmd.y;
      events.push({
        kind: 'lineTo',
        x: cx,
        y: cy,
      },);
    }
    else if (cmd.type
      === 'Z') {
      events.push({ kind: 'close', },);
    }
  },);

  return events;
}
