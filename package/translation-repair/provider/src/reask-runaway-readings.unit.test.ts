/**
 Tests for the census readings a reasoning runaway's re-ask is ordered by (ledger B1075).

 A SHARE IS READ ONLY FROM A MEASURED MODEL: one with a row and at least `MIN_CALLS_TO_MEASURE` calls since the caps.
 Every other model is `unmeasured` and is placed after the measured ones by `alternateModelsFor`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ROSTER_MODEL_IDS, } from '@monochromatic-dev/translation-repair-shared-contract';

import {
  MIN_CALLS_TO_MEASURE,
  RUNAWAY_READINGS,
  runawayShareOf,
} from '../dist/final/node/index.mjs';

//region Runaway reading tests

await describe({
  name: runawayShareOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS THE RUNAWAYS OVER THE CALLS of a model the table holds, as a fraction',
      fn: async () => {
        expect(runawayShareOf({ modelId: 'deepseek-v4.1-flash', },),).toBeCloseTo(438 / 78_100, 12,);
        expect(runawayShareOf({ modelId: 'gemma-4-26b-a4b-it', },),).toBe(0,);
      },
    },),

    it({
      name: 'READS A MODEL WITH NO ROW, OR FEWER CALLS THAN CAN BE MEASURED, AS UNMEASURED, and a model at the floor as '
        + 'measured',
      fn: async () => {
        expect(runawayShareOf({ modelId: 'claude-haiku-5-5', },),).toBe('unmeasured',);
        expect(runawayShareOf({
          modelId: 'minimax-m3',
          readings: [{ modelId: 'minimax-m3', calls: MIN_CALLS_TO_MEASURE - 1, runaways: 0, },],
        },),).toBe('unmeasured',);
        expect(runawayShareOf({
          modelId: 'minimax-m3',
          readings: [{ modelId: 'minimax-m3', calls: MIN_CALLS_TO_MEASURE, runaways: 5, },],
        },),).toBe(0.05,);
      },
    },),

    it({
      name: 'HOLDS ONE ROW FOR EACH CHAT MODEL THE CENSUS SEATED, in roster order, none with more runaways than calls',
      fn: async () => {
        /**
         Roster order of the models the table names.
         */
        const positions = RUNAWAY_READINGS.map(function positionOf(reading,): number {
          return ROSTER_MODEL_IDS.indexOf(reading.modelId,);
        },);
        expect(positions.every(function isKnown(position,): boolean {
          return position >= 0;
        },),).toBe(true,);
        expect(positions,).toEqual(positions.toSorted(function ascending(
          left,
          right,
        ): number {
          return left - right;
        },),);
        expect(new Set(RUNAWAY_READINGS.map(function nameOf(reading,) {
          return reading.modelId;
        },),).size,).toBe(RUNAWAY_READINGS.length,);
        expect(RUNAWAY_READINGS.every(function hasNoMoreRunawaysThanCalls(reading,): boolean {
          return reading.runaways <= reading.calls;
        },),).toBe(true,);
      },
    },),
  ],
},);

//endregion Runaway reading tests
