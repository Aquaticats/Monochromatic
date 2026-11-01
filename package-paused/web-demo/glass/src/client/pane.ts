/**
 * Glass panes: the instant-hole break and the spawn/recycle loop.
 *
 * A strike blasts the cells around the impact out immediately and leaves
 * a cracked rim holding around a real hole; the rim collapses on its
 * hold timer, on the next hit, or under a body smash. The pane system
 * owns both waves and emits staged shatter events; debris, sparks,
 * audio, and score react in the main loop.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  type Scene,
  Vector3,
} from 'three/webgpu';

import { dropOverlay, } from './crack-overlay.ts';
import {
  fractureCells,
  type PaneCell,
  type PanePoint,
  type RandomSource,
} from './fracture.ts';
import {
  assemblePane,
  createPaneMaterials,
  paneFrame,
} from './pane-assembly.ts';
import {
  type Pane,
  PANE_TUNING,
  type PaneState,
  type PaneSystem,
  type ShatterEvent,
} from './pane-model.ts';
import { dropPaneRim, } from './pane-rim.ts';
import { strikePane, } from './pane-strike.ts';
import { WORLD_TUNING, } from './scene.ts';

export {
  type Pane,
  PANE_TUNING,
  type PaneState,
  type PaneSystem,
  type ShatterEvent,
} from './pane-model.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the pane system.
 */
const l = tagged({
  tag: 'pane',
  l: parentLogger,
},);

/**
 * Collapses a pane's remaining glass into a shatter event.
 *
 * @param pane - pane to collapse
 *
 * @param cells - cells becoming shards: the rim, or a fresh full fracture
 *
 * @param impactLocal - impact point in pane-local meters
 *
 * @param ballVelocity - driving velocity in world m/s
 *
 * @mutates pane - `pane.group.remove(pane.sheet)` drops the sheet; `dropOverlay` and `dropPaneRim` dispose and unmount the overlay and rim, each documenting its own edits; the state latch flips to shattered; and `paneFrame` runs `pane.glass.matrixWorld.decompose(...)`, which only reads the matrix.
 *
 * @mutates ballVelocity - `ballVelocity.clone()` is a three.js method the analyzer cannot inspect; it only reads components.
 *
 * @returns shatter event for the main loop
 */
function collapse(
  {
    pane,
    cells,
    impactLocal,
    ballVelocity,
  }: Readonly<{
    pane: Pane;
    cells: readonly PaneCell[];
    impactLocal: PanePoint;
    ballVelocity: Vector3;
  }>,
): ShatterEvent {
  pane.state = 'shattered';
  dropOverlay(pane,);
  dropPaneRim(pane,);
  pane.group
    .remove(pane.sheet,);
  /**
   * Glass world transform with the unit-box scale stripped: the glass
   * mesh is a scaled unit box, so its raw matrixWorld would double-apply
   * pane dimensions to meter-space fracture data.
   */
  const paneMatrix = paneFrame(pane,);
  /**
   * Impact point lifted into world space through the meters-space frame.
   */
  const impactWorld = new Vector3(
    impactLocal.x,
    impactLocal.y,
    0,
  )
    .applyMatrix4(paneMatrix,);
  return {
    stage: 'collapse',
    cells,
    paneMatrix,
    impactWorld,
    ballVelocity: ballVelocity.clone(),
  };
}

/**
 * Creates the pane system.
 *
 * @param scene - scene receiving pane assemblies
 *
 * @param random - uniform random source for layout and fracture
 *
 * @mutates scene - `assemblePane` runs `scene.add(group)` and the recycler runs `scene.remove(pane.group)` on the caller's scene graph.
 *
 * @mutates random - layout and fracture draws advance the caller-supplied generator state.
 *
 * @returns pane system handle
 *
 * @example
 * ```ts
 * const panes = createPanes({ scene, random: Math.random },);
 * ```
 */
