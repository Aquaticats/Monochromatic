/**
 Tests for the fan-out window: which seats a round asks first.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  askingWindow,
  benchRotation,
  FANOUT_SPARE,
  firstRoundWindow,
  rotatedBench,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 A seven-seat bench in roster order.
 */
const BENCH: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: benchRotation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ROTATES deterministically by the prompt, within the bench, and differently for a different '
            + 'prompt at least somewhere, so a re-run asks the same seats and slices spread their first asks',
          fn: async () => {
            /**
             Rotation for one prompt, read twice.
             */
            const first = benchRotation({ messages: [{ role: 'user', content: 'meow', },], size: 7, },);
            expect(first,).toBe(benchRotation({ messages: [{ role: 'user', content: 'meow', },], size: 7, },),);
            expect(first,).toBeGreaterThanOrEqual(0,);
            expect(first,).toBeLessThan(7,);
            /**
             Rotations over many distinct prompts.
             */
            const seen = new Set(
              Array.from({ length: 40, }, function rotationOf(_,
                at,): number {
                return benchRotation({ messages: [{ role: 'user', content: `prompt ${String(at,)}`, },], size: 7, },);
              },),
            );
            expect(seen.size,).toBeGreaterThan(1,);
            expect(benchRotation({ messages: [], size: 0, },),).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: rotatedBench.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS every seat exactly once, starting at the prompt\'s rotation',
          fn: async () => {
            /**
             Bench as one prompt orders it.
             */
            const order = rotatedBench({ modelIds: BENCH, messages: [{ role: 'user', content: 'meow', },], },);
            expect([...order,].toSorted(),).toEqual([...BENCH,].toSorted(),);
            /**
             Where the rotation started.
             */
            const start = benchRotation({ messages: [{ role: 'user', content: 'meow', },], size: BENCH.length, },);
            expect(order[0],).toBe(BENCH[start],);
          },
        },),

        it({
          name: 'STARTS at the seat a launch pins, whatever the prompt says, and keeps every seat once, so a fixture is moved by no '
            + 'sheet text (n166, Z3)',
          fn: async () => {
            for (const start of [0, 3, BENCH.length - 1,]) {
              /**
               Bench as the pinned start orders it.
               */
              const order = rotatedBench({ modelIds: BENCH, messages: [{ role: 'user', content: 'meow', },], start, },);
              expect(order,).toEqual([...BENCH.slice(start,), ...BENCH.slice(0, start,),],);
              expect(
                rotatedBench({ modelIds: BENCH, messages: [{ role: 'user', content: 'a different prompt entirely', },], start, },),
              ).toEqual(order,);
            }
          },
        },),

        it({
          name: 'REFUSES A PINNED START THAT IS NOT A WHOLE NUMBER FROM 0 TO BELOW THE SIZE OF THE BENCH, naming the value and the '
            + 'size: -1, NaN, 1.5 and the size itself, where the rotation silently counted from the end, stayed unrotated or '
            + 'truncated; and accepts 0 and the size less one',
          fn: async () => {
            /**
             What each pinned start comes to: the error's name and message, or the order the bench was asked in.
             */
            const outcomes = [-1, Number.NaN, 1.5, BENCH.length,].map(function refusalOf(start,): unknown {
              try {
                return rotatedBench({ modelIds: BENCH, messages: [{ role: 'user', content: 'meow', },], start, },);
              }
              catch (error) {
                return Error.isError(error,) ? { name: error.name, message: error.message, } : error;
              }
            },);
            expect(outcomes.map(function nameOf(outcome,): unknown {
              return (outcome as { readonly name?: unknown; }).name;
            },),).toEqual(Array.from({ length: 4, }, () => 'StageBenchStartError',),);
            expect(outcomes.map(function namesValueAndSize(outcome,): readonly boolean[] {
              const { message, } = outcome as { readonly message: string; };
              return [message.includes(String(BENCH.length,),), message.includes('whole number',),];
            },),).toEqual(Array.from({ length: 4, }, () => [true, true,],),);
            expect(outcomes.map(function valueNamed(outcome, at,): boolean {
              return (outcome as { readonly message: string; }).message.includes(String([-1, Number.NaN, 1.5, BENCH.length,][at],),);
            },),).toEqual([true, true, true, true,],);
            expect([0, BENCH.length - 1,].map(function acceptedOrder(start,): readonly RosterModelId[] {
              return rotatedBench({ modelIds: BENCH, messages: [{ role: 'user', content: 'meow', },], start, },);
            },),).toEqual([BENCH, [BENCH.at(-1,) as RosterModelId, ...BENCH.slice(0, -1,),],],);
          },
        },),
      ],
    },),

    describe({
      name: askingWindow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES what quorum still needs plus the spare from the front, never more than is pending, '
            + 'and the spare alone when nothing is needed',
          fn: async () => {
            expect(askingWindow({ pending: BENCH, needed: 4, },),).toEqual(BENCH.slice(0, 4 + FANOUT_SPARE,),);
            expect(askingWindow({ pending: BENCH.slice(0, 2,), needed: 4, },),).toEqual(BENCH.slice(0, 2,),);
            expect(askingWindow({ pending: BENCH, needed: 0, },),).toEqual(BENCH.slice(0, FANOUT_SPARE,),);
            expect(askingWindow({ pending: [], needed: 3, },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: firstRoundWindow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS quorum plus the spare, capped at the bench, so six seats cost half plus the spare, '
            + 'three seats cost all three and an empty bench costs nothing',
          fn: async () => {
            expect(firstRoundWindow({ benchSize: 6, },),).toBe(3 + FANOUT_SPARE,);
            expect(firstRoundWindow({ benchSize: 4, },),).toBe(2 + FANOUT_SPARE,);
            expect(firstRoundWindow({ benchSize: 3, },),).toBe(3,);
            expect(firstRoundWindow({ benchSize: 0, },),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
