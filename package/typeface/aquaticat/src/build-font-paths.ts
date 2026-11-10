/**
 Path construction utilities for converting SVG glyph data into OpenType path commands.
 
 @example
 ```ts
 import { computeLocalXBounds, addFilledPath } from "./build-font-paths.ts";
 ```
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type * as opentype from 'opentype.js';

import { resolveAbsolutePoints, } from './build-font-resolve-points.ts';
import { toFontCoordinate, } from './font-coordinate.ts';
import {
  type CellPath,
  parseSvgPathD,
  type SVGPathCommand,
} from './parse-svg.ts';

//region Path construction

/**
 Computes the X bounding box of all paths in a cell, in local cell
 coordinates, using points resolved via {@link resolveAbsolutePoints}. For
 stroked paths, the bounds are expanded by half the stroke width.
 
 @param paths - cell paths with their stroke widths
 
 @param cellX - X offset of the cell in SVG coordinates
 
 @returns min and max X in local cell coordinates
 
 @example
 ```ts
 const { minX, maxX } = computeLocalXBounds({
   paths: cell.paths,
   cellX: cell.xOffset,
 });
 ```
 */
export function computeLocalXBounds({
  paths,
  cellX,
}: {
  readonly paths: readonly CellPath[];
  readonly cellX: number;
},): {
  minX: number;
  maxX: number;
} {
  /**
   Running minimum of local X across every point of every path.
   
   Declared as `let` because the value is reduced across two nested forEach
   loops; seeded to `+Infinity` so the first comparison always wins.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- accumulator across nested forEach loops; seed +Infinity guarantees first comparison wins
  let minX = Infinity;
  /**
   Running maximum of local X, the symmetric partner of `minX`.
   
   Seeded to `-Infinity` so the first comparison always wins.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- accumulator across nested forEach loops; seed -Infinity guarantees first comparison wins
  let maxX = -Infinity;

  paths.forEach(function measurePath(pathData,) {
    /**
     Parsed command list for this path, used to drive {@link resolveAbsolutePoints}.
     */
    const commands = parseSvgPathD(pathData.d,);
    /**
     Absolute point coordinates for this path (H/V already expanded).
     */
    const points = resolveAbsolutePoints(commands,);
    /**
     Half the stroke width: the amount each point's bounding box extends past the centreline.
     */
    const halfStroke = pathData.strokeWidth
      / 2;

    points.forEach(function updateBounds([px,],) {
      /**
       Point X translated into cell-local coordinates so bounds are independent of cell position.
       */
      const localX = px - cellX;
      minX = Math.min(
        minX,
        localX - halfStroke,
      );
      maxX = Math.max(
        maxX,
        localX + halfStroke,
      );
    },);
  },);

  return {
    minX,
    maxX,
  };
}

/**
 Adds a filled SVG path to an opentype Path, applying coordinate transforms.
 
 @param otPath - opentype path to append to
 
 @param commands - parsed SVG path commands
 
 @param cellX - X offset of the cell in SVG coordinates
 
 @param xShift - horizontal shift to apply for proportional spacing
 
 @mutates otPath - `otPath.moveTo`, `otPath.lineTo`, and `otPath.close` append glyph commands
 
 @example
 ```ts
 addFilledPath({
   otPath: glyphPath,
   commands: parseSvgPathD('M0 0 L10 0 L10 10 Z'),
   cellX: 0,
   xShift: 40,
 });
 ```
 */
export function addFilledPath({
  otPath,
  commands,
  cellX,
  xShift,
}: ForeignBorrowed<Readonly<{
  otPath: opentype.Path;
  commands: readonly SVGPathCommand[];
  cellX: number;
  xShift: number;
}>>,): void {
  /**
   X component of the pen cursor while emitting OpenType path commands.
   
   Declared as `let` because SVG H rewrites only this axis, and the cursor
   is read back into the opentype `moveTo`/`lineTo` calls for the next vertex.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- pen cursor: H rewrites only x, each command consumes the prior cursor across forEach iterations
  let cx = 0;
  /**
   Y component of the pen cursor.
   
   Declared as `let` for the same reason as `cx`: V rewrites only this axis,
   and the cursor is read back on every subsequent command.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- pen cursor: V rewrites only y, each command consumes the prior cursor across forEach iterations
  let cy = 0;

  commands.forEach(function traceFilledCommand(cmd,) {
    if (cmd.type
      === 'M') {
      cx = cmd.x;
      cy = cmd.y;
      /**
       Glyph-space point of the pen cursor after the move.
       */
      const [glyphX, glyphY,] = toFontCoordinate({
        x: cx,
        y: cy,
        cellX,
        xShift,
      },);
      otPath.moveTo(
        glyphX,
        glyphY,
      );
    }
    else if (cmd.type
      === 'L') {
      cx = cmd.x;
      cy = cmd.y;
      /**
       Glyph-space point of the pen cursor after the line.
       */
      const [glyphX, glyphY,] = toFontCoordinate({
        x: cx,
        y: cy,
        cellX,
        xShift,
      },);
      otPath.lineTo(
        glyphX,
        glyphY,
      );
    }
    else if (cmd.type
      === 'H') {
      cx = cmd.x;
      /**
       Glyph-space point after the horizontal command, whose Y is unchanged.
       */
      const [glyphX, glyphY,] = toFontCoordinate({
        x: cx,
        y: cy,
        cellX,
        xShift,
      },);
      otPath.lineTo(
        glyphX,
        glyphY,
      );
    }
    else if (cmd.type
      === 'V') {
      cy = cmd.y;
      /**
       Glyph-space point after the vertical command, whose X is unchanged.
       */
      const [glyphX, glyphY,] = toFontCoordinate({
        x: cx,
        y: cy,
        cellX,
        xShift,
      },);
      otPath.lineTo(
        glyphX,
        glyphY,
      );
    }
    else if (cmd.type
      === 'Z') {
      otPath.close();
    }
  },);
}

//endregion Path construction
