/**
 * Tests for the pure shard launch math.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEBRIS_TUNING,
  launchShardBody,
} from './shard-launch.ts';

/**
 * Constant random source pinning every draw to the midpoint, which zeroes
 * the symmetric jitter terms.
 *
 * @returns 0.5 on every draw
 *
 * @example
 * ```ts
 * const body = launchShardBody({ ...input, random: midRandom },);
 * ```
 */
function midRandom(): number {
  return 1 / 2;
}

/**
 * Constant random source above the midpoint so spin draws come out
 * nonzero.
 *
 * @returns 0.9 on every draw
 *
 * @example
 * ```ts
 * const body = launchShardBody({ ...input, random: highRandom },);
 * ```
 */
function highRandom(): number {
  return 0.9;
}

/**
 * Baseline launch input: shard half a meter to the right of the impact,
 * ball flying straight down the corridor.
 */
const BASE_INPUT = {
  pivot: {
    x: 0.5,
    y: 1.6,
    z: -12,
  },
  impact: {
    x: 0,
    y: 1.6,
    z: -12,
  },
  ballVelocity: {
    x: 0,
    y: 0,
    z: -16,
  },
  impactDistance: 0.5,
  thickness: 0.012,
  size: 0.12,
  random: midRandom,
} as const;

await describe({
  name: launchShardBody.name,
  children: [
    it({
      name: 'transfers more ball momentum near the impact than far away',
      fn: async function transfersMomentumByDistance(): Promise<void> {
        /**
         * Shard essentially at the impact point.
         */
        const near = launchShardBody({
          ...BASE_INPUT,
          impactDistance: 0.02,
        },);
        /**
         * Shard a full pane away from the impact.
         */
        const far = launchShardBody({
          ...BASE_INPUT,
          impactDistance: 1.6,
        },);
        expect(Math.abs(near.vz,),).toBeGreaterThan(Math.abs(far.vz,),);
        expect(near.vz,).toBeLessThan(0,);
      },
    },),

    it({
      name: 'bursts radially away from the impact',
      fn: async function burstsRadially(): Promise<void> {
        /**
         * Shard offset along +x from the impact.
         */
        const body = launchShardBody(BASE_INPUT,);
        expect(body.vx,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'gives the center cell no radial kick',
      fn: async function centerCellStaysRadialFree(): Promise<void> {
        /**
         * Shard exactly at the impact point.
         */
        const body = launchShardBody({
          ...BASE_INPUT,
          pivot: BASE_INPUT.impact,
          impactDistance: 0,
        },);
        /**
         * Expected pure momentum-transfer z velocity: full punch, no
         * radial term, and midpoint random zeroes the jitter.
         */
        const expectedVz = BASE_INPUT.ballVelocity
          .z
          * DEBRIS_TUNING.punchTransfer;
        expect(body.vx,).toBeCloseTo(0, 10,);
        expect(body.vz,).toBeCloseTo(expectedVz, 10,);
      },
    },),

    it({
      name: 'spins small shards faster than large plates',
      fn: async function spinsSmallShardsFaster(): Promise<void> {
        /**
         * Random source with one fixed draw sequence per call so both
         * bodies read identical spin draws.
         */
        const tiny = launchShardBody({
          ...BASE_INPUT,
          size: 0.03,
          random: highRandom,
        },);
        /**
         * Large plate for the comparison.
         */
        const plate = launchShardBody({
          ...BASE_INPUT,
          size: 0.5,
          random: highRandom,
        },);
        expect(Math.abs(tiny.wx,),).toBeGreaterThan(Math.abs(plate.wx,),);
      },
    },),

    it({
      name: 'starts unsettled with a size-scaled rest height',
      fn: async function startsUnsettled(): Promise<void> {
        /**
         * Baseline launch.
         */
        const body = launchShardBody(BASE_INPUT,);
        expect(body.settled,).toBe(false,);
        expect(body.restHeight,).toBeCloseTo(
          (BASE_INPUT.thickness / 2)
            + (BASE_INPUT.size * DEBRIS_TUNING.restHeightFactor),
          10,
        );
      },
    },),
  ],
},);
