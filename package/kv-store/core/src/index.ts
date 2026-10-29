/**
 Multi-backend key-value store with majority consensus and self-healing.
 
 Persists serialized values across one or more storage backends. Reads compute a
 canonical value by majority vote across all backends, fall back to a majority within
 the highest-priority tier on a cross-tier tie, then heal divergent backends to match.
 SuperJSON serialization round-trips structured values through string backends, with
 opt-in lossy handling for circular graphs. Both a synchronous and an asynchronous
 variant are provided.
 
 Hashing, serialization, and cycle detection are internal helpers, not public exports.
 
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
