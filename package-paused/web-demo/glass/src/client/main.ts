/**
 * Client entry: wires renderer, world, panes, balls, debris, effects,
 * audio, and HUD into the walk-and-throw loop.
 *
 * The camera does the walking; the world stands still. Every moving thing
 * else reacts to two event streams: ball impacts (crack or collapse) and
 * pane shatter events (debris, sparks, sound, score).
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { Vector3, } from 'three/webgpu';

import { createAudio, } from './audio.ts';
import { createBalls, } from './ball.ts';
import { createCorridor, } from './corridor.ts';
import { createDebris, } from './debris.ts';
import {
  createFx,
  FX_TUNING,
} from './fx.ts';
import {
  createPanes,
  PANE_TUNING,
} from './pane.ts';
import { prewarmPipelines, } from './prewarm.ts';
import {
  bootstrapScene,
  WORLD_TUNING,
} from './scene.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the entry module.
 */
const l = tagged({
  tag: 'main',
  l: parentLogger,
},);

/**
 * Walk bob and loop constants.
 */
const LOOP_TUNING = {
  /**
   * Step frequency in Hz driving the head bob.
   */
  stepFrequency: 1.85,
  /**
   * Vertical bob amplitude in meters.
   */
  bobAmplitude: 0.018,
  /**
   * Lateral sway amplitude in meters.
   */
  swayAmplitude: 0.011,
  /**
   * Longest simulated timestep in seconds; tab-switch spikes clamp here.
   */
  maxTimestep: 0.05,
  /**
   * Fallback frame rate assumed on the very first frame, Hz.
   */
  fallbackFps: 60,
  /**
   * Milliseconds per second, for the animation-loop timestamp.
   */
  msPerSecond: 1_000,
  /**
   * How far ahead of the camera the slab centers sit, meters.
   */
  slabAhead: 40,
  /**
   * Normalized device z for unprojecting the pointer ray.
   */
  pointerRayDepth: 0.5,
} as const;

/**
 * Page canvas the renderer draws into.
 */
const stage = nonNullishOrThrow(
  document.querySelector<HTMLCanvasElement>('#stage',),
);
/**
 * HUD element showing the shattered pane count.
 */
const scoreValue = nonNullishOrThrow(document.querySelector<HTMLElement>('#score-value',),);
/**
 * HUD hint hidden after the first throw.
 */
const hint = nonNullishOrThrow(document.querySelector<HTMLElement>('#hint',),);
/**
 * HUD element naming the active backend.
 */
const backendLabel = nonNullishOrThrow(document.querySelector<HTMLElement>('#backend',),);
/**
 * Error overlay shown when the renderer cannot start.
 */
const errorOverlay = nonNullishOrThrow(
  document.querySelector<HTMLElement>('#error',),
);
/**
 * Error overlay message body.
 */
const errorCopy = nonNullishOrThrow(document.querySelector<HTMLElement>('#error-copy',),);

/**
 * Starts the demo: bootstraps the scene, builds every system, installs
 * input and resize handlers, and enters the animation loop.
 *
 * @example
 * ```ts
 * await startDemo();
 * ```
 */
