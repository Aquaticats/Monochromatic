/**
 * Impact garnish: additive spark points and camera shake.
 *
 * The pulverized-center cells that are too small to be solid shards come
 * back as a glitter burst; combined with a few frames of camera shake
 * this covers the sensory gap where sound alone cannot carry the hit.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  AdditiveBlending,
  BufferAttribute,
  BufferGeometry,
  Points,
  PointsMaterial,
  type Scene,
  type Vector3,
} from 'three/webgpu';

import type { RandomSource, } from './fracture.ts';
import { GRAVITY, } from './physics.ts';

/**
 * Components per position, color, or velocity vector in the flat buffers.
 */
const XYZ = 3;

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the effects module.
 */
const l = tagged({
  tag: 'fx',
  l: parentLogger,
},);

/**
 * Spark pool and shake constants.
 */
export const FX_TUNING = {
  /**
   * Spark pool capacity across all bursts.
   */
  maxSparks: 720,
  /**
   * Sparks emitted per shatter burst.
   */
  burstSize: 110,
  /**
   * Sparks emitted per crack tick.
   */
  crackSize: 30,
  /**
   * Spark lifetime in seconds.
   */
  sparkLife: 0.8,
  /**
   * Gravity fraction sparks feel; glass dust drifts more than it drops.
   */
  sparkGravity: 0.55,
  /**
   * Initial shake amplitude per shatter, meters.
   */
  shakeAmplitude: 0.035,
  /**
   * Exponential shake decay rate per second.
   */
  shakeDecay: 9,
  /**
   * Depth sparks park at when dead, far below the corridor.
   */
  parkDepth: -1_000,
  /**
   * Upward bias subtracted less than the full range so bursts hang.
   */
  upwardBias: 0.6,
  /**
   * Slowest fraction of the burst speed a spark gets.
   */
  speedFloor: 0.25,
  /**
   * Random speed fraction on top of the floor.
   */
  speedSpread: 0.75,
  /**
   * Red channel floor for fresh sparks.
   */
  redFloor: 0.85,
  /**
   * Random red spread on top of the floor.
   */
  redSpread: 0.15,
  /**
   * Green channel for fresh sparks.
   */
  green: 0.95,
  /**
   * Shortest life fraction a spark draws.
   */
  lifeFloor: 0.5,
  /**
   * Red fade multiplier while dying.
   */
  fadeRed: 0.9,
  /**
   * Green fade multiplier while dying.
   */
  fadeGreen: 0.95,
  /**
   * Shake amplitude under which the shake snaps to zero, meters.
   */
  shakeFloor: 0.0004,
} as const;

/**
 * Effects system handle.
 */
export type FxSystem = {
  /**
   * Emits a spark burst at a world position.
   *
   * @param at - burst origin
   *
   * @param count - sparks to emit
   *
   * @param speed - burst speed scale in m/s
   */
  readonly burst: (input: Readonly<{
    at: Vector3;
    count: number;
    speed: number;
  }>,) => void;
  /**
   * Kicks the camera shake to at least the shatter amplitude.
   */
  readonly kickShake: () => void;
  /**
   * Advances sparks and shake.
   *
   * @param dt - timestep in seconds
   *
   * @returns current shake offset amplitude in meters
   */
  readonly update: (dt: number,) => number;
};

/**
 * Creates the spark pool and shake state.
 *
 * @param scene - scene receiving the points object
 *
 * @param random - uniform random source
 *
 * @mutates scene - `scene.add(points)` registers the spark pool in the caller's scene graph.
 *
 * @mutates random - spark emission draws advance the caller-supplied generator state.
 *
 * @returns effects system handle
 *
 * @example
 * ```ts
 * const fx = createFx({ scene, random: Math.random },);
 * ```
 */
