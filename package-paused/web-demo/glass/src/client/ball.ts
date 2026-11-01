/**
 * Thrown balls: pooled meshes, ballistic flight, floor bounces, and
 * strike resolution through the swept-segment collider.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  Mesh,
  MeshStandardMaterial,
  type PerspectiveCamera,
  type Scene,
  SphereGeometry,
  Vector3,
} from 'three/webgpu';

import {
  type BallImpact,
  SWEEP_MISS,
  sweepBallAgainstPanes,
} from './ball-sweep.ts';
import { BALL_TUNING, } from './ball-tuning.ts';
import type { PaneSystem, } from './pane-model.ts';
import {
  type BallBody,
  stepBallFlight,
} from './physics.ts';
import { WORLD_TUNING, } from './scene.ts';

export { type BallImpact, } from './ball-sweep.ts';
export { BALL_TUNING, } from './ball-tuning.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the ball system.
 */
const l = tagged({
  tag: 'ball',
  l: parentLogger,
},);

/**
 * One live ball.
 */
type BallSlot = {
  /**
   * Pooled mesh shown while the ball lives.
   */
  readonly mesh: Mesh;
  /**
   * Simulated point mass.
   */
  readonly body: BallBody;
  /**
   * Seconds since the throw.
   */
  age: number;
};

/**
 * One throw's parameters.
 */
export type ThrowInput = {
  /**
   * Walking camera the throw starts from.
   */
  readonly camera: PerspectiveCamera;
  /**
   * Normalized world-space throw direction.
   */
  readonly direction: Vector3;
};

/**
 * One ball-system frame's parameters.
 */
export type BallUpdateInput = {
  /**
   * Timestep in seconds.
   */
  readonly dt: number;
  /**
   * Wall-clock seconds.
   */
  readonly now: number;
  /**
   * Pane system the balls collide with.
   */
  readonly panes: PaneSystem;
  /**
   * Camera z for retirement distance checks.
   */
  readonly cameraZ: number;
};

/**
 * Ball system handle.
 */
export type BallSystem = {
  /**
   * Throws a ball from the camera toward the pointer ray.
   */
  readonly throwBall: (input: ThrowInput,) => void;
  /**
   * Advances all balls one frame and resolves pane strikes.
   *
   * @returns impacts this frame
   */
  readonly update: (input: BallUpdateInput,) => BallImpact[];
};

/**
 * Scratch segment start for the per-frame sweep.
 */
const SCRATCH_FROM = new Vector3();
/**
 * Scratch segment end for the per-frame sweep.
 */
const SCRATCH_TO = new Vector3();

/**
 * Creates the ball pool.
 *
 * @param scene - scene receiving ball meshes
 *
 * @mutates scene - `scene.add(mesh)` on throw and `scene.remove(slot.mesh)` on retire edit the caller's scene graph.
 *
 * @returns ball system handle
 *
 * @example
 * ```ts
 * const balls = createBalls({ scene },);
 * ```
 */
