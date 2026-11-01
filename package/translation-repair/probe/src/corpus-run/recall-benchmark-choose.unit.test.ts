/**
 Tests for choosing the recall benchmark's entries across the size bands,
 read against a throwaway clone holding invented cat pages, never the pinned
 corpus.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { chooseRecallEntries, } from '../../dist/final/node/index.mjs';
import {
  type BenchPages,
  benchWorld,
} from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-run/bench-world.test-fixture.ts';

/**
 English page whose two sentences are long enough to delete.
 */
const ENGLISH = 'The kitten dozes on the warm windowsill every sunny afternoon. '
  + 'Its tail hangs down to the floor beside the cushion basket.\n';

/**
 Entry whose original page sits in the small band.
 */
const SMALL: BenchPages = {
  source: '小猫在窗台上打盹。\n',
  english: ENGLISH,
};

/**
 Entry whose original page sits in the medium band.
 */
const MEDIUM: BenchPages = {
  source: '猫'.repeat(800,),
  english: ENGLISH,
};

/**
 Entry whose original page sits in the large band.
 */
const LARGE: BenchPages = {
  source: '猫'.repeat(1_300,),
  english: ENGLISH,
};

await describe({
  name: chooseRecallEntries.name,
  concurrency: 1,
  children: [
    it({
      name: 'CHOOSES three entries a band in corpus order, setting aside the unseedable and the half-written, '
        + 'and stops at nine without reading the next entry',
      fn: async () => {
        await using world = await benchWorld({
          entries: {
            'a-meowing': {
              source: SMALL.source,
              english: 'Meow.\n',
            },
            'b-small': SMALL,
            'c-small': SMALL,
            'd-small': SMALL,
            'e-small': SMALL,
            'f-medium': MEDIUM,
            'g-medium': MEDIUM,
            'h-medium': MEDIUM,
            'i-large': LARGE,
            'j-large': LARGE,
            'k-large': LARGE,
            // Read, it would refuse: the nine before it already fill every band.
            'l-tangled': {
              source: SMALL.source,
              english: `---\ntitle: [unclosed\n---\n${ENGLISH}`,
            },
          },
          originalOnly: { 'a-half': SMALL.source, },
        },);

        const { chosen, perBand, } = await chooseRecallEntries({ pin: world.pin, },);

        expect(chosen.map(function idOf(entry,): string {
          return entry.entryId;
        },),).toEqual([
          'b-small',
          'c-small',
          'd-small',
          'f-medium',
          'g-medium',
          'h-medium',
          'i-large',
          'j-large',
          'k-large',
        ],);
        expect(perBand,).toEqual({
          small: 3,
          medium: 3,
          large: 3,
        },);
      },
    },),

    it({
      name: 'CHOOSES what the corpus offers when it holds fewer than nine seedable entries, band by band',
      fn: async () => {
        await using world = await benchWorld({
          entries: {
            'a-small': SMALL,
            'b-large': LARGE,
          },
        },);

        const { chosen, perBand, } = await chooseRecallEntries({ pin: world.pin, },);

        expect(chosen.map(function idOf(entry,): string {
          return entry.entryId;
        },),).toEqual([
          'a-small',
          'b-large',
        ],);
        expect(perBand,).toEqual({
          small: 1,
          medium: 0,
          large: 1,
        },);
      },
    },),

    it({
      name: 'CHOOSES nothing from a corpus with nobody in it, with every band counted at zero',
      fn: async () => {
        await using world = await benchWorld({ entries: {}, },);

        expect(await chooseRecallEntries({ pin: world.pin, },),).toEqual({
          chosen: [],
          perBand: {
            small: 0,
            medium: 0,
            large: 0,
          },
        },);
      },
    },),
  ],
},);
