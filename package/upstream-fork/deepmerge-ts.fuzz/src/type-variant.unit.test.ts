/**
 Type-level tests for the non-default entry points: `deepmergeInto`'s
 assertion, the `FastUnsafe` variants, and `deepmergeCustom` with custom
 merge-function URIs and `DeepMergeNoFilteringURI`.

 Divergences live in `./type-known-defect.unit.test.ts`.

 @module
 */

import type {
  DeepMergeFunctionsURIs,
  DeepMergeLeaf,
  DeepMergeNoFilteringURI,
} from 'deepmerge-ts';

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

/**
 Registers a custom result-type URI the way `docs/deepmergeCustom.md` shows:
 by augmenting `DeepMergeFunctionURItoKind`.
 */
declare module 'deepmerge-ts' {
  // oxlint-disable-next-line typescript/consistent-type-definitions -- declaration merging needs an interface.
  interface DeepMergeFunctionURItoKind<Ts extends readonly unknown[], Fs extends DeepMergeFunctionsURIs, in out M> {
    /**
     Last value wins, used by the custom `mergeArrays` case.
     */
    readonly FuzzSidecarLastValueURI: DeepMergeLeaf<Ts, Fs, M>;
  }
}

await describe({
  name: 'deepmerge-ts variant entry point types',
  children: [
    it({
      name: 'deepmergeInto asserts the target gains the merged keys',
      fn: async () => {
        const merged = { a: 1, };
        target.deepmergeInto(merged, { b: 'x', },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: number; b: string; }>();
        expect(merged,).toEqual({ a: 1, b: 'x', },);
      },
    },),
    it({
      name: 'deepmergeInto with same-typed values keeps the target type',
      fn: async () => {
        const counts: { a: number; } = { a: 1, };
        target.deepmergeInto(counts, { a: 2, },);
        expectTypeOf(counts,).toEqualTypeOf<{ a: number; }>();
        expect(counts,).toEqual({ a: 2, },);
      },
    },),
    it({
      name: 'deepmergeInto returns void',
      fn: async () => {
        expectTypeOf(target.deepmergeInto({ a: 1, }, { b: 2, },),).toEqualTypeOf<void>();
      },
    },),
    it({
      name: 'deepmergeIntoFastUnsafe asserts nested keys as an intersection',
      fn: async () => {
        const nested = { r: { x: 1, }, };
        target.deepmergeIntoFastUnsafe(nested, { r: { y: 's', }, },);
        expectTypeOf(nested,).toEqualTypeOf<{ r: { x: number; } & { x: number; y: string; }; }>();
        expect(nested,).toEqual({ r: { x: 1, y: 's', }, },);
      },
    },),
    it({
      name: 'deepmergeFastUnsafe types match deepmerge for records and tuples',
      fn: async () => {
        const tuples = target.deepmergeFastUnsafe({ a: [1,] as const, }, { a: ['s',] as const, },);
        expectTypeOf(tuples,).toEqualTypeOf<{ a: [1, 's',]; }>();
        expectTypeOf(tuples,).toEqualTypeOf<ReturnType<typeof target.deepmerge<[{ a: readonly [1,]; }, { a: readonly ['s',]; },]>>>();
        expect(tuples,).toEqual({ a: [1, 's',], },);

        const records = target.deepmergeFastUnsafe({ a: 1, }, { b: 'b', }, { a: true, },);
        expectTypeOf(records,).toEqualTypeOf<{ a: boolean; b: string; }>();
      },
    },),
    it({
      name: 'a custom mergeArrays with a registered URI changes the array result type',
      fn: async () => {
        const lastArray = target.deepmergeCustom<unknown, { DeepMergeArraysURI: 'FuzzSidecarLastValueURI'; }>({
          mergeArrays: (values,) => values.at(-1,),
        },);
        const merged = lastArray({ a: [1,], }, { a: ['s',], },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: string[]; }>();
        expect(merged,).toEqual({ a: ['s',], },);
      },
    },),
    it({
      name: 'DeepMergeNoFilteringURI keeps undefined in the type when filterValues is false',
      fn: async () => {
        const unfiltered = target.deepmergeCustom<unknown, { DeepMergeFilterValuesURI: DeepMergeNoFilteringURI; }>({
          filterValues: false,
        },);
        const merged = unfiltered({ a: 1, }, { a: undefined, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: undefined; }>();
        expect(merged,).toEqual({ a: undefined, },);
      },
    },),
    it({
      name: 'deepmergeCustom with no options types like deepmerge',
      fn: async () => {
        const plain = target.deepmergeCustom({},);
        const merged = plain({ a: [1,], }, { a: ['s',], b: new Set([1,],), },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: (string | number)[]; b: Set<number>; }>();
      },
    },),
  ],
},);
