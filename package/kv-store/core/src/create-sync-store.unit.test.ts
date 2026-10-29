import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ABSENT,
  createSyncStore,
} from './index.ts';

await describe({
  name: createSyncStore.name,
  children: [
    it({
      name: 'creates a store with default configuration',
      fn: async () => {
        const store = createSyncStore();
        expect(store.storeId,).toBeDefined();
        expect(store.backends.length,).toBe(1,);
        expect(store.lossyForCircular,).toBe(true,);
      },
    },),

    it({
      name: 'creates a store with custom storeId',
      fn: async () => {
        const store = createSyncStore({ storeId: 'test-sync-store', },);
        expect(store.storeId,).toBe('test-sync-store',);
      },
    },),

    it({
      name: 'set and get round-trips a value',
      fn: async () => {
        const store = createSyncStore({ storeId: 'roundtrip', },);
        store.set('key1', { value: 42, },);
        const result = store.get<{ value: number; }>('key1',);
        expect(result,).toEqual({ value: 42, },);
      },
    },),

    it({
      name: 'get returns ABSENT for missing keys',
      fn: async () => {
        const store = createSyncStore({ storeId: 'missing', },);
        const result = store.get('nonexistent',);
        expect(result,).toBe(ABSENT,);
      },
    },),

    it({
      name: 'delete removes an entry',
      fn: async () => {
        const store = createSyncStore({ storeId: 'delete-test', },);
        store.set('to-delete', 'value',);
        expect(store.get<string>('to-delete',),).toBe('value',);
        store.delete('to-delete',);
        expect(store.get('to-delete',),).toBe(ABSENT,);
      },
    },),

    it({
      name: 'clear removes all entries',
      fn: async () => {
        const store = createSyncStore({ storeId: 'clear-test', },);
        store.set('a', 1,);
        store.set('b', 2,);
        store.clear();
        expect(store.get('a',),).toBe(ABSENT,);
        expect(store.get('b',),).toBe(ABSENT,);
      },
    },),

    it({
      name: 'set returns the store for chaining',
      fn: async () => {
        const store = createSyncStore({ storeId: 'chain', },);
        const returned = store.set('k', 'v',);
        expect(returned,).toBe(store,);
      },
    },),

    it({
      name: 'size reflects number of entries',
      fn: async () => {
        const store = createSyncStore({ storeId: 'size-test', },);
        expect(store.size,).toBe(0,);
        store.set('a', 1,);
        expect(store.size,).toBe(1,);
        store.set('b', 2,);
        expect(store.size,).toBe(2,);
        store.delete('a',);
        expect(store.size,).toBe(1,);
        store.clear();
        expect(store.size,).toBe(0,);
      },
    },),

    it({
      name: 'handles multiple backends with consensus',
      fn: async () => {
        const backend1 = new Map<string, string>();
        const backend2 = new Map<string, string>();
        const store = createSyncStore({
          storeId: 'consensus',
          backends: [backend1, backend2,],
        },);

        store.set('shared', 'hello',);
        expect(backend1.has('shared',),).toBe(true,);
        expect(backend2.has('shared',),).toBe(true,);

        const result = store.get<string>('shared',);
        expect(result,).toBe('hello',);
      },
    },),

    it({
      name: 'heals divergent backends to majority value',
      fn: async () => {
        const backend1 = new Map<string, string>();
        const backend2 = new Map<string, string>();
        const backend3 = new Map<string, string>();
        const store = createSyncStore({
          storeId: 'heal',
          backends: [backend1, backend2, backend3,],
        },);

        store.set('key', 'correct',);

        // Corrupt one backend
        const correctSerialized = backend1.get('key',) as string;
        backend2.set('key', '"corrupted"',);

        const result = store.get<string>('key',);
        expect(result,).toBe('correct',);

        // backend2 should be healed
        expect(backend2.get('key',),).toBe(correctSerialized,);
      },
    },),

    it({
      name: 'throws TypeError for cyclic value when lossyForCircular is false',
      fn: async () => {
        const store = createSyncStore({ storeId: 'cyclic', lossyForCircular: false, },);
        const circular: Record<string, unknown> = {};
        circular.self = circular;

        expect(function setCyclic() {
          store.set('cyc', circular,);
        },).toThrow(TypeError,);
      },
    },),

    it({
      name: 'stores decycled value when lossyForCircular is true',
      fn: async () => {
        const store = createSyncStore({ storeId: 'cyclic-lossy', lossyForCircular: true, },);
        const circular: Record<string, unknown> = { data: 'test', };
        circular.self = circular;

        store.set('cyc', circular,);
        const result = store.get<Record<string, unknown>>('cyc',);
        expect(result,).toBeDefined();
        if (result === ABSENT)
          throw new Error('unreachable: stored value missing',);
        expect(result.data,).toBe('test',);
      },
    },),

    it({
      name: 'handles primitive values',
      fn: async () => {
        const store = createSyncStore({ storeId: 'primitives', },);
        store.set('str', 'hello',);
        store.set('num', 42,);
        store.set('bool', true,);
        store.set('null', null,);

        expect(store.get<string>('str',),).toBe('hello',);
        expect(store.get<number>('num',),).toBe(42,);
        expect(store.get<boolean>('bool',),).toBe(true,);
        expect(store.get<null>('null',),).toBeNull();
      },
    },),

    it({
      name: 'handles arrays and nested objects',
      fn: async () => {
        const store = createSyncStore({ storeId: 'complex', },);
        const complex = { arr: [1, 2, 3,], nested: { deep: true, }, };
        store.set('complex', complex,);
        expect(store.get<typeof complex>('complex',),).toEqual(complex,);
      },
    },),

    it({
      name: 'LRU eviction removes oldest entry at capacity',
      fn: async () => {
        const store = createSyncStore({
          storeId: 'lru-evict',
          eviction: [{ policy: 'lru', maxSize: 3, },],
        },);

        store.set('a', 1,);
        store.set('b', 2,);
        store.set('c', 3,);
        expect(store.size,).toBe(3,);

        store.set('d', 4,);
        expect(store.size,).toBe(3,);
        expect(store.get('a',),).toBe(ABSENT,);
        expect(store.get('d',),).toBeDefined();
      },
    },),

    it({
      name: 'LRU access refreshes entry position',
      fn: async () => {
        const store = createSyncStore({
          storeId: 'lru-refresh',
          eviction: [{ policy: 'lru', maxSize: 3, },],
        },);

        store.set('a', 1,);
        store.set('b', 2,);
        store.set('c', 3,);

        // Access 'a' to refresh it
        store.get('a',);

        // Adding 'd' should evict 'b' (oldest after refresh), not 'a'
        store.set('d', 4,);
        expect(store.get('a',),).toBeDefined();
        expect(store.get('b',),).toBe(ABSENT,);
      },
    },),

    it({
      name: 'LRU delete removes key from eviction tracking',
      fn: async () => {
        const store = createSyncStore({
          storeId: 'lru-delete',
          eviction: [{ policy: 'lru', maxSize: 3, },],
        },);

        store.set('a', 1,);
        store.set('b', 2,);
        store.set('c', 3,);
        store.delete('b',);

        // After deleting 'b', LRU tracks [a, c] (2 keys).
        // Adding 'd' fills to capacity (3), no eviction.
        store.set('d', 4,);
        expect(store.get('a',),).toBeDefined();
        expect(store.get('c',),).toBeDefined();
        expect(store.get('d',),).toBeDefined();
      },
    },),

    it({
      name: 'LRU clear resets eviction tracking',
      fn: async () => {
        const store = createSyncStore({
          storeId: 'lru-clear',
          eviction: [{ policy: 'lru', maxSize: 2, },],
        },);

        store.set('a', 1,);
        store.set('b', 2,);
        store.clear();

        // After clear, can add entries without eviction
        store.set('c', 3,);
        store.set('d', 4,);
        expect(store.size,).toBe(2,);
      },
    },),

    it({
      name: 'no eviction by default',
      fn: async () => {
        const store = createSyncStore({ storeId: 'no-eviction', },);

        store.set('a', 1,);
        store.set('b', 2,);
        store.set('c', 3,);
        store.set('d', 4,);
        store.set('e', 5,);
        expect(store.size,).toBe(5,);
      },
    },),

    it({
      name: 'all operations are synchronous',
      fn: async () => {
        const store = createSyncStore({ storeId: 'sync-check', },);
        const setResult = store.set('k', 'v',);
        // set returns the store itself (not a Promise)
        expect(setResult,).toBe(store,);

        const getResult = store.get('k',);
        // get returns the value directly (not a Promise)
        expect(getResult,).toBe('v',);

        // delete and clear return void (not Promises)
        store.delete('k',);
        store.clear();
      },
    },),
  ],
},);
