/**
 Cycle-resolution paths that upstream's suite and every earlier sidecar test
 left open, found by mutation testing
 (`doc/audit/deepmerge-ts-mutation-2026-09-24.md`).

 Each test pins 8.0.2 behaviour and names the Stryker mutant ids of that run
 it kills. The `deepmergeInto` cases merge two sources into a target that
 lacks the key, because only there does the recursion's result differ from
 its first input, which is what separates a remap onto the target from a
 reference back into a source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cyclicSelf,
  ownProto,
  read,
} from './mutation-helper.ts';
import { target, } from './target.ts';

/**
 Record whose `self` key holds `inner`, whose `back` key references the record.

 @param inner - Extra fields of the `self` value, besides `back`.

 @returns Record `r` with `r.self.back === r`.

 @example
 ```ts
 const outer = backReferenced({ x: 1, }); // outer.self.back === outer
 ```
 */
function backReferenced(inner: Record<string, unknown>,): Record<string, unknown> {
  /**
   Record the nested value points back to.
   */
  const outer: Record<string, unknown> = {};
  outer.self = { ...inner, back: outer, };
  return outer;
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
   Target lacking `n`, so the recursion seeds a fresh record for it.
   */
  const into: Record<string, unknown> = {};
  target.deepmergeInto(into, { n: first, }, { n: second, },);
  return into;
}

