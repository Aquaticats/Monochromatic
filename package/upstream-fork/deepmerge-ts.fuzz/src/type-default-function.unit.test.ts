/**
 Custom merge functions that delegate to every `utils.defaultMergeFunctions`
 member, typed through the published declarations, on all four custom entry
 points: each must type-check with the documented call shape and merge like
 the matching default entry point.

 The historical-recall audit (`doc/audit/deepmerge-ts-recall-2026-09-24.md`,
 row `ca94270` / `#482`) found no sidecar custom function called
 `utils.defaultMergeFunctions`, so a declaration that rejected the documented
 delegation went unnoticed.

 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

/**
 Fresh inputs reaching records, arrays, Sets, Maps, and other values.

 @returns Two records sharing every key.

 @example
 ```ts
 target.deepmerge(...inputs(),);
 ```
 */
function inputs(): readonly [Record<string, unknown>, Record<string, unknown>,] {
  return [
    { a: [1,], m: new Map([['k', { x: 1, },],],), n: 1, r: { x: 1, }, s: new Set([1,],), },
    { a: [2,], m: new Map([['k', { y: 2, },],],), n: 2, r: { y: 2, }, s: new Set([2,],), },
  ];
}

await describe({
  name: 'custom functions delegating to utils.defaultMergeFunctions',
  children: [
    it({
      name: 'deepmergeCustom delegating every member merges like deepmerge',
      fn: async () => {
        /**
         Merge whose every function delegates to its default.
         */
        const delegating = target.deepmergeCustom({
          // Array, Set, and other defaults take values only: see the known-defect case in this file.
          mergeArrays: (values, utils,) => utils.defaultMergeFunctions.mergeArrays(values,),
          mergeCircularReferences: (values, cyclicDepths, utils, meta,) =>
            utils.defaultMergeFunctions.mergeCircularReferences(values, cyclicDepths, utils, meta,),
          mergeMaps: (values, utils, meta,) => utils.defaultMergeFunctions.mergeMaps(values, utils, meta,),
          mergeOthers: (values, utils,) => utils.defaultMergeFunctions.mergeOthers(values,),
          mergeRecords: (values, utils, meta,) => utils.defaultMergeFunctions.mergeRecords(values, utils, meta,),
          mergeSets: (values, utils,) => utils.defaultMergeFunctions.mergeSets(values,),
        },);
        expect(
          delegating(...inputs(),),
        ).toEqual(
          target.deepmerge(...inputs(),),
        );
        /**
         Self-referencing inputs, so the cycle function delegates too.
         */
        const left: Record<string, unknown> = { v: 1, };
        left.self = left;
        const right: Record<string, unknown> = { w: 2, };
        right.self = right;
        expect(delegating(left, right,),).toEqual(target.deepmerge(left, right,),);
      },
    },),
    it({
      name: 'known defect: default array, Set, and other functions reject the utils their optional parameter declares',
      // `mergeArrays`, `mergeSets`, and `mergeOthers` are declared as standalone generics
      // (`utils?: U, meta?: M`) whose `M` is inferred apart from the invariant `M` of the `utils`
      // a custom function receives, so passing that `utils` is TS2345. `mergeRecords`, `mergeMaps`,
      // and `mergeCircularReferences`, declared inline, accept the same call. Runtime ignores `utils`.
      // Upstream reporting: the combined issue draft, held locally for the user.
      fn: async () => {
        /**
         Delegation passing everything the declaration allows.
         */
        const merge = target.deepmergeCustom({
          // @ts-expect-error -- TS2345: Argument of type 'U' is not assignable (metaDataUpdater return types differ).
          mergeArrays: (values, utils, meta,) => utils.defaultMergeFunctions.mergeArrays(values, utils, meta,),
          // @ts-expect-error -- TS2345, as for mergeArrays.
          mergeOthers: (values, utils, meta,) => utils.defaultMergeFunctions.mergeOthers(values, utils, meta,),
          // @ts-expect-error -- TS2345, as for mergeArrays.
          mergeSets: (values, utils, meta,) => utils.defaultMergeFunctions.mergeSets(values, utils, meta,),
        },);
        expect(
          merge(...inputs(),),
        ).toEqual(
          target.deepmerge(...inputs(),),
        );
      },
    },),
    it({
      name: 'the documented mergeOthers filter (docs/deepmergeCustom.md) type-checks and filters',
      fn: async () => {
        /**
         Drops `null` and `undefined` before the default `mergeOthers`, as the docs show.
         */
        const merge = target.deepmergeCustom({
          mergeOthers: (values, utils,) => utils.defaultMergeFunctions.mergeOthers(values.filter(function present(value,) {
            return (value !== undefined) && (value !== null);
          },),),
        },);
        expectTypeOf(merge,).toBeFunction();
        expect(merge({ a: 1, }, { a: null, },),).toEqual({ a: 1, },);
      },
    },),
    it({
      name: 'deepmergeFastUnsafeCustom delegating every member merges like deepmergeFastUnsafe',
      fn: async () => {
        /**
         FastUnsafe merge whose every function delegates to its default.
         */
        const delegating = target.deepmergeFastUnsafeCustom({
          mergeArrays: (values, utils,) => utils.defaultMergeFunctions.mergeArrays(values,),
          mergeMaps: (values, utils,) => utils.defaultMergeFunctions.mergeMaps(values, utils, undefined,),
          mergeOthers: (values, utils,) => utils.defaultMergeFunctions.mergeOthers(values,),
          mergeRecords: (values, utils,) => utils.defaultMergeFunctions.mergeRecords(values, utils, undefined,),
          mergeSets: (values, utils,) => utils.defaultMergeFunctions.mergeSets(values,),
        },);
        expect(
          delegating(...inputs(),),
        ).toEqual(
          target.deepmergeFastUnsafe(...inputs(),),
        );
      },
    },),
    it({
      name: 'deepmergeIntoCustom delegating every member merges like deepmergeInto',
      fn: async () => {
        /**
         Into merge whose every function delegates to its default.
         */
        const delegating = target.deepmergeIntoCustom({
          mergeArrays: (mutTarget, values, utils,) => utils.defaultMergeFunctions.mergeArrays(mutTarget, values,),
          mergeCircularReferences: (mutTarget, values, utils, meta,) =>
            utils.defaultMergeFunctions.mergeCircularReferences(mutTarget, values, utils, meta,),
          mergeMaps: (mutTarget, values, utils, meta,) => utils.defaultMergeFunctions.mergeMaps(mutTarget, values, utils, meta,),
          mergeOthers: (mutTarget, values, utils,) => utils.defaultMergeFunctions.mergeOthers(mutTarget, values,),
          mergeRecords: (mutTarget, values, utils, meta,) => utils.defaultMergeFunctions.mergeRecords(mutTarget, values, utils, meta,),
          mergeSets: (mutTarget, values, utils,) => utils.defaultMergeFunctions.mergeSets(mutTarget, values,),
        },);
        /**
         Target and source merged through the delegating functions.
         */
        const [delegated, delegatedSource,] = inputs();
        delegating(delegated, delegatedSource,);
        /**
         Separate target and source merged by the default entry point.
         */
        const [defaulted, defaultedSource,] = inputs();
        target.deepmergeInto(defaulted, defaultedSource,);
        expect(delegated,).toEqual(defaulted,);
      },
    },),
    it({
      name: 'deepmergeIntoFastUnsafeCustom delegating every member merges like deepmergeIntoFastUnsafe',
      fn: async () => {
        /**
         FastUnsafe into merge whose every function delegates to its default.
         */
        const delegating = target.deepmergeIntoFastUnsafeCustom({
          mergeArrays: (mutTarget, values, utils,) => utils.defaultMergeFunctions.mergeArrays(mutTarget, values,),
          mergeMaps: (mutTarget, values, utils,) => utils.defaultMergeFunctions.mergeMaps(mutTarget, values, utils, undefined,),
          mergeOthers: (mutTarget, values, utils,) => utils.defaultMergeFunctions.mergeOthers(mutTarget, values,),
          mergeRecords: (mutTarget, values, utils,) => utils.defaultMergeFunctions.mergeRecords(mutTarget, values, utils, undefined,),
          mergeSets: (mutTarget, values, utils,) => utils.defaultMergeFunctions.mergeSets(mutTarget, values,),
        },);
        /**
         Target and source merged through the delegating functions.
         */
        const [delegated, delegatedSource,] = inputs();
        delegating(delegated, delegatedSource,);
        /**
         Separate target and source merged by the default entry point.
         */
        const [defaulted, defaultedSource,] = inputs();
        target.deepmergeIntoFastUnsafe(defaulted, defaultedSource,);
        expect(delegated,).toEqual(defaulted,);
      },
    },),
  ],
},);
