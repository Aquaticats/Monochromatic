/**
 Multi-backend key-value store with majority consensus and self-healing.
 
 Persists serialized values across one or more storage backends. Reads compute a
 canonical value by majority vote across all backends, fall back to a majority within
 the highest-priority tier on a cross-tier tie, then heal divergent backends to match.
 SuperJSON serialization round-trips structured values through string backends, with
 opt-in lossy handling for circular graphs. Both a synchronous and an asynchronous
 variant are provided, as hand-written twins rather than one core with two
 adapters: the split and its reason are recorded in
 `doc/decision/memoize-sync-async-parallel-impls.md`.
 
 Hashing, serialization, and cycle detection are internal helpers; the ones the
 built-artifact tests exercise are exported at the end of this module, marked
 `@internal`.
 
 @example
 Async store:
 ```ts
 import { createStore } from '\@monochromatic-dev/kv-store-core';
 
 const store = await createStore({
   storeId: 'cache',
   eviction: [{ policy: 'lru', maxSize: 256 }],
 });
 await store.set('user-1', { name: 'Ada' });
 const user = await store.get<{ name: string }>('user-1');
 ```
 
 @example
 Sync store:
 ```ts
 import { createSyncStore } from '\@monochromatic-dev/kv-store-core';
 
 const store = createSyncStore({ storeId: 'sync-cache' });
 store.set('answer', 42);
 const answer = store.get<number>('answer');
 ```
 
 @packageDocumentation
 */

export type {
  BaseStoreConfig,
  BaseStoreFields,
  Deserializer,
  EvictionPolicy,
  LruEvictionPolicy,
  Serializer,
  StorageBackend,
  Store,
  StoreConfig,
  SyncStorageBackend,
  SyncStore,
  SyncStoreConfig,
} from './types.ts';

export type { DefaultBackendsBuilder, } from './backends-async.ts';

export { configureDefaultBackendsBuilder, } from './backends-async.ts';

export { ABSENT, } from './constants.ts';

export { createStore, } from './create-store.ts';

export { createSyncStore, } from './create-sync-store.ts';

//region Built-artifact test surface

/**
 Circular-reference detection as {@link serializeValue} applies it, exported so
 built-artifact tests can reach it.
 
 @internal
 */
export { hasCycle, } from './has-cycle.ts';

/**
 Value serialization including circular-graph handling, exported so
 built-artifact tests can reach it.
 
 @internal
 */
export { serializeValue, } from './serialize.ts';

//endregion
