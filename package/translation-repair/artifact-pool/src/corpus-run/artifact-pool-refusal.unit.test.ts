/**
 Tests for the noun a pool refusal counts entries with.

 WHY THIS FILE EXISTS. A refusal names how many entries each built pipeline
 holds, and someone reads that listing to decide which generation to keep.
 Measured on 2026-08-25, always saying `entries` failed no case, so the one
 generation holding a single entry read as `1 entries` with nothing to catch
 it.

 THE COUNT ITSELF is pinned in `artifact-pool-generation-lines.unit.test.ts`,
 which is a separate file because `await describe` throws and one GFP round
 cannot read two suites that share one.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pluralEntries, } from '../../dist/final/node/index.mjs';

await describe({
  name: pluralEntries.name,
  children: [
    it({
      name: 'SAYS entry at one and entries otherwise, including at zero, which is the count a refusal '
        + 'most often prints and the one an English plural still takes',
      fn: async () => {
        expect(pluralEntries({ count: 1, },),).toBe('entry',);
        expect(pluralEntries({ count: 0, },),).toBe('entries',);
        expect(pluralEntries({ count: 2, },),).toBe('entries',);
      },
    },),
  ],
},);
