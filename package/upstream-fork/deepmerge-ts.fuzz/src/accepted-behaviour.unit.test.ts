/**
 Characterization tests for behaviour accepted as intended but otherwise
 untested upstream (`doc/handover/deepmerge-ts-hardening.md`, Q10 and Q12).

 - Past the default `maxDepth` of 1000 the last value wins silently.
 - Getters are read and flattened into data properties; a throwing getter
   propagates its error.
 - `deepmergeInto` into a frozen target throws `TypeError`; frozen sources
   merge and the result is not frozen.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { DEFAULT_MAX_DEPTH, } from './model.ts';
import { target, } from './target.ts';

/**
 Build a chain of records `depth` levels deep, ending in `leaf`.

 @param depth - Number of nested `n` records above the leaf record.
 @param leaf - Record placed at the bottom of the chain.

 @returns Root of the chain.

 @example
 ```ts
 chain({ depth: 2, leaf: { end: true, }, }); // { n: { n: { end: true } } }
 ```
 */
function chain({ depth, leaf, }: { readonly depth: number; readonly leaf: object; },): object {
  return Array.from({ length: depth, },).reduce<object>((inner,) => ({ n: inner, }), leaf,);
}

/**
 Follow `n` links from a merged chain.

 @param root - Merged chain root.
 @param depth - Links to follow.

 @returns Node `depth` links below the root.

 @example
 ```ts
 descend({ root: { n: { x: 1, }, }, depth: 1, }); // { x: 1 }
 ```
 */
function descend({ root, depth, }: { readonly root: unknown; readonly depth: number; },): unknown {
  return Array.from({ length: depth, },).reduce<unknown>((node,) => Reflect.get(node as object, 'n',), root,);
}

await describe({
  name: 'deepmerge-ts accepted behaviour',
  children: [
    it({
      name: 'the default maxDepth merges up to depth 1000 and then lets the last value win',
      fn: async () => {
        /**
         Bottom record of the later input, reached at the default limit.
         */
        const laterLeaf = { later: true, };
        /**
         Merge of two chains whose leaves sit exactly at the limit.
         */
        const atLimit = target.deepmerge(
          chain({ depth: DEFAULT_MAX_DEPTH, leaf: { earlier: true, }, },),
          chain({ depth: DEFAULT_MAX_DEPTH, leaf: laterLeaf, },),
        );
        expect(descend({ root: atLimit, depth: DEFAULT_MAX_DEPTH, },),).toBe(laterLeaf,);

        /**
         Merge of the same chains one level shallower, still merged.
         */
        const belowLimit = target.deepmerge(
          chain({ depth: DEFAULT_MAX_DEPTH - 1, leaf: { earlier: true, }, },),
          chain({ depth: DEFAULT_MAX_DEPTH - 1, leaf: laterLeaf, },),
        );
        expect(descend({ root: belowLimit, depth: DEFAULT_MAX_DEPTH - 1, },),).toEqual({ earlier: true, later: true, },);
      },
    },),
    it({
      name: 'getters are read once per merge and flattened into data properties',
      fn: async () => {
        /**
         Number of getter invocations observed.
         */
        let reads = 0;
        /**
         Source exposing `a` through a getter.
         */
        const source = {
          get a() {
            reads += 1;
            return 1;
          },
        };
        /**
         Merge with a second record so the getter's owner is rebuilt.
         */
        const merged = target.deepmerge(source, { b: 2, },);
        expect(Reflect.getOwnPropertyDescriptor(merged, 'a',),).toEqual({
          configurable: true,
          enumerable: true,
          value: 1,
          writable: true,
        },);
        expect(reads,).toBe(1,);
      },
    },),
    it({
      name: 'a throwing getter propagates its error',
      fn: async () => {
        /**
         Error raised from the getter, compared by identity.
         */
        const raised = new Error('getter failed',);
        /**
         Source whose getter throws.
         */
        const source = {
          get a(): never {
            throw raised;
          },
        };
        expect(() => target.deepmerge(source, { b: 2, },),).toThrow(raised.message,);
      },
    },),
    it({
      name: 'deepmergeInto rejects a frozen target and accepts frozen sources',
      fn: async () => {
        expect(() => target.deepmergeInto(Object.freeze({ a: 1, },), { b: 2, },),).toThrow(TypeError,);
        /**
         Writable target merged from a frozen source.
         */
        const writable: Record<string, unknown> = { a: 1, };
        target.deepmergeInto(writable, Object.freeze({ b: Object.freeze({ c: 3, },), },),);
        expect(writable,).toEqual({ a: 1, b: { c: 3, }, },);
        expect(
          Object.isFrozen(target.deepmerge(Object.freeze({ a: 1, },), Object.freeze({ b: 2, },),),),
        ).toBe(false,);
      },
    },),
  ],
},);
