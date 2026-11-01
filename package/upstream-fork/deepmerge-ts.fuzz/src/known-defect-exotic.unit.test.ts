/**
 Known divergences on exotic inputs, each asserted to still reproduce, with
 the same contract as `./known-defect.unit.test.ts`: the suite stays green
 while upstream is unfixed and turns red when behaviour changes.

 Reproduced on deepmerge-ts 8.0.2 in a capped container (2026-09-24).

 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  foreign,
  taggedArray,
  taggedMap,
  taggedSet,
} from './exotic-arbitraries.ts';
import { target, } from './target.ts';

/**
 Spread a Set-like value into an array, failing when it is not iterable.

 @param value - Merge result expected to be a Set.

 @returns Its elements, in order.

 @throws When the value is not iterable.

 @example
 ```ts
 elements(new Set([1,])); // [1]
 ```
 */
function elements(value: unknown,): readonly unknown[] {
  if (((typeof value) !== 'object') || (value === null) || (!(Symbol.iterator in value)))
    throw new TypeError('elements: value is not iterable',);
  return [...(value as Iterable<unknown>),];
}

await describe({
  name: 'deepmerge-ts exotic known defects still reproduce',
  children: [
    it({
      name: 'defect: Sets and Maps from another realm are leaves, while foreign arrays and records merge',
      // Excluded region: ./exotic-arbitraries.ts generates no foreign Sets or Maps.
      // Cause: getObjectType in src/utils.ts tests `instanceof Set` and `instanceof Map`, which are
      // realm-bound, while Array.isArray and isRecord's toString checks work across realms.
      fn: async () => {
        expect(
          elements(target.deepmerge(foreign('new Set([1])',), foreign('new Set([2])',),),),
        ).toEqual([2,],);
        expect(
          elements(target.deepmerge(new Set([1,],), foreign('new Set([2])',),),),
        ).toEqual([2,],);
        expect(
          elements(target.deepmergeFastUnsafe(foreign('new Set([1])',), foreign('new Set([2])',),),),
        ).toEqual([2,],);
        /**
         Foreign Maps whose shared key holds records that would merge in this realm.
         */
        const mergedMaps = target.deepmerge(foreign('new Map([["k",{x:1}]])',), foreign('new Map([["k",{y:2}]])',),);
        expect(
          JSON.stringify(elements(mergedMaps,),),
        ).toBe('[["k",{"y":2}]]',);
        // Foreign arrays and records do merge, which is what makes the Set and Map behaviour a divergence.
        expect(target.deepmerge(foreign('[1]',), foreign('[2]',),),).toEqual([1, 2,],);
        expect(
          JSON.stringify(target.deepmerge(foreign('({a:1})',), foreign('({b:2})',),),),
        ).toBe('{"a":1,"b":2}',);
        /**
         Foreign Set as a deepmergeInto target: left unchanged without an error.
         */
        const foreignTarget = foreign('new Set([1])',) as Set<unknown>;
        target.deepmergeInto(foreignTarget, foreign('new Set([2])',) as Set<unknown>,);
        expect(elements(foreignTarget,),).toEqual([1,],);
      },
    },),
    it({
      name: 'defect: deepmergeInto silently ignores sources whose top-level kind differs from the target',
      // Excluded region: ./exotic-merge.property.unit.test.ts only merges into plain record targets
      // from record sources.
      // Cause: customizedDeepmergeInto in src/deepmerge-into.ts passes a throwaway `{ value: target }`
      // wrapper; mergeOthersInto replaces only `wrapper.value`, which nothing reads afterwards.
      // README "Merging into a Target" says the target's type is asserted to have changed, so the
      // static type also claims a change the runtime never makes.
      fn: async () => {
        /**
         Record target merged with an array source.
         */
        const recordTarget = { a: 1, };
        target.deepmergeInto(recordTarget, [1, 2,],);
        expectTypeOf(recordTarget,).toHaveProperty('length',);
        expect(recordTarget,).toEqual({ a: 1, },);
        expect(target.deepmerge({ a: 1, }, [1, 2,],),).toEqual([1, 2,],);
        /**
         Set target merged with a Map source.
         */
        const setTarget = new Set([1,],);
        target.deepmergeInto(setTarget, new Map([[1, 1,],],),);
        expect([...setTarget,],).toEqual([1,],);
        /**
         Typed-array target merged with a typed-array source, through the FastUnsafe variant.
         */
        const typedTarget = new Uint8Array([1,],);
        target.deepmergeIntoFastUnsafe(typedTarget, new Uint8Array([2,],),);
        expect([...typedTarget,],).toEqual([1,],);
      },
    },),
    it({
      name: 'intent question: container subclasses become base-class results in deepmerge but keep their class in deepmergeInto',
      // Not excluded: the model agrees with deepmerge here (subclasses merge like their base).
      // Cause: mergeArrays uses values.flat() and mergeSets/mergeMaps build `new Set`/`new Map`
      // (src/defaults/general.ts, src/defaults/vanilla.ts); deepmergeInto mutates the target in place.
      fn: async () => {
        /**
         Array subclass instance used as both inputs' prototype source.
         */
        const tagged = taggedArray([1,],);
        expect(
          Object.getPrototypeOf(target.deepmerge(tagged, taggedArray([2,],),),),
        ).toBe(Array.prototype,);
        expect(
          Object.getPrototypeOf(target.deepmerge(taggedSet([1,],), taggedSet([2,],),),),
        ).toBe(Set.prototype,);
        expect(
          Object.getPrototypeOf(target.deepmerge(taggedMap([[1, 1,],],), taggedMap([[2, 2,],],),),),
        ).toBe(Map.prototype,);
        // A single input passes through by reference and keeps its class.
        expect(target.deepmerge(tagged,),).toBe(tagged,);
        /**
         Subclass target mutated in place.
         */
        const intoTarget = taggedArray([1,],);
        target.deepmergeInto(intoTarget, [2,],);
        expect(Object.getPrototypeOf(intoTarget,),).toBe(Object.getPrototypeOf(tagged,),);
        expect([...intoTarget,],).toEqual([1, 2,],);
      },
    },),
  ],
},);
