/**
 Parity between the two hand-written store variants.

 `createStore` (`create-store.ts`) and `createSyncStore` (`create-sync-store.ts`)
 are deliberate twins, not one core with two adapters: the split and its reason
 are recorded in `doc/decision/memoize-sync-async-parallel-impls.md`. Because the
 bodies are separate, every behavioural difference must be proved intentional,
 and these cases fix the decisions both sides must make identically.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ABSENT,
  createStore,
  createSyncStore,
  EmptyKeyError,
} from '../dist/final/node/index.mjs';

/**
 Backend contents rendered as sorted key/value text.
 
 Compares what the two variants did to their backends without comparing backend
 identity, so one assertion covers every backend of both stores.
 
 @param backends - backends of one store, in the order that store holds them
 
 @returns one line per backend, each listing its sorted entries
 */
function snapshotBackends(backends: readonly Map<string, string>[],): string {
  return backends
    .map(function renderBackend(backend,) {
      return JSON.stringify(
        [...backend.entries(),]
          .toSorted(function byKey(
            [leftKey,],
            [rightKey,],
          ) {
            return leftKey.localeCompare(rightKey,);
          },),
      );
    },)
    .join('\n',);
}

/**
 Serialized form the majority of backends disagree with, so a read has to repair
 the backend holding it.
 */
const DIVERGENT_SERIALIZED = '"divergent"';

/**
 Serialized form only a minority of backends hold, so the majority verdict is
 absence and a read has to drop it.
 */
const MINORITY_SERIALIZED = '"minority"';

await describe({
  name: 'createStore and createSyncStore parity',
  children: [
    it({
      name: 'rewrites a divergent backend to the same contents in both variants',
      fn: async () => {
        /** Backends of the async store; the middle one is corrupted after the write. */
        const asyncBackends = [
          new Map<string, string>(),
          new Map<string, string>(),
          new Map<string, string>(),
        ] as const;
        /** Backends of the sync store, in the same starting shape as the async ones. */
        const syncBackends = [
          new Map<string, string>(),
          new Map<string, string>(),
          new Map<string, string>(),
        ] as const;
        /** Async store under test. */
        const asyncStore = await createStore({
          storeId: 'parity-heal-async',
          backends: asyncBackends,
        },);
        /** Sync store under test. */
        const syncStore = createSyncStore({
          storeId: 'parity-heal-sync',
          backends: syncBackends,
        },);
        /** Value both stores persist before the read that heals them. */
        const value = { value: 42, };

        await asyncStore.set('key', value,);
        syncStore.set('key', value,);
        asyncBackends[1].set('key', DIVERGENT_SERIALIZED,);
        syncBackends[1].set('key', DIVERGENT_SERIALIZED,);

        /** Value the async store reads back. */
        const asyncRead = await asyncStore.get('key',);
        /** Value the sync store reads back. */
        const syncRead = syncStore.get('key',);

        expect(asyncRead,).toEqual(value,);
        expect(syncRead,).toEqual(value,);
        expect(snapshotBackends(syncBackends,),).toBe(snapshotBackends(asyncBackends,),);
      },
    },),

    it({
      name: 'drops a minority entry to the same contents in both variants',
      fn: async () => {
        /** Backends of the async store; only the first holds the key. */
        const asyncBackends = [
          new Map<string, string>([['orphan', MINORITY_SERIALIZED,],],),
          new Map<string, string>(),
          new Map<string, string>(),
        ] as const;
        /** Backends of the sync store, in the same starting shape as the async ones. */
        const syncBackends = [
          new Map<string, string>([['orphan', MINORITY_SERIALIZED,],],),
          new Map<string, string>(),
          new Map<string, string>(),
        ] as const;
        /** Async store under test. */
        const asyncStore = await createStore({
          storeId: 'parity-absent-async',
          backends: asyncBackends,
        },);
        /** Sync store under test. */
        const syncStore = createSyncStore({
          storeId: 'parity-absent-sync',
          backends: syncBackends,
        },);

        expect(await asyncStore.get('orphan',),).toBe(ABSENT,);
        expect(syncStore.get('orphan',),).toBe(ABSENT,);
        expect(snapshotBackends(syncBackends,),).toBe(snapshotBackends(asyncBackends,),);
      },
    },),

    it({
      name: 'rejects an empty key for set, get and delete in both variants',
      fn: async () => {
        /** Backend of the async store, kept empty so a write would show up. */
        const asyncBackend = new Map<string, string>();
        /** Backend of the sync store, in the same starting shape as the async one. */
        const syncBackend = new Map<string, string>();
        /** Async store under test. */
        const asyncStore = await createStore({
          storeId: 'parity-empty-key-async',
          backends: [asyncBackend,],
        },);
        /** Sync store under test. */
        const syncStore = createSyncStore({
          storeId: 'parity-empty-key-sync',
          backends: [syncBackend,],
        },);
        /** Value both stores refuse to persist under an empty key. */
        const value = 'unaddressable';

        await expect(asyncStore.set('', value,),).rejects.toThrow(EmptyKeyError,);
        expect(function setEmptySyncKey() {
          syncStore.set('', value,);
        },).toThrow(EmptyKeyError,);
        await expect(asyncStore.get('',),).rejects.toThrow(EmptyKeyError,);
        expect(function getEmptySyncKey() {
          syncStore.get('',);
        },).toThrow(EmptyKeyError,);
        await expect(asyncStore.delete('',),).rejects.toThrow(EmptyKeyError,);
        expect(function deleteEmptySyncKey() {
          syncStore.delete('',);
        },).toThrow(EmptyKeyError,);

        expect(snapshotBackends([syncBackend,],),).toBe(snapshotBackends([asyncBackend,],),);
        expect(asyncBackend.size,).toBe(0,);
      },
    },),

    it({
      name: 'round-trips a shared reference with its identity intact in both variants',
      fn: async () => {
        /** One subobject referenced from two properties, which serializers must keep shared. */
        const shared = { depth: 1, };
        /** Acyclic value with a shared reference. */
        const value = { left: shared, right: shared, };
        /** Async store under test. */
        const asyncStore = await createStore({ storeId: 'parity-shared-async', },);
        /** Sync store under test. */
        const syncStore = createSyncStore({ storeId: 'parity-shared-sync', },);

        await asyncStore.set('dag', value,);
        syncStore.set('dag', value,);
        /** Value the async store reads back. */
        const asyncRead = await asyncStore.get<typeof value>('dag',);
        /** Value the sync store reads back. */
        const syncRead = syncStore.get<typeof value>('dag',);

        expect(asyncRead,).toEqual(value,);
        expect(syncRead,).toEqual(value,);
        if ((asyncRead === ABSENT) || (syncRead === ABSENT))
          throw new Error('unreachable: stored value missing',);
        expect(asyncRead.left,).toBe(asyncRead.right,);
        expect(syncRead.left,).toBe(syncRead.right,);
      },
    },),
  ],
},);
