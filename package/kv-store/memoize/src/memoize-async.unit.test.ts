import { createStore, } from '@monochromatic-dev/kv-store-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { memoizeAsync, } from './index.ts';

await describe({
  name: memoizeAsync.name,
  children: [
    it({
      name: 'returns computed value on first call',
      fn: async () => {
        async function double(x: number,): Promise<number> {
          return x * 2;
        }
        const memoized = await memoizeAsync({ fn: double, keyFn: String, },);
        expect(await memoized({ args: [5,], salt: 'v1', },),).toBe(10,);
      },
    },),

    it({
      name: 'returns cached value on subsequent calls',
      fn: async () => {
        const calls: number[] = [];
        async function double(x: number,): Promise<number> {
          calls.push(x,);
          return x * 2;
        }
        const memoized = await memoizeAsync({ fn: double, keyFn: String, },);

        expect(await memoized({ args: [5,], salt: 'v1', },),).toBe(10,);
        expect(await memoized({ args: [5,], salt: 'v1', },),).toBe(10,);
        expect(calls.length,).toBe(1,);
      },
    },),

    it({
      name: 'deduplicates concurrent in-flight calls',
      fn: async () => {
        const calls: number[] = [];
        async function double(x: number,): Promise<number> {
          calls.push(x,);
          return x * 2;
        }
        const memoized = await memoizeAsync({ fn: double, keyFn: String, },);

        const [result1, result2,] = await Promise.all([
          memoized({ args: [5,], salt: 'v1', },),
          memoized({ args: [5,], salt: 'v1', },),
        ],);
        expect(result1,).toBe(10,);
        expect(result2,).toBe(10,);
        expect(calls.length,).toBe(1,);
      },
    },),

    it({
      name: 'rejection clears the inflight entry and recomputes on retry',
      fn: async () => {
        const calls: number[] = [];
        async function failFirst(x: number,): Promise<number> {
          calls.push(x,);
          if (calls.length
            === 1)
            throw new Error('first call fails',);
          return x * 2;
        }
        const memoized = await memoizeAsync({ fn: failFirst, keyFn: String, },);

        await expect(memoized({ args: [5,], salt: 'v1', },),).rejects.toThrow(
          'first call fails',
        );

        // The store was never populated (set is gated on fn resolving), and the
        // inflight entry was cleared on throw, so the retry recomputes.
        const result = await memoized({ args: [5,], salt: 'v1', },);
        expect(result,).toBe(10,);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'different salt recomputes for same args',
      fn: async () => {
        const calls: number[] = [];
        async function identity(x: number,): Promise<number> {
          calls.push(x,);
          return x;
        }
        const memoized = await memoizeAsync({ fn: identity, keyFn: String, },);

        expect(await memoized({ args: [1,], salt: 'a', },),).toBe(1,);
        expect(await memoized({ args: [1,], salt: 'a', },),).toBe(1,);
        expect(await memoized({ args: [1,], salt: 'b', },),).toBe(1,);
        expect(await memoized({ args: [1,], salt: 'b', },),).toBe(1,);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'clear empties the cache',
      fn: async () => {
        const calls: number[] = [];
        async function identity(x: number,): Promise<number> {
          calls.push(x,);
          return x;
        }
        const memoized = await memoizeAsync({ fn: identity, keyFn: String, },);

        await memoized({ args: [1,], salt: 'v1', },);
        await memoized.clear();

        await memoized({ args: [1,], salt: 'v1', },);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'delete removes a specific entry',
      fn: async () => {
        const calls: number[] = [];
        async function identity(x: number,): Promise<number> {
          calls.push(x,);
          return x;
        }
        const memoized = await memoizeAsync({ fn: identity, keyFn: String, },);

        await memoized({ args: [1,], salt: 'v1', },);
        await memoized.delete('1:v1',);

        await memoized({ args: [1,], salt: 'v1', },);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'store provides access to the underlying Store',
      fn: async () => {
        async function identity(x: number,): Promise<number> {
          return x;
        }
        const memoized = await memoizeAsync({ fn: identity, keyFn: String, },);

        expect(memoized.store,).toBeDefined();
        expect(memoized.store.storeId,).toBeDefined();
      },
    },),

    it({
      name: 'different args produce separate cache entries',
      fn: async () => {
        const calls: number[] = [];
        async function double(x: number,): Promise<number> {
          calls.push(x,);
          return x * 2;
        }
        const memoized = await memoizeAsync({ fn: double, keyFn: String, },);

        expect(await memoized({ args: [1,], salt: 'v1', },),).toBe(2,);
        expect(await memoized({ args: [2,], salt: 'v1', },),).toBe(4,);
        expect(calls.length,).toBe(2,);
      },
    },),

    it({
      name: 'LRU eviction at capacity recomputes evicted entries',
      fn: async () => {
        const calls: number[] = [];
        async function identity(x: number,): Promise<number> {
          calls.push(x,);
          return x;
        }
        const store = await createStore({
          storeId: 'memoize-lru-async-test',
          eviction: [{ policy: 'lru', maxSize: 2, },],
        },);
        const memoized = await memoizeAsync({ fn: identity, keyFn: String, store, },);

        await memoized({ args: [1,], salt: 'v1', },);
        await memoized({ args: [2,], salt: 'v1', },);
        await memoized({ args: [3,], salt: 'v1', },);

        // Entry 1 was evicted when 3 pushed past maxSize 2, so re-fetching 1 recomputes.
        const before = calls.length;
        await memoized({ args: [1,], salt: 'v1', },);
        expect(calls.length
          - before,).toBe(1,);
      },
    },),

    it({
      name: 'undefined resolution is cached and recomputes once',
      fn: async () => {
        const calls: number[] = [];
        async function resolvesUndefined(x: number,): Promise<undefined> {
          calls.push(x,);
        }
        const memoized = await memoizeAsync({ fn: resolvesUndefined, keyFn: String, },);

        expect(await memoized({ args: [1,], salt: 'v1', },),).toBeUndefined();
        expect(await memoized({ args: [1,], salt: 'v1', },),).toBeUndefined();
        expect(calls.length,).toBe(1,);
      },
    },),
  ],
},);
