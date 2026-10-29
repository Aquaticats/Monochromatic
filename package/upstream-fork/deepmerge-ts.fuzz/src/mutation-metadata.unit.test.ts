/**
 `deepmergeInto` cycle remapping under a custom `metaDataUpdater` whose
 hierarchy entries carry no `result`, found by mutation testing
 (`doc/audit/deepmerge-ts-mutation-2026-09-24.md`).

 With the built-in updater every entry has a `result`, so the
 `?? parents[0]` fallbacks in `src/deepmerge-into.ts` and
 `src/defaults/into.ts` never run. Without it, each remap falls back to the
 first input at the ancestor level: here a source record, because the target
 lacks the key. The test pins that fallback and names the Stryker mutant ids
 of that run it kills.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cyclicSelf,
  read,
} from './mutation-helper.ts';
import { target, } from './target.ts';

/**
 Hierarchy entry without `result`.
 */
type BareEntry = {
  readonly key: unknown;
  readonly parents: unknown;
  readonly values: unknown;
};

/**
 Metadata with a hierarchy of bare entries.
 */
type BareMeta = { readonly hierarchy: readonly BareEntry[]; };

/**
 Whether parent metadata is a bare-entry hierarchy; at the root it is absent.

 @param value - Parent metadata the library passes back.

 @returns Whether `value` carries a hierarchy array.

 @example
 ```ts
 isBareMeta({ hierarchy: [], }); // true
 ```
 */
function isBareMeta(value: unknown,): value is BareMeta {
  return ((typeof value) === 'object')
    && (value !== null)
    && Array.isArray(Reflect.get(value, 'hierarchy',),);
}

/**
 Custom `metaDataUpdater` keeping the hierarchy but dropping `result`.

 @param previous - Parent metadata, absent at the root.

 @param info - Merge info for the child position; `result` is ignored.

 @returns Metadata whose hierarchy gains one bare entry.

 @example
 ```ts
 deepmergeIntoCustom({ metaDataUpdater: withoutResult, });
 ```
 */
function withoutResult(previous: unknown, info: BareEntry,): BareMeta {
  /**
   Entry for the child position.
   */
  const entry: BareEntry = {
    key: info.key,
    parents: info.parents,
    values: info.values,
  };
  return { hierarchy: [...(isBareMeta(previous,) ? previous.hierarchy : []), entry,], };
}

/**
 Merge two sources, each holding its value under `n`, into a fresh target.

 @param first - Value under `n` in the first source.

 @param second - Value under `n` in the second source.

 @returns Target after the merge.

 @example
 ```ts
 const into = mergeUnderN({ first: {}, second: {}, });
 ```
 */
function mergeUnderN(
  {
    first,
    second,
  }: {
    readonly first: unknown;
    readonly second: unknown;
  },
): Record<string, unknown> {
  /**
   Target lacking `n`.
   */
  const into: Record<string, unknown> = {};
  target.deepmergeIntoCustom({ metaDataUpdater: withoutResult as never, },)(into, { n: first, }, { n: second, },);
  return into;
}

await describe({
  name: 'deepmerge-ts into cycle remap without hierarchy results',
  children: [
    it({
      name: 'every into remap falls back to the first input of the ancestor level',
      // Kills 257 (deepmerge-into.ts) and 759, 804, and 811 (defaults/into.ts).
      fn: async () => {
        /**
         Cyclic first source at `n`, the fallback of every case below.
         */
        const looped = cyclicSelf(1,);
        expect(read({ holder: mergeUnderN({ first: looped, second: {}, },).n, key: 'self', },),).toBe(looped,);
        expect(read({ holder: mergeUnderN({ first: looped, second: cyclicSelf(2,), },).n, key: 'self', },),)
          .toBe(looped,);
        /**
         Plain first source at `n` with a cyclic second.
         */
        const plainFirst = { self: { x: 1, }, };
        expect(read({ holder: mergeUnderN({ first: plainFirst, second: cyclicSelf(2,), },).n, key: 'self', },),)
          .toBe(plainFirst,);
        /**
         Second source whose `self.back` references it.
         */
        const backed: Record<string, unknown> = {};
        backed.self = { back: backed, };
        /**
         Resolved `self` of the second source.
         */
        const resolved = read({ holder: mergeUnderN({ first: looped, second: backed, },).n, key: 'self', },);
        expect(read({ holder: resolved, key: 'back', },),).toBe(looped,);
      },
    },),
  ],
},);
