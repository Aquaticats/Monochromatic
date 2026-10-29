/**
 Behavioural claims in upstream docs/API.md, the README, and the TSDoc in
 `src/`, each run against the build under test.

 `conformance:` cases hold and guard the claim. `docs error:` and
 `docs imprecision:` cases pin where the build does something other than the
 text says, with the same contract as `./known-defect.unit.test.ts`: green
 while upstream is unchanged, red when behaviour or the claim's subject changes.

 Checked against deepmerge-ts 8.0.2 (docs at upstream `17fc99cb`), 2026-09-24.

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
 Record that refers to itself, the smallest cyclic input.

 @returns Fresh `{ v: 1, self: <itself> }`.

 @example
 ```ts
 const node = selfLoop();
 node.self === node; // true
 ```
 */
function selfLoop(): Record<string, unknown> {
  /**
   Node under construction.
   */
  const node: Record<string, unknown> = { v: 1, };
  node.self = node;
  return node;
}

await describe({
  name: 'deepmerge-ts documentation claims',
  children: [
    it({
      name: 'docs error: the deepmergeCustom TSDoc example says concatenation "instead of last-wins", but the default already concatenates',
      // src/deepmerge.ts deepmergeCustom @example: `mergeArrays: (values) => values.flat()` under the
      // comment "Merge arrays by concatenation instead of last-wins"; mergeArrays in
      // src/defaults/vanilla.ts is already `values.flat()`, and README's default example prints a
      // concatenated array.
      fn: async () => {
        /**
         The TSDoc example's merge function.
         */
        const concatenate = target.deepmergeCustom({
          mergeArrays: function flat(values,) {
            return values.flat();
          },
        },);
        expect(concatenate({ tags: ['a',], }, { tags: ['b',], },),).toEqual({ tags: ['a', 'b',], },);
        expect(target.deepmerge({ tags: ['a',], }, { tags: ['b',], },),).toEqual({ tags: ['a', 'b',], },);
      },
    },),
    it({
      name: 'docs error: objectHasProperty answers false for own non-enumerable properties and for functions',
      // API.md: "Returns whether the given object has the given property."
      // src/utils.ts: `typeof object === "object" && propertyIsEnumerable.call(object, property)`.
      fn: async () => {
        /**
         Record with one non-enumerable own property.
         */
        const hidden = Object.defineProperty({}, 'h', { enumerable: false, value: 1, },);
        expect(Object.hasOwn(hidden, 'h',),).toBe(true,);
        expect(target.objectHasProperty(hidden, 'h',),).toBe(false,);
        /**
         Function with an own enumerable property.
         */
        const callable = Object.assign(function noop() {}, { p: 1, },);
        expect(Object.hasOwn(callable, 'p',),).toBe(true,);
        expect(target.objectHasProperty(callable, 'p',),).toBe(false,);
        expect(target.objectHasProperty({ a: 1, }, 'a',),).toBe(true,);
      },
    },),
    it({
      name: 'docs imprecision: FastUnsafe handles one cyclic input; only cycles met in two inputs at one position overflow',
      // API.md and each FastUnsafe TSDoc: "Circular structures will result in a stack overflow."
      // A single cyclic value is passed through by reference, since nothing else merges with it.
      fn: async () => {
        /**
         Cyclic input.
         */
        const loop = selfLoop();
        /**
         Result of merging the cyclic input with a plain one.
         */
        const merged = target.deepmergeFastUnsafe(loop, { v: 2, },);
        expect(merged.v,).toBe(2,);
        expect(merged.self,).toBe(loop,);
        expect(function bothCyclic() {
          return target.deepmergeFastUnsafe(selfLoop(), selfLoop(),);
        },).toThrow(RangeError,);
      },
    },),
    it({
      name: 'docs gap: FastUnsafe custom merge functions receive undefined meta, so meta-based examples cannot fire',
      // API.md says the FastUnsafe custom variants take the same options except metaDataUpdater,
      // maxDepth, and mergeCircularReferences, and "No metadata tracking"; it does not say that
      // `meta` is always undefined (typed and at runtime), so deepmergeCustom.md's skipme example, which
      // reads `meta?.key`, neither compiles against these types nor skips when forced through.
      fn: async () => {
        /**
         Metadata each call received.
         */
        const seen: unknown[] = [];
        /**
         Merge that records the metadata it receives.
         */
        const recordMeta = target.deepmergeFastUnsafeCustom({
          mergeOthers: function record(_values, utils, meta,) {
            expectTypeOf(meta,).toEqualTypeOf<undefined>();
            seen.push(meta,);
            return utils.actions.defaultMerge;
          },
        },);
        expect(recordMeta({ skipme: new Date(1,), }, { skipme: new Date(2,), },),).toEqual({ skipme: new Date(2,), },);
        expect(seen,).toEqual([undefined,],);
        // The standard variant hands the same function the key.
        target.deepmergeCustom({
          mergeOthers: function record(_values, utils, meta,) {
            seen.push(meta?.key,);
            return utils.actions.defaultMerge;
          },
        },)({ skipme: 1, }, { skipme: 2, },);
        expect(seen,).toEqual([undefined, 'skipme',],);
      },
    },),
    it({
      name: 'conformance: into merge functions see the target value first and only actions.defaultMerge',
      // deepmergeCustom.md: "values includes all the values, including the target's value";
      // API.md: DeepMergeIntoUtils actions has only defaultMerge.
      fn: async () => {
        /**
         Values and action names each call received.
         */
        const seen: unknown[] = [];
        target.deepmergeIntoCustom({
          mergeOthers: function record(_slot, values, utils,) {
            seen.push([[...values,], Object.keys(utils.actions,),],);
            return utils.actions.defaultMerge;
          },
        },)({ a: 1, }, { a: 2, }, { a: 3, },);
        expect(seen,).toEqual([[[1, 2, 3,], ['defaultMerge',],],],);
      },
    },),
    it({
      name: 'conformance: rootMetaData reaches root merges and utils.deepmerge is the customized function',
      // API.md: "The given meta data value will be passed to root level merges" and
      // "deepmerge: This is your top level customized deepmerge function".
      fn: async () => {
        /**
         Metadata and utils the root merge received.
         */
        const seen: unknown[] = [];
        /**
         Customized function under test.
         */
        const merge = target.deepmergeCustom(
          {
            mergeOthers: function record(values, utils, meta,) {
              seen.push(meta, utils.deepmerge,);
              return values.at(-1,);
            },
          },
          { key: 'root', },
        );
        merge(1, 2,);
        expect(seen,).toEqual([{ key: 'root', }, merge,],);
      },
    },),
  ],
},);
