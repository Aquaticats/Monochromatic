/**
 * Tests for ball flight, shard ground contact, and pane crossing.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type BallBody,
  GRAVITY,
  PANE_MISS,
  paneSegmentCrossing,
  SHARD_CONTACT_TUNING,
  type ShardBody,
  stepBallFlight,
  stepShardBody,
} from './physics.ts';

/**
 * Fresh ball fixture: level flight at 10 m/s along -z.
 *
 * @returns independent ball body per call so tests never share state
 *
 * @example
 * ```ts
 * const ball = levelBall();
 * ```
 */
function levelBall(): BallBody {
  return {
    px: 0,
    py: 1.6,
    pz: 0,
    vx: 0,
    vy: 0,
    vz: -10,
  };
}

/**
 * Fresh shard fixture dropping straight down from the given height.
 *
 * @param height - starting y in meters
 *
 * @param fallSpeed - downward speed in m/s, positive numbers fall
 *
 * @returns independent shard body per call
 *
 * @example
 * ```ts
 * const shard = droppingShard({ height: 0.5, fallSpeed: 3 },);
 * ```
 */
function droppingShard(
  {
    height,
    fallSpeed,
  }: Readonly<{
    height: number;
    fallSpeed: number;
  }>,
): ShardBody {
  return {
    px: 0,
    py: height,
    pz: 0,
    vx: 1,
    vy: -fallSpeed,
    vz: 0.5,
    wx: 2,
    wy: 3,
    wz: 4,
    restHeight: 0.02,
    settled: false,
  };
}

/**
 * Pane extents shared across crossing tests.
 */
const PANE = {
  halfWidth: 1,
  halfHeight: 1.3,
} as const;

await describe({
  name: stepBallFlight.name,
  children: [
    it({
      name: 'applies gravity and advances position',
      fn: async function appliesGravity(): Promise<void> {
        /**
         * Ball advanced one full second for round numbers.
         */
        const ball = levelBall();
        stepBallFlight({
          body: ball,
          dt: 1,
        },);
        expect(ball.vy,).toBeCloseTo(-GRAVITY, 10,);
        expect(ball.pz,).toBeCloseTo(-10, 10,);
        expect(ball.py,).toBeCloseTo(1.6 - GRAVITY, 10,);
      },
    },),
  ],
},);

await describe({
  name: stepShardBody.name,
  children: [
    it({
      name: 'falls freely above the floor',
      fn: async function fallsFreely(): Promise<void> {
        /**
         * Shard well above the floor.
         */
        const shard = droppingShard({
          height: 2,
          fallSpeed: 1,
        },);
        /**
         * Whether the step reported floor contact.
         */
        const contact = stepShardBody({
          body: shard,
          dt: 1 / 60,
          floorY: 0,
        },);
        expect(contact,).toBe(false,);
        expect(shard.py,).toBeLessThan(2,);
        expect(shard.settled,).toBe(false,);
      },
    },),

    it({
      name: 'bounces with restitution and friction when hitting fast',
      fn: async function bouncesWhenFast(): Promise<void> {
        /**
         * Shard about to slam into the floor.
         */
        const shard = droppingShard({
          height: 0.02,
          fallSpeed: 3,
        },);
        /**
         * Whether the step reported floor contact.
         */
        const contact = stepShardBody({
          body: shard,
          dt: 1 / 1_000,
          floorY: 0,
        },);
        expect(contact,).toBe(true,);
        expect(shard.settled,).toBe(false,);
        expect(shard.vy,).toBeGreaterThan(0,);
        expect(shard.vy,).toBeCloseTo(
          (3 + (GRAVITY / 1_000)) * SHARD_CONTACT_TUNING.restitution,
          6,
        );
        expect(shard.vx,).toBeCloseTo(SHARD_CONTACT_TUNING.tangentialKeep, 10,);
        expect(shard.wx,).toBeCloseTo(2 * SHARD_CONTACT_TUNING.spinKeep, 10,);
        expect(shard.py,).toBeCloseTo(0.02, 10,);
      },
    },),

    it({
      name: 'settles instead of bouncing below the settle speed',
      fn: async function settlesWhenSlow(): Promise<void> {
        /**
         * Shard drifting down slowly onto the floor.
         */
        const shard = droppingShard({
          height: 0.02,
          fallSpeed: 0.2,
        },);
        stepShardBody({
          body: shard,
          dt: 1 / 1_000,
          floorY: 0,
        },);
        expect(shard.settled,).toBe(true,);
        expect(shard.vx,).toBe(0,);
        expect(shard.vy,).toBe(0,);
        expect(shard.wz,).toBe(0,);
      },
    },),

    it({
      name: 'leaves settled shards untouched',
      fn: async function skipsSettled(): Promise<void> {
        /**
         * Already-settled shard that must not move again.
         */
        const shard = droppingShard({
          height: 0.02,
          fallSpeed: 0,
        },);
        shard.settled = true;
        stepShardBody({
          body: shard,
          dt: 1,
          floorY: 0,
        },);
        expect(shard.py,).toBeCloseTo(0.02, 10,);
        expect(shard.vy,).toBeCloseTo(0, 10,);
      },
    },),
  ],
},);

await describe({
  name: paneSegmentCrossing.name,
  children: [
    it({
      name: 'reports the crossing point and parameter for a through shot',
      fn: async function reportsThroughShot(): Promise<void> {
        /**
         * Crossing of a straight front-to-back segment.
         */
        const hit = paneSegmentCrossing({
          fromX: 0.5,
          fromY: -0.5,
          fromZ: 2,
          toX: 0.5,
          toY: -0.5,
          toZ: -2,
          ...PANE,
        },);
        if (hit === PANE_MISS)
          throw new Error('expected a crossing for a through shot',);
        expect(hit.x,).toBeCloseTo(0.5, 10,);
        expect(hit.y,).toBeCloseTo(-0.5, 10,);
        expect(hit.t,).toBeCloseTo(1 / 2, 10,);
      },
    },),

    it({
      name: 'misses when the crossing lies outside the pane rectangle',
      fn: async function missesOutsideRect(): Promise<void> {
        expect(paneSegmentCrossing({
          fromX: 2,
          fromY: 0,
          fromZ: 1,
          toX: 2,
          toY: 0,
          toZ: -1,
          ...PANE,
        },),).toBe(PANE_MISS,);
      },
    },),

    it({
      name: 'misses when both endpoints sit on one side',
      fn: async function missesSameSide(): Promise<void> {
        expect(paneSegmentCrossing({
          fromX: 0,
          fromY: 0,
          fromZ: 2,
          toX: 0,
          toY: 0,
          toZ: 1,
          ...PANE,
        },),).toBe(PANE_MISS,);
      },
    },),
  ],
},);
