/**
 Known divergences between deepmerge-ts and its documented semantics.

 Each test asserts that the divergence still reproduces, beside the documented
 outcome it should have, so the unit suite stays green while upstream is
 unfixed and turns red the moment upstream changes behaviour. When one fails
 after an upgrade: confirm the fix, delete the test, and re-include the
 excluded region in the generators named by its comment.

 Upstream reporting: one combined issue draft, held locally for the user
 (`doc/handover/deepmerge-ts-hardening.md`, Q3 and confirmation constraints).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { point, } from './arbitraries.ts';
import { target, } from './target.ts';

/**
 Build a record from entries with `defineProperty` semantics for readability.

 @param entries - Key and value pairs in insertion order.

 @returns Plain record holding every entry.

 @example
 ```ts
 plain([['a', 1,],]); // { a: 1 }
 ```
 */
function plain(entries: readonly (readonly [string, unknown,])[],): Record<string, unknown> {
  return Object.fromEntries(entries,);
}

/**
 Read one property of a merge result whose static type is opaque.

 @param holder - Value expected to be an object.

 @param key - Property to read.

 @returns Property value.

 @throws When `holder` is not an object, which means the merge shape changed.

 @example
 ```ts
 property({ holder: { a: 1, }, key: 'a', }); // 1
 ```
 */
function property(
  {
    holder,
    key,
  }: {
    readonly holder: unknown;
    readonly key: PropertyKey;
  },
): unknown {
  if (((typeof holder) !== 'object') || (holder === null))
    throw new Error(`expected an object holding ${String(key,)}, got ${String(holder,)}`,);
  /**
   Read value, typed `unknown` rather than `any`.
   */
  const value: unknown = Reflect.get(
    holder,
    key,
  );
  return value;
}

