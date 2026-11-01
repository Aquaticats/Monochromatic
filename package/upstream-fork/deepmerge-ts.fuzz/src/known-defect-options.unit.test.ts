/**
 Known divergences in the customization surface (`deepmergeCustom`,
 `deepmergeIntoCustom`, and the FastUnsafe custom variants).

 Same contract as `./known-defect.unit.test.ts`: each test asserts the current
 behaviour, so the suite stays green while upstream is unchanged and turns
 red when upstream changes it. Defects and intent questions are named as
 such; each comment names the cause and the generator region
 `./options-arbitraries.ts` or `./options-model.ts` leaves out or pins.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { IntoActionUtils, } from './mutation-helper.ts';
import { target, } from './target.ts';

/**
 Minimal utils view used by the custom functions below.
 */
type ActionUtils = { readonly actions: { readonly defaultMerge: symbol; readonly skip: symbol; }; };

/**
 Custom merge function that always returns `actions.skip`.

 @param _values - Ignored values.
 @param utils - Utils carrying the action symbols.

 @returns The skip action.

 @example
 ```ts
 deepmergeCustom({ mergeRecords: alwaysSkip, });
 ```
 */
function alwaysSkip(_values: unknown, utils: ActionUtils,): symbol {
  return utils.actions.skip;
}

