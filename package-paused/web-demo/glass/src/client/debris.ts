/**
 * Pooled glass debris on one BatchedMesh.
 *
 * Every shard from every shattered pane lives in a single draw call with
 * one shared material: the alternative (a mesh and material per shard)
 * stalls exactly at the impact frame while dozens of pipelines compile.
 * Shard motion integrates on the CPU through the pure physics module and
 * lands in per-instance matrices.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  BatchedMesh,
  Matrix4,
  MeshPhysicalMaterial,
  Quaternion,
  type Scene,
  Vector3,
} from 'three/webgpu';

import type {
  DebrisSystem,
  ShardSlot,
  SpawnShardsInput,
} from './debris-model.ts';
import {
  type PaneCell,
  polygonArea,
} from './fracture.ts';
import { stepShardBody, } from './physics.ts';
import { prismFromPolygon, } from './prism.ts';
import {
  allocateShard,
  POOL_EXHAUSTED,
} from './shard-alloc.ts';
import {
  DEBRIS_TUNING,
  launchShardBody,
} from './shard-launch.ts';

export {
  type DebrisSystem,
  type SpawnShardsInput,
} from './debris-model.ts';
export { DEBRIS_TUNING, } from './shard-launch.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the debris system.
 */
const l = tagged({
  tag: 'debris',
  l: parentLogger,
},);

/**
 * Scratch matrix reused across matrix writes to avoid per-frame allocation.
 */
const SCRATCH_MATRIX = new Matrix4();

/**
 * Scratch position for matrix composition and spawn transforms.
 */
const SCRATCH_POSITION = new Vector3();
/**
 * Scratch for the shard scale during fade-out.
 */
const SCRATCH_SCALE = new Vector3();
/**
 * Scratch for the per-frame incremental spin rotation.
 */
const SCRATCH_SPIN = new Quaternion();
/**
 * Scratch axis for building the incremental spin rotation.
 */
const SCRATCH_AXIS = new Vector3();

/**
 * Creates the batched debris pool.
 *
 * @param scene - scene receiving the batched mesh
 *
 * @mutates scene - `scene.add(batched)` registers the shard pool mesh in the caller's scene graph.
 *
 * @returns debris system handle
 *
 * @example
 * ```ts
 * const debris = createDebris({ scene },);
 * ```
 */
