/**
 Tests for the draw of the whole editor fidelity pool: the seeded order entries are visited in, where the draw stops,
 and what it counts. Entries are invented; no clone is read.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  drawPool,
  seededEntryOrder,
} from '../../dist/final/node/index.mjs';
import {
  DRAW_PIN,
  readInventedEntry,
} from '../editor-fidelity-pool.test-fixture.ts';

/**
 Entry ids these draws list: seven that can be drawn, one that is not a plain id, one unreadable, one of unequal
 shape.
 */
const ENTRY_IDS: readonly string[] = [
  'Acorn',
  'Briar',
  'Cinder',
  'Dapple',
  'Ember',
  'Fennel',
  'Ginger',
  'bad id!',
  'ghost',
  'lopsided',
];

/**
 Draws the invented entries.

 @param seed - seed of the draw

 @param want - trials wanted in all

 @returns The draw

 @example
 ```ts
 const draw = await drawnWith({ seed: 'a', want: 5, },);
 ```
 */
function drawnWith({ seed, want, }: { readonly seed: string; readonly want: number; },): ReturnType<typeof drawPool> {
  return drawPool({
    entryIds: ENTRY_IDS,
    readEntry: function readEntry({ entryId, },) {
      return Promise.resolve(readInventedEntry({ entryId, },),);
    },
    seed,
    want,
    reviewedTrials: 0,
    reviewedEntryIds: [],
    corpusSha: DRAW_PIN.commitSha,
    reviewedOn: '2026-10-23',
  },);
}

await describe({
  name: drawPool.name,
  children: [
    it({
      name: 'STOPS AFTER THE ENTRY THAT TAKES THE POOL TO THE TRIALS WANTED, visiting entries in the seeded order and '
        + 'taking two trials of each, and counts every entry that could have been drawn, not only those it took',
      fn: async () => {
        const draw = await drawnWith({
          seed: 'a seed',
          want: 5,
        },);
        /**
         The plain, readable, equal-shape entries in the order the seed visits them.
         */
        const order = seededEntryOrder({
          entryIds: ENTRY_IDS.filter(function drawable(id,): boolean {
            return !['bad id!', 'ghost', 'lopsided',].includes(id,);
          },),
          seed: 'a seed',
        },);
        expect(draw.drawnEntries,).toStrictEqual(order.slice(0, 3,),);
        expect(draw.trials,).toBe(6,);
        expect(draw.eligibleEntries,).toBe(7,);
        expect(draw.kinds,).toStrictEqual({
          insertion: 3,
          alteration: 3,
        },);
      },
    },),

    it({
      name: 'COUNTS THE REVIEWED TRIALS toward the pool and leaves the reviewed entries out of the draw',
      fn: async () => {
        const draw = await drawPool({
          entryIds: ENTRY_IDS,
          readEntry: function readEntry({ entryId, },) {
            return Promise.resolve(readInventedEntry({ entryId, },),);
          },
          seed: 'a seed',
          want: 5,
          reviewedTrials: 4,
          reviewedEntryIds: ['Acorn',],
          corpusSha: DRAW_PIN.commitSha,
          reviewedOn: '2026-10-23',
        },);
        expect(draw.trials,).toBe(2,);
        expect(draw.drawnEntries.includes('Acorn',),).toBe(false,);
        expect(draw.eligibleEntries,).toBe(6,);
      },
    },),

    it({
      name: 'SAYS WHY ENTRIES WERE LEFT OUT, each reason with how many entries it took',
      fn: async () => {
        const draw = await drawnWith({
          seed: 'a seed',
          want: 5,
        },);
        expect(draw.skipped,).toStrictEqual({
          'entry id not plain': 1,
          'entry unreadable': 1,
          'entry not equal shape': 1,
        },);
      },
    },),

    it({
      name: 'VISITS ENTRIES IN ANOTHER ORDER FOR ANOTHER SEED and so can draw other entries, and draws the same ones '
        + 'again for the same seed',
      fn: async () => {
        const first = await drawnWith({
          seed: 'a seed',
          want: 5,
        },);
        const again = await drawnWith({
          seed: 'a seed',
          want: 5,
        },);
        const other = await drawnWith({
          seed: 'another seed',
          want: 5,
        },);
        expect(again.specs,).toStrictEqual(first.specs,);
        expect(other.drawnEntries,).not.toStrictEqual(first.drawnEntries,);
      },
    },),
  ],
},);
