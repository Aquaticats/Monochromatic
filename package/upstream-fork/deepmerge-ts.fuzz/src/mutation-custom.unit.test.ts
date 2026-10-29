/**
 Customization paths that upstream's suite and every earlier sidecar test left
 open, found by mutation testing (`doc/audit/deepmerge-ts-mutation-2026-09-24.md`).

 Each test asserts documented or current behaviour of 8.0.2 and names the
 Stryker mutant ids of that run it kills. They cover implicit default merging
 per merge function, `actions.defaultMerge` requested from into merge
 functions, `filterValues` resolution in the into variants, `rootMetaData`,
 `actions.skip` on the FastUnsafe three-record path and on Maps, and the keys
 `utils.mergeFunctions` exposes.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ActionUtils,
  type IntoActionUtils,
  cyclicSelf,
  keepNonStrings,
  markOthers,
  returnsUndefined,
} from './mutation-helper.ts';
import { target, } from './target.ts';

await describe({
  name: 'deepmerge-ts customization paths found by mutation testing',
  children: [
    it({
      name: 'filterValues dropping every value at a key leaves the target value in place',
      // Kills 19 and 21 (deepmerge-fast.ts), 107 and 109 (deepmerge-into-fast.ts), 220 and 222 (deepmerge-into.ts),
      // 201 and 203 (deepmerge-into.ts: a custom filterValues is used).
      fn: async () => {
        // The default mergeOthers also yields undefined for no values, so only a custom one shows the call.
        expect(target.deepmergeFastUnsafeCustom({ filterValues: keepNonStrings, mergeOthers: markOthers, },)('x', 'y',),)
          .toBe(undefined,);
        /**
         Into target whose only key receives two strings, both filtered out.
         */
        const into: Record<string, unknown> = { a: 'x', };
        target.deepmergeIntoCustom({ filterValues: keepNonStrings, },)(into, { a: 'y', },);
        expect(into,).toEqual({ a: 'x', },);
        /**
         Same case through the FastUnsafe into variant.
         */
        const fastInto: Record<string, unknown> = { a: 'x', };
        target.deepmergeIntoFastUnsafeCustom({ filterValues: keepNonStrings, },)(fastInto, { a: 'y', },);
        expect(fastInto,).toEqual({ a: 'x', },);
      },
    },),
    it({
      name: 'implicit default merging is off unless enabled',
      // Kills 366 (deepmerge.ts) and 6 (deepmerge-fast.ts).
      fn: async () => {
        expect(target.deepmergeCustom({ mergeOthers: returnsUndefined, },)(1, 2,),).toBe(undefined,);
        expect(target.deepmergeFastUnsafeCustom({ mergeOthers: returnsUndefined, },)(1, 2,),).toBe(undefined,);
      },
    },),
    it({
      name: 'implicit default merging falls back for mergeSets, mergeMaps, and mergeCircularReferences',
      // Kills 75 and 80 (deepmerge-fast.ts) and 500 (deepmerge.ts).
      fn: async () => {
        /**
         FastUnsafe merge whose custom Set and Map functions defer by returning undefined.
         */
        const fast = target.deepmergeFastUnsafeCustom({
          enableImplicitDefaultMerging: true,
          mergeMaps: returnsUndefined,
          mergeSets: returnsUndefined,
        },);
        expect([...fast(new Set([1,],), new Set([2,],),) as Set<unknown>,],).toEqual([1, 2,],);
        expect([...fast(new Map([['a', 1,],],), new Map([['b', 2,],],),) as Map<unknown, unknown>,],)
          .toEqual([['a', 1,], ['b', 2,],],);
        /**
         Merge of two self-referencing inputs whose custom cycle function defers.
         */
        const merged: unknown = target.deepmergeCustom({
          enableImplicitDefaultMerging: true,
          mergeCircularReferences: returnsUndefined,
        },)(cyclicSelf(1,), cyclicSelf(2,),);
        expect(Reflect.get(merged as object, 'self',),).toBe(merged,);
      },
    },),
    it({
      name: 'mergeCircularReferences returning actions.defaultMerge runs the default',
      // Kills 499 and 501 (deepmerge.ts).
      fn: async () => {
        /**
         Merge whose custom cycle function always requests the default.
         */
        const merged: unknown = target.deepmergeCustom({
          mergeCircularReferences: function requestDefault(
            _values: readonly unknown[],
            _depths: readonly number[],
            utils: ActionUtils,
          ) {
            return utils.actions.defaultMerge;
          },
        },)(cyclicSelf(1,), cyclicSelf(2,),);
        expect(Reflect.get(merged as object, 'self',),).toBe(merged,);
      },
    },),
    it({
      name: 'a custom mergeSets or mergeMaps replaces the default',
      // Kills 488 and 493 (deepmerge.ts) and 78 (deepmerge-fast.ts).
      fn: async () => {
        /**
         First Set and Map, which the first-value custom functions return.
         */
        const firstSet = new Set([1,],);
        /**
         First Map, which the first-value custom functions return.
         */
        const firstMap = new Map([['a', 1,],],);
        /**
         Custom functions returning the first value unmerged.
         */
        const firstWins = {
          mergeMaps: function firstMapWins(values: readonly unknown[],) {
            return values[0];
          },
          mergeSets: function firstSetWins(values: readonly unknown[],) {
            return values[0];
          },
        };
        expect(target.deepmergeCustom(firstWins,)(firstSet, new Set([2,],),),).toBe(firstSet,);
        expect(target.deepmergeCustom(firstWins,)(firstMap, new Map([['b', 2,],],),),).toBe(firstMap,);
        expect(target.deepmergeFastUnsafeCustom(firstWins,)(firstMap, new Map([['b', 2,],],),),).toBe(firstMap,);
      },
    },),
    it({
      name: 'into merge functions run the default only on request',
      // Kills 175, 180, and 181 (deepmerge-into-fast.ts: mergeOthers) and 324 (deepmerge-into.ts: mergeSets);
      // the Map case guards the matching mergeMaps wrapper.
      fn: async () => {
        /**
         Target whose leaf the custom mergeOthers overwrites itself.
         */
        const custom: Record<string, unknown> = { a: 1, };
        target.deepmergeIntoFastUnsafeCustom({
          mergeOthers: function writeCustom(slot: { value: unknown; },) {
            slot.value = 'custom';
          },
        },)(custom, { a: 2, },);
        expect(custom,).toEqual({ a: 'custom', },);
        /**
         Target whose leaf the custom mergeOthers hands back to the default.
         */
        const requested: Record<string, unknown> = { a: 1, };
        target.deepmergeIntoFastUnsafeCustom({
          mergeOthers: function writeDefaultMerge(slot: { value: unknown; }, _values: unknown, utils: IntoActionUtils,) {
            slot.value = utils.actions.defaultMerge;
          },
        },)(requested, { a: 2, },);
        expect(requested,).toEqual({ a: 2, },);
        /**
         Target whose Map a no-op custom mergeMaps leaves untouched.
         */
        const maps = { m: new Map([['a', 1,],],), };
        target.deepmergeIntoCustom({
          mergeMaps: function leaveMap() {
            // Deliberately does nothing and requests nothing.
          },
        },)(maps, { m: new Map([['b', 2,],],), },);
        expect([...maps.m.keys(),],).toEqual(['a',],);
        /**
         Target whose Set a no-op custom mergeSets leaves untouched.
         */
        const sets = { s: new Set(['a',],), };
        target.deepmergeIntoCustom({
          mergeSets: function leaveSet() {
            // Deliberately does nothing and requests nothing.
          },
        },)(sets, { s: new Set(['b',],), },);
        expect([...sets.s,],).toEqual(['a',],);
      },
    },),
    it({
      name: 'into mergeCircularReferences requesting the default, by return or by slot, runs it',
      // Kills 337, 338, 339, 341, 343, and 344 (deepmerge-into.ts).
      fn: async () => {
        /**
         Target lacking the cyclic key; the cycle function returns the action.
         */
        const returned: Record<string, unknown> = {};
        target.deepmergeIntoCustom({
          mergeCircularReferences: function returnDefaultMerge(
            _slot: { value: unknown; },
            _values: unknown,
            utils: IntoActionUtils,
          ) {
            return utils.actions.defaultMerge;
          },
        },)(returned, cyclicSelf(1,), cyclicSelf(2,),);
        // Only the default remaps the key onto the target itself.
        expect(returned.self,).toBe(returned,);
        /**
         Target lacking the cyclic key; the cycle function writes the action into its slot.
         */
        const written: Record<string, unknown> = {};
        target.deepmergeIntoCustom({
          mergeCircularReferences: function writeDefaultMerge(
            slot: { value: unknown; },
            _values: unknown,
            utils: IntoActionUtils,
          ) {
            slot.value = utils.actions.defaultMerge;
          },
        },)(written, cyclicSelf(1,), cyclicSelf(2,),);
        expect(written.self,).toBe(written,);
      },
    },),
    it({
      name: 'into filterValues false keeps undefined, and the FastUnsafe into default filters it',
      // Kills 197 and 199 (deepmerge-into.ts) and 99 (deepmerge-into-fast.ts).
      fn: async () => {
        /**
         Target of an unfiltered merge: undefined wins as the last value.
         */
        const unfiltered: Record<string, unknown> = { a: 1, };
        target.deepmergeIntoCustom({ filterValues: false, },)(unfiltered, { a: undefined, },);
        expect(Object.hasOwn(unfiltered, 'a',),).toBe(true,);
        expect(unfiltered.a,).toBe(undefined,);
        /**
         FastUnsafe into targets, where undefined must be neutral by default.
         */
        const fastTargets: Record<string, unknown>[] = [{ a: 1, }, { a: 1, },];
        target.deepmergeIntoFastUnsafe(fastTargets[0] ?? {}, { a: undefined, },);
        target.deepmergeIntoFastUnsafeCustom({},)(fastTargets[1] ?? {}, { a: undefined, },);
        expect(fastTargets,).toEqual([{ a: 1, }, { a: 1, },],);
      },
    },),
    it({
      name: 'rootMetaData reaches the root merge function',
      // Kills 358 (deepmerge.ts).
      fn: async () => {
        /**
         Root metadata the custom mergeOthers returns when it sees it.
         */
        const rootMeta = { tag: 'root', };
        /**
         Result of a leaf merge whose mergeOthers returns its metadata.
         */
        const seen: unknown = target.deepmergeCustom({
          mergeOthers: function returnMeta(_values: readonly unknown[], _utils: unknown, meta: unknown,) {
            return meta;
          },
        }, rootMeta,)(1, 2,);
        expect(seen,).toBe(rootMeta,);
      },
    },),
    it({
      name: 'actions.skip drops keys on the FastUnsafe three-record path and Map entries',
      // Kills 843 and 845 (defaults/vanilla-fast.ts) and 932 and 934 (defaults/vanilla.ts).
      fn: async () => {
        /**
         Custom mergeOthers skipping every leaf.
         */
        const skipLeaves = {
          mergeOthers: function skipAll(_values: readonly unknown[], utils: ActionUtils,) {
            return utils.actions.skip;
          },
        };
        expect(target.deepmergeFastUnsafeCustom(skipLeaves,)({ a: 1, }, { a: 2, }, { a: 3, },),).toEqual({},);
        /**
         Map merge whose only shared entry is skipped.
         */
        const map = target.deepmergeCustom(skipLeaves,)(new Map([['a', 1,],],), new Map([['a', 2,],],),);
        expect((map as Map<unknown, unknown>).size,).toBe(0,);
      },
    },),
    it({
      name: 'utils.mergeFunctions exposes only the merge functions, not other or unset options',
      // Kills 559 and 572 (defaults/general.ts).
      fn: async () => {
        /**
         Options with a non-function option and a merge function explicitly set to undefined, as a spread of
         partial options produces.
         */
        const options = {
          maxDepth: 5,
          mergeArrays: undefined,
          mergeOthers: function listKeys(_values: readonly unknown[], utils: { readonly mergeFunctions: object; },) {
            return Object.keys(utils.mergeFunctions,).toSorted();
          },
        };
        /**
         Keys a custom mergeOthers sees on utils.mergeFunctions.
         */
        const keys: unknown = target.deepmergeCustom(options as never,)(1, 2,);
        expect(keys,).toEqual([
          'mergeArrays',
          'mergeCircularReferences',
          'mergeMaps',
          'mergeOthers',
          'mergeRecords',
          'mergeSets',
        ],);
      },
    },),
  ],
},);
