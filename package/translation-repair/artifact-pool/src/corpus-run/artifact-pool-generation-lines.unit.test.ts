/**
 Tests for the line a refusal prints per built pipeline.

 WHY THIS SITS APART from `artifact-pool-refusal.unit.test.ts`: `await
 describe` throws, so two suites in one file cannot both be read by a single
 GFP round, and each of these pins a different decision.

 WHAT IT PINS is that the line carries the entry COUNT beside the abbreviated
 digest. Measured on 2026-08-25, dropping the count failed no case: the
 listing still named every generation and still read as a report, while the
 reader lost the one number the refusal exists to offer.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type GenerationCensus,
  generationLines,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Census holding two generations, one of them a single entry.
 */
const CENSUS = {
  groups: [
    {
      digest: 'aa11bb22',
      entryIds: [
        'Mittens',
        'Whiskers',
        'Biscuit',
      ],
    },
    {
      digest: 'cc33dd44',
      entryIds: ['Pepper',],
    },
  ],
  total: 4,
  tipByEntry: new Map(),
} as unknown as GenerationCensus;

/**
 Abbreviator standing in for the one a report sizes over every digest.

 @param id - digest to shorten

 @returns First four characters of it

 @example
 ```ts
 const label = short({ id: 'aa11bb22', },);
 ```
 */
function short({ id, }: { readonly id: string; },): string {
  return id.slice(
    0,
    4,
  );
}

//endregion Fixtures

await describe({
  name: generationLines.name,
  children: [
    it({
      name: 'NAMES the entry count beside each pipeline, and agrees the noun with it, so a reader '
        + 'choosing which generation to keep can see which one holds the bulk',
      fn: async () => {
        expect(generationLines({
          census: CENSUS,
          short,
        },),).toEqual([
          '  aa11  3 entries',
          '  cc33  1 entry',
        ],);
      },
    },),
  ],
},);