export function createPanes(
  {
    scene,
    random,
  }: {
    readonly scene: Scene;
    readonly random: RandomSource;
  },
): PaneSystem {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: createPanes.name,
    l,
  },);
  /**
   * Shared materials every pane assembly reuses.
   */
  const materials = createPaneMaterials();
  /**
   * All live panes, any state.
   */
  const panes: Pane[] = [];
  /**
   * Instant strike bursts waiting for the next update to hand them to
   * the main loop alongside the collapse events.
   */
  const pending: ShatterEvent[] = [];
  /**
   * Spawn frontiers, marching toward -infinity as the camera advances.
   */
  const frontier = {
    /**
     * Next spawn z for gate frames.
     */
    gateZ: -12,
    /**
     * Next spawn z for wall panes.
     */
    wallZ: -8,
  };
  innerL.info('pane system ready',);
  return {
    collidables: function collidables(): readonly Pane[] {
      return panes.filter(function canBeHit(pane: Pane,): boolean {
        return pane.state !== 'shattered';
      },);
    },
    crackedCount: function crackedCount(): number {
      return panes.filter(function isCracked(pane: Pane,): boolean {
        return pane.state === 'cracked';
      },)
        .length;
    },
    /**
     * {@inheritDoc PaneSystem.strike}
     *
     * @mutates input - `strikePane` advances `input.pane` and clones `input.ballVelocity` through the three.js methods it documents; instant bursts queue for the next update.
     */
    strike: function strike(input,): PaneState {
      /**
       * Stage entered plus the instant burst, when glass flew.
       */
      const outcome = strikePane({
        pane: input.pane,
        impactLocal: input.impactLocal,
        ballVelocity: input.ballVelocity,
        now: input.now,
        random,
      },);
      if (outcome.burst !== undefined)
        pending.push(outcome.burst,);
      return outcome.state;
    },
    update: function update(input,): ShatterEvent[] {
      /**
       * Shatter events emitted this frame, starting with the instant
       * bursts queued by strikes since the last update.
       */
      const events: ShatterEvent[] = pending.splice(0,);
      //region Collapse expired rim holds
      for (const pane of panes)
        if ((pane.state === 'cracked') && (input.now >= pane.holdUntil))
          events.push(collapse({
            pane,
            cells: pane.rimCells ?? [],
            impactLocal: pane.impactLocal ?? {
              x: 0,
              y: 0,
            },
            ballVelocity: pane.impactVelocity ?? new Vector3(
              0,
              0,
              -PANE_TUNING.fallbackImpactSpeed,
            ),
          },),);
      //endregion
      //region Body smash: walking into unbroken gate glass shatters it
      for (const pane of panes)
        if (
          (pane.kind === 'gate')
          && (pane.state !== 'shattered')
            && (Math.abs(pane.group
              .position
              .z
              - input.cameraZ,)
            < PANE_TUNING.bodySmashDistance)
        ) {
          /**
           * Impact point of a body smash: chest height at the walk line.
           */
          const smashImpact = {
            x: -pane.group
              .position
              .x,
            y: 0,
          };
          events.push(collapse({
            pane,
            cells: pane.state === 'intact'
              ? fractureCells({
                halfWidth: pane.halfWidth,
                halfHeight: pane.halfHeight,
                impact: smashImpact,
                random,
              },)
              : pane.rimCells ?? [],
            impactLocal: smashImpact,
            ballVelocity: new Vector3(
              0,
              0,
              -PANE_TUNING.bodySmashSpeed,
            ),
          },),);
        }
      //endregion
      //region Recycle panes behind the camera
      /**
       * Panes fallen behind the camera; spliced after the scan so the
       * loop never mutates the list it iterates.
       */
      const stale = panes.filter(function isBehind(pane: Pane,): boolean {
        return pane.group
          .position
          .z
          > (input.cameraZ
          + PANE_TUNING.recycleBehind);
      },);
      for (const pane of stale) {
        dropOverlay(pane,);
        dropPaneRim(pane,);
        scene.remove(pane.group,);
        panes.splice(
          panes.indexOf(pane,),
          1,
        );
      }
      //endregion
      //region Spawn gates and wall panes ahead
      while (frontier.gateZ > (input.cameraZ - PANE_TUNING.spawnAhead)) {
        assemblePane({
          scene,
          materials,
          registry: panes,
          spec: {
            kind: 'gate',
            halfWidth: PANE_TUNING.gate
              .halfWidthBase
              + (random()
                * PANE_TUNING.gate
                .halfWidthSpread),
            halfHeight: PANE_TUNING.gate
              .halfHeight,
            centerY: PANE_TUNING.gate
              .centerY,
            x: (random() - (1
              / 2))
              * 2
              * PANE_TUNING.gate
              .offsetSpread,
            z: frontier.gateZ,
            rotationY: 0,
          },
        },);
        frontier.gateZ -= PANE_TUNING.gateSpacing;
      }
      while (frontier.wallZ > (input.cameraZ - PANE_TUNING.spawnAhead)) {
        for (
          const side of [
            -1,
            1,
          ]
        )
          assemblePane({
            scene,
            materials,
            registry: panes,
            spec: {
              kind: 'wall',
              halfWidth: PANE_TUNING.wall
                .halfWidth,
              halfHeight: PANE_TUNING.wall
                .halfHeight,
              centerY: PANE_TUNING.wall
                .centerY,
              x: side * (WORLD_TUNING.corridorHalfWidth
                - PANE_TUNING.wall
                .inset),
              z: frontier.wallZ,
              rotationY: side === 1 ? (-Math.PI) / 2 : Math.PI / 2,
            },
          },);
        frontier.wallZ -= PANE_TUNING.wallSpacing;
      }
      //endregion
      return events;
    },
  };
}
