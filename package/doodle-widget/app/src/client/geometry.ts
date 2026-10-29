/**
 2D geometry helpers for segment distance and intersection tests.
 
 Used by the eraser to detect proximity between the eraser travel
 path and stroke geometry (both individual points and line segments).
 */

/**
 Computes the squared distance from point P to the closest point
 on line segment AB, in pixel space.
 
 Uses projection clamped to [0, 1] to find the nearest point on the
 segment rather than the infinite line.
 
 @param px - point x in pixels
 
 @param py - point y in pixels
 
 @param ax - segment start x in pixels
 
 @param ay - segment start y in pixels
 
 @param bx - segment end x in pixels
 
 @param by - segment end y in pixels
 
 @returns squared pixel distance from P to nearest point on AB
 
 @example
 ```ts
 const distSq = distToSegmentSq({ px: 5, py: 5, ax: 0, ay: 0, bx: 10, by: 0 });
 ```
 */
export function distToSegmentSq(
  {
    px,
    py,
    ax,
    ay,
    bx,
    by,
  }: {
    readonly px: number;
    readonly py: number;
    readonly ax: number;
    readonly ay: number;
    readonly bx: number;
    readonly by: number;
  },
): number {
  /**
   Cached so {@link lenSq} and the projection share one subtraction.
   */
  const dx = bx - ax;
  /**
   Companion to {@link dx} on the y axis.
   */
  const dy = by - ay;
  /**
   Squared length of segment AB; zero means degenerate (A == B)
   */
  const lenSq = (dx * dx) + (dy * dy);

  if (lenSq === 0)
    return ((px - ax) ** 2) + ((py - ay) ** 2);

  /**
   Projection parameter clamped to [0, 1] so closest point stays on segment
   */
  const t = Math.max(
    0,
    Math.min(
      1,
      (((px - ax) * dx) + ((py - ay) * dy)) / lenSq,
    ),
  );
  /**
   Closest point on segment
   */
  const cx = ax + (t * dx);
  /**
   Companion to {@link cx} on the y axis.
   */
  const cy = ay + (t * dy);

  return ((px - cx) ** 2) + ((py - cy) ** 2);
}

/**
 2D cross product of vectors (B - A) and (C - A).
 
 Positive when C is counter-clockwise from AB, negative when clockwise,
 zero when collinear.
 
 @param ax - first point x
 
 @param ay - first point y
 
 @param bx - second point x
 
 @param by - second point y
 
 @param cx - third point x
 
 @param cy - third point y
 
 @returns signed area of the parallelogram spanned by AB and AC
 */
function cross(
  {
    ax,
    ay,
    bx,
    by,
    cx,
    cy,
  }: {
    readonly ax: number;
    readonly ay: number;
    readonly bx: number;
    readonly by: number;
    readonly cx: number;
    readonly cy: number;
  },
): number {
  return ((bx - ax) * (cy - ay)) - ((by - ay) * (cx - ax));
}

/**
 Parameter shape shared by {@link segmentsIntersect} and {@link segToSegDistSq}.
 */
export type TwoSegments = {
  readonly a1x: number;
  readonly a1y: number;
  readonly a2x: number;
  readonly a2y: number;
  readonly b1x: number;
  readonly b1y: number;
  readonly b2x: number;
  readonly b2y: number;
};

/**
 Tests whether two line segments properly intersect (cross each other).
 
 Uses the orientation (cross-product sign) test: segments AB and CD
 intersect when A and B are on opposite sides of line CD, and C and D
 are on opposite sides of line AB. Collinear/touching cases return
 false because the radius check in the caller already covers near misses.
 
 @param a1x - segment 1 start x
 
 @param a1y - segment 1 start y
 
 @param a2x - segment 1 end x
 
 @param a2y - segment 1 end y
 
 @param b1x - segment 2 start x
 
 @param b1y - segment 2 start y
 
 @param b2x - segment 2 end x
 
 @param b2y - segment 2 end y
 
 @returns true when the segments properly cross each other
 
 @example
 ```ts
 segmentsIntersect({ a1x: 0, a1y: 0, a2x: 10, a2y: 10, b1x: 10, b1y: 0, b2x: 0, b2y: 10 });
 ```
 */
export function segmentsIntersect(
  {
    a1x,
    a1y,
    a2x,
    a2y,
    b1x,
    b1y,
    b2x,
    b2y,
  }: TwoSegments,
): boolean {
  /**
   Orientation of A1 relative to segment B
   */
  const d1 = cross({
    ax: b1x,
    ay: b1y,
    bx: b2x,
    by: b2y,
    cx: a1x,
    cy: a1y,
  },);
  /**
   Orientation of A2 relative to segment B
   */
  const d2 = cross({
    ax: b1x,
    ay: b1y,
    bx: b2x,
    by: b2y,
    cx: a2x,
    cy: a2y,
  },);
  /**
   Orientation of B1 relative to segment A
   */
  const d3 = cross({
    ax: a1x,
    ay: a1y,
    bx: a2x,
    by: a2y,
    cx: b1x,
    cy: b1y,
  },);
  /**
   Orientation of B2 relative to segment A
   */
  const d4 = cross({
    ax: a1x,
    ay: a1y,
    bx: a2x,
    by: a2y,
    cx: b2x,
    cy: b2y,
  },);

  return (((d1 > 0) && (d2 < 0)) || ((d1 < 0) && (d2 > 0)))
    && (((d3 > 0) && (d4 < 0)) || ((d3 < 0) && (d4 > 0)));
}