await describe({
  name: 'deepmerge-ts custom option divergences still reproduce',
  children: [
    it({
      name: 'defect: actions.skip returned at the root leaks the internal symbol to the caller',
      // Excluded region: planned skips return actions.defaultMerge when the call has no metadata (src/options-model.ts).
      // Cause: mergeUnknowns in src/deepmerge.ts (and mergeUnknownsFast) return the merge function's result as is;
      // only the default record and Map merges in src/defaults/vanilla.ts interpret actions.skip.
      fn: async () => {
        /**
         Skip symbol the library exposes to custom functions.
         */
        const skip: unknown = target.deepmergeCustom({ mergeOthers: alwaysSkip, },)(1, 2,);
        expect(typeof skip,).toBe('symbol',);
        expect(String(skip,),).toBe('Symbol(deepmerge-ts: skip)',);
        expect(target.deepmergeCustom({ mergeRecords: alwaysSkip, },)({ a: 1, }, { b: 2, },),).toBe(skip,);
        expect(target.deepmergeCustom({ mergeOthers: alwaysSkip, },)({ a: 1, },),).toBe(skip,);
        expect(target.deepmergeFastUnsafeCustom({ mergeRecords: alwaysSkip, },)({ a: 1, }, { b: 2, },),).toBe(skip,);
      },
    },),
    it({
      name: 'defect: an into call whose root merge goes to mergeOthers silently leaves the target unchanged',
      // Excluded region: into plans never set mergeRecords to false or maxDepth to 0 (src/options-arbitraries.ts).
      // Cause: customizedDeepmergeInto in src/deepmerge-into.ts wraps the target as { value: target }; mergeOthersInto
      // in src/defaults/general.ts only reassigns wrapper.value, and the wrapper is dropped.
      fn: async () => {
        /**
         Target of `mergeRecords: false`; `deepmergeCustom` gives `{ b: 2 }`.
         */
        const recordsOff: Record<string, unknown> = { a: 1, };
        target.deepmergeIntoCustom({ mergeRecords: false, },)(recordsOff, { b: 2, },);
        expect(recordsOff,).toEqual({ a: 1, },);
        expect(target.deepmergeCustom({ mergeRecords: false, },)({ a: 1, }, { b: 2, },),).toEqual({ b: 2, },);

        /**
         Target of `maxDepth: 0`.
         */
        const depthZero: Record<string, unknown> = { a: 1, };
        target.deepmergeIntoCustom({ maxDepth: 0, },)(depthZero, { b: 2, },);
        expect(depthZero,).toEqual({ a: 1, },);

        /**
         Array target of `mergeArrays: false`.
         */
        const arraysOff = [1,];
        target.deepmergeIntoCustom({ mergeArrays: false, },)(arraysOff, [2,],);
        expect(arraysOff,).toEqual([1,],);

        /**
         FastUnsafe target of `mergeRecords: false`.
         */
        const fastOff: Record<string, unknown> = { a: 1, };
        target.deepmergeIntoFastUnsafeCustom({ mergeRecords: false, },)(fastOff, { b: 2, },);
        expect(fastOff,).toEqual({ a: 1, },);
      },
    },),
    it({
      name: 'intent question: an option set to false uses the default mergeOthers, not a custom one',
      // Pinned by the model: `false` resolves to the last value (src/options-model.ts, dispatch).
      // Cause: resolveCustomMergeFunctions in src/defaults/general.ts maps false to defaultMergeFunctions.mergeOthers.
      fn: async () => {
        /**
         Custom mergeOthers that marks every value it handles.
         */
        const marked = target.deepmergeCustom({
          mergeArrays: false,
          mergeOthers: function marker() {
            return 'custom mergeOthers';
          },
        },);
        expect(marked({ a: [1,], }, { a: [2,], },),).toEqual({ a: [2,], },);
        expect(marked({ n: 1, }, { n: 2, },),).toEqual({ n: 'custom mergeOthers', },);
      },
    },),
    it({
      name: 'intent question: a container in only one input, or at maxDepth, never reaches its custom merge function',
      // Pinned by the model: one value, or depth >= maxDepth, goes to mergeOthers (src/options-model.ts, mergeAt).
      // Cause: mergeUnknowns in src/deepmerge.ts calls mergeOthers for a single value and at the depth limit.
      fn: async () => {
        /**
         De-duplicating array merge, a typical custom mergeArrays.
         */
        const dedupe = target.deepmergeCustom({
          mergeArrays: function unique(values: readonly (readonly unknown[])[],) {
            return [...new Set(values.flat(),),];
          },
        },);
        expect(dedupe({ a: [1, 1,], }, { a: [2, 2,], },),).toEqual({ a: [1, 2,], },);
        // The same function never sees an array present in only one input.
        expect(dedupe({ a: [1, 1,], }, { b: [2, 2,], },),).toEqual({ a: [1, 1,], b: [2, 2,], },);

        /**
         Kinds mergeOthers received, recorded to show it gets containers.
         */
        const seen: string[] = [];
        target.deepmergeCustom({
          maxDepth: 1,
          mergeOthers: function record(values: readonly unknown[], utils: ActionUtils,) {
            seen.push(values.map(function kindName(value,) {
              return Array.isArray(value,) ? 'array' : typeof value;
            },).join(',',),);
            return utils.actions.defaultMerge;
          },
        },)({ r: { x: 1, }, l: [1,], }, { r: { y: 2, }, },);
        expect(seen,).toEqual(['object,object', 'array',],);
      },
    },),
    it({
      name: 'intent question: invalid maxDepth values silently fall back to 1000',
      // Pinned by the model: effectiveMaxDepth (src/options-model.ts).
      // Cause: getUtils in src/deepmerge.ts accepts only a non-NaN number >= 0 and otherwise uses 1000.
      fn: async () => {
        for (const maxDepth of [-1, Number.NaN, '1',]) {
          expect(
            target.deepmergeCustom({ maxDepth: maxDepth as number, },)({ r: { x: 1, }, }, { r: { y: 2, }, },),
          )
            .toEqual({ r: { x: 1, y: 2, }, },);
        }
      },
    },),
    it({
      name: 'intent question: mergeCircularReferences false makes the result reference an input',
      // Generated by ./alias-options.property.unit.test.ts, which holds false to a last-value cycle function.
      // Cause: false resolves to the default mergeOthers, whose last value is the input's own cyclic object.
      fn: async () => {
        /**
         First cyclic input.
         */
        const first: Record<string, unknown> = { v: 1, };
        first.self = first;
        /**
         Second cyclic input, whose self-reference the result ends up holding.
         */
        const second: Record<string, unknown> = { w: 2, };
        second.self = second;
        /**
         Result of merging with cycle resolution off.
         */
        const merged = target.deepmergeCustom({ mergeCircularReferences: false, },)(first, second,) as Record<string, unknown>;
        expect(merged.self,).toBe(second,);
        expect(merged.self,).not.toBe(merged,);
      },
    },),
    it({
      name: 'intent question: deepmergeInto never passes a Map entry only the target holds to a custom merge function',
      // Excluded region: the into model reports such entries as excluded when the custom function would change
      // them (src/options-model.ts). Found by a fresh-seed round of ./options.property.unit.test.ts (seed 101).
      // Cause: mergeMapsInto (src/defaults/into.ts) collects entries from the source Maps only, skipping the
      // target's own Map, while mergeRecordsInto visits every key of every value, the target's included, and
      // deepmerge's Map merge visits every key of every Map.
      fn: async () => {
        /**
         Custom into mergeOthers that marks every position it sees.
         */
        const markOthers = {
          mergeOthers: function mark(slot: { value: unknown; },): void {
            slot.value = 'custom';
          },
        };
        /**
         Target holding a key and a Map entry that the source lacks.
         */
        const into = {
          m: new Map([['x', 1,],],),
          r: { x: 1, },
        };
        target.deepmergeIntoCustom(markOthers,)(
          into,
          {
            m: new Map(),
            r: {},
          },
        );
        expect(into.r.x,).toBe('custom',);
        expect(into.m.get('x',),).toBe(1,);
        /**
         Same target shape for the FastUnsafe into variant.
         */
        const fastInto = {
          m: new Map([['x', 1,],],),
          r: { x: 1, },
        };
        target.deepmergeIntoFastUnsafeCustom(markOthers,)(
          fastInto,
          {
            m: new Map(),
            r: {},
          },
        );
        expect(fastInto.r.x,).toBe('custom',);
        expect(fastInto.m.get('x',),).toBe(1,);
        /**
         deepmerge with the matching returning function reaches the single-Map entry.
         */
        const merged = target.deepmergeCustom({
          mergeOthers: function mark(): string {
            return 'custom';
          },
        },)(
          {
            m: new Map([['x', 1,],],),
          },
          { m: new Map(), },
        ) as { readonly m: ReadonlyMap<string, unknown>; };
        expect(merged.m.get('x',),).toBe('custom',);
      },
    },),
    it({
      name: 'intent question: an into array, Set, or Map function writing actions.defaultMerge into the slot leaves the symbol',
      // Not generated: `slotDefault` in ./options-plan.ts is offered to into mergeOthers only. Cause:
      // mergeOthersInto and mergeCircularReferencesInto (src/deepmerge-into.ts) also honour the action written
      // into the slot, while mergeArraysInto, mergeSetsInto, and mergeMapsInto only honour a returned action.
      fn: async () => {
        /**
         Into merge functions that request the default by writing the action into the slot.
         */
        const slotDefaults = (['mergeArrays', 'mergeSets', 'mergeMaps',] as const).map(function slotDefault(name,) {
          return {
            [name]: function writeDefault(slot: { value: unknown; }, _values: unknown, utils: IntoActionUtils,): void {
              slot.value = utils.actions.defaultMerge;
            },
          };
        },);
        /**
         Targets after each call, keyed like the functions.
         */
        const results = slotDefaults.map(function mergeWith(options,) {
          /**
           Target holding one container of each kind.
           */
          const into: Record<string, unknown> = {
            a: [1,],
            m: new Map([[1, 1,],],),
            s: new Set([1,],),
          };
          target.deepmergeIntoCustom(options,)(
            into,
            {
              a: [2,],
              m: new Map([[2, 2,],],),
              s: new Set([2,],),
            },
          );
          return into;
        },);
        expect(results.map(function kept(into, index,) {
          return typeof into[['a', 's', 'm',][index] ?? ''];
        },),).toEqual(['symbol', 'symbol', 'symbol',],);
        /**
         mergeOthers honours the same slot write.
         */
        const others: Record<string, unknown> = { a: 1, };
        target.deepmergeIntoCustom({
          mergeOthers: function writeDefault(slot: { value: unknown; }, _values: unknown, utils: IntoActionUtils,): void {
            slot.value = utils.actions.defaultMerge;
          },
        },)(others, { a: 2, },);
        expect(others.a,).toBe(2,);
      },
    },),
  ],
},);