async function startDemo(): Promise<void> {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: startDemo.name,
    l,
  },);
  /**
   * Renderer, scene, camera, and slabs from the bootstrap.
   */
  const kit = await bootstrapScene({ canvas: stage, },);
  backendLabel.textContent = kit.usingWebGpu
    ? 'WebGPU'
    : 'WebGL2 fallback';
  /**
   * Shared uniform random source for every visual system.
   */
  const {random} = Math;
  /**
   * Corridor furniture and its recycler.
   */
  const corridor = createCorridor({ scene: kit.scene, },);
  /**
   * Pane system owning the two-stage break.
   */
  const panes = createPanes({
    scene: kit.scene,
    random,
  },);
  /**
   * Batched debris pool.
   */
  const debris = createDebris({ scene: kit.scene, },);
  /**
   * Ball pool.
   */
  const balls = createBalls({ scene: kit.scene, },);
  /**
   * Sparks and camera shake.
   */
  const fx = createFx({
    scene: kit.scene,
    random,
  },);
  /**
   * Procedural sound.
   */
  const audio = createAudio();
  /**
   * Mutable run state: HUD counters and the walking clock.
   */
  const run: {
    /**
     * Shattered pane count shown in the HUD.
     */
    shattered: number;
    /**
     * Whether the player threw at least once; hides the hint.
     */
    thrown: boolean;
    /**
     * Walk phase in radians driving bob and sway.
     */
    phase: number;
    /**
     * Camera z excluding bob and shake; the authoritative walk position.
     */
    z: number;
    /**
     * Previous frame timestamp in seconds, for dt; absent on frame one.
     */
    lastTime?: number;
  } = {
    shattered: 0,
    thrown: false,
    phase: 0,
    z: 0,
  };
  prewarmPipelines({
    debris,
    fx,
    random,
  },);
  //region Input: pointer throws, first gesture unlocks audio
  stage.addEventListener(
    'pointerdown',
    function onPointerDown(event: PointerEvent,): void {
      audio.unlock();
      if (!run.thrown) {
        run.thrown = true;
        hint.classList
          .add('hidden',);
      }
      /**
       * Canvas-relative pointer in normalized device coordinates.
       */
      const bounds = stage.getBoundingClientRect();
      /**
       * Throw direction through the click point on the near plane.
       */
      const direction = new Vector3(
        (((event.clientX - bounds.left) / bounds.width) * 2) - 1,
        ((-((event.clientY - bounds.top) / bounds.height)) * 2) + 1,
        LOOP_TUNING.pointerRayDepth,
      )
        .unproject(kit.camera,)
        .sub(kit.camera
          .position,)
        .normalize();
      balls.throwBall({
        camera: kit.camera,
        direction,
      },);
      audio.playThrow();
    },
    { passive: true, },
  );
  //endregion
  //region Resize
  /**
   * Applies the current viewport size and pixel ratio.
   */
  function applySize(): void {
    /**
     * Viewport size with a lower bound so the renderer never gets 0.
     */
    const width = Math.max(
      1,
      window.innerWidth,
    );
    /**
     * Viewport height with the same lower bound.
     */
    const height = Math.max(
      1,
      window.innerHeight,
    );
    kit.camera
      .aspect = width / height;
    kit.camera
      .updateProjectionMatrix();
    kit.renderer
      .setPixelRatio(
      Math.min(
        window.devicePixelRatio || 1,
        WORLD_TUNING.maxPixelRatio,
      ),
    );
    kit.renderer
      .setSize(
        width,
        height,
        false,
      );
  }
  applySize();
  window.addEventListener(
    'resize',
    applySize,
  );
  //endregion
  void kit.renderer
    .setAnimationLoop(function frame(timeMs: number,): void {
    /**
     * Frame time in seconds.
     */
    const now = timeMs / LOOP_TUNING.msPerSecond;
    /**
     * Clamped timestep so tab switches do not teleport physics.
     */
    const dt = Math.min(
      LOOP_TUNING.maxTimestep,
      run.lastTime === undefined
        ? 1 / LOOP_TUNING.fallbackFps
        : now - run.lastTime,
    );
    run.lastTime = now;
    //region Walk: constant forward speed plus bob, sway, and shake
    run.z -= WORLD_TUNING.walkSpeed * dt;
    run.phase += dt * Math.PI
      * 2
      * LOOP_TUNING.stepFrequency;
    /**
     * Camera shake amplitude from the effects system this frame.
     */
    const shake = fx.update(dt,);
    kit.camera
      .position
      .set(
      (Math.sin(run.phase / 2,) * LOOP_TUNING.swayAmplitude)
        + ((random() - (1
          / 2)) * 2
          * shake),
      WORLD_TUNING.eyeHeight
        + (Math.abs(Math.sin(run.phase,),)
        * LOOP_TUNING.bobAmplitude)
        + ((random() - (1
          / 2))
        * 2
          * shake),
      run.z,
    );
    kit.floor
      .position
      .z = run.z - LOOP_TUNING.slabAhead;
    kit.ceiling
      .position
      .z = run.z - LOOP_TUNING.slabAhead;
    corridor.recycle(run.z,);
    //endregion
    //region Systems: balls strike, panes collapse, debris flies
    /**
     * Ball impacts this frame.
     */
    const impacts = balls.update({
      dt,
      now,
      panes,
      cameraZ: run.z,
    },);
    for (const impact of impacts)
      if (impact.result === 'cracked')
        audio.playCrack();
    /**
     * Staged shatter events this frame: instant hole bursts from
     * strikes, plus rim collapses from holds, hits, and body smashes.
     */
    const shatters = panes.update({
      cameraZ: run.z,
      now,
    },);
    for (const event of shatters) {
      debris.spawnShards({
        cells: event.cells,
        thickness: PANE_TUNING.thickness,
        paneMatrix: event.paneMatrix,
        impactWorld: event.impactWorld,
        ballVelocity: event.ballVelocity,
        random,
      },);
      if (event.stage === 'hole') {
        // The blast at the impact is the shower moment: big spark
        // burst, camera shock, and the crash sound, all at strike time.
        fx.burst({
          at: event.impactWorld,
          count: FX_TUNING.burstSize,
          speed: 3.2,
        },);
        fx.kickShake();
        audio.playShatter();
      }
      else {
        fx.burst({
          at: event.impactWorld,
          count: FX_TUNING.crackSize,
          speed: 1.6,
        },);
        audio.playShatter();
        run.shattered++;
        scoreValue.textContent = String(run.shattered,);
      }
    }
    /**
     * Shard floor bounces this frame, for glass ticks.
     */
    const bounces = debris.update({
      dt,
      cameraZ: run.z,
    },);
    if (bounces > 0)
      audio.playTick();
    //endregion
    kit.renderer
      .render(
        kit.scene,
        kit.camera,
      );
  },);
  //region Probe: read-only state hook for automated verification
  Reflect.set(
    globalThis,
    'glassDemoProbe',
    function glassDemoProbe(): Record<string, unknown> {
      return {
        backend: kit.usingWebGpu ? 'webgpu' : 'webgl2',
        shattered: run.shattered,
        walkZ: run.z,
        collidablePanes: panes.collidables()
          .length,
        crackedPanes: panes.crackedCount(),
      };
    },
  );
  //endregion
  innerL.info('demo running',);
}

try {
  await startDemo();
}
catch (error) {
  l.error(`demo failed to start: ${caughtValueText(error,)}`,);
  errorCopy.textContent = caughtValueText(error,);
  errorOverlay.hidden = false;
  throw error;
}