/**
 Computes the squared minimum distance between two line segments.
 
 Handles two cases:
 1. Segments intersect; distance is 0
 2. Otherwise; minimum of the four endpoint-to-opposite-segment distances
 
 @param a1x - segment 1 start x
 
 @param a1y - segment 1 start y
 
 @param a2x - segment 1 end x
 
 @param a2y - segment 1 end y
 
 @param b1x - segment 2 start x
 
 @param b1y - segment 2 start y
 
 @param b2x - segment 2 end x
 
 @param b2y - segment 2 end y
 
 @returns squared pixel distance between the closest points on the two segments
 
 @example
 ```ts
 const distSq = segToSegDistSq({ a1x: 0, a1y: 0, a2x: 5, a2y: 0, b1x: 3, b1y: 1, b2x: 3, b2y: 5 });
 ```
 */
export function segToSegDistSq(
  {
    a1x,
    a1y,
    a2x,
    a2y,
    b1x,
    b1y,
    b2x,
    b2y,
  }: TwoSegments,
): number {
  if (segmentsIntersect({
    a1x,
    a1y,
    a2x,
    a2y,
    b1x,
    b1y,
    b2x,
    b2y,
  },)) {
    return 0;
  }

  return Math.min(
    distToSegmentSq({
      px: a1x,
      py: a1y,
      ax: b1x,
      ay: b1y,
      bx: b2x,
      by: b2y,
    },),
    distToSegmentSq({
      px: a2x,
      py: a2y,
      ax: b1x,
      ay: b1y,
      bx: b2x,
      by: b2y,
    },),
    distToSegmentSq({
      px: b1x,
      py: b1y,
      ax: a1x,
      ay: a1y,
      bx: a2x,
      by: a2y,
    },),
    distToSegmentSq({
      px: b2x,
      py: b2y,
      ax: a1x,
      ay: a1y,
      bx: a2x,
      by: a2y,
    },),
  );
}

/**
 Tests whether a line segment intersects an axis-aligned rectangle.
 
 Checks the segment against all four edges of the rectangle using
 {@link segmentsIntersect}. Does **not** check whether either endpoint
 is inside the rect; the caller handles that separately.
 
 @param sx - segment start x
 
 @param sy - segment start y
 
 @param ex - segment end x
 
 @param ey - segment end y
 
 @param left - rectangle left edge (min x)
 
 @param top - rectangle top edge (min y)
 
 @param right - rectangle right edge (max x)
 
 @param bottom - rectangle bottom edge (max y)
 
 @returns true when the segment crosses any edge of the rectangle
 
 @example
 ```ts
 segmentIntersectsRect({ sx: 0, sy: 0, ex: 10, ey: 10, left: 5, top: 0, right: 15, bottom: 10 });
 ```
 */
export function segmentIntersectsRect(
  {
    sx,
    sy,
    ex,
    ey,
    left,
    top,
    right,
    bottom,
  }: {
    readonly sx: number;
    readonly sy: number;
    readonly ex: number;
    readonly ey: number;
    readonly left: number;
    readonly top: number;
    readonly right: number;
    readonly bottom: number;
  },
): boolean {
  /**
   Top edge: left,top -> right,top
   */
  if (segmentsIntersect({
    a1x: sx,
    a1y: sy,
    a2x: ex,
    a2y: ey,
    b1x: left,
    b1y: top,
    b2x: right,
    b2y: top,
  },)) {
    return true;
  }
  /**
   Bottom edge: left,bottom -> right,bottom
   */
  if (segmentsIntersect({
    a1x: sx,
    a1y: sy,
    a2x: ex,
    a2y: ey,
    b1x: left,
    b1y: bottom,
    b2x: right,
    b2y: bottom,
  },)) {
    return true;
  }
  /**
   Left edge: left,top -> left,bottom
   */
  if (segmentsIntersect({
    a1x: sx,
    a1y: sy,
    a2x: ex,
    a2y: ey,
    b1x: left,
    b1y: top,
    b2x: left,
    b2y: bottom,
  },)) {
    return true;
  }
  /**
   Right edge: right,top -> right,bottom
   */
  if (segmentsIntersect({
    a1x: sx,
    a1y: sy,
    a2x: ex,
    a2y: ey,
    b1x: right,
    b1y: top,
    b2x: right,
    b2y: bottom,
  },)) {
    return true;
  }

  return false;
}
