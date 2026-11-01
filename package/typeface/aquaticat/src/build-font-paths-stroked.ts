/**
 Stroked path expansion for converting SVG stroke-based glyphs into filled outlines.
 Separated from {@link ./build-font-paths.ts} to stay within the max-lines budget.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type * as opentype from 'opentype.js';

import { fontY, } from './build-font-metrics.ts';
import { resolveAbsolutePoints, } from './build-font-resolve-points.ts';
import { offsetPolygon, } from './expand-stroke.ts';
import type { SVGPathCommand, } from './parse-svg.ts';

/**
 Adds a stroked polygon to an opentype Path as an expanded filled outline.
 Resolves the centreline with {@link resolveAbsolutePoints}, then offsets
 it outward and inward with {@link offsetPolygon} to trace both contours.
 
 @param otPath - opentype path to append to
 
 @param commands - parsed SVG path commands
 
 @param strokeWidth - stroke width in SVG units
 
 @param cellX - X offset of the cell in SVG coordinates
 
 @param xShift - horizontal shift to apply for proportional spacing
 
 @mutates otPath - `otPath.moveTo`, `otPath.lineTo`, and `otPath.close` append glyph commands
 
 @example
 ```ts
 addStrokedPath({
   otPath: glyphPath,
   commands: parseSvgPathD('M0 0 L10 0 L10 10 L0 10 Z'),
   strokeWidth: 2,
   cellX: 0,
   xShift: 40,
 });
 ```
 */
export function addStrokedPath({
  otPath,
  commands,
  strokeWidth,
  cellX,
  xShift,
}: ForeignBorrowed<Readonly<{
  otPath: opentype.Path;
  commands: readonly SVGPathCommand[];
  strokeWidth: number;
  cellX: number;
  xShift: number;
}>>,): void {
  /**
   Half the stroke width, the signed distance each side is shifted from the centreline.
   */
  const halfWidth = strokeWidth / 2;
  /**
   Absolute coordinates of the stroke centreline, with H/V already expanded.
   */
  const points = resolveAbsolutePoints(commands,);
  /**
   First and last centreline points, extracted so the explicit closing vertex can be detected.
   */
  const [first,] = points;
  /**
   Trailing centreline point, paired with `first` to detect an explicit close.
   */
  const last = points.at(-1,);
  /**
   Centreline polygon used for offsetting.
   
   SVG's Z command implies a close, so an explicit duplicate of the first vertex
   at the end would cause `offsetPolygon` to emit a zero-length edge; drop it
   when present so every edge has a real direction.
   */
  const vertices = (
    (first !== undefined)
      && (last !== undefined)
      && (points.length
        > 1)
      && (first[0]
        === last[0])
      && (first[1]
        === last[1])
  )
    ? points.slice(
      0,
      -1,
    )
    : points;

  /**
   Outer contour: centreline expanded outward by `halfWidth`.
   */
  const outerVerts = offsetPolygon({
    vertices,
    offset: halfWidth,
  },);
  /**
   Inner contour: centreline shrunk inward by `halfWidth`, traced in reverse to form a hole.
   */
  const innerVerts = offsetPolygon({
    vertices,
    offset: -halfWidth,
  },);

  /**
   Traces a polygon contour onto the opentype path.
   
   @param verts - ordered vertices of the contour polygon
   */
  function traceContour(verts: readonly (readonly [
    number,
    number,
  ])[],): void {
    verts.forEach(function traceVertex(
      vert,
      vertIndex,
    ) {
      /**
       Glyph-space X: vertex shifted from SVG coords into the glyph's local origin.
       */
      const fx = (vert[0]
        - cellX) + xShift;
      /**
       Glyph-space Y: vertex flipped into font Y-up coordinates by {@link fontY}.
       */
      const fy = fontY(vert[1],);
      if (vertIndex === 0) {
        otPath.moveTo(
          fx,
          fy,
        );
      }
      else {
        otPath.lineTo(
          fx,
          fy,
        );
      }
    },);
    otPath.close();
  }

  // Outer contour (forward order)
  traceContour(outerVerts,);
  // Inner contour (reversed to create the hole via opposite winding)
  traceContour([...innerVerts,].toReversed(),);
}
