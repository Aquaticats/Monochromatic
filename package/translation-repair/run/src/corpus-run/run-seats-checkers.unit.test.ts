/**
 Tests the checker bench each provider reading seats: three places filled
 from the order the checker benchmark of 2026-09-27 ranks, served seats
 first, and never a model that benchmark measured calling unchanged text
 fixed.

 THE CASE: with Synthetic and Hyper dry and Bedrock and OpenRouter wet, the
 reading five of the six runs before 2026-09-27 read, the bench seated
 Qwen3.8-27B with no provider serving it beside `gemma-4-26b-a4b-it` and
 `google.gemma-4-e2b`. On 85 settled fixes and the 85 unchanged archive texts
 beside them, those two resolved 35 fixes and wrongly resolved 9 unchanged
 texts; `gemma-4-26b-a4b-it` alone called 40 of 85 unchanged texts fixed and
 `google.gemma-4-e2b` called 50 of 85 real fixes not fixed (ledger L1).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type BudgetView,
  NO_PROVIDER,
  providerServing,
  reachOf,
} from '@monochromatic-dev/translation-repair-provider';
import { judgeSeatsFor, } from '../../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Providers in the order a dryness mask names them.
 */
const PROVIDERS = [
  'synthetic',
  'hyper',
  'bedrock',
  'openrouter',
] as const;

/**
 Checkers the benchmark measured calling unchanged text fixed or real fixes
 not fixed, far below every seated checker.
 */
const MEASURED_OUT: readonly RosterModelId[] = [
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_BEDROCK_ONLY_TEXT,
];

/**
 Every dryness reading, one per subset of the four providers.

 @returns Sixteen readings
 */
function everyReading(): readonly BudgetView[] {
  /**
   Readings count: one bit per provider.
   */
  const readings = 2 ** PROVIDERS.length;
  return Array.from({ length: readings, }, function readingAt(
    _unused,
    mask,
  ): BudgetView {
    /**
     Whether the provider at one bit is dry in this reading.

     @param bit - provider position

     @returns True when that provider is dry
     */
    function dryAt(bit: number,): boolean {
      return (mask & (1 << bit)) !== 0;
    }
    return {
      neuralwatt: true,
      synthetic: dryAt(0,),
      hyper: dryAt(1,),
      bedrock: dryAt(2,),
      deepseek: dryAt(3,),
      xiaomi: dryAt(3,),
      anthropic: dryAt(3,),
      openrouter: dryAt(3,),
    };
  },);
}

/**
 Whether some wet provider serves a seat under one reading.

 @param modelId - seat

 @param dry - reading

 @returns True when a call to it would be routed somewhere
 */
function servedUnder(
  {
    modelId,
    dry,
  }: {
    readonly modelId: RosterModelId;
    readonly dry: BudgetView;
  },
): boolean {
  return providerServing({
    reach: reachOf({ modelId, },),
    dry,
  },) !== NO_PROVIDER;
}

await describe({
  name: 'checker bench per reading',
  children: [
    it({
      name: 'SEATS THREE SERVED CHECKERS with Synthetic and Hyper dry, the reading five of the six runs '
        + 'before 2026-09-27 read, where the bench had one seat no provider served',
      fn: async () => {
        /** The dominant reading. */
        const dry: BudgetView = {
          neuralwatt: true,
          synthetic: true,
          hyper: true,
          bedrock: false,
          deepseek: false,
          xiaomi: false,
          anthropic: false,
          openrouter: false,
        };
        /** Bench it seats. */
        const { checkers, } = judgeSeatsFor({ dry, },);
        expect(checkers,).toHaveLength(3,);
        for (const modelId of checkers) {
          expect(servedUnder({
            modelId,
            dry,
          },),).toBe(true,);
        }
      },
    },),

    it({
      name: 'NEVER SEATS A CHECKER THE BENCHMARK MEASURED OUT, and seats every served checker before '
        + 'any seat no provider serves, on every reading the seats can be derived for',
      fn: async () => {
        for (const dry of everyReading()) {
          /** Bench this reading seats, or nothing when the contract refuses it. */
          const checkers = (function benchOrNone(): readonly RosterModelId[] {
            try {
              return judgeSeatsFor({ dry, },).checkers;
            }
            catch (error) {
              expect(String(error,),).toContain('checker',);
              return [];
            }
          })();
          for (const modelId of checkers)
            expect(MEASURED_OUT,).not.toContain(modelId,);
          /** Whether each seat is served, in bench order. */
          const served = checkers.map(function servedSeat(modelId,): boolean {
            return servedUnder({
              modelId,
              dry,
            },);
          },);
          expect(served,).toEqual(served.toSorted(function servedFirst(left, right,): number {
            return Number(right,) - Number(left,);
          },),);
        }
      },
    },),

    it({
      name: 'SEATS THE TOP THREE OF THE MEASURED ORDER when every provider is wet',
      fn: async () => {
        /** Nobody dry. */
        const dry: BudgetView = {
          neuralwatt: false,
          synthetic: false,
          hyper: false,
          bedrock: false,
          deepseek: false,
          xiaomi: false,
          anthropic: false,
          openrouter: false,
        };
        expect(judgeSeatsFor({ dry, },).checkers,).toEqual([
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_OPENROUTER_ONLY_CHECKER,
          SEAT_SYNTHETIC_VISION_WITHHELD,
        ],);
      },
    },),
  ],
},);
