/**
 Tests for the rule the editor fidelity pool is drawn by: which entry ids take part, the order a seeded draw visits
 entries in, and the order donors are offered in. Ids are invented.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  DEFAULT_POOL_SEED,
  furthestFirst,
  isPlainEntryId,
  seededEntryOrder,
} from '../dist/final/node/index.mjs';

/**
 Eight invented entry ids, in code-unit order.
 */
const IDS: readonly string[] = ['Acorn', 'Briar', 'Cinder', 'Dapple', 'Ember', 'Fennel', 'Ginger', 'Heron',];

await describe({
  name: 'editor fidelity pool rule',
  children: [
    it({
      name: 'TAKES AN ID OF LETTERS, DIGITS, UNDERSCORES, DOTS AND HYPHENS that does not begin with a dot, because the '
        + 'id becomes a file name in the clone and a spec id in the manifest',
      fn: async () => {
        for (const entryId of ['gqt', 'MTF_0615', 'Everythings99', 'a.b-c',])
          expect(isPlainEntryId({ entryId, },),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES AN EMPTY ID, ONE WITH A SPACE, A SLASH OR A NON-ASCII LETTER, AND ONE THAT BEGINS WITH A DOT',
      fn: async () => {
        for (const entryId of ['', 'two words', 'a/b', 'café', '.hidden', '..',])
          expect(isPlainEntryId({ entryId, },),).toBe(false,);
      },
    },),

    it({
      name: 'VISITS THE SAME ENTRIES IN THE SAME ORDER FOR THE SAME SEED, whatever order the ids are listed in, so the '
        + 'draw is reproducible from the seed alone',
      fn: async () => {
        /**
         The ids listed backwards.
         */
        const reversed = IDS.toReversed();
        expect(seededEntryOrder({
          entryIds: reversed,
          seed: DEFAULT_POOL_SEED,
        },),).toStrictEqual(seededEntryOrder({
          entryIds: IDS,
          seed: DEFAULT_POOL_SEED,
        },),);
        expect(seededEntryOrder({
          entryIds: IDS,
          seed: DEFAULT_POOL_SEED,
        },).toSorted(),).toStrictEqual(IDS,);
      },
    },),

    it({
      name: 'IS A SAMPLE AND NOT A PREFIX: another seed visits the entries in another order, and neither order is '
        + 'the alphabetical one',
      fn: async () => {
        /**
         Order under the default seed and under another.
         */
        const first = seededEntryOrder({
          entryIds: IDS,
          seed: DEFAULT_POOL_SEED,
        },);
        const second = seededEntryOrder({
          entryIds: IDS,
          seed: 'another seed',
        },);
        expect(first,).not.toStrictEqual(second,);
        expect(first,).not.toStrictEqual(IDS,);
        expect(second,).not.toStrictEqual(IDS,);
      },
    },),

    it({
      name: 'OFFERS THE FURTHEST SECTION FIRST and, between two equally far, the later one first, so a donor is as '
        + 'far from the damaged section as the page allows',
      fn: async () => {
        expect(furthestFirst({
          indices: [0, 1, 2, 3, 4,],
          from: 2,
        },),).toStrictEqual([4, 0, 3, 1,],);
        expect(furthestFirst({
          indices: [0, 1, 2,],
          from: 0,
        },),).toStrictEqual([2, 1,],);
      },
    },),
  ],
},);
