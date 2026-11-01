import {
  ABSENT,
  createSyncStore,
} from '@monochromatic-dev/kv-store-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { memoize, } from './index.ts';

await describe({
  name: memoize.name,
  children: [
    it({
      name: 'returns computed value on first call',
      fn: async () => {
        function double(x: number,): number {
          return x * 2;
        }
        const memoized = memoize({ fn: double, keyFn: String, },);
        expect(memoized({ args: [5,], salt: 'v1', },),).toBe(10,);
      },
    },),

    it({
      name: 'returns cached value on subsequent calls with same args and salt',
      fn: async () => {
        const calls: number[] = [];
        function double(x: number,): number {
          calls.push(x,);
          return x * 2;
        }
        const memoized = memoize({ fn: double, keyFn: String, },);

        expect(memoized({ args: [5,], salt: 'v1', },),).toBe(10,);
        expect(memoized({ args: [5,], salt: 'v1', },),).toBe(10,);
        expect(calls.length,).toBe(1,);
      },
    },),

    it({
      name: 'computes separately for different args',
      fn: async () => {
        const calls: number[] = [];
        function double(x: number,): number {
          calls.push(x,);
          return x * 2;
        }
        const memoized = memoize({ fn: double, keyFn: String, },);

        expect(memoized({ args: [5,], salt: 'v1', },),).toBe(10,);
        expect(memoized({ args: [6,], salt: 'v1', },),).toBe(12,);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'different salt produces different cache entries',
      fn: async () => {
        const calls: number[] = [];
        function double(x: number,): number {
          calls.push(x,);
          return x * 2;
        }
        const memoized = memoize({ fn: double, keyFn: String, },);

        memoized({ args: [5,], salt: 'v1', },);
        memoized({ args: [5,], salt: 'v2', },);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'composite keyFn over a multi-field argument',
      fn: async () => {
        const calls: number[] = [];
        function add({
          a,
          b,
        }: Readonly<{ a: number; b: number; }>,): number {
          calls.push(a + b,);
          return a + b;
        }
        function keyOf({
          a,
          b,
        }: Readonly<{ a: number; b: number; }>,): string {
          return `${String(a,)}:${String(b,)}`;
        }
        const memoized = memoize({ fn: add, keyFn: keyOf, },);

        expect(memoized({ args: [{ a: 1, b: 2, },], salt: 'v1', },),).toBe(3,);
        expect(memoized({ args: [{ a: 1, b: 2, },], salt: 'v1', },),).toBe(3,);
        expect(memoized({ args: [{ a: 2, b: 1, },], salt: 'v1', },),).toBe(3,);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'size reflects cache size',
      fn: async () => {
        function identity(x: number,): number {
          return x;
        }
        const memoized = memoize({ fn: identity, keyFn: String, },);

        expect(memoized.size,).toBe(0,);
        memoized({ args: [1,], salt: 'v1', },);
        expect(memoized.size,).toBe(1,);
        memoized({ args: [2,], salt: 'v1', },);
        expect(memoized.size,).toBe(2,);
        memoized({ args: [1,], salt: 'v1', },);
        expect(memoized.size,).toBe(2,);
      },
    },),

    it({
      name: 'clear empties the cache',
      fn: async () => {
        function identity(x: number,): number {
          return x;
        }
        const memoized = memoize({ fn: identity, keyFn: String, },);

        memoized({ args: [1,], salt: 'v1', },);
        memoized({ args: [2,], salt: 'v1', },);
        expect(memoized.size,).toBe(2,);
        memoized.clear();
        expect(memoized.size,).toBe(0,);
      },
    },),

    it({
      name: 'delete removes a specific entry',
      fn: async () => {
        function identity(x: number,): number {
          return x;
        }
        const memoized = memoize({ fn: identity, keyFn: String, },);

        memoized({ args: [1,], salt: 'v1', },);
        memoized({ args: [2,], salt: 'v1', },);
        expect(memoized.size,).toBe(2,);
        memoized.delete('1:v1',);
        expect(memoized.size,).toBe(1,);
      },
    },),

    it({
      name: 'store provides access to the underlying SyncStore',
      fn: async () => {
        function triple(x: number,): number {
          return x * 3;
        }
        const memoized = memoize({ fn: triple, keyFn: String, },);

        memoized({ args: [7,], salt: 's', },);
        expect(memoized.store.get<number>('7:s',),).toBe(21,);
      },
    },),

    it({
      name: 'LRU eviction removes oldest entry at capacity',
      fn: async () => {
        function identity(x: number,): number {
          return x;
        }
        const store = createSyncStore({
          storeId: 'memoize-lru-test',
          eviction: [{ policy: 'lru', maxSize: 3, },],
        },);
        const memoized = memoize({ fn: identity, keyFn: String, store, },);

        memoized({ args: [1,], salt: 'v1', },);
        memoized({ args: [2,], salt: 'v1', },);
        memoized({ args: [3,], salt: 'v1', },);
        expect(memoized.size,).toBe(3,);

        memoized({ args: [4,], salt: 'v1', },);
        expect(memoized.size,).toBe(3,);
        expect(memoized.store.get('1:v1',),).toBe(ABSENT,);
        expect(memoized.store.get('4:v1',),).toBeDefined();
      },
    },),

    it({
      name: 'LRU access refreshes entry position',
      fn: async () => {
        function identity(x: number,): number {
          return x;
        }
        const store = createSyncStore({
          storeId: 'memoize-lru-refresh-test',
          eviction: [{ policy: 'lru', maxSize: 3, },],
        },);
        const memoized = memoize({ fn: identity, keyFn: String, store, },);

        memoized({ args: [1,], salt: 'v1', },);
        memoized({ args: [2,], salt: 'v1', },);
        memoized({ args: [3,], salt: 'v1', },);

        // Access 1 to refresh its position before adding a fourth entry.
        memoized({ args: [1,], salt: 'v1', },);

        // Adding 4 should evict 2 (oldest after refresh), not 1.
        memoized({ args: [4,], salt: 'v1', },);
        expect(memoized.store.get('1:v1',),).toBeDefined();
        expect(memoized.store.get('2:v1',),).toBe(ABSENT,);
      },
    },),

    it({
      name: 'same args with different salt are cached independently',
      fn: async () => {
        const calls: number[] = [];
        function identity(x: number,): number {
          calls.push(x,);
          return x;
        }
        const memoized = memoize({ fn: identity, keyFn: String, },);

        memoized({ args: [1,], salt: 'a', },);
        memoized({ args: [1,], salt: 'a', },);
        memoized({ args: [1,], salt: 'b', },);
        memoized({ args: [1,], salt: 'b', },);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'custom store is used when provided',
      fn: async () => {
        const customStore = createSyncStore({ storeId: 'memoize-custom', },);
        function double(x: number,): number {
          return x * 2;
        }
        const memoized = memoize({ fn: double, keyFn: String, store: customStore, },);

        memoized({ args: [5,], salt: 'v1', },);
        expect(customStore.get<number>('5:v1',),).toBe(10,);
        expect(memoized.store,).toBe(customStore,);
      },
    },),

    it({
      name: 'undefined return value is cached and recomputes once',
      fn: async () => {
        const calls: number[] = [];
        function returnsUndefined(x: number,): undefined {
          calls.push(x,);
        }
        const memoized = memoize({ fn: returnsUndefined, keyFn: String, },);

        expect(memoized({ args: [1,], salt: 'v1', },),).toBeUndefined();
        expect(memoized({ args: [1,], salt: 'v1', },),).toBeUndefined();
        expect(calls.length,).toBe(1,);
      },
    },),
  ],
},);
