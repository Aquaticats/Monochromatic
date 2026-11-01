/**
 Aliasing behaviour accepted as documented by upstream's own tests
 (`tests/deepmerge-semantics.ts`), pinned so a change is noticed.

 - A key present in a single input is shared by reference, so mutating the
   `deepmerge` result there mutates that input ("shares references for keys
   present in a single source").
 - `deepmergeInto` merges into the target's own containers in place
   ("mutates the target's existing nested containers in place"), so a
   container the target reaches at two positions changes at both; the graph
   properties use tree targets for that reason.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

await describe({
  name: 'deepmerge-ts accepted aliasing',
  children: [
    it({
      name: 'a deepmerge result shares single-input subtrees with the input',
      fn: async () => {
        /**
         Input whose `k` only it provides.
         */
        const input = { k: { x: 1, }, };
        /**
         Merge result sharing `input.k`.
         */
        const merged = target.deepmerge(input, { other: 1, },);
        expect(merged.k,).toBe(input.k,);
        target.deepmergeInto(merged, { k: { y: 2, }, },);
        expect(input.k,).toEqual({ x: 1, y: 2, },);
      },
    },),
    it({
      name: 'deepmergeInto into a container the target reaches twice changes both positions',
      fn: async () => {
        /**
         Container reached at `a` and `b`.
         */
        const shared = { x: 1, };
        /**
         Target sharing it.
         */
        const intoTarget = { a: shared, b: shared, };
        target.deepmergeInto(intoTarget, { b: { y: 2, }, },);
        expect(intoTarget.a,).toEqual({ x: 1, y: 2, },);
        expect(intoTarget.a,).toBe(intoTarget.b,);
      },
    },),
  ],
},);