export function createBalls(
  { scene, }: Readonly<{ scene: Scene; }>,
): BallSystem {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: createBalls.name,
    l,
  },);
  /**
   * Shared ball geometry.
   */
  const geometry = new SphereGeometry(
    BALL_TUNING.radius,
    BALL_TUNING.sphereSegments
      .width,
    BALL_TUNING.sphereSegments
      .height,
  );
  /**
   * Shared ball material: bright with a touch of emissive so the ball
   * reads against the dark corridor.
   */
  const material = new MeshStandardMaterial({
    color: '#f4fbff',
    emissive: '#9fd8ff',
    emissiveIntensity: 0.55,
    metalness: 0.05,
    roughness: 0.3,
  },);
  /**
   * Live balls, oldest first.
   */
  const balls: BallSlot[] = [];
  /**
   * Removes a ball from the world and the pool.
   *
   * @param slot - ball to retire
   *
   * @mutates slot - `scene.remove(slot.mesh)` detaches the pooled mesh from the scene graph.
   */
  function retire(slot: BallSlot,): void {
    scene.remove(slot.mesh,);
    balls.splice(
      balls.indexOf(slot,),
      1,
    );
  }
  /**
   * {@inheritDoc BallSystem.throwBall}
   *
   * @param input - camera and normalized throw direction
   *
   * @mutates input - `eye.clone` copies the camera position and `start.addScaledVector` reads `input.direction`; both are three.js methods the analyzer cannot inspect and only read their inputs.
   */
  function throwBall(input: ThrowInput,): void {
      /**
       * Oldest ball retired when the pool is full.
       */
      const [oldest,] = balls;
      if ((balls.length >= BALL_TUNING.poolSize) && (oldest !== undefined))
        retire(oldest,);
      /**
       * Fresh ball mesh sharing geometry and material.
       */
      const mesh = new Mesh(
        geometry,
        material,
      );
      /**
       * Eye position alias; its clone becomes the launch point.
       */
      const eye = input.camera
        .position;
      /**
       * Launch point: slightly ahead of and below the eye, like a hand.
       */
      const start = eye.clone();
      start.addScaledVector(
        input.direction,
        BALL_TUNING.hand
          .forward,
      );
      start.add(new Vector3(
        BALL_TUNING.hand
          .right,
        -BALL_TUNING.hand
          .down,
        0,
      ),);
      mesh.position
        .copy(start,);
      scene.add(mesh,);
      balls.push({
        mesh,
        body: {
          px: start.x,
          py: start.y,
          pz: start.z,
          vx: input.direction
            .x
            * BALL_TUNING.throwSpeed,
          vy: (input.direction
            .y
            * BALL_TUNING.throwSpeed) + BALL_TUNING.throwLift,
          vz: (input.direction
            .z
            * BALL_TUNING.throwSpeed) - WORLD_TUNING.walkSpeed,
        },
        age: 0,
      },);
      innerL.debug(`threw ball, live ${String(balls.length,)}`,);
  }
  /**
   * {@inheritDoc BallSystem.update}
   *
   * @param input - timestep, clock, pane system, and camera z
   *
   * @returns impacts resolved this frame
   */
  function update(input: BallUpdateInput,): BallImpact[] {
      /**
       * Impacts collected this frame.
       */
      const impacts: BallImpact[] = [];
      /**
       * Panes a ball can still hit this frame.
       */
      const targets = input.panes
        .collidables();
      /**
       * Balls to retire after the loop; splicing mid-iteration skips
       * elements.
       */
      const expired: BallSlot[] = [];
      for (const slot of balls) {
        slot.age += input.dt;
        SCRATCH_FROM.set(
          slot.body
            .px,
          slot.body
            .py,
          slot.body
            .pz,
        );
        stepBallFlight({
          body: slot.body,
          dt: input.dt,
        },);
        SCRATCH_TO.set(
          slot.body
            .px,
          slot.body
            .py,
          slot.body
            .pz,
        );
        /**
         * First pane strike along this ball's frame segment, when any.
         */
        const impact = sweepBallAgainstPanes({
          from: SCRATCH_FROM,
          to: SCRATCH_TO,
          body: slot.body,
          targets,
          strike: input.panes
            .strike,
          now: input.now,
        },);
        if (impact !== SWEEP_MISS)
          impacts.push(impact,);
        //region Floor bounce and retirement
        if ((slot.body
          .py
          < BALL_TUNING.radius) && (slot.body
            .vy
            < 0)) {
          slot.body
            .py = BALL_TUNING.radius;
          slot.body
            .vy = (-slot.body
              .vy) * BALL_TUNING.floorRestitution;
          slot.body
            .vx *= BALL_TUNING.floorDrag;
          slot.body
            .vz *= BALL_TUNING.floorDrag;
        }
        if (
          (slot.age > BALL_TUNING.maxLifeSeconds)
          || (slot.body
            .pz
            < (input.cameraZ
            - BALL_TUNING.cullDistance))
            || (slot.body
              .pz
              > (input.cameraZ
              + 10))
        ) {
          expired.push(slot,);
          continue;
        }
        slot.mesh
          .position
          .set(
            slot.body
              .px,
            slot.body
              .py,
            slot.body
              .pz,
          );
        //endregion
      }
      for (const slot of expired)
        retire(slot,);
      return impacts;
  }
  return {
    throwBall,
    update,
  };
}
