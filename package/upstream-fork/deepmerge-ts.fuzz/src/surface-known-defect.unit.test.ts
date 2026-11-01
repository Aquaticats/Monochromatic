/**
 Known divergences found by the unsearched-surfaces workstream
 (`doc/audit/deepmerge-ts-surface-2026-09-24.md`), each asserted to still
 reproduce, with the same contract as `./known-defect.unit.test.ts`: the
 suite stays green while upstream is unfixed and turns red when behaviour
 changes. Type-level halves are checked by `lint:types`.

 Reproduced on deepmerge-ts 8.0.2 in a capped container (2026-09-24).

 @module
 */

import type * as DeepmergeTs from 'deepmerge-ts';

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mapView,
  setView,
} from './surface-collection-view.ts';
import { target, } from './target.ts';

/**
 Configuration shape declared as an interface, as README "TypeScript
 Interfaces" describes.
 */
// oxlint-disable-next-line typescript/consistent-type-definitions -- the interface form is the case under test.
interface InterfaceConfig {
  /**
   Numbers that a later input extends.
   */
  list: number[];
  /**
   Key only this input has.
   */
  name: string;
}

await describe({
  name: 'deepmerge-ts surface known defects still reproduce',
  children: [
    it({
      name: 'defect: the declaration files export ObjectType as a value that neither build provides',
      // Cause: src/index.ts re-exports `type ObjectType` (a const enum in src/utils.ts), but
      // dist/index.d.mts and dist/index.d.cts declare `export declare const enum ObjectType`.
      // tsc inlines the members, so plain tsc builds work; isolatedModules and verbatimModuleSyntax
      // report TS2748, and per-file transpilers (Node type stripping, esbuild, Bun, Deno) keep the
      // import, which fails at load in ESM and reads undefined in CJS. API.md lists ObjectType under
      // "Utility Functions ... exported for use in custom merge functions".
      fn: async () => {
        // A type query on the value side compiles only while the declaration exports a value.
        expectTypeOf<(typeof DeepmergeTs.ObjectType)['RECORD']>().toEqualTypeOf<DeepmergeTs.ObjectType.RECORD>();
        /**
         Export name widened to `string`, because a literal key into the namespace type reads the
         const enum as a value (TS2475).
         */
        const exportName: string = 'ObjectType';
        expect(Object.keys(target,),).not.toContain(exportName,);
        expect(Reflect.get(target, exportName,),).toBeUndefined();
        // The numbering the enum declares is what getObjectType returns.
        expect([1, {}, [], new Set(), new Map(), new Date(0,),].map(function kindOf(value,) {
          return target.getObjectType(value,);
        },),).toEqual([0, 1, 2, 3, 4, 5,],);
      },
    },),
    it({
      name: 'defect: values typed ReadonlySet or ReadonlyMap that are not Set or Map instances are typed as merged Sets and Maps but kept as the last value',
      // Excluded region: the declared-type generator (./declared-type-arbitrary.ts) draws no class types.
      // Cause: IsSet and IsMap in index.d.mts test `extends ReadonlySet` and `extends ReadonlyMap`,
      // while getObjectType (src/utils.ts) tests `instanceof Set` and `instanceof Map`, so a view is
      // "set" statically and "other" (last value wins) at runtime. The static type then offers
      // `add` and `set`, which throw.
      fn: async () => {
        /**
         Later set view, which the runtime keeps by reference.
         */
        const laterSet = setView([1,],);
        /**
         Record holding set views of different element types.
         */
        const sets = target.deepmerge({ tags: setView(['a',],), }, { tags: laterSet, },);
        expectTypeOf(sets.tags,).toEqualTypeOf<Set<string | number>>();
        expect(sets.tags,).toBe(laterSet,);
        expect(Reflect.get(sets.tags, 'add',),).toBeUndefined();
        /**
         Later map view, which the runtime keeps by reference.
         */
        const laterMap = mapView([['b', 'x',],],);
        /**
         Record holding map views of different value types.
         */
        const maps = target.deepmerge({ m: mapView([['a', 1,],],), }, { m: laterMap, },);
        expectTypeOf(maps.m,).toEqualTypeOf<Map<string, number | string>>();
        expect(maps.m,).toBe(laterMap,);
        expect(Reflect.get(maps.m, 'set',),).toBeUndefined();
      },
    },),
    it({
      name: 'defect: a fixed input followed by a spread array is typed as the fixed input alone',
      // Excluded region: every generated call site passes a fixed argument list.
      // Cause: `[First, ...Rest[]]` passes IsTuple (index.d.mts), so DeepMergeHKT does not fall back
      // to `unknown` (API.md's documented result for non-tuple arguments); the record helpers read
      // only the fixed element. deepmergeInto's assertion keeps the target type for the same reason.
      fn: async () => {
        /**
         Inputs known only as an array type.
         */
        const rest: readonly { readonly b: string; readonly list: readonly string[]; }[] = [{ b: 'x', list: ['s',], },];
        /**
         Fixed input, then the spread.
         */
        const leading = target.deepmerge({ a: 1, list: [1,], }, ...rest,);
        expectTypeOf(leading,).toEqualTypeOf<{ a: number; list: number[]; }>();
        expect(leading,).toEqual({ a: 1, list: [1, 's',], b: 'x', },);
        // Spread first is typed unknown, as API.md says for non-tuple inputs.
        expectTypeOf(target.deepmerge(...rest, { a: 1, },),).toEqualTypeOf<unknown>();
        /**
         deepmergeInto target merged with a spread.
         */
        const intoTarget = { a: 1, list: [1,], };
        target.deepmergeInto(intoTarget, ...rest,);
        expectTypeOf(intoTarget,).toEqualTypeOf<{ a: number; list: number[]; }>();
        expect(intoTarget,).toEqual({ a: 1, list: [1, 's',], b: 'x', },);
      },
    },),
    it({
      name: 'intent question: an interface-typed input makes the result type the last input only',
      // Not excluded: the declared-type generator emits type aliases only.
      // README "TypeScript Interfaces" says interfaces "may not appear to merge correctly"; the type
      // is not just imprecise: it drops the interface's keys and claims string elements for an
      // array that holds numbers. Cause: IsRecord needs an index signature, which interfaces lack,
      // so the input is "other" and DeepMergeLeaf picks the last type.
      fn: async () => {
        /**
         Interface-typed first input.
         */
        const first: InterfaceConfig = { list: [1,], name: 'a', };
        /**
         Merge with a record whose array holds strings.
         */
        const merged = target.deepmerge(first, { list: ['s',], },);
        expectTypeOf(merged,).toEqualTypeOf<{ list: string[]; }>();
        expect(merged,).toEqual({ list: [1, 's',], name: 'a', },);
      },
    },),
  ],
},);
