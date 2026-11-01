/**
 * BatchedMesh slot allocation for shard prisms.
 *
 * Wraps geometry upload and instance allocation with a sentinel result so
 * the spawn loop needs no uninitialized bindings and no try/catch of its
 * own when the pool is exhausted.
 */
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  type BatchedMesh,
  BufferAttribute,
  BufferGeometry,
} from 'three/webgpu';

import type { PrismMesh, } from './prism.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for shard allocation.
 */
const l = tagged({
  tag: 'shard-alloc',
  l: parentLogger,
},);

/**
 * Components per position or normal vector in the prism buffers.
 */
const XYZ = 3;

/**
 * Sentinel reported when the batched pool rejected the shard geometry,
 * usually because the vertex or instance budget ran out mid-burst.
 */
export const POOL_EXHAUSTED: unique symbol = Symbol(
  'batched pool rejected the shard geometry',
);

/**
 * Pooled ids of one allocated shard instance.
 */
export type ShardAllocation = {
  /**
   * Geometry slot in the batch.
   */
  readonly geometryId: number;
  /**
   * Instance slot in the batch.
   */
  readonly instanceId: number;
};

/**
 * Uploads one prism into the batch and allocates its instance.
 *
 * @param batched - batch receiving the shard
 *
 * @param prism - prism arrays to upload
 *
 * @mutates batched - `batched.addGeometry(geometry)` and `batched.addInstance(geometryId)` claim pool slots.
 *
 * @returns pooled ids, or {@link POOL_EXHAUSTED} when the pool rejected
 *   the geometry
 *
 * @example
 * ```ts
 * const allocation = allocateShard({ batched, prism },);
 * if (allocation !== POOL_EXHAUSTED)
 *   batched.setMatrixAt(allocation.instanceId, matrix,);
 * ```
 */
export function allocateShard(
  {
    batched,
    prism,
  }: {
    readonly batched: BatchedMesh;
    readonly prism: PrismMesh;
  },
): ShardAllocation | typeof POOL_EXHAUSTED {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: allocateShard.name,
    l,
  },);
  /**
   * Source geometry copied into the batch, then dropped.
   */
  const geometry = new BufferGeometry();
  geometry.setAttribute(
    'position',
    new BufferAttribute(
      prism.positions,
      XYZ,
    ),
  );
  geometry.setAttribute(
    'normal',
    new BufferAttribute(
      prism.normals,
      XYZ,
    ),
  );
  geometry.setIndex(new BufferAttribute(
    prism.indices,
    1,
  ),);
  try {
    /**
     * Geometry slot claimed in the batch.
     */
    const geometryId = batched.addGeometry(geometry,);
    /**
     * Instance slot claimed with the geometry.
     */
    const instanceId = batched.addInstance(geometryId,);
    geometry.dispose();
    return {
      geometryId,
      instanceId,
    };
  }
  catch (error) {
    innerL.warn(`debris pool exhausted, skipping shard: ${caughtValueText(error,)}`,);
    geometry.dispose();
    return POOL_EXHAUSTED;
  }
}
