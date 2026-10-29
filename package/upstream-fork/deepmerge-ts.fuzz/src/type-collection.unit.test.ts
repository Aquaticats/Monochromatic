/**
 Type-level tests for arrays, tuples, Sets, and Maps through `deepmerge`.

 Static result types are pinned with `expectTypeOf` (checked by
 `lint:types`) beside the runtime value. Divergences live in
 `./type-known-defect.unit.test.ts`.

 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

await describe({
  name: 'deepmerge collection result types',
  children: [
    it({
      name: 'mutable arrays concatenate into an array of the element union',
      fn: async () => {
        const numbers: number[] = [1,];
        const strings: string[] = ['a',];
        const merged = target.deepmerge(numbers, strings,);
        expectTypeOf(merged,).toEqualTypeOf<(number | string)[]>();
        expect(merged,).toEqual([1, 'a',],);
      },
    },),
    it({
      name: 'const tuples concatenate into a tuple',
      fn: async () => {
        const merged = target.deepmerge([1, 2,] as const, ['a',] as const, [true,] as const,);
        expectTypeOf(merged,).toEqualTypeOf<[1, 2, 'a', true,]>();
        expect(merged,).toEqual([1, 2, 'a', true,],);
      },
    },),
    it({
      name: 'a tuple followed by an array spreads the array into the rest',
      fn: async () => {
        const rest: number[] = [3, 4,];
        const merged = target.deepmerge(['x',] as const, rest,);
        expectTypeOf(merged,).toEqualTypeOf<['x', ...number[],]>();
        expect(merged,).toEqual(['x', 3, 4,],);
      },
    },),
    it({
      name: 'readonly arrays concatenate into a mutable array',
      fn: async () => {
        const first: readonly number[] = [1,];
        const second: readonly string[] = ['b',];
        const merged = target.deepmerge(first, second,);
        expectTypeOf(merged,).toEqualTypeOf<(number | string)[]>();
      },
    },),
    it({
      name: 'arrays nested in records concatenate per key',
      fn: async () => {
        const merged = target.deepmerge({ list: [1,], }, { list: ['two',], },);
        expectTypeOf(merged,).toEqualTypeOf<{ list: (number | string)[]; }>();
        expect(merged,).toEqual({ list: [1, 'two',], },);
      },
    },),
    it({
      name: 'arrays of records are concatenated, never merged element-wise',
      fn: async () => {
        const merged = target.deepmerge([{ a: 1, },], [{ b: 'b', },],);
        expectTypeOf(merged,).toEqualTypeOf<({ a: number; } | { b: string; })[]>();
        expect(merged,).toEqual([{ a: 1, }, { b: 'b', },],);
      },
    },),
    it({
      name: 'Sets union their element types',
      fn: async () => {
        const merged = target.deepmerge(new Set([1,],), new Set(['a',],),);
        expectTypeOf(merged,).toEqualTypeOf<Set<number | string>>();
        expect([...merged,],).toEqual([1, 'a',],);
      },
    },),
    it({
      name: 'readonly Sets still produce a mutable Set',
      fn: async () => {
        const first: ReadonlySet<number> = new Set([1,],);
        const second: ReadonlySet<boolean> = new Set([true,],);
        const merged = target.deepmerge(first, second,);
        expectTypeOf(merged,).toEqualTypeOf<Set<number | boolean>>();
      },
    },),
    it({
      name: 'Maps union key and value types',
      fn: async () => {
        const merged = target.deepmerge(new Map([['k', 1,],],), new Map([[2, 'v',],],),);
        expectTypeOf(merged,).toEqualTypeOf<Map<string | number, number | string>>();
        expect([...merged,],).toEqual([['k', 1,], [2, 'v',],],);
      },
    },),
    it({
      name: 'identical Map types short-circuit to that type',
      fn: async () => {
        const first = new Map<string, { a: number; }>([['k', { a: 1, },],],);
        const second = new Map<string, { a: number; }>([['k', { a: 2, },],],);
        const merged = target.deepmerge(first, second,);
        expectTypeOf(merged,).toEqualTypeOf<Map<string, { a: number; }>>();
        expect(merged.get('k',),).toEqual({ a: 2, },);
      },
    },),
    it({
      name: 'a collection kind mismatch takes the last type',
      fn: async () => {
        const arrayThenSet = target.deepmerge({ v: [1,], }, { v: new Set(['s',],), },);
        expectTypeOf(arrayThenSet,).toEqualTypeOf<{ v: Set<string>; }>();
        expect(arrayThenSet.v,).toBeInstanceOf(Set,);

        const setThenMap = target.deepmerge({ v: new Set([1,],), }, { v: new Map([[1, 1,],],), },);
        expectTypeOf(setThenMap,).toEqualTypeOf<{ v: Map<number, number>; }>();

        const recordThenArray = target.deepmerge({ v: { a: 1, }, }, { v: [true,], },);
        expectTypeOf(recordThenArray,).toEqualTypeOf<{ v: boolean[]; }>();
        expect(recordThenArray,).toEqual({ v: [true,], },);
      },
    },),
    it({
      name: 'empty arrays and Sets contribute nothing to the element type',
      fn: async () => {
        const merged = target.deepmerge([] as never[], [1,],);
        expectTypeOf(merged,).toEqualTypeOf<number[]>();
        const sets = target.deepmerge(new Set<never>(), new Set([1,],),);
        expectTypeOf(sets,).toEqualTypeOf<Set<number>>();
      },
    },),
  ],
},);