export function createFx(
  {
    scene,
    random,
  }: {
    readonly scene: Scene;
    readonly random: RandomSource;
  },
): FxSystem {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: createFx.name,
    l,
  },);
  /**
   * Spark positions, xyz per spark; dead sparks park far underground.
   */
  const positions = new Float32Array(FX_TUNING.maxSparks * XYZ,).fill(FX_TUNING.parkDepth,);
  /**
   * Spark colors, rgb per spark, faded toward black as sparks die so
   * additive blending dissolves them without per-point alpha.
   */
  const colors = new Float32Array(FX_TUNING.maxSparks * XYZ,);
  /**
   * Spark velocities, xyz per spark, meters per second.
   */
  const velocities = new Float32Array(FX_TUNING.maxSparks * XYZ,);
  /**
   * Remaining life per spark in seconds; 0 or less is dead.
   */
  const lives = new Float32Array(FX_TUNING.maxSparks,);
  /**
   * Mutable pool state: the round-robin overwrite cursor and the decaying
   * camera shake amplitude.
   */
  const state = {
    /**
     * Next pool slot to overwrite, advancing round-robin.
     */
    cursor: 0,
    /**
     * Current shake amplitude in meters, decaying exponentially.
     */
    shake: 0,
  };
  /**
   * Geometry exposing position and color to the points material.
   */
  const geometry = new BufferGeometry();
  geometry.setAttribute(
    'position',
    new BufferAttribute(
      positions,
      XYZ,
    ),
  );
  geometry.setAttribute(
    'color',
    new BufferAttribute(
      colors,
      XYZ,
    ),
  );
  /**
   * Additive glitter material shared by the whole pool.
   */
  const material = new PointsMaterial({
    size: 0.035,
    vertexColors: true,
    transparent: true,
    depthWrite: false,
    blending: AdditiveBlending,
    sizeAttenuation: true,
  },);
  /**
   * Single points object owning every spark.
   */
  const points = new Points(
    geometry,
    material,
  );
  points.frustumCulled = false;
  scene.add(points,);
  innerL.info(`spark pool ready: ${String(FX_TUNING.maxSparks,)} slots`,);
  return {
    burst: function burst(input,): void {
      for (let emitted = 0; emitted < input.count; emitted++) {
        /**
         * Pool slot this spark overwrites.
         */
        const slot = state.cursor;
        state.cursor = (state.cursor + 1) % FX_TUNING.maxSparks;
        positions.set(
          [
            input.at
              .x,
            input.at
              .y,
            input.at
              .z,
          ],
          slot * XYZ,
        );
        /**
         * Random unit-ish direction, upward-biased so glitter hangs.
         */
        const dirX = (random() * 2) - 1;
        /**
         * Vertical direction component before the upward bias.
         */
        const dirY = (random() * 2) - FX_TUNING.upwardBias;
        /**
         * Depth direction component.
         */
        const dirZ = (random() * 2) - 1;
        /**
         * Direction length guard against a zero vector.
         */
        const length = Math.hypot(
          dirX,
          dirY,
          dirZ,
        ) || 1;
        /**
         * Spark speed within the burst cone.
         */
        const speed = input.speed * (FX_TUNING.speedFloor + (random()
          * FX_TUNING.speedSpread));
        velocities.set(
          [
            (dirX / length) * speed,
            (dirY / length) * speed,
            (dirZ / length) * speed,
          ],
          slot * XYZ,
        );
        colors.set(
          [
            FX_TUNING.redFloor + (random()
              * FX_TUNING.redSpread),
            FX_TUNING.green,
            1,
          ],
          slot * XYZ,
        );
        lives[slot] = FX_TUNING.sparkLife * (FX_TUNING.lifeFloor + (random()
          * (1 - FX_TUNING.lifeFloor)));
      }
      geometry.getAttribute('position',)
        .needsUpdate = true;
      geometry.getAttribute('color',)
        .needsUpdate = true;
    },
    kickShake: function kickShake(): void {
      state.shake = Math.max(
        state.shake,
        FX_TUNING.shakeAmplitude,
      );
    },
    update: function update(dt: number,): number {
      for (let slot = 0; slot < FX_TUNING.maxSparks; slot++) {
        /**
         * Life read once; typed-array reads narrow without assertions.
         */
        const life = lives[slot];
        if ((life === undefined) || (life <= 0))
          continue;
        /**
         * Life left after this step.
         */
        const remaining = life - dt;
        lives[slot] = remaining;
        if (remaining <= 0) {
          positions[(slot * XYZ) + 1] = FX_TUNING.parkDepth;
          continue;
        }
        /**
         * Vertical velocity after this step's gravity.
         */
        const fallSpeed = (velocities[(slot * XYZ) + 1] ?? 0)
          - (GRAVITY
          * FX_TUNING.sparkGravity
            * dt);
        velocities[(slot * XYZ) + 1] = fallSpeed;
        positions[slot * XYZ] = (positions[slot * XYZ] ?? 0)
          + ((velocities[slot * XYZ] ?? 0)
          * dt);
        positions[(slot * XYZ) + 1] = (positions[(slot * XYZ) + 1] ?? 0)
          + (fallSpeed
          * dt);
        positions[(slot * XYZ) + 2] = (positions[(slot * XYZ) + 2] ?? 0)
          + ((velocities[(slot * XYZ) + 2] ?? 0)
          * dt);
        /**
         * Life fraction driving the additive fade to black.
         */
        const fade = remaining / FX_TUNING.sparkLife;
        colors[slot * XYZ] = FX_TUNING.fadeRed * fade;
        colors[(slot * XYZ) + 1] = FX_TUNING.fadeGreen * fade;
        colors[(slot * XYZ) + 2] = fade;
      }
      geometry.getAttribute('position',)
        .needsUpdate = true;
      geometry.getAttribute('color',)
        .needsUpdate = true;
      state.shake *= Math.exp((-FX_TUNING.shakeDecay) * dt,);
      if (state.shake < FX_TUNING.shakeFloor)
        state.shake = 0;
      return state.shake;
    },
  };
}
