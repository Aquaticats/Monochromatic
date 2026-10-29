/**
 Type-level divergences between deepmerge-ts's result types and its runtime
 results.

 Unsound cases pin a static type the runtime value contradicts; imprecise
 cases pin a type wider than any runtime value. Each pins the current type
 with `expectTypeOf` and the contradicting runtime value with `expect`, so
 `lint:types` fails when upstream changes the type and `test:unit` fails when
 the runtime changes. On failure after an upgrade: confirm the fix, move the
 case into the matching `./type-*.unit.test.ts` file with the corrected type.

 Upstream reporting: the combined issue draft, held locally for the user.

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
 Return the argument typed as `T`, keeping a union-typed value from being
 narrowed to its initializer.

 @param value - Value to widen.

 @returns Same value, typed as `T`.

 @example
 ```ts
 const either = widen<number[] | undefined>([1,]);
 ```
 */
function widen<T,>(value: T,): T {
  return value;
}

await describe({
  name: 'deepmerge-ts type-level known divergences',
  children: [
    it({
      name: 'unsound: a string index signature absorbs a later known key of another type',
      fn: async () => {
        const dictionary: Record<string, number> = { a: 1, };
        const merged = target.deepmerge(dictionary, { b: 'b', },);
        expectTypeOf(merged,).toEqualTypeOf<Record<string, number>>();
        // Typed number, holds a string.
        expect(merged.b,).toBe('b',);
      },
    },),
    it({
      name: 'unsound: a union-typed array value is replaced in the type but concatenated at runtime',
      fn: async () => {
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- the nullable input type is the case under test.
        const maybeNumbers = widen<number[] | undefined>([1,],);
        const merged = target.deepmerge({ a: maybeNumbers, }, { a: ['s',], },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: string[]; }>();
        expect(merged.a,).toEqual([1, 's',],);

        const arrayOrRecord = widen<number[] | { k: 1; }>([1,],);
        const mixed = target.deepmerge({ a: arrayOrRecord, }, { a: ['s',], },);
        expectTypeOf(mixed,).toEqualTypeOf<{ a: string[]; }>();
        expect(mixed.a,).toEqual([1, 's',],);
      },
    },),
    it({
      name: 'unsound: a union-typed Set or nested record value is replaced in the type but merged at runtime',
      fn: async () => {
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- the nullable input type is the case under test.
        const maybeSet = widen<Set<number> | undefined>(new Set([1,],),);
        const sets = target.deepmerge({ a: maybeSet, }, { a: new Set(['s',],), },);
        expectTypeOf(sets,).toEqualTypeOf<{ a: Set<string>; }>();
        expect([...sets.a,],).toEqual([1, 's',],);

        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- the nullable input type is the case under test.
        const maybeRecord = widen<{ x: number[]; } | undefined>({ x: [1,], },);
        const records = target.deepmerge({ a: maybeRecord, }, { a: { x: ['s',], }, },);
        expectTypeOf(records,).toEqualTypeOf<{ a: { x: string[]; }; }>();
        expect(records.a.x,).toEqual([1, 's',],);
      },
    },),
    it({
      name: 'unsound: an optional record key followed by a required one claims the earlier nested keys',
      fn: async () => {
        const optionalRecord = widen<{ a?: { x: number; }; }>({},);
        const merged = target.deepmerge(optionalRecord, { a: { y: 's', }, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a?: { x: number; y: string; }; }>();
        // `x` is typed as present, and `a` as optional, yet the runtime has `a` without `x`.
        expect(merged,).toEqual({ a: { y: 's', }, },);
      },
    },),
    it({
      name: 'unsound: a custom maxDepth is not reflected in the result type',
      fn: async () => {
        const later = { y: 2, };
        const merged = target.deepmergeCustom({ maxDepth: 1, },)({ r: { x: 1, }, }, { r: later, },);
        expectTypeOf(merged,).toEqualTypeOf<{ r: { x: number; y: number; }; }>();
        expect(merged.r,).toBe(later,);
      },
    },),
    it({
      name: 'unsound: mergeRecords false and filterValues false are not reflected in the result type',
      fn: async () => {
        const recordsReplaced = target.deepmergeCustom({ mergeRecords: false, },)({ r: { x: 1, }, }, { r: { y: 2, }, },);
        expectTypeOf(recordsReplaced,).toEqualTypeOf<{ r: { x: number; y: number; }; }>();
        expect(recordsReplaced,).toEqual({ r: { y: 2, }, },);

        const unfiltered = target.deepmergeCustom({ filterValues: false, },)({ a: 1, }, { a: undefined, },);
        expectTypeOf(unfiltered,).toEqualTypeOf<{ a: number; }>();
        expect(unfiltered,).toEqual({ a: undefined, },);
      },
    },),
    it({
      name: 'unsound: mergeArrays false still concatenates tuples in the type',
      fn: async () => {
        const merged = target.deepmergeCustom({ mergeArrays: false, },)({ t: [1,] as const, }, { t: ['s',] as const, },);
        expectTypeOf(merged,).toEqualTypeOf<{ t: [1, 's',]; }>();
        expect(merged,).toEqual({ t: ['s',], },);
      },
    },),
    it({
      name: 'unsound: an any input yields the other input\'s concrete type',
      fn: async () => {
        const loose: any = 5;
        const merged = target.deepmerge({ a: 1, }, loose,);
        expectTypeOf(merged,).toEqualTypeOf<{ a: number; }>();
        expect(merged,).toBe(5,);
      },
    },),
    it({
      name: 'unsound: deepmergeInto asserts Target & Merged, so a changed value type becomes never or keeps the old elements',
      fn: async () => {
        const replaced = { a: 1, };
        target.deepmergeInto(replaced, { a: 'x', },);
        expectTypeOf(replaced,).toEqualTypeOf<{ a: never; }>();
        expect(replaced,).toEqual({ a: 'x', },);

        const concatenated = { list: [1,], };
        target.deepmergeInto(concatenated, { list: ['s',], },);
        expectTypeOf(concatenated.list,).items.toEqualTypeOf<number>();
        // Elements read as number (or absent), yet index 1 holds a string.
        expect(concatenated.list,).toEqual([1, 's',],);
      },
    },),
    it({
      name: 'unsound: deepmergeIntoCustom leaves the target type unchanged',
      fn: async () => {
        const replacedArray = { a: [1,], };
        target.deepmergeIntoCustom({ mergeArrays: false, },)(replacedArray, { a: ['s',], },);
        expectTypeOf(replacedArray,).toEqualTypeOf<{ a: number[]; }>();
        expect(replacedArray,).toEqual({ a: ['s',], },);
      },
    },),
    it({
      name: 'inherent: an object typed by a type alias is a record to the types even when its prototype makes it a leaf',
      fn: async () => {
        type Shaped = { readonly boxed: number; };
        // A Date carrying an own field: a leaf at runtime, a plain shape to the type system.
        const instance: Shaped = Object.assign(new Date(0,), { boxed: 1, },);
        const merged = target.deepmerge({ a: { w: 2, }, }, { a: instance, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: { w: number; boxed: number; }; }>();
        expect(merged.a,).toBe(instance,);
      },
    },),
    it({
      name: 'imprecise: a key whose only values are undefined is typed unknown',
      fn: async () => {
        const merged = target.deepmerge({ b: 1, }, { a: undefined, },);
        expectTypeOf(merged,).toEqualTypeOf<{ b: number; a: unknown; }>();
        expect(merged,).toEqual({ b: 1, a: undefined, },);
      },
    },),
    it({
      name: 'imprecise: optional then required stays optional, although required then optional becomes required',
      fn: async () => {
        const optionalNumbers = widen<{ a?: number[]; }>({},);
        const merged = target.deepmerge(optionalNumbers, { a: ['s',], },);
        expectTypeOf(merged,).toEqualTypeOf<{ a?: (string | number)[]; }>();
        expect(merged,).toEqual({ a: ['s',], },);
      },
    },),
    it({
      name: 'imprecise: mergeSets false and mergeMaps false still union element types',
      fn: async () => {
        const sets = target.deepmergeCustom({ mergeSets: false, },)({ s: new Set([1,],), }, { s: new Set(['x',],), },);
        expectTypeOf(sets.s,).toEqualTypeOf<Set<number | string>>();
        expect([...sets.s,],).toEqual(['x',],);
      },
    },),
  ],
},);
