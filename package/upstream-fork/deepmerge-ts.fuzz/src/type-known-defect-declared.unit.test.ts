/**
 Type-level divergences found by the declared-type fuzzer
 (`./declared-type-campaign.ts`), beyond the classes already pinned in
 `./type-known-defect.unit.test.ts`.

 Same contract as that file: each case pins the current static type with
 `expectTypeOf` (checked by `lint:types`) and the contradicting runtime value
 with `expect`, so a change to either side fails. Root causes point into
 deepmerge-ts 8.0.2's `src/types/`.

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
 Return the argument typed as `T`, so a declared union or optional type is
 not narrowed to its initializer by control-flow analysis.

 @param value - Value to widen.

 @returns Same value, typed as `T`.

 @example
 ```ts
 const either = widen<{ a?: string[]; }>({},);
 ```
 */
function widen<T,>(value: T,): T {
  return value;
}

await describe({
  name: 'deepmerge-ts declared-type known divergences',
  children: [
    it({
      name: 'unsound: a union-of-objects input contributes only the keys all its members share',
      // Cause: RecordKeysOf and ValuesForKey in src/types/defaults.ts read `keyof Head`, and keyof of a union is
      // the intersection of its members' keys, so every other key of that input vanishes from the result type.
      fn: async () => {
        const later = target.deepmerge({ a: [1,], }, widen<{ a: string[]; } | { b: 1; }>({ a: ['s',], },),);
        expectTypeOf(later,).toEqualTypeOf<{ a: number[]; }>();
        expect(later.a,).toEqual([1, 's',],);

        const earlier = target.deepmerge(widen<{ a: string[]; } | { b: 1; }>({ a: ['s',], },), { a: [1,], },);
        expectTypeOf(earlier,).toEqualTypeOf<{ a: number[]; }>();
        expect(earlier.a,).toEqual(['s', 1,],);
      },
    },),
    it({
      // The same holds when the other inputs are keyless (`{}` or `Record<never, never>`); lint bans those spellings here.
      name: 'unsound: union-of-objects inputs whose members share no key, with no other known key, are typed never',
      // Cause: the same keyof-of-a-union reading leaves RecordKeysOf empty, and DeepMergeRecordsDefaultHKT in
      // src/types/defaults.ts returns never when IsNever<RecordKeysOf<Ts>>, so any assertion on the result passes.
      fn: async () => {
        const merged = target.deepmerge(widen<{ a: 1; } | { b: 2; }>({ a: 1, },), widen<{ c: 3; } | { d: 4; }>({ d: 4, },),);
        expectTypeOf(merged,).toBeNever();
        expect(merged,).toEqual({ a: 1, d: 4, },);
      },
    },),
    it({
      name: 'unsound: a key optional in a later input is typed by PreciseOrUnion instead of the merge',
      // Cause: DeepMergeRecordPropertyMetaDefaultHKTGetValuesHelper in src/types/defaults.ts folds values after an
      // optional one with PreciseOrUnion (the narrower type when assignable, else a union), while the runtime merges.
      fn: async () => {
        const arrays = target.deepmerge(widen<{ a: number[]; }>({ a: [1,], },), widen<{ a?: string[]; }>({ a: ['s',], },),);
        expectTypeOf(arrays,).toEqualTypeOf<{ a: string[] | number[]; }>();
        expect(arrays.a,).toEqual([1, 's',],);

        const sets = target.deepmerge(widen<{ a: Set<number>; }>({ a: new Set([1,],), },), widen<{ a?: Set<string>; }>({ a: new Set(['s',],), },),);
        expectTypeOf(sets,).toEqualTypeOf<{ a: Set<string> | Set<number>; }>();
        expect([...sets.a,],).toEqual([1, 's',],);

        // A wider later type loses to the narrower earlier one, although the later value wins at runtime.
        const wider = target.deepmerge(widen<{ a: Set<'b'>; }>({ a: new Set(['b',] as const,), },), widen<{ a?: object; }>({ a: {}, },),);
        expectTypeOf(wider,).toEqualTypeOf<{ a: Set<'b'>; }>();
        expect(wider.a,).toEqual({},);

        // Both optional: the narrower tuple type replaces the concatenation.
        const tuples = target.deepmerge(widen<{ a?: string[]; }>({ a: ['x',], },), widen<{ a?: ['y',]; }>({ a: ['y',], },),);
        expectTypeOf(tuples,).toEqualTypeOf<{ a?: ['y',]; }>();
        expect(tuples.a,).toEqual(['x', 'y',],);
      },
    },),
    it({
      name: 'imprecise: an input\'s index signatures are dropped when it also declares known keys',
      // Cause: RecordKeys in src/types/defaults.ts keeps only KnownKeys when an input has any, discarding its
      // string, number, symbol, and template-literal index signatures; the runtime keeps those keys.
      fn: async () => {
        const numbered = target.deepmerge(widen<{ a: 1; [key: number]: string; }>({ a: 1, 0: 'z', },), { b: 2, },);
        expectTypeOf(numbered,).toEqualTypeOf<{ a: 1; b: number; }>();
        expect(Reflect.get(numbered, '0',),).toBe('z',);
      },
    },),
    it({
      name: 'imprecise: mutually assignable inputs are typed as the first one, dropping later optional keys',
      // Cause: DeepMergeSameTypeShortcut in src/types/merging.ts tests sameness with AllSameTypes, which is mutual
      // assignability, and a type is mutually assignable with the same type plus optional keys; the shortcut then
      // returns the first input's type, at the root and for each property value.
      fn: async () => {
        const root = target.deepmerge(widen<{ a: number; }>({ a: 1, },), widen<{ a: number; b?: string; }>({ a: 2, b: 's', },),);
        expectTypeOf(root,).toEqualTypeOf<{ a: number; }>();
        expect(root,).toEqual({ a: 2, b: 's', },);

        const nested = target.deepmerge(widen<{ x: { c?: 2; }; y: 1; }>({ x: {}, y: 1, },), widen<{ x: { b?: 1; c?: 2; }; z: 2; }>({ x: { b: 1, }, z: 2, },),);
        expectTypeOf(nested,).toEqualTypeOf<{ x: { c?: 2; }; y: 1; z: 2; }>();
        expect(nested.x,).toEqual({ b: 1, },);
      },
    },),
    it({
      name: 'unsound: deepmergeInto keeps the target type when every source is assignable to it, so tuples grow unseen',
      // Cause: the first deepmergeInto overload in src/deepmerge-into.ts, (target: T, ...objects: ReadonlyArray<T>)
      // returning void, wins whenever the sources fit the target type, so the Target & Merged assertion never runs.
      fn: async () => {
        const tuples = widen<{ a: [number]; }>({ a: [1,], },);
        target.deepmergeInto(tuples, widen<{ a: [number]; }>({ a: [2,], },),);
        expectTypeOf(tuples,).toEqualTypeOf<{ a: [number]; }>();
        expect(tuples.a,).toEqual([1, 2,],);

        // Imprecise variant: an `object` target type absorbs every source key.
        const loose = widen<object>({},);
        target.deepmergeInto(loose, widen<{ b: 1; }>({ b: 1, },),);
        expectTypeOf(loose,).toEqualTypeOf<object>();
        expect(loose,).toEqual({ b: 1, },);
      },
    },),
    it({
      name: 'unsound: deepmergeInto with a non-object source asserts the target became that source\'s type',
      // Same Target & Merged assertion as the pinned key-level case, at the root: the merge result is the last
      // value, so the target is typed as a Map (or number), while the runtime leaves the target untouched.
      fn: async () => {
        const intoMap = widen<{ b: number; }>({ b: 1, },);
        target.deepmergeInto(intoMap, widen<Map<string, 'b'>>(new Map([['m', 'b',],],),),);
        expectTypeOf(intoMap,).toExtend<Map<string, 'b'>>();
        expect(intoMap instanceof Map,).toBe(false,);
        expect(intoMap,).toEqual({ b: 1, },);
      },
    },),
  ],
},);
