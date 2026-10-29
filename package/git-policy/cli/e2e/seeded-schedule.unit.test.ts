import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createSeededRandom,
  deriveSeed,
  parseSeed,
  SeededRandomError,
} from './seeded-random-fixture.ts';
import {
  interleaveSequences,
  planFaultOffset,
  planStartOffsets,
} from './seeded-schedule-fixture.ts';

//region Fixtures

/**
 Draw count used to compare sequences.
 */
const DRAWS = 16;

/**
 Draws floats from a fresh source.

 @param seed - seed

 @returns drawn floats

 @example
 ```ts
 draws(1);
 ```
 */
function draws(seed: number,): readonly number[] {
  /**
   Source under test.
   */
  const random = createSeededRandom(seed,);
  return Array.from({ length: DRAWS, }, function draw() {
    return random.next();
  },);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: createSeededRandom.name,
      children: [
        it({
          name: 'replays the same sequence for the same seed and differs across seeds',
          fn: async () => {
            expect(draws(42,),).toEqual(draws(42,),);
            expect(draws(42,),).not.toEqual(draws(43,),);
            expect(draws(7,).every(function unitInterval(value,) {
              return (value >= 0) && (value < 1);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'keeps integers inside the inclusive range and reaches both ends',
          fn: async () => {
            const random = createSeededRandom(3,);
            const values = Array.from({ length: 200, }, function draw() {
              return random.integer({ min: 2, max: 4, },);
            },);
            expect(new Set(values,),).toEqual(new Set([2, 3, 4,],),);
          },
        },),
        it({
          name: 'shuffles into a permutation without mutating the input',
          fn: async () => {
            const input = [1, 2, 3, 4, 5, 6,] as const;
            const shuffled = createSeededRandom(9,).shuffle(input,);
            expect(shuffled.toSorted(function ascending(left, right,) {
              return left - right;
            },),).toEqual([...input,],);
            expect(input,).toEqual([1, 2, 3, 4, 5, 6,],);
          },
        },),
        it({
          name: 'forks independent labelled sources deterministically',
          fn: async () => {
            const random = createSeededRandom(5,);
            expect(random.fork('a',).next(),).toBe(createSeededRandom(5,).fork('a',).next(),);
            expect(random.fork('a',).next(),).not.toBe(random.fork('b',).next(),);
          },
        },),
        it({
          name: 'rejects invalid seeds, ranges, and empty picks',
          fn: async () => {
            expect(function negativeSeed() {
              return createSeededRandom(-1,);
            },).toThrow(SeededRandomError,);
            expect(function invertedRange() {
              return createSeededRandom(1,).integer({ min: 3, max: 2, },);
            },).toThrow(SeededRandomError,);
            expect(function emptyPick() {
              return createSeededRandom(1,).pick([],);
            },).toThrow('cannot pick from an empty array',);
          },
        },),
      ],
    },),
    describe({
      name: deriveSeed.name,
      children: [
        it({
          name: 'is stable and label-sensitive',
          fn: async () => {
            expect(deriveSeed({ seed: 1, label: 'x', },),).toBe(deriveSeed({ seed: 1, label: 'x', },),);
            expect(deriveSeed({ seed: 1, label: 'x', },),).not.toBe(deriveSeed({ seed: 1, label: 'y', },),);
            expect(deriveSeed({ seed: 1, label: 'x', },),).not.toBe(deriveSeed({ seed: 2, label: 'x', },),);
          },
        },),
      ],
    },),
    describe({
      name: parseSeed.name,
      children: [
        it({
          name: 'generates a seed only for empty text',
          fn: async () => {
            expect(parseSeed({ text: ' ', fresh: () => 11, },),).toEqual({ seed: 11, generated: true, },);
            expect(parseSeed({ text: '42', fresh: () => 11, },),).toEqual({ seed: 42, generated: false, },);
          },
        },),
        it({
          name: 'rejects non-decimal and out-of-range text',
          fn: async () => {
            expect(function word() {
              return parseSeed({ text: 'abc', fresh: () => 0, },);
            },).toThrow(SeededRandomError,);
            expect(function tooLarge() {
              return parseSeed({ text: String(2 ** 32,), fresh: () => 0, },);
            },).toThrow(SeededRandomError,);
            ['0x2a', '0b101', '0o17', '1e3', '4.0', '+5', '-0', '1_0',].forEach(function nonDecimal(text,) {
              expect(function spelledOtherwise() {
                return parseSeed({ text, fresh: () => 0, },);
              },).toThrow(SeededRandomError,);
            },);
          },
        },),
      ],
    },),
    describe({
      name: planStartOffsets.name,
      children: [
        it({
          name: 'plans one bounded offset per worker, replayably',
          fn: async () => {
            const offsets = planStartOffsets({ random: createSeededRandom(1,), count: 5, maxJitterMs: 30, },);
            expect(offsets,).toHaveLength(5,);
            expect(offsets.every(function bounded(offset,) {
              return (offset >= 0) && (offset <= 30);
            },),).toBe(true,);
            expect(planStartOffsets({ random: createSeededRandom(1,), count: 5, maxJitterMs: 30, },),).toEqual(offsets,);
          },
        },),
      ],
    },),
    describe({
      name: interleaveSequences.name,
      children: [
        it({
          name: 'schedules every operation once and keeps each actor in order',
          fn: async () => {
            const steps = interleaveSequences({ random: createSeededRandom(8,), lengths: [3, 2, 4,], },);
            expect(steps,).toHaveLength(9,);
            [0, 1, 2,].forEach(function actorOrder(actor,) {
              expect(steps.filter(function byActor(step,) {
                return step.actor === actor;
              },).map(function position(step,) {
                return step.position;
              },),).toEqual(Array.from({ length: [3, 2, 4,][actor] ?? 0, }, function index(_unused, value,) {
                return value;
              },),);
            },);
          },
        },),
        it({
          name: 'reaches different interleavings across seeds and replays each seed exactly',
          fn: async () => {
            const orders = Array.from({ length: 20, }, function order(_unused, seed,) {
              return JSON.stringify(interleaveSequences({ random: createSeededRandom(seed,), lengths: [2, 2,], },),);
            },);
            expect(new Set(orders,).size,).toBeGreaterThan(1,);
            expect(
              JSON.stringify(interleaveSequences({ random: createSeededRandom(4,), lengths: [2, 2,], },),),
            ).toBe(orders[4],);
          },
        },),
        it({
          name: 'returns no steps for empty sequences',
          fn: async () => {
            expect(interleaveSequences({ random: createSeededRandom(1,), lengths: [0, 0,], },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: planFaultOffset.name,
      children: [
        it({
          name: 'stays within the window',
          fn: async () => {
            const offset = planFaultOffset({ random: createSeededRandom(2,), minMs: 20, maxMs: 25, },);
            expect(offset,).toBeGreaterThanOrEqual(20,);
            expect(offset,).toBeLessThanOrEqual(25,);
          },
        },),
      ],
    },),
  ],
},);
