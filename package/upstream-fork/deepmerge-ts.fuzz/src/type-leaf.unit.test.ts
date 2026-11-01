/**
 Type-level tests for leaf objects, unions, and special types through
 `deepmerge`.

 Leaf objects (dates, regular expressions, errors) are never merged: the last value
 wins in both the type and the runtime. Unions distribute when every member
 shares a kind. Divergences live in `./type-known-defect.unit.test.ts`.

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
 Return the argument typed as `T`, so a union-typed test value is not
 narrowed to its initializer's member by control-flow analysis.

 @param value - Value to widen.

 @returns Same value, typed as the full union.

 @example
 ```ts
 const either = widen<number | string>(1); // number | string
 ```
 */
function widen<T,>(value: T,): T {
  return value;
}

await describe({
  name: 'deepmerge leaf and union result types',
  children: [
    it({
      name: 'dates are leaves in both directions',
      fn: async () => {
        const when = new Date(0,);
        const dateThenRecord = target.deepmerge({ a: when, }, { a: { w: 2, }, },);
        expectTypeOf(dateThenRecord,).toEqualTypeOf<{ a: { w: number; }; }>();
        expect(dateThenRecord,).toEqual({ a: { w: 2, }, },);

        const recordThenDate = target.deepmerge({ a: { w: 2, }, }, { a: when, },);
        expectTypeOf(recordThenDate,).toEqualTypeOf<{ a: Date; }>();
        expect(recordThenDate.a,).toBe(when,);
      },
    },),
    it({
      name: 'regular expressions and errors are leaves',
      fn: async () => {
        // oxlint-disable-next-line no-restricted-syntax/no-regex -- a RegExp object is the leaf input under test; it never matches anything here.
        const pattern = /deepmerge/gu;
        // oxlint-disable-next-line no-restricted-syntax/no-regex -- RegExp objects are leaf inputs under test; they never match anything here.
        const merged = target.deepmerge({ p: /other/u, e: new Error('first',), }, { p: pattern, e: new TypeError('second',), },);
        expectTypeOf(merged,).toEqualTypeOf<{ p: RegExp; e: TypeError; }>();
        expect(merged.p,).toBe(pattern,);
      },
    },),
    it({
      name: 'null and top-level scalars resolve to the last type',
      fn: async () => {
        const recordThenNull = target.deepmerge({ a: 1, }, null,);
        expectTypeOf(recordThenNull,).toEqualTypeOf<null>();
        expect(recordThenNull,).toBeNull();

        const nullThenRecord = target.deepmerge(null, { a: 1, },);
        expectTypeOf(nullThenRecord,).toEqualTypeOf<{ a: number; }>();

        const scalars = target.deepmerge(1, 'two', false,);
        expectTypeOf(scalars,).toEqualTypeOf<boolean>();
        expect(scalars,).toBe(false,);
      },
    },),
    it({
      name: 'unions of one array kind distribute over each member',
      fn: async () => {
        const numbersOrStrings = widen<number[] | string[]>([1,],);
        const merged = target.deepmerge(numbersOrStrings, [true,],);
        expectTypeOf(merged,).toEqualTypeOf<(string | boolean)[] | (number | boolean)[]>();
        expect(merged,).toEqual([1, true,],);
      },
    },),
    it({
      name: 'a scalar-or-record earlier value yields the later record type',
      fn: async () => {
        const recordOrString = widen<{ x: number; } | string>({ x: 1, },);
        const merged = target.deepmerge({ a: recordOrString, }, { a: { y: 1, }, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: { y: number; }; }>();
        // The runtime value also carries `x`; extra keys keep it assignable.
        expect(merged,).toEqual({ a: { x: 1, y: 1, }, },);

        const laterUnion = target.deepmerge({ a: { x: 1, }, }, { a: recordOrString, },);
        expectTypeOf(laterUnion,).toEqualTypeOf<{ a: string | { x: number; }; }>();
      },
    },),
    it({
      name: 'a later scalar-or-undefined value widens instead of replacing',
      fn: async () => {
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- the nullable input type is the case under test.
        const maybeString = widen<string | undefined>(undefined,);
        const merged = target.deepmerge({ a: 1, }, { a: maybeString, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: string | number; }>();
        expect(merged,).toEqual({ a: 1, },);
      },
    },),
    it({
      name: 'unknown inputs make the whole result unknown',
      fn: async () => {
        const opaque: unknown = { b: 2, };
        const merged = target.deepmerge({ a: 1, }, opaque,);
        expectTypeOf(merged,).toEqualTypeOf<unknown>();
        expect(merged,).toEqual({ a: 1, b: 2, },);
      },
    },),
    it({
      name: 'undefined between two tuples is skipped in the type',
      fn: async () => {
        const merged = target.deepmerge({ a: [1,] as const, }, { a: undefined, }, { a: ['z',] as const, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: [1, 'z',]; }>();
        expect(merged,).toEqual({ a: [1, 'z',], },);
      },
    },),
    it({
      name: 'deep optional arrays concatenate and stay optional',
      fn: async () => {
        const partial: { a: { x?: number[]; }; } = { a: { x: [1,], }, };
        const merged = target.deepmerge(partial, { a: { x: ['s',], }, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: { x?: (string | number)[]; }; }>();
        expect(merged,).toEqual({ a: { x: [1, 's',], }, },);
      },
    },),
  ],
},);