await describe({
  name: 'deepmerge-ts known defects still reproduce',
  children: [
    it({
      name: 'shared subobject reachable from another input\'s ancestor is mistaken for a cycle',
      // Excluded region: generators never share a container between inputs (src/arbitraries.ts).
      // Cause: getCyclicReferenceDepth in src/utils.ts matches any input's ancestors via parents.includes.
      fn: async () => {
        /**
         Subobject placed at two different depths across the inputs.
         */
        const shared = plain([['v', 1,], ['k2', plain([['w', 2,],],),],],);
        /**
         Merge result; documented semantics give `r.k.k2 === shared`.
         */
        const merged = target.deepmerge(plain([['k', shared,],],), plain([['k', plain([['k2', shared,],],),],],),);
        /**
         Node that should be `shared` but is the merged `k` record itself.
         */
        const nested = property({
          holder: property({
            holder: merged,
            key: 'k',
          },),
          key: 'k2',
        },);
        expect(nested,).toBe(property({
          holder: merged,
          key: 'k',
        },),);
        expect(() => JSON.stringify(merged,),).toThrow(TypeError,);
      },
    },),
    it({
      name: 'deepmergeInto seeds an absent key from its first value and merges later records into that copy',
      // Excluded region: deepmergeInto properties use objectLeaves: false (src/merge-model.property.unit.test.ts).
      // Cause: mergeRecordsInto seeds emptyLike(propValues[0]) in src/defaults/into.ts, and mergeUnknownsInto
      // types the merge by that seeded {} without checking the first value's own kind.
      fn: async () => {
        /**
         Record that must win outright, by documented last-value semantics.
         */
        const last = Object.create(null,) as object;
        /**
         `deepmerge` result: the last value itself, as documented.
         */
        const merged = target.deepmerge(plain([['a', point(0,),],],), plain([['a', last,],],),);
        expect(property({
          holder: merged,
          key: 'a',
        },),).toBe(last,);

        /**
         `deepmergeInto` target lacking the key.
         */
        const intoTarget = {};
        target.deepmergeInto(intoTarget, plain([['a', point(0,),],],), plain([['a', last,],],),);
        /**
         Value `deepmergeInto` stored instead of `last`.
         */
        const stored = property({
          holder: intoTarget,
          key: 'a',
        },);
        if (((typeof stored) !== 'object') || (stored === null))
          throw new Error('deepmergeInto stored a non-object at a',);
        expect(stored,).not.toBe(last,);
        expect(Object.getPrototypeOf(stored,),).toBe(Object.prototype,);
        expect(Reflect.ownKeys(stored,),).toEqual(['x',],);
      },
    },),
    it({
      name: 'deepmergeInto replaces instead of merging when the first value at a key is undefined',
      // Excluded region: deepmergeInto targets and sources use undefinedLeaves: false
      // (src/merge-model.property.unit.test.ts). Found by the campaign (seeds 1256317701, 655631496).
      // Cause: mergeRecordsInto in src/defaults/into.ts (and into-fast.ts) seeds the key from the target's
      // value or emptyLike(first value) before undefined is filtered; mergeUnknownsInto types the merge by
      // that undefined seed, so it falls to mergeOthers and only the last value survives.
      fn: async () => {
        expect(target.deepmerge({ a: undefined, }, { a: { x: 1, }, }, { a: { y: 2, }, },),).toEqual({ a: { x: 1, y: 2, }, },);
        /**
         Target whose key exists but holds `undefined`.
         */
        const intoTarget: Record<string, unknown> = { a: undefined, };
        target.deepmergeInto(intoTarget, { a: { x: 1, }, }, { a: { y: 2, }, },);
        expect(intoTarget,).toEqual({ a: { y: 2, }, },);
        /**
         Same target for the FastUnsafe variant, which shares the seeding.
         */
        const fastTarget: Record<string, unknown> = { a: undefined, };
        target.deepmergeIntoFastUnsafe(fastTarget, { a: [0,], }, { a: [1,], },);
        expect(fastTarget,).toEqual({ a: [1,], },);
        /**
         Target lacking the key, whose first source holds `undefined` there.
         */
        const absentTarget: Record<string, unknown> = {};
        target.deepmergeInto(absentTarget, { a: undefined, }, { a: [0,], }, { a: [1,], },);
        expect(absentTarget,).toEqual({ a: [1,], },);
        expect(target.deepmerge({}, { a: undefined, }, { a: [0,], }, { a: [1,], },),).toEqual({ a: [0, 1,], },);
      },
    },),
    it({
      name: 'array holes are dropped instead of concatenated (upstream intent question)',
      // Excluded region: generators build dense arrays only (src/arbitraries.ts).
      // Cause: mergeArrays uses Array.prototype.flat in src/defaults/general.ts, which skips holes.
      fn: async () => {
        /**
         Array with a hole at index 0.
         */
        // oxlint-disable-next-line no-sparse-arrays -- The hole is the input under test.
        const holed = [, 1,];
        // Concatenation keeps every slot, holes included: length is the sum of input lengths.
        expect([
          ...holed,
          2,
        ],).toHaveLength(holed.length + 1,);
        expect(target.deepmerge(holed, [2,],),).toEqual([1, 2,],);
      },
    },),
    it({
      name: 'a cycle deeper than one level in a single input is passed through, not remapped (upstream intent question)',
      // Excluded region: the cycle property merges inputs cyclic along the same chain only
      // (src/merge-invariant.property.unit.test.ts).
      // Observation: a direct self-reference is remapped onto the result, a two-level cycle keeps
      // pointing at the input, so the merged graph mixes result and input nodes.
      fn: async () => {
        /**
         Input whose `self.self` returns to it.
         */
        const looped: Record<string, unknown> = { extra: 1, };
        looped.self = { self: looped, };
        /**
         Merge with a record contributing a key the input lacks.
         */
        const merged = target.deepmerge(looped, { added: true, },);
        /**
         Node two `self` hops from the result.
         */
        const twoHops = property({
          holder: property({
            holder: merged,
            key: 'self',
          },),
          key: 'self',
        },);
        expect(twoHops,).toBe(looped,);
        expect(twoHops,).not.toBe(merged,);

        /**
         Direct self-reference, which is remapped onto the result.
         */
        const direct: Record<string, unknown> = { extra: 1, };
        direct.self = direct;
        /**
         Merge of the direct self-reference with the same extra record.
         */
        const mergedDirect = target.deepmerge(direct, { added: true, },);
        expect(property({
          holder: mergedDirect,
          key: 'self',
        },),).toBe(mergedDirect,);
      },
    },),
  ],
},);
