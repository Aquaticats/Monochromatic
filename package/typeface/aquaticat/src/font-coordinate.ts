/**
 Coordinates in the two spaces the font build works in.

 The master glyph strip places every glyph at a cell offset in SVG coordinates,
 with Y growing downward from the top of the strip. An OpenType glyph outline
 starts at its own origin with Y growing upward from the baseline. `Coordinate`
 is the pair both spaces use, and `toFontCoordinate` is the only place the
 conversion between them is written.

 @module
 */

import { fontY, } from './build-font-metrics.ts';

/**
 Cartesian coordinate pair.
 */
export type Coordinate = readonly [
  number,
  number,
];

/**
 Converts one strip coordinate into glyph space.
 
 @param x - X coordinate in strip space
 
 @param y - Y coordinate in strip space
 
 @param cellX - X offset of the cell the point belongs to
 
 @param xShift - horizontal shift that places the glyph at its side bearing
 
 @returns the same point in glyph space
 
 @example
 ```ts
 toFontCoordinate({ x: 700, y: 750, cellX: 645, xShift: 40 }); // [95, 0]
 ```
 */
export function toFontCoordinate({
  x,
  y,
  cellX,
  xShift,
}: {
  readonly x: number;
  readonly y: number;
  readonly cellX: number;
  readonly xShift: number;
},): Coordinate {
  return [
    (x - cellX) + xShift,
    fontY(y,),
  ];
}
