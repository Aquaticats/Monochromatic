/**
 * Convex polygon to prism mesh arrays for glass shards.
 *
 * Each Voronoi cell from the fracture becomes a thin prism: front face,
 * back face, and side walls. Emitting plain typed arrays keeps this module
 * three.js-free and unit-testable; the debris system wraps the arrays in a
 * BufferGeometry.
 */
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type PaneCell,
  type PanePoint,
  polygonCentroid,
} from './fracture.ts';

/**
 * Components per position or normal vector in the flat buffers.
 */
const XYZ = 3;

/**
 * Fewest vertices a polygon can have and still enclose area.
 */
const MIN_POLYGON_VERTICES = 3;

/**
 * Vertices each side wall carries so it gets flat outward normals.
 */
const WALL_VERTICES = 4;

/**
 * Indices each side wall emits: two triangles.
 */
const WALL_INDICES = 6;

/**
 * Mesh arrays for one shard prism, ready for BufferGeometry attributes.
 * Positions are centered: the polygon centroid maps to the local origin so
 * shard instances rotate around their own center of mass.
 */
export type PrismMesh = {
  /**
   * Interleaved xyz vertex positions, meters, shard-local.
   */
  readonly positions: Float32Array;
  /**
   * Per-vertex unit normals matching {@link PrismMesh.positions}.
   */
  readonly normals: Float32Array;
  /**
   * Triangle indices into the vertex arrays.
   */
  readonly indices: Uint16Array;
  /**
   * Centroid the polygon was recentered around, in pane-local meters.
   */
  readonly pivot: PanePoint;
};

/**
 * Builds prism mesh arrays from one convex fracture cell.
 *
 * Layout per cell of n vertices: n front vertices, n back vertices, then
 * 4 vertices per side wall so walls carry flat outward normals instead of
 * sharing smoothed face normals. Front and back faces triangulate as fans,
 * which is valid because Voronoi cells are convex.
 *
 * @param polygon - convex cell, counterclockwise winding, at least 3 vertices
 *
 * @param thickness - pane thickness in meters, extruded symmetrically
 *
 * @returns mesh arrays plus the pivot used for recentering
 *
 * @throws RangeError when the polygon has fewer than 3 vertices
 *
 * @example
 * ```ts
 * const mesh = prismFromPolygon({
 *   polygon: [{ x: 0, y: 0 }, { x: 0.1, y: 0 }, { x: 0.05, y: 0.12 }],
 *   thickness: 0.01,
 * },);
 * ```
 */
export function prismFromPolygon(
  {
    polygon,
    thickness,
  }: Readonly<{
    polygon: PaneCell;
    thickness: number;
  }>,
): PrismMesh {
  if (polygon.length < MIN_POLYGON_VERTICES)
    throw new RangeError(`prism needs at least 3 vertices, got ${String(polygon.length,)}`,);
  /**
   * Recenter pivot so the shard spins around its own center of mass.
   */
  const pivot = polygonCentroid(polygon,);
  /**
   * Cell vertices shifted so the pivot sits at the local origin.
   */
  const ring = polygon.map(function recenter(vertex: PanePoint,): PanePoint {
    return {
      x: vertex.x - pivot.x,
      y: vertex.y - pivot.y,
    };
  },);
  /**
   * Ring vertex count, reused for every block offset below.
   */
  const count = ring.length;
  /**
   * Half thickness: faces sit symmetrically around the pane midplane.
   */
  const half = thickness / 2;
  /**
   * Vertex total: front ring, back ring, and 4 vertices per side wall.
   */
  const vertexTotal = (count * 2) + (count * WALL_VERTICES);
  /**
   * Index total: two fan faces plus two triangles per side wall.
   */
  const indexTotal = ((count - 2) * XYZ
    * 2) + (count * WALL_INDICES);
  /**
   * Position buffer filled block by block below.
   */
  const positions = new Float32Array(vertexTotal * XYZ,);
  /**
   * Normal buffer parallel to {@link positions}.
   */
  const normals = new Float32Array(vertexTotal * XYZ,);
  /**
   * Index buffer; Uint16 suffices because shard prisms stay tiny.
   */
  const indices = new Uint16Array(indexTotal,);
  //region Front and back rings: flat +z / -z faces sharing the cell outline
  for (const [index, vertex,] of ring.entries()) {
    positions.set(
      [
        vertex.x,
        vertex.y,
        half,
      ],
      index * XYZ,
    );
    normals.set(
      [
        0,
        0,
        1,
      ],
      index * XYZ,
    );
    positions.set(
      [
        vertex.x,
        vertex.y,
        -half,
      ],
      (count + index) * XYZ,
    );
    normals.set(
      [
        0,
        0,
        -1,
      ],
      (count + index) * XYZ,
    );
  }
  /**
   * Index slots taken by one fan face; the back fan and the walls start
   * at fixed offsets computed from it, so no cursor mutates.
   */
  const fanIndexCount = (count - 2) * XYZ;
  for (let fan = 1; fan < (count
    - 1); fan++) {
    indices.set(
      [
        0,
        fan,
        fan + 1,
      ],
      (fan - 1) * XYZ,
    );
  }
  for (let fan = 1; fan < (count
    - 1); fan++) {
    indices.set(
      [
        count,
        count + fan
          + 1,
        count + fan,
      ],
      fanIndexCount + ((fan - 1) * XYZ),
    );
  }
  //endregion
  //region Side walls: one quad per edge with a flat outward normal
  for (const [index, vertex,] of ring.entries()) {
    /**
     * Edge partner: next ring vertex, wrapping to close the outline.
     */
    const next = nonNullishOrThrow(ring[(index + 1) % count],);
    /**
     * Outward wall normal: edge direction rotated -90 degrees, normalized.
     * Counterclockwise winding puts the outside on this side.
     */
    const edgeLength = Math.hypot(
      next.x - vertex.x,
      next.y - vertex.y,
    ) || 1;
    /**
     * Wall normal x component after rotation and normalization.
     */
    const wallX = (next.y - vertex.y) / edgeLength;
    /**
     * Wall normal y component after rotation and normalization.
     */
    const wallY = (-(next.x - vertex.x)) / edgeLength;
    /**
     * First vertex slot of this wall's 4-vertex block.
     */
    const base = (count * 2) + (index * WALL_VERTICES);
    positions.set(
      [
        vertex.x,
        vertex.y,
        half,
        next.x,
        next.y,
        half,
        next.x,
        next.y,
        -half,
        vertex.x,
        vertex.y,
        -half,
      ],
      base * XYZ,
    );
    for (let corner = 0; corner < WALL_VERTICES; corner++)
      normals.set(
        [
          wallX,
          wallY,
          0,
        ],
        (base + corner) * XYZ,
      );
    indices.set(
      [
        base,
        base + 2,
        base + 1,
        base,
        base + (WALL_VERTICES - 1),
        base + 2,
      ],
      (fanIndexCount * 2) + (index * WALL_INDICES),
    );
  }
  //endregion
  return {
    positions,
    normals,
    indices,
    pivot,
  };
}
