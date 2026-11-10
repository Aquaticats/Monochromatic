import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parse as superjsonParse,
  stringify as superjsonStringify,
} from 'superjson';

import {
  hasCycle,
  serializeValue,
} from '../dist/final/node/index.mjs';

/**
 Marker `decircular` writes in place of a reference that closes a cycle, so a
 lossy serialization can be asserted without depending on traversal order.
 */
const DISCARDED_CYCLE_MARKER = '[Circular *]';

/**
 Depth of the deepest acyclic chain the walker must survive without a stack
 overflow; far past any realistic value and past the default call-stack depth
 of the runtimes the package targets.
 */
const DEEP_CHAIN_DEPTH = 20_000;

await describe({
  name: 'hasCycle and serializeValue',
  children: [
    it({
      name: 'reports a value whose two properties share one object as acyclic',
      fn: async () => {
        /** Single subobject referenced twice, which is a shared reference, not a back-edge. */
        const shared = { depth: 1, };
        /** Diamond: both branches reach the same object. */
        const diamond = { left: shared, right: shared, };

        expect(hasCycle(diamond,),).toBe(false,);
      },
    },),

    it({
      name: 'reports a repeated reference at different depths as acyclic',
      fn: async () => {
        /** Leaf reached once directly and once through a nested branch. */
        const leaf = { value: 1, };
        /** Acyclic value whose repeated reference sits at two depths. */
        const value = { shallow: leaf, deep: { nested: { leaf, }, }, };

        expect(hasCycle(value,),).toBe(false,);
      },
    },),

    it({
      name: 'reports a repeated array reference as acyclic',
      fn: async () => {
        /** Array referenced from two properties of one value. */
        const items = [1, 2, 3,];
        expect(hasCycle({ first: items, second: items, },),).toBe(false,);
      },
    },),

    it({
      name: 'serializes a shared-reference value with the plain serializer when lossyForCircular is false',
      fn: async () => {
        /** Shared subobject that must survive as one serialization, not two copies. */
        const shared = { a: 1, };
        /** Acyclic value with a shared reference. */
        const diamond = { x: shared, y: shared, };

        /** Strict-mode serialization of an acyclic value. */
        const serialized = serializeValue({
          value: diamond,
          serializer: JSON.stringify,
          lossyForCircular: false,
        },);

        expect(serialized,).toBe(JSON.stringify(diamond,),);
      },
    },),

    it({
      name: 'keeps a shared reference distinct from a cycle when lossyForCircular is true',
      fn: async () => {
        /** Shared subobject whose identity the serializer must preserve. */
        const shared = { a: 1, };
        /** Acyclic value with a shared reference. */
        const diamond = { x: shared, y: shared, };

        /** Run through the lossy path, which must leave an acyclic value untouched. */
        const serialized = serializeValue({
          value: diamond,
          serializer: superjsonStringify,
          lossyForCircular: true,
        },);
        /** Value revived from `serialized`; superjson records referential equalities. */
        const revived = superjsonParse<{ x: { a: number; }; y: { a: number; }; }>(serialized,);

        expect(revived.x,).toBe(revived.y,);
        expect(revived,).toEqual(diamond,);
      },
    },),

    it({
      name: 'detects a self-referencing value',
      fn: async () => {
        /** Value whose `self` property points back at itself. */
        const cyclic: Record<string, unknown> = {};
        cyclic.self = cyclic;

        expect(hasCycle(cyclic,),).toBe(true,);
      },
    },),

    it({
      name: 'detects a cycle whose head is also shared',
      fn: async () => {
        /** Cycle member whose `b` property is set below. */
        const a: Record<string, unknown> = {};
        /** Second cycle member closing the loop back to `a`. */
        const b: Record<string, unknown> = { a, };
        a.b = b;
        /** Acyclic-looking top level that references the cycle head twice. */
        const root = { first: a, second: a, };

        expect(hasCycle(root,),).toBe(true,);
      },
    },),

    it({
      name: 'throws TypeError for a self-referencing value when lossyForCircular is false',
      fn: async () => {
        /** Value whose `self` property points back at itself. */
        const cyclic: Record<string, unknown> = {};
        cyclic.self = cyclic;

        expect(function serializeCyclic() {
          serializeValue({
            value: cyclic,
            serializer: JSON.stringify,
            lossyForCircular: false,
          },);
        },).toThrow(TypeError,);
      },
    },),

    it({
      name: 'stores a decycled value when lossyForCircular is true',
      fn: async () => {
        /** Value whose `self` property points back at itself. */
        const cyclic: Record<string, unknown> = { data: 'test', };
        cyclic.self = cyclic;

        /** Lossy serialization of a genuinely cyclic value. */
        const serialized = serializeValue({
          value: cyclic,
          serializer: JSON.stringify,
          lossyForCircular: true,
        },);
        /** Revived lossy value whose cycle became the marker. */
        const revived = JSON.parse(serialized,) as Record<string, unknown>;

        expect(revived.data,).toBe('test',);
        expect(revived.self,).toBe(DISCARDED_CYCLE_MARKER,);
      },
    },),

    it({
      name: 'reports non-object values as acyclic',
      fn: async () => {
        expect(hasCycle(42,),).toBe(false,);
        expect(hasCycle('text',),).toBe(false,);
        expect(hasCycle(true,),).toBe(false,);
        expect(hasCycle(null,),).toBe(false,);
        expect(hasCycle(undefined,),).toBe(false,);
      },
    },),

    it({
      name: 'walks a deep acyclic chain without exhausting the call stack',
      fn: async () => {
        /** Head of the chain handed to the walker. */
        const root: Record<string, unknown> = {};
        /** Node being extended; moved down the chain as it is built. */
        let cursor = root;
        for (let level = 0; level < DEEP_CHAIN_DEPTH; level += 1) {
          /** Child of the current node, linked in on this iteration. */
          const next: Record<string, unknown> = {};
          cursor.next = next;
          cursor = next;
        }

        expect(hasCycle(root,),).toBe(false,);
      },
    },),
  ],
},);
