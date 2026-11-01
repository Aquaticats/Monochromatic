/**
 Model properties: every default merge entry point agrees with the reference
 model in `./model.ts` on generated trees.

 - `deepmerge` on exotic trees (accessors, hidden keys, null prototypes,
   frozen records, `__proto__` keys).
 - `deepmergeCustom({ maxDepth })` for small depths, pinning the documented
   last-value fallback (Q10).
 - `deepmergeInto` on a writable plain target: the target ends up equal to
   the model's merge of its snapshot and the sources.
 - `deepmergeFastUnsafe` and `deepmergeIntoFastUnsafe` on the same inputs
   without `__proto__` keys, which `docs/API.md` excludes for those variants.

 Run plan and seed policy: see `./fuzz-budget.ts`.

 @module
 */

import {
  array,
  assert,
  integer,
  property,
  tuple,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  treeArbitraries,
} from './arbitraries.ts';
import { mergeArgumentsArbitrary, } from './argument-arbitraries.ts';
import { fuzzRunPlan, } from './fuzz-budget.ts';
import { modelMerge, } from './model.ts';
import {
  NO_MISMATCH,
  shapeMismatch,
  snapshotObject,
} from './shape.ts';
import { target, } from './target.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Largest custom `maxDepth` exercised; generated trees nest up to four
 containers, so depths past five behave like the default.
 */
const MAX_CUSTOM_DEPTH = 5;

/**
 Most sources merged into one `deepmergeInto` target.
 */
const MAX_INTO_SOURCES = 3;

/**
 Exotic argument lists, `__proto__` keys included.
 */
const exoticArguments = mergeArgumentsArbitrary({ exotic: true, },);

/**
 Exotic argument lists without `__proto__` keys, for the `FastUnsafe` variants.
 */
const fastArguments = mergeArgumentsArbitrary({ exotic: true, protoKey: false, },);

/**
 `deepmergeInto` inputs: a writable plain target record plus exotic sources.

 Dates and class instances are left out: when the target lacks a key,
 `deepmergeInto` seeds it from the first value as a plain object and then
 merges later records into it instead of letting the last value win. The
 inputs hold no `undefined` values: when the first value at a key is
 `undefined`, `deepmergeInto` replaces instead of merging at that key. Both are
 known defects, see `./known-defect.unit.test.ts`.

 @param protoKey - Whether records may use the `__proto__` key.

 @returns Generator of a target and its sources.

 @example
 ```ts
 const inputs = intoArguments({ protoKey: true, });
 ```
 */
function intoArguments({ protoKey, }: { readonly protoKey: boolean; },) {
  return tuple(
    treeArbitraries({ exotic: false, objectLeaves: false, protoKey, undefinedLeaves: false, },).record.filter((record,) => Object.getPrototypeOf(record,) === Object.prototype),
    array(treeArbitraries({ exotic: true, objectLeaves: false, protoKey, undefinedLeaves: false, },).record, { minLength: 0, maxLength: MAX_INTO_SOURCES, },),
  );
}

//endregion Constants and arbitraries

await describe({
  name: 'deepmerge-ts agrees with the reference model',
  children: [
    it({
      name: 'deepmerge matches the model on exotic trees',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(exoticArguments, function deepmergeMatchesModel(values,) {
            expect(shapeMismatch({ actual: target.deepmerge(...values,), expected: modelMerge({ values, },), },),)
              .toBe(NO_MISMATCH,);
          },),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmergeCustom({ maxDepth }) falls back to the last value at the limit',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            exoticArguments,
            integer({ min: 1, max: MAX_CUSTOM_DEPTH, },),
            function maxDepthMatchesModel(values, maxDepth,) {
              expect(shapeMismatch({
                actual: target.deepmergeCustom({ maxDepth, },)(...values,),
                expected: modelMerge({ values, maxDepth, },),
              },),)
                .toBe(NO_MISMATCH,);
            },
          ),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmergeInto leaves the target equal to the model merge',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(intoArguments({ protoKey: true, },), function deepmergeIntoMatchesModel([generated, sources,],) {
            // fast-check may replay one generated value while shrinking, so mutate a copy.
            /**
             Private target this run may mutate.
             */
            const mutableTarget = snapshotObject(generated,);
            /**
             Model prediction from the untouched target.
             */
            const expected = modelMerge({ values: [generated, ...sources,], },);
            target.deepmergeInto(mutableTarget, ...sources,);
            expect(shapeMismatch({ actual: mutableTarget, expected, },),).toBe(NO_MISMATCH,);
          },),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmergeFastUnsafe matches the model without __proto__ keys',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(fastArguments, function fastMatchesModel(values,) {
            expect(shapeMismatch({ actual: target.deepmergeFastUnsafe(...values,), expected: modelMerge({ values, },), },),)
              .toBe(NO_MISMATCH,);
          },),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmergeIntoFastUnsafe matches the model without __proto__ keys',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(intoArguments({ protoKey: false, },), function fastIntoMatchesModel([generated, sources,],) {
            /**
             Private target this run may mutate.
             */
            const mutableTarget = snapshotObject(generated,);
            /**
             Model prediction from the untouched target.
             */
            const expected = modelMerge({ values: [generated, ...sources,], },);
            target.deepmergeIntoFastUnsafe(mutableTarget, ...sources,);
            expect(shapeMismatch({ actual: mutableTarget, expected, },),).toBe(NO_MISMATCH,);
          },),
          RUN.params,
        );
      },
    },),
  ],
},);