export function createDebris(
  { scene, }: Readonly<{ scene: Scene; }>,
): DebrisSystem {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: createDebris.name,
    l,
  },);
  /**
   * Shared shard material: reflective transparent glass without
   * transmission, which small fast shards do not need and which would
   * multiply the transmission pass cost.
   */
  const material = new MeshPhysicalMaterial({
    color: '#c3dde9',
    metalness: 0,
    roughness: 0.06,
    transparent: true,
    opacity: 0.32,
    envMapIntensity: 2.6,
  },);
  /**
   * Single batched mesh owning every shard instance.
   */
  const batched = new BatchedMesh(
    DEBRIS_TUNING.maxInstances,
    DEBRIS_TUNING.maxVertices,
    DEBRIS_TUNING.maxIndices,
    material,
  );
  batched.frustumCulled = false;
  scene.add(batched,);
  /**
   * Live shard slots in spawn order, oldest first.
   */
  const slots: ShardSlot[] = [];
  /**
   * Writes one shard's matrix from its body, orientation, and fade scale.
   *
   * @param slot - shard to write
   *
   * @mutates slot - `SCRATCH_MATRIX.compose(...)` reads slot.orientation through a three.js method the analyzer cannot inspect.
   */
  function writeMatrix(slot: ShardSlot,): void {
    /**
     * Fade shrink factor, 1 while alive and easing to 0 while fading.
     */
    const shrink = Number.isNaN(slot.fadingFor,)
      ? 1
      : Math.max(
        0,
        1 - (slot.fadingFor
          / DEBRIS_TUNING.fadeSeconds),
      );
    /**
     * Body alias keeping the position write on one line.
     */
    const { body, } = slot;
    SCRATCH_POSITION.set(
      body.px,
      body.py,
      body.pz,
    );
    SCRATCH_SCALE.setScalar(shrink,);
    SCRATCH_MATRIX.compose(
      SCRATCH_POSITION,
      slot.orientation,
      SCRATCH_SCALE,
    );
    batched.setMatrixAt(
      slot.instanceId,
      SCRATCH_MATRIX,
    );
  }
  /**
   * Releases a shard's pooled ids.
   *
   * @param slot - shard to release
   */
  function release(slot: ShardSlot,): void {
    batched.deleteInstance(slot.instanceId,);
    batched.deleteGeometry(slot.geometryId,);
  }
  /**
   * Begins the fade-out of the oldest shards until the pool drops under
   * the soft cap, so bursts never exhaust the buffers mid-shatter.
   *
   * @param needed - instance count about to be added
   */
  function makeRoom(needed: number,): void {
    /**
     * Oldest-first candidates not already fading.
     */
    const solid = slots.filter(function notFading(slot: ShardSlot,): boolean {
      return Number.isNaN(slot.fadingFor,);
    },);
    /**
     * How many shards must start fading now.
     */
    const excess = (slots.length + needed) - DEBRIS_TUNING.softCap;
    for (
      const slot of solid.slice(
        0,
        Math.max(
          0,
          excess,
        ),
      )
    )
      slot.fadingFor = 0;
  }
  /**
   * {@inheritDoc DebrisSystem.spawnShards}
   *
   * @param input - one shatter's worth of spawn parameters
   *
   * @mutates input - `SCRATCH_POSITION.copy(input.impactWorld)`, `SCRATCH_MATRIX.copy(input.paneMatrix)`, `new Quaternion().setFromRotationMatrix(input.paneMatrix)`, `pivotWorld.applyMatrix4(input.paneMatrix)`, and `launchShardBody` are three.js and launch calls the analyzer cannot inspect; they only read the inputs, and `input.random()` advances caller generator state.
   */
  function spawnShards(input: SpawnShardsInput,): void {
      /**
       * Cells big enough to be solid shards; the rest is spark dust.
       */
      const solidCells = input.cells
        .filter(function bigEnough(cell: PaneCell,): boolean {
          return polygonArea(cell,) >= DEBRIS_TUNING.minShardArea;
        },);
      makeRoom(solidCells.length,);
      /**
       * Impact point in pane-local space, for per-shard distance falloff.
       */
      const impactLocal = SCRATCH_POSITION
        .copy(input.impactWorld,)
        .applyMatrix4(SCRATCH_MATRIX.copy(input.paneMatrix,)
          .invert(),)
        .clone();
      /**
       * Pane orientation for rotating shard frames into the world.
       */
      const paneRotation = new Quaternion().setFromRotationMatrix(input.paneMatrix,);
      for (const cell of solidCells) {
        /**
         * Prism arrays for this cell.
         */
        const prism = prismFromPolygon({
          polygon: cell,
          thickness: input.thickness,
        },);
        /**
         * Pooled ids for this shard, or the exhaustion sentinel.
         */
        const allocation = allocateShard({
          batched,
          prism,
        },);
        if (allocation === POOL_EXHAUSTED)
          continue;
        /**
         * Shard pivot in world space.
         */
        const pivotWorld = new Vector3(
          prism.pivot
            .x,
          prism.pivot
            .y,
          0,
        );
        pivotWorld.applyMatrix4(input.paneMatrix,);
        /**
         * Pane-local distance from impact to this shard, driving both
         * momentum transfer and the radial burst.
         */
        const impactDistance = Math.hypot(
          prism.pivot
            .x
            - impactLocal.x,
          prism.pivot
            .y
            - impactLocal.y,
        );
        /**
         * Shard characteristic size for spin scaling.
         */
        const size = Math.sqrt(polygonArea(cell,),);
        /**
         * Fresh slot registered with the pool; the pure launch math owns
         * every velocity and spin decision.
         */
        const slot: ShardSlot = {
          instanceId: allocation.instanceId,
          geometryId: allocation.geometryId,
          body: launchShardBody({
            pivot: pivotWorld,
            impact: input.impactWorld,
            ballVelocity: input.ballVelocity,
            impactDistance,
            thickness: input.thickness,
            size,
            random: input.random,
          },),
          orientation: paneRotation.clone(),
          age: 0,
          settledFor: Number.NaN,
          flatYaw: input.random() * Math.PI
            * 2,
          flatHeight: (input.thickness / 2) + DEBRIS_TUNING.flatLift,
          fadingFor: Number.NaN,
        };
        slots.push(slot,);
        writeMatrix(slot,);
      }
      innerL.debug(`spawned shards, live total ${String(slots.length,)}`,);
  }
  return {
    spawnShards,
    update: function update(
      input: Readonly<{
        dt: number;
        cameraZ: number;
      }>,
    ): number {
      /**
       * Timestep alias keeping the physics lines short.
       */
      const { dt, } = input;
      /**
       * Floor bounces observed this frame, returned for audio ticks.
       * Object-wrapped so the counter mutates without a root `let`.
       */
      const tally = { bounces: 0, };
      /**
       * Slots finished fading this frame, released after iteration.
       */
      const finished: ShardSlot[] = [];
      for (const slot of slots) {
        /**
         * Body alias keeping the physics reads on single lines.
         */
        const { body, } = slot;
        slot.age += dt;
        // Shards behind the camera are invisible forever on a forward
        // walk: release them instantly, no fade needed. Everything the
        // player can still see stays put; real debris does not evaporate.
        if (
          body.pz > (input.cameraZ
            + DEBRIS_TUNING.cullBehind)
        ) {
          finished.push(slot,);
          continue;
        }
        if (!Number.isNaN(slot.fadingFor,)) {
          slot.fadingFor += dt;
          if (slot.fadingFor >= DEBRIS_TUNING.fadeSeconds) {
            finished.push(slot,);
            continue;
          }
        }
        else if (
          (slot.age >= DEBRIS_TUNING.maxLifeSeconds)
          || (slot.settledFor >= DEBRIS_TUNING.persistSeconds)
        )
          slot.fadingFor = 0;
        if (body.settled) {
          slot.settledFor = Number.isNaN(slot.settledFor,)
            ? 0
            : slot.settledFor + dt;
          if (slot.flatTarget !== undefined) {
            slot.orientation
              .slerp(
                slot.flatTarget,
                Math.min(
                  1,
                  dt / DEBRIS_TUNING.flattenSeconds,
                ),
              );
          }
          writeMatrix(slot,);
          continue;
        }
        /**
         * Whether this step touched the floor.
         */
        const contact = stepShardBody({
          body,
          dt,
          floorY: 0,
        },);
        if (contact)
          tally.bounces++;
        if (body.settled) {
          // Lie flat: align the plate normal upward with a random yaw so
          // resting glass tiles the floor the way real debris does.
          SCRATCH_SPIN.setFromAxisAngle(
            SCRATCH_AXIS.set(
              1,
              0,
              0,
            ),
            (-Math.PI) / 2,
          );
          slot.flatTarget = new Quaternion()
            .setFromAxisAngle(
              SCRATCH_AXIS.set(
                0,
                1,
                0,
              ),
              slot.flatYaw,
            )
            .multiply(SCRATCH_SPIN,);
          body.py = slot.flatHeight;
        }
        /**
         * Spin magnitude this frame; skipped when negligible.
         */
        const spinSpeed = Math.hypot(
          body.wx,
          body.wy,
          body.wz,
        );
        if (spinSpeed > DEBRIS_TUNING.spinEpsilon) {
          SCRATCH_AXIS.set(
            body.wx,
            body.wy,
            body.wz,
          );
          SCRATCH_AXIS.divideScalar(spinSpeed,);
          SCRATCH_SPIN.setFromAxisAngle(
            SCRATCH_AXIS,
            spinSpeed * dt,
          );
          slot.orientation
            .premultiply(SCRATCH_SPIN,);
        }
        writeMatrix(slot,);
      }
      if (finished.length > 0) {
        for (const slot of finished) {
          release(slot,);
          slots.splice(
            slots.indexOf(slot,),
            1,
          );
        }
        if (slots.length === 0)
          batched.optimize();
      }
      // An empty BatchedMesh still issues a draw and warns about missing
      // vertex attributes; hide it whenever the pool drains.
      batched.visible = slots.length > 0;
      return tally.bounces;
    },
  };
}
