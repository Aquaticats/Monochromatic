/**
 Example tests for `./alias-rewire.ts`: edge snapshots detect writes by
 identity, and `sharedWithRoot` finds exactly the input nodes a result
 reaches. The campaign counterexamples (seeds 1395258848 and 1831879258)
 double as positive controls for the narrowed property in
 `./alias-graph.property.unit.test.ts`, and a write into an unshared source
 node shows the narrowing still reports one.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  reachableContainers,
  rewiredNodes,
  snapshotEdges,
  sharedWithRoot,
} from './alias-rewire.ts';
import { target, } from './target.ts';

/**
 Four-node cyclic graph from the campaign counterexample.

 @returns Fresh nodes of one graph.

 @example
 ```ts
 const { n1, } = counterexampleGraph();
 ```
 */
function counterexampleGraph(): Readonly<Record<'n0' | 'n1' | 'n2' | 'n4', Record<string, unknown>>> {
  /**
   Node reached from `n1` through `c`.
   */
  const n0: Record<string, unknown> = {};
  /**
   Source root.
   */
  const n1: Record<string, unknown> = {};
  /**
   Target root shape.
   */
  const n2: Record<string, unknown> = {};
  /**
   Node shared by `n1.b` and `n2.c`.
   */
  const n4: Record<string, unknown> = {};
  n0.a = n2;
  n1.c = n0;
  n1.b = n4;
  n2.c = n4;
  n2.b = n1;
  n4.b = n0;
  return { n0, n1, n2, n4, };
}

/**
 Five-node cyclic graph from the second campaign counterexample.

 @returns Fresh nodes of one graph.

 @example
 ```ts
 const { s4, } = ladderGraph();
 ```
 */
function ladderGraph(): Readonly<Record<'s1' | 's2' | 's3' | 's4', Record<string, unknown>>> {
  /**
   Target root shape; its two keys share `s2`.
   */
  const s1: Record<string, unknown> = {};
  /**
   Node reached from `s1` through both keys.
   */
  const s2: Record<string, unknown> = {};
  /**
   Node closing the cycle back to `s1`.
   */
  const s3: Record<string, unknown> = {};
  /**
   Source root; its two keys share `s3`.
   */
  const s4: Record<string, unknown> = {};
  s1.c = s2;
  s1.b = s2;
  s2.c = s3;
  s3.a = s1;
  s4.c = s3;
  s4.b = s3;
  return { s1, s2, s3, s4, };
}

await describe({
  name: 'alias rewire helpers',
  children: [
    it({
      name: 'reachableContainers visits each container of a cyclic graph once',
      fn: async () => {
        /**
         Graph under test.
         */
        const graph = counterexampleGraph();
        expect(reachableContainers([graph.n1, 1, graph.n1,],).size,).toBe(4,);
      },
    },),
    it({
      name: 'rewiredNodes reports added keys and same-key identity changes only',
      fn: async () => {
        /**
         Node that gains a key.
         */
        const grown: Record<string, unknown> = { a: 1, };
        /**
         Node whose child is replaced by an equal-looking object.
         */
        const replaced: Record<string, unknown> = { a: {}, };
        /**
         Node left alone.
         */
        const untouched: Record<string, unknown> = { a: 1, };
        /**
         Edges before the writes.
         */
        const edges = snapshotEdges(new Set([grown, replaced, untouched,],),);
        grown.b = 2;
        replaced.a = {};
        expect(rewiredNodes(edges,),).toEqual([grown, replaced,],);
      },
    },),
    it({
      name: 'the second campaign counterexample rewires a shared source node the target does not store directly',
      fn: async () => {
        /**
         Graph whose `s1` is the target root.
         */
        const into = ladderGraph();
        /**
         Graph whose `s4` is the source root.
         */
        const source = ladderGraph();
        /**
         Source edges before the merge.
         */
        const edges = snapshotEdges(reachableContainers([source.s4,],),);
        target.deepmergeInto(into.s1, source.s4,);
        // The target stores `s1` at `c.a` and reaches `s2` through it; `s3` and `s4` stay unshared.
        expect(into.s2.a,).toBe(source.s1,);
        expect([...sharedWithRoot({ edges, root: into.s1, },),],).toEqual([source.s1, source.s2,],);
        expect(rewiredNodes(edges,),).toEqual([source.s2,],);
      },
    },),
    it({
      name: 'a write into a source node the result does not reach stays outside the shared set',
      fn: async () => {
        /**
         Graph whose `s1` is the target root.
         */
        const into = ladderGraph();
        /**
         Graph whose `s4` is the source root.
         */
        const source = ladderGraph();
        /**
         Source edges before the merge.
         */
        const edges = snapshotEdges(reachableContainers([source.s4,],),);
        target.deepmergeInto(into.s1, source.s4,);
        source.s3.extra = 1;
        /**
         Source nodes the target reaches.
         */
        const shared = sharedWithRoot({ edges, root: into.s1, },);
        expect(rewiredNodes(edges,).filter(function unexplained(node,) {
          return !shared.has(node,);
        },),).toEqual([source.s3,],);
      },
    },),
    it({
      name: 'the campaign counterexample rewires exactly the source node the target stores',
      fn: async () => {
        /**
         Graph whose `n2` is the target root.
         */
        const into = counterexampleGraph();
        /**
         Graph whose `n1` is the source root.
         */
        const source = counterexampleGraph();
        /**
         Source edges before the merge.
         */
        const edges = snapshotEdges(reachableContainers([source.n1,],),);
        target.deepmergeInto(into.n2, source.n1,);
        expect([...sharedWithRoot({ edges, root: into.n2, },),],).toEqual([source.n2,],);
        expect(rewiredNodes(edges,),).toEqual([source.n2,],);
      },
    },),
  ],
},);