await describe({
  name: 'deepmerge-ts cycle resolution paths found by mutation testing',
  children: [
    it({
      name: 'a cyclic value in the second position is remapped onto the result',
      // Kills 445 (deepmerge.ts) and 279 (deepmerge-into.ts).
      fn: async () => {
        /**
         Input whose `k` key references itself.
         */
        const looped: Record<string, unknown> = {};
        looped.k = looped;
        /**
         Merge with a plain record first at `k`.
         */
        const merged: unknown = target.deepmerge({ k: { x: 1, }, }, looped,);
        expect(read({ holder: merged, key: 'k', },),).toBe(merged,);
        /**
         Into target with a plain record at `k`.
         */
        const into: Record<string, unknown> = { k: { x: 1, }, };
        target.deepmergeInto(into, looped,);
        expect(into.k,).toBe(into,);
      },
    },),
    it({
      name: 'three values where only the last is cyclic, or only the last is not, resolve by the last',
      // Kills 461 (deepmerge.ts) and 989 (defaults/vanilla.ts).
      fn: async () => {
        /**
         Input whose `k` key references itself.
         */
        const looped: Record<string, unknown> = {};
        looped.k = looped;
        /**
         Merge whose last value at `k` is the cycle.
         */
        const lastCyclic: unknown = target.deepmerge({ k: { x: 1, }, }, { k: { y: 1, }, }, looped,);
        expect(read({ holder: lastCyclic, key: 'k', },),).toBe(lastCyclic,);
        /**
         Second self-referencing input.
         */
        const loopedAgain: Record<string, unknown> = {};
        loopedAgain.k = loopedAgain;
        /**
         Plain value last at `k`, which resolution leaves unchanged.
         */
        const plain = { x: 1, };
        /**
         Merge whose last value at `k` is plain.
         */
        const lastPlain: unknown = target.deepmerge(looped, loopedAgain, { k: plain, },);
        expect(read({ holder: lastPlain, key: 'k', },),).toBe(plain,);
      },
    },),
    it({
      name: 'deepmergeInto remaps nested cycles onto the target, never onto a source',
      // Kills 253 and 255 (deepmerge-into.ts), 798, 801, 805, and 808 (defaults/into.ts).
      fn: async () => {
        /**
         Cycle present in the first source only: a single value at `self`.
         */
        const single = mergeUnderN({ first: cyclicSelf(1,), second: {}, },);
        expect(read({ holder: single.n, key: 'self', },),).toBe(single.n,);
        /**
         Cycles at the same depth in both sources.
         */
        const both = mergeUnderN({ first: cyclicSelf(1,), second: cyclicSelf(2,), },);
        expect(read({ holder: both.n, key: 'self', },),).toBe(both.n,);
        /**
         Plain value first, cycle last.
         */
        const lastCyclic = mergeUnderN({ first: { self: { x: 1, }, }, second: cyclicSelf(2,), },);
        expect(read({ holder: lastCyclic.n, key: 'self', },),).toBe(lastCyclic.n,);
      },
    },),
    it({
      name: 'deepmergeInto resolves a plain last value holding an ancestor reference onto the target',
      // Kills 737, 741, 742, 744, 755, 757, 761, 762, 763, 765, 767, 769, and 770 (defaults/into.ts).
      fn: async () => {
        /**
         Result whose `n.self.back` must be the target's own `n`.
         */
        const into = mergeUnderN({ first: cyclicSelf(1,), second: backReferenced({ x: 1, },), },);
        /**
         Resolved copy of the second source's `self`.
         */
        const resolved = read({ holder: into.n, key: 'self', },);
        expect(read({ holder: resolved, key: 'back', },),).toBe(into.n,);
        expect(read({ holder: resolved, key: 'x', },),).toBe(1,);
      },
    },),
    it({
      name: 'resolution returns an unchanged plain value by reference',
      // Kills 764, 766, and 768 (defaults/into.ts) and 961, 963, and 965 (defaults/vanilla.ts).
      fn: async () => {
        /**
         Plain value at `self` with no ancestor reference.
         */
        const plain = { x: 1, };
        expect(read({ holder: mergeUnderN({ first: cyclicSelf(1,), second: { self: plain, }, },).n, key: 'self', },),)
          .toBe(plain,);
        expect(read({ holder: target.deepmerge(cyclicSelf(1,), { self: plain, },), key: 'self', },),).toBe(plain,);
      },
    },),
    it({
      name: 'resolution leaves an array holding an ancestor reference as that array',
      // Kills 760 (defaults/into.ts) and 957 (defaults/vanilla.ts). Arrays are not resolved at all:
      // the array keeps pointing at the source ancestor, see the audit report.
      fn: async () => {
        /**
         Source whose `self.list` array references the source.
         */
        const source: Record<string, unknown> = {};
        /**
         Array holding the ancestor reference.
         */
        const list: unknown[] = [source,];
        source.self = { list, };
        /**
         Merge result, resolved at `self` because the other input is cyclic there.
         */
        const merged = target.deepmerge(cyclicSelf(1,), source,);
        expect(read({ holder: read({ holder: merged, key: 'self', },), key: 'list', },),).toBe(list,);
        /**
         Same case through `deepmergeInto`.
         */
        const into = mergeUnderN({ first: cyclicSelf(1,), second: source, },);
        expect(read({ holder: read({ holder: into.n, key: 'self', },), key: 'list', },),).toBe(list,);
      },
    },),
    it({
      name: 'resolution copies an own __proto__ key as an own enumerable writable configurable key',
      // Kills 773, 775, 777, 778, 779, and 780 (defaults/into.ts) and 970, 972, 974, 975, 976, and 977
      // (defaults/vanilla.ts).
      fn: async () => {
        /**
         Record under the `__proto__` key.
         */
        const inner = { p: 1, };
        /**
         Resolved records from both entry points.
         */
        const resolved = [
          read({ holder: target.deepmerge(cyclicSelf(1,), backReferenced(ownProto(inner,),),), key: 'self', },),
          read({
            holder: mergeUnderN({ first: cyclicSelf(1,), second: backReferenced(ownProto(inner,),), },).n,
            key: 'self',
          },),
        ];
        for (const record of resolved) {
          expect(Object.getPrototypeOf(record,),).toBe(Object.prototype,);
          expect(Object.getOwnPropertyDescriptor(record, '__proto__',),).toEqual({
            configurable: true,
            enumerable: true,
            value: inner,
            writable: true,
          },);
        }
      },
    },),
  ],
},);
