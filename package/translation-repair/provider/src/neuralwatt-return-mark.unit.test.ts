/**
 Tests for the mark that keeps when NeuralWatt's spent day starts again, beside the reading that named it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { createNeuralWattReturnMark, } from '../dist/final/node/index.mjs';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

await describe({
  name: createNeuralWattReturnMark.name,
  concurrency: 1,
  children: [
    it({
      name: 'NAMES NO RETURN UNTIL A READING NAMES ONE, and counts the wait down on the clock it is given',
      fn: async () => {
        const clock = { now: 1_000, };
        const mark = createNeuralWattReturnMark({ now: () => clock.now, },);
        expect([mark.passed(), mark.leftMs(),],).toEqual([false, 0,],);
        mark.replace({ dryness: { kind: 'spent-day', dayStart: '2026-10-23T19:15:08.000Z', returnsInMs: 48_000, }, },);
        expect([mark.passed(), mark.leftMs(),],).toEqual([false, 48_000,],);
        clock.now += 47_997;
        expect([mark.passed(), mark.leftMs(),],).toEqual([false, 3,],);
        clock.now += 3;
        expect([mark.passed(), mark.leftMs(),],).toEqual([true, 0,],);
      },
    },),

    it({
      name: 'COUNTS THE RETURN AS PASSED TWO MILLISECONDS EARLY, the slack for a timer that fires a hair '
        + 'before its time',
      fn: async () => {
        const clock = { now: 1_000, };
        const mark = createNeuralWattReturnMark({ now: () => clock.now, },);
        mark.replace({ dryness: { kind: 'spent-day', dayStart: '2026-10-23T19:15:08.000Z', returnsInMs: 48_000, }, },);
        clock.now += 47_997;
        expect(mark.passed(),).toBe(false,);
        clock.now += 1;
        expect(mark.passed(),).toBe(true,);
      },
    },),

    it({
      name: 'LOGS THE RETURN WHEN IT CHANGES AND NOT ON EVERY READING that names the same day start: a minute\'s '
        + 'readings repeat the line otherwise',
      fn: async () => {
        const lines: string[] = [];
        const clock = { now: 1_000, };
        const mark = createNeuralWattReturnMark({ now: () => clock.now, l: levelCapturingLogger({ lines, },), },);
        mark.replace({ dryness: { kind: 'spent-day', dayStart: '2026-10-23T19:15:08.000Z', returnsInMs: 48_000, }, },);
        clock.now += 1_000;
        mark.replace({ dryness: { kind: 'spent-day', dayStart: '2026-10-23T19:15:08.000Z', returnsInMs: 47_000, }, },);
        expect(lines.length,).toBe(1,);
        mark.replace({ dryness: { kind: 'wet', }, },);
        mark.replace({
          dryness: { kind: 'spent-day', dayStart: '2026-10-24T19:15:08.000Z', returnsInMs: 86_000_000, },
        },);
        expect(lines.length,).toBe(2,);
      },
    },),

    it({
      name: 'FORGETS THE RETURN WHEN THE NEXT READING NAMES NONE, so a return never outlives the reading that named it',
      fn: async () => {
        const mark = createNeuralWattReturnMark({ now: () => 1_000, },);
        mark.replace({ dryness: { kind: 'spent-day', dayStart: '2026-10-23T19:15:08.000Z', returnsInMs: 48_000, }, },);
        mark.replace({ dryness: { kind: 'wet', }, },);
        expect([mark.passed(), mark.leftMs(),],).toEqual([false, 0,],);
        expect(mark.dryness(),).toEqual({ kind: 'wet', },);
      },
    },),

    it({
      name: 'COUNTS THE DISTANCE DOWN in the cause it reports, and keeps the other causes as the reading gave them',
      fn: async () => {
        const clock = { now: 1_000, };
        const mark = createNeuralWattReturnMark({ now: () => clock.now, },);
        mark.replace({ dryness: { kind: 'spent-day', dayStart: '2026-10-23T19:15:08.000Z', returnsInMs: 48_000, }, },);
        clock.now += 8_000;
        expect(mark.dryness(),).toEqual({
          kind: 'spent-day',
          dayStart: '2026-10-23T19:15:08.000Z',
          returnsInMs: 40_000,
        },);
        mark.replace({ dryness: { kind: 'key-blocked', until: '2026-10-24T05:00:00.000Z', }, },);
        expect([mark.dryness(), mark.leftMs(),],).toEqual([
          { kind: 'key-blocked', until: '2026-10-24T05:00:00.000Z', },
          0,
        ],);
        mark.replace({ dryness: { kind: 'reserved', }, },);
        expect(mark.dryness(),).toEqual({ kind: 'reserved', },);
      },
    },),

    it({
      name: 'NAMES NO RETURN FOR A SPENT DAY THE ACCOUNT STILL SHOWS AFTER ITS RETURN PASSED: the same day start read '
        + 'again past its return is the account lagging, not a new day, so no 24 h wait follows',
      fn: async () => {
        const clock = { now: 1_000, };
        const mark = createNeuralWattReturnMark({ now: () => clock.now, },);
        mark.replace({ dryness: { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 48_000, }, },);
        clock.now += 48_000;
        mark.replace({
          dryness: { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 86_400_000, },
        },);
        expect([mark.passed(), mark.leftMs(),],).toEqual([false, 0,],);
        expect(mark.dryness(),).toEqual({ kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', },);
      },
    },),
  ],
},);
