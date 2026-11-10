/**
 Polygon offset utilities for expanding stroked SVG paths into filled outlines.
 Used by the font build to convert stroke-based glyphs (O, Q hexagons) into
 closed contours suitable for OpenType glyph paths.
 */

/**
 Cartesian coordinate pair.
 */
type Point = readonly [
  number,
  number,
];

/**
 Intersect two infinite lines, each defined by a point and direction vector.
 
 @param p1 - point on line 1
 
 @param d1 - direction of line 1
 
 @param p2 - point on line 2
 
 @param d2 - direction of line 2
 
 @returns intersection point
 
 @throws When the two directions are parallel, so the lines never meet
 
 @example
 ```ts
 lineIntersection({
   p1: [0, 0],
   d1: [1, 0],
   p2: [10, 5],
   d2: [0, 1],
 });
 // [10, 0]
 ```
 */
function lineIntersection({
  p1,
  d1,
  p2,
  d2,
}: {
  readonly p1: Point;
  readonly d1: Point;
  readonly p2: Point;
  readonly d2: Point;
},): Point {
  /**
   2D cross product of the two direction vectors; zero means parallel lines.
   */
  const cross = (d1[0]
    * d2[1]) - (d1[1]
      * d2[0]);
  if (cross === 0) {
    throw new Error(
      `consecutive polygon edges are collinear, so their offset lines never meet: [${d1.join(',',)}] and [${d2.join(',',)}]`,
    );
  }
  /**
   X offset from `p1` to `p2`, used to project the gap onto `d1`'s parameter axis.
   */
  const dx = p2[0]
    - p1[0];
  /**
   Y offset from `p1` to `p2`, paired with `dx` to form the gap vector.
   */
  const dy = p2[1]
    - p1[1];
  /**
   Parameter along line 1: the multiple of `d1` that lands on the intersection point.
   */
  const t = ((dx * d2[1]) - (dy * d2[0])) / cross;
  return [
    p1[0]
      + (t * d1[0]),
    p1[1]
      + (t * d1[1]),
  ];
}

/**
 Offset every edge of a closed polygon by a signed distance along its outward normal,
 then intersect consecutive offset edges via {@link lineIntersection} to
 produce the new vertex ring. Positive offset expands a clockwise (SVG
 Y-down) polygon outward; negative offset shrinks it inward.
 
 @param vertices - ordered polygon vertices (clockwise in SVG coordinate space)
 
 @param offset - signed offset distance (positive = outward, negative = inward)
 
 @returns new polygon vertices at the requested offset
 
 @throws When two consecutive vertices coincide, or two consecutive edges are collinear
 
 @example
 ```ts
 const square: [number, number][] = [[0, 0], [10, 0], [10, 10], [0, 10]];
 const expanded = offsetPolygon({ vertices: square, offset: 2 });
 ```
 */
export function offsetPolygon({
  vertices,
  offset,
}: {
  readonly vertices: readonly Point[];
  readonly offset: number;
},): Point[] {
  /**
   Cached vertex count used as the modulus for wrap-around edge indexing.
   */
  const vertexCount = vertices.length;

  /**
   Each edge of the source polygon shifted along its outward normal by `offset`,
   stored as `{ point, direction }` so the next pass can intersect consecutive edges.
   */
  const offsetEdges = vertices.map(function shiftEdge(
    vertex,
    vertexIndex,
  ) {
    /**
     Successor vertex; wraps around so the last edge connects back to vertex 0.
     */
    const next = vertices[(vertexIndex + 1) % vertexCount];
    if (next === undefined)
      throw new Error('unreachable: vertex index out of bounds',);
    /**
     Edge direction X component, used both for the normal and as the line direction.
     */
    const dx = next[0]
      - vertex[0];
    /**
     Edge direction Y component, paired with `dx` to define the edge vector.
     */
    const dy = next[1]
      - vertex[1];
    /**
     Edge length, used to normalise the direction vector before scaling by `offset`.
     */
    const len = Math.hypot(
      dx,
      dy,
    );
    if (len === 0) {
      throw new Error(
        `polygon edge ${vertexIndex} is a zero-length edge: vertices ${vertexIndex} and ${(vertexIndex + 1) % vertexCount} coincide at [${vertex.join(',',)}]`,
      );
    }
    /**
     X component of the outward normal scaled by `offset` (CW polygon, Y-down SVG coords).
     */
    const nx = (dy / len) * offset;
    /**
     Y component of the outward normal scaled by `offset`.
     */
    const ny = ((-dx) / len) * offset;
    return {
      point: [
        vertex[0]
          + nx,
        vertex[1]
          + ny,
      ] satisfies Point,
      direction: [
        dx,
        dy,
      ] satisfies Point,
    };
  },);

  // Intersect consecutive offset edges to find new vertices
  return offsetEdges.map(function intersectEdge(
    edge: Readonly<(typeof offsetEdges)[number]>,
    edgeIndex,
  ) {
    /**
     Successor offset edge; pairs with the current edge to produce one new vertex via intersection.
     */
    const nextEdge = offsetEdges[(edgeIndex + 1) % vertexCount];
    if (nextEdge === undefined)
      throw new Error('unreachable: edge index out of bounds',);
    return lineIntersection({
      p1: edge.point,
      d1: edge.direction,
      p2: nextEdge.point,
      d2: nextEdge.direction,
    },);
  },);
}
