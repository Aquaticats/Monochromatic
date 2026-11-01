/**
 * Merged prism arrays for a pane's surviving rim.
 *
 * After a strike blasts out the hole cells, the rim cells still stand as
 * real glass with a real opening. Rendering them needs one mesh in
 * pane-local meters: every rim cell's prism, un-recentered back to its
 * true position, concatenated into single position/normal/index buffers.
 * Pure typed-array math, so the whole module unit-tests in node.
 */
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { PaneCell, } from './fracture.ts';
import {
  prismFromPolygon,
  type PrismMesh,
} from './prism.ts';

/**
 * Components per position or normal vector in the flat buffers.
 */
const XYZ = 3;

/**
 * Largest vertex index a Uint16 index buffer can address.
 */
const MAX_UINT16 = 65_535;

/**
 * Merged mesh arrays for one rim, ready for BufferGeometry attributes.
 * Positions live in pane-local meters: each cell prism is shifted back by
 * its own centroid so the union sits exactly where the sheet was.
 */
export type RimMeshArrays = {
  /**
   * Interleaved xyz vertex positions, meters, pane-local.
   */
  readonly positions: Float32Array;
  /**
   * Per-vertex unit normals matching {@link RimMeshArrays.positions}.
   */
  readonly normals: Float32Array;
  /**
   * Triangle indices into the merged vertex arrays.
   */
  readonly indices: Uint16Array;
};

/**
 * Builds one merged mesh from every rim cell's prism.
 *
 * @param cells - surviving rim cells in pane-local meters
 *
 * @param thickness - pane thickness in meters, extruded symmetrically
 *
 * @returns merged arrays covering every rim cell
 *
 * @throws RangeError when the merged vertex count exceeds Uint16 indexing
 *
 * @example
 * ```ts
 * const arrays = rimMeshArrays({
 *   cells: rim,
 *   thickness: 0.012,
 * },);
 * ```
 */
export function rimMeshArrays(
  {
    cells,
    thickness,
  }: {
    readonly cells: readonly PaneCell[];
    readonly thickness: number;
  },
): RimMeshArrays {
  /**
   * Per-cell prisms in shard-local space, merged below.
   */
  const prisms = cells.map(function prismForCell(cell: PaneCell,): PrismMesh {
    return prismFromPolygon({
      polygon: cell,
      thickness,
    },);
  },);
  /**
   * Merged vertex count across every prism.
   */
  const vertexTotal = prisms.reduce(
    function sumVertices(
      sum: number,
      prism: (typeof prisms)[number],
    ): number {
      return sum + (prism.positions
        .length
        / XYZ);
    },
    0,
  );
  if (vertexTotal > MAX_UINT16)
    throw new RangeError(
      `rim mesh needs ${String(vertexTotal,)} vertices, above the Uint16 limit`,
    );
  /**
   * Merged index count across every prism.
   */
  const indexTotal = prisms.reduce(
    function sumIndices(
      sum: number,
      prism: (typeof prisms)[number],
    ): number {
      /**
       * Index count alias keeping the sum a single-line expression.
       */
      const { length, } = prism.indices;
      return sum + length;
    },
    0,
  );
  /**
   * Merged position buffer, filled prism by prism.
   */
  const positions = new Float32Array(vertexTotal * XYZ,);
  /**
   * Merged normal buffer parallel to {@link positions}.
   */
  const normals = new Float32Array(vertexTotal * XYZ,);
  /**
   * Merged index buffer with per-prism vertex offsets applied.
   */
  const indices = new Uint16Array(indexTotal,);
  /**
   * Write cursors advancing prism by prism; object-wrapped so the merge
   * loop mutates no root binding.
   */
  const cursor = {
    vertex: 0,
    index: 0,
  };
  for (const prism of prisms) {
    /**
     * Cell centroid this prism was recentered around, restored here.
     */
    const { pivot, } = prism;
    /**
     * Vertex count this prism contributes.
     */
    const count = prism.positions
      .length
      / XYZ;
    normals.set(
      prism.normals,
      cursor.vertex * XYZ,
    );
    for (let vertex = 0; vertex < count; vertex++) {
      /**
       * Source slot of this vertex inside the prism buffer.
       */
      const source = vertex * XYZ;
      positions.set(
        [
          nonNullishOrThrow(prism.positions[source],) + pivot.x,
          nonNullishOrThrow(prism.positions[source + 1],) + pivot.y,
          nonNullishOrThrow(prism.positions[source + 2],),
        ],
        (cursor.vertex + vertex) * XYZ,
      );
    }
    for (const [slot, index,] of prism.indices
      .entries())
      indices[cursor.index + slot] = index + cursor.vertex;
    cursor.vertex += count;
    cursor.index += prism.indices
      .length;
  }
  return {
    positions,
    normals,
    indices,
  };
}
