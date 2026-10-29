/**
 Divergences and intent questions found by mutation testing
 (`doc/audit/deepmerge-ts-mutation-2026-09-24.md`).

 Same contract as `./known-defect.unit.test.ts`: each test asserts the current
 behaviour, so the suite stays green while upstream is unchanged and turns
 red when upstream changes it. Each also names the Stryker mutant ids of that
 run it kills, since no other test observes these paths.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

/**
 Custom `filterValues` removing arrays.

 @param values - Values at one position.

 @returns Values that are not arrays.

 @example
 ```ts
 keepNonArrays([[1,], 2,]); // [2]
 ```
 */
function keepNonArrays(values: readonly unknown[],): unknown[] {
  return values.filter(function isNotArray(value,) {
    return !Array.isArray(value,);
  },);
}

await describe({
  name: 'deepmerge-ts divergences found by mutation testing still reproduce',
  children: [
    it({
      name: 'defect: filterValues removing every value at a key the target lacks leaves an empty container',
      // Kills 511 (defaults/general.ts). Not generated: option plans never filter out containers.
      // Cause: mergeRecordsInto in src/defaults/into.ts seeds the slot with emptyLike(propValues[0]) before
      // mergeUnknownsInto filters, and returns early on zero values, so the seed is assigned to the target.
      fn: async () => {
        /**
         Target lacking `a`.
         */
        const into: Record<string, unknown> = {};
        target.deepmergeIntoCustom({ filterValues: keepNonArrays, },)(into, { a: [1,], }, { a: [2,], },);
        expect(into,).toEqual({ a: [], },);
        // deepmerge with the same option gives undefined at `a`.
        expect(target.deepmergeCustom({ filterValues: keepNonArrays, },)({ a: [1,], }, { a: [2,], },),)
          .toEqual({ a: undefined, },);
      },
    },),
    it({
      name: 'intent question: invalid maxDepth values in deepmergeIntoCustom silently fall back to 1000',
      // Kills 207 and 209 (deepmerge-into.ts). Same question as the deepmergeCustom case in
      // ./known-defect-options.unit.test.ts; a string passes the typeof check only in the mutants.
      // Cause: getUtils in src/deepmerge-into.ts accepts only a non-NaN number >= 0 and otherwise uses 1000.
      fn: async () => {
        for (const maxDepth of [-1, Number.NaN, '1',]) {
          /**
           Target merged under the invalid limit.
           */
          const into: Record<string, unknown> = { r: { x: 1, }, };
          target.deepmergeIntoCustom({ maxDepth: maxDepth as number, },)(into, { r: { y: 2, }, },);
          expect(into,).toEqual({ r: { x: 1, y: 2, }, },);
        }
      },
    },),
  ],
},);
