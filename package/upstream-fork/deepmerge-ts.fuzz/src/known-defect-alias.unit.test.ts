/**
 Known aliasing and cycle divergences in deepmerge-ts, found by the graph
 properties in `./alias-graph.property.unit.test.ts`.

 Same contract as `./known-defect.unit.test.ts`: each test asserts that the
 divergence still reproduces beside the outcome it should have, so the unit
 suite stays green while upstream is unfixed and turns red when behaviour
 changes. The comment on each test names the cause and the excluded region.

 Several false-cycle variants here are closed by the fork branch
 `fix/false-cycle-detection` (built with `fork:build`); against that build
 those tests turn red, while the ones marked "not closed by the fix branch"
 stay green.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

/**
 Read one own property of a merge result for assertions.

 @param value - Merge result or nested node.
 @param key - Property to read.

 @returns Property value.

 @example
 ```ts
 at({ value: { a: 1, }, key: 'a', }); // 1
 ```
 */
function at({ value, key, }: { readonly value: unknown; readonly key: PropertyKey; },): unknown {
  if (((typeof value) !== 'object') || (value === null))
    throw new TypeError(`at: cannot read ${String(key,)} of ${String(value,)}`,);
  return Reflect.get(value, key,);
}

await describe({
  name: 'deepmerge-ts aliasing and cycle defects still reproduce',
  children: [
    it({
      name: 'repeated deepmergeInto calls mutate the sources of earlier calls',
      // Cause: deepmergeInto stores a single source's nested container in the target by reference
      // (upstream test "shares references for keys present in a single source"), and a later call merges
      // into the target's containers in place (upstream test "mutates the target's existing nested
      // containers in place"), so the second call writes into the first call's source. Contradicts upstream
      // test "does not mutate sources across repeated calls into a reused target" for single-source keys.
      // Not reached by the properties, which check one call at a time.
      fn: async () => {
        /**
         Defaults merged first, then left untouched by the caller.
         */
        const defaults = {
          env: new Map([['k', { v: 1, },],],),
          flags: new Set(['x',],),
          plugins: ['a',],
          server: { port: 80, },
        };
        /**
         Configuration built by two calls into one target.
         */
        const config = {};
        target.deepmergeInto(config, defaults,);
        target.deepmergeInto(config, {
          env: new Map([['k', { w: 2, },],],),
          flags: new Set(['y',],),
          plugins: ['b',],
          server: { host: 'h', },
        },);
        expect(defaults.server,).toEqual({ host: 'h', port: 80, },);
        expect(defaults.plugins,).toEqual(['a', 'b',],);
        expect([...defaults.flags,],).toEqual(['x', 'y',],);
        expect(defaults.env.get('k',),).toEqual({ v: 1, w: 2, },);
      },
    },),
    it({
      name: 'one deepmergeInto call rewires a cyclic source node it stored in the target by reference',
      // Cause: the same single-source reference sharing, within one call: `a` exists only in the source, so
      // the target stores the source's node there, and cycle resolution later in the call writes the
      // target's nodes into it. Found by the campaign (seed 1395258848) in "deepmergeInto never mutates
      // its graph sources within one call"; that property now skips mutations of source nodes the target
      // references. Not closed by fix/false-cycle-detection.
      fn: async () => {
        /**
         Four-node cyclic graph from the campaign counterexample; `n2` is reached from `n1` via `c.a`.

         @returns Fresh nodes of one graph.
         */
        function graph(): Readonly<Record<'n0' | 'n1' | 'n2' | 'n4', Record<string, unknown>>> {
          /**
           Node reached from `n1` through `c`.
           */
          const n0: Record<string, unknown> = {};
          /**
           Source root.
           */
          const n1: Record<string, unknown> = {};
          /**
           Target root shape; the node the call rewires in the source.
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
         Graph whose `n2` is the target root.
         */
        const into = graph();
        /**
         Graph whose `n1` is the source root.
         */
        const source = graph();
        target.deepmergeInto(into.n2, source.n1,);
        // Documented: sources are never written. Actual: the target holds source `n2` at `c.a` and rewired it.
        expect(at({ key: 'a', value: into.n4, },),).toBe(source.n2,);
        expect(source.n2.c,).toBe(into.n1,);
        expect(source.n2.b,).toBe(into.n2,);
      },
    },),
    it({
      name: 'one deepmergeInto call rewires a cyclic source node reached through a node it stored by reference',
      // Cause: the same sharing one level further: the target stores source `s1` at `c.a`, then the call
      // merges into the target's graph again and writes a target node into `s2`, which the target reaches
      // only through `s1`. Found by the campaign (seed 1831879258) after the property skipped only directly
      // stored nodes; it now skips every source node the target reaches. Not closed by
      // fix/false-cycle-detection.
      fn: async () => {
        /**
         Five-node cyclic graph from the campaign counterexample; `s0` stays empty and unreached.

         @returns Fresh nodes of one graph.
         */
        function graph(): Readonly<Record<'s1' | 's2' | 's3' | 's4', Record<string, unknown>>> {
          /**
           Target root shape; its two keys share `s2`.
           */
          const s1: Record<string, unknown> = {};
          /**
           Node reached from `s1` through both keys; the node the call rewires in the source.
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
        /**
         Graph whose `s1` is the target root.
         */
        const into = graph();
        /**
         Graph whose `s4` is the source root.
         */
        const source = graph();
        target.deepmergeInto(into.s1, source.s4,);
        // Documented: sources are never written. Actual: the target holds source `s1` at `c.a`, and
        // source `s2` (reached through it) now points at the target's `s2`.
        expect(into.s2.a,).toBe(source.s1,);
        expect(source.s2.c,).toBe(into.s2,);
        expect(source.s1.c,).toBe(source.s2,);
      },
    },),
    it({
      name: 'false cycle: an input\'s value that is another input\'s ancestor or descendant resolves to an ancestor merge',
      // Cause: getCyclicReferenceDepth (src/utils.ts) matches any input's parents. Excluded region:
      // falseCycle in ./alias-bisim.ts. Closed by fix/false-cycle-detection.
      fn: async () => {
        /**
         Record whose child is another input's value at the same key.
         */
        const parent = { x: { z: 1, }, };
        /**
         Parent and child merged at the same key.
         */
        const merged = target.deepmerge({ a: parent, }, { a: parent.x, },);
        // Documented: `a` merges `parent` with `parent.x`, so `a.x` is `parent.x` merged with nothing new.
        expect(at({ key: 'x', value: at({ key: 'a', value: merged, },), },),).toBe(at({ key: 'a', value: merged, },),);

        /**
         Map value that is the other input's parent at that key.
         */
        const shared = { v: 1, };
        /**
         Maps merged where one value is the other's parent.
         */
        const maps = target.deepmerge(new Map([['k', shared,],],), new Map([['k', { k: shared, },],],),);
        /**
         Merged value under `k`.
         */
        const merged2 = maps.get('k',);
        expect(at({ key: 'k', value: merged2, },),).toBe(merged2,);
      },
    },),
    it({
      name: 'interlocking cycles across inputs resolve to the root merge instead of the swapped merge',
      // Cause: the same getCyclicReferenceDepth matching; a.o is b and b.o is a, so position `o` merges
      // (b, a), which upstream resolves to the root merge of (a, b). Closed by fix/false-cycle-detection.
      fn: async () => {
        /**
         First input, pointing at the second.
         */
        const first: Record<string, unknown> = { n: 'a', };
        /**
         Second input, pointing back at the first.
         */
        const second: Record<string, unknown> = { n: 'b', o: first, };
        first.o = second;
        /**
         Root merge; documented `o` is merge(b, a), whose `n` is 'a'.
         */
        const merged = target.deepmerge(first, second,);
        expect(at({ key: 'o', value: merged, },),).toBe(merged,);
        expect(at({ key: 'n', value: at({ key: 'o', value: merged, },), },),).toBe('b',);
      },
    },),
    it({
      name: 'not closed by the fix branch: a shared self-looping object from another input resolves to the root merge',
      // Cause: the value is a genuine self-loop, so the fix's key walk from it returns to it, yet it is
      // input 1's value at a position where input 0 holds the same object as an ancestor; the position
      // merges (loop, loop) but resolves to the root merge, gaining input 1's other keys.
      fn: async () => {
        /**
         Self-looping record.
         */
        const loop: Record<string, unknown> = { v: 1, };
        loop.a = loop;
        /**
         Merge with an input that shares `loop` at `a` and adds `c`.
         */
        const merged = target.deepmerge(loop, { a: loop, c: 1, },);
        expect(at({ key: 'a', value: merged, },),).toBe(merged,);
        // Documented unfolding: `a` is merge(loop, loop), which has no `c`.
        expect(at({ key: 'c', value: at({ key: 'a', value: merged, },), },),).toBe(1,);
      },
    },),
    it({
      name: 'intent question: a cycle among a subset of inputs pulls in keys from inputs that do not recur',
      // Cause: default mergeCircularReferences (src/defaults/vanilla.ts) resolves a position whose values
      // share a cyclic depth to that ancestor's merge, which included inputs absent at this position.
      // The graph properties accept upstream's cycle rule (upstream-rule mode) instead of asserting this.
      fn: async () => {
        /**
         Cyclic inputs recurring at `self`.
         */
        const first: Record<string, unknown> = { fromA: 1, };
        first.self = first;
        /**
         Second cyclic input recurring at `self`.
         */
        const second: Record<string, unknown> = { fromB: 2, };
        second.self = second;
        /**
         Merge with a third input that has no `self`.
         */
        const merged = target.deepmerge(first, second, { fromC: 3, },);
        expect(at({ key: 'self', value: merged, },),).toBe(merged,);
        // Unfolding: `self` is merge(first, second), which has no `fromC`.
        expect(at({ key: 'fromC', value: at({ key: 'self', value: merged, },), },),).toBe(3,);
      },
    },),
    it({
      name: 'intent question: a position with one cyclic value drops the other inputs\' values there',
      // Cause: default mergeCircularReferences resolves a position whose last value is cyclic to that
      // value's ancestor merge, discarding earlier non-cyclic values at the same position.
      fn: async () => {
        /**
         Non-cyclic value of the first input at `a`.
         */
        const plain = { fromN1: true, };
        /**
         Second input whose `a` is itself.
         */
        const looped: Record<string, unknown> = { b: plain, };
        looped.a = looped;
        /**
         Merge; unfolding makes `a` merge(plain, looped) with `fromN1`.
         */
        const merged = target.deepmerge({ a: plain, }, looped,);
        expect(at({ key: 'a', value: merged, },),).toBe(merged,);
        expect(
          Object.keys(at({ key: 'a', value: merged, },) as object,),
        ).toEqual(['a', 'b',],);
      },
    },),
  ],
},);
