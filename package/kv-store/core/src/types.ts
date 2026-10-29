import type { Promisable, } from 'type-fest';

import type { ABSENT, } from './constants.ts';

/**
 Serialize unknown data to deterministic string form.
 
 @example
 ```ts
 const serialize: Serializer = JSON.stringify;
 ```
 */
export type Serializer = (toSerialize: unknown,) => string;

/* oxlint-disable typescript/no-unnecessary-type-parameters -- T is the caller-specified return type for typed reads; a single use is the intended call-site-inference shape, not redundancy */
/**
 Deserialize string data into typed value.
 
 @typeParam T - target value type
 
 @example
 ```ts
 const deserialize: Deserializer = JSON.parse;
 ```
 */
export type Deserializer = <const T = unknown,>(toDeserialize: string,) => T;
/* oxlint-enable typescript/no-unnecessary-type-parameters */

/**
 LRU eviction policy that removes the least recently accessed entry
 when capacity is exceeded.
 
 @example
 ```ts
 const lru: LruEvictionPolicy = { policy: 'lru', maxSize: 1024 };
 ```
 */
export type LruEvictionPolicy = {
  /**
   Eviction strategy discriminant.
   */
  readonly policy: 'lru';
  /**
   Maximum entries before the oldest is evicted.
   */
  readonly maxSize: number;
};

/**
 Discriminated union of eviction policies.
 Multiple policies can be active simultaneously on a single store
 by passing an array to {@link BaseStoreConfig.eviction}.
 
 @example
 ```ts
 const policy: EvictionPolicy = { policy: 'lru', maxSize: 256 };
 ```
 */
export type EvictionPolicy = LruEvictionPolicy;

/**
 Shared configuration fields for both sync and async store constructors.
 Extended by {@link StoreConfig} and {@link SyncStoreConfig} which add their
 backend-specific `backends` field.
 
 @example
 ```ts
 const base: BaseStoreConfig = {
   storeId: 'my-cache',
   lossyForCircular: false,
 };
 ```
 */
export type BaseStoreConfig = {
  /**
   Serializer for persisted values. Defaults to `superjson.stringify`.
   */
  readonly serializer?: Serializer;
  /**
   Deserializer for loaded values. Defaults to `superjson.parse`.
   */
  readonly deserializer?: Deserializer;
  /**
   When true, cyclic graphs are decycled and persisted lossy instead of throwing.
   Defaults to `true`.
   */
  readonly lossyForCircular?: boolean;
  /**
   Unique identifier used for namespacing. Defaults to random UUID.
   */
  readonly storeId?: string;
  /**
   Eviction policies for bounding store capacity.
   Multiple policies can be active simultaneously.
   Defaults to no eviction (unbounded).
   
   @example
   ```ts
   { eviction: [{ policy: 'lru', maxSize: 256 }] }
   ```
   */
  readonly eviction?: readonly EvictionPolicy[];
};

/**
 Shared readonly fields exposed on both sync and async store instances.
 Extended by {@link Store} and {@link SyncStore} which add their method signatures.
 
 @typeParam TBackend - storage backend type
 
 @example
 ```ts
 function logStoreInfo(store: BaseStoreFields<StorageBackend>): void {
   console.log(store.storeId, store.backends.length);
 }
 ```
 */
export type BaseStoreFields<TBackend,> = {
  /**
   Unique identifier for this store instance.
   */
  readonly storeId: string;
  /**
   Serializer used when persisting values.
   */
  readonly serializer: Serializer;
  /**
   Deserializer used when loading values.
   */
  readonly deserializer: Deserializer;
  /**
   Whether to accept lossy serialization of cyclic graphs.
   */
  readonly lossyForCircular: boolean;
  /**
   Ordered backends used by this store.
   */
  readonly backends: readonly [
    TBackend,
    ...TBackend[],
  ];
};

/**
 Backend storage interface for the async {@link Store}.
 Compatible with `Map<string, string>` and the Web Storage API shape.
 
 @example
 ```ts
 const backend: StorageBackend = new Map<string, string>();
 ```
 */
export type StorageBackend = {
  /**
   Retrieve serialized value by key.
   
   @param key - lookup key
   @returns serialized value or undefined/null when missing
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors the JS stdlib backend APIs this contract accepts directly: `Map<string,string>.get` returns `string | undefined` and Web Storage `getItem` returns `string | null`; raw `Map` is a documented backend, so the return must admit both `undefined` and `null`
  readonly get: (key: string,) => Promisable<string | undefined | null>;

  /**
   Persist serialized value by key.
   
   @param key - storage key
   @param value - serialized value to store
   */
  readonly set: (
    key: string,
    value: string,
  ) => Promisable<unknown>;

  /**
   Remove entry by key.
   
   @param key - key to remove
   */
  readonly delete: (key: string,) => Promisable<unknown>;

  /**
   Optional priority tier for consensus voting.
   Higher values indicate higher trust.
   Defaults to `0` when omitted.
   */
  readonly priority?: number;
};

/**
 Synchronous backend storage interface for the {@link SyncStore}.
 All operations return synchronously.
 Compatible with `Map<string, string>`.
 
 @example
 ```ts
 const backend: SyncStorageBackend = new Map<string, string>();
 ```
 */
export type SyncStorageBackend = {
  /**
   Retrieve serialized value by key.
   
   @param key - lookup key
   @returns serialized value or undefined when missing
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors `Map<string,string>.get` (`string | undefined`), the JS stdlib backend this contract accepts directly; raw `Map` is a documented sync backend, so the return must admit `undefined`
  readonly get: (key: string,) => string | undefined;

  /**
   Persist serialized value by key.
   
   @param key - storage key
   @param value - serialized value to store
   */
  readonly set: (
    key: string,
    value: string,
  ) => unknown;

  /**
   Remove entry by key.
   
   @param key - key to remove
   */
  readonly delete: (key: string,) => unknown;

  /**
   Optional priority tier for consensus voting.
   Higher values indicate higher trust.
   Defaults to `0` when omitted.
   */
  readonly priority?: number;
};

/**
 Configuration for constructing an async {@link Store}.
 
 @example
 ```ts
 const config: StoreConfig = {
   backends: [new Map<string, string>()],
   storeId: 'my-cache',
 };
 ```
 */
export type StoreConfig = BaseStoreConfig & {
  /**
   Ordered storage backends with optional priorities for consensus.
   Falls back to a single in-memory `Map` when omitted.
   */
  readonly backends?: readonly [
    StorageBackend,
    ...StorageBackend[],
  ];
};

/**
 Configuration for constructing a sync {@link SyncStore}.
 
 @example
 ```ts
 const config: SyncStoreConfig = {
   backends: [new Map<string, string>()],
   storeId: 'my-sync-cache',
 };
 ```
 */
export type SyncStoreConfig = BaseStoreConfig & {
  /**
   Ordered synchronous storage backends with optional priorities for consensus.
   Falls back to a single in-memory `Map` when omitted.
   */
  readonly backends?: readonly [
    SyncStorageBackend,
    ...SyncStorageBackend[],
  ];
};

/**
 Multi-backend key-value store with consensus and self-healing.
 
 Persists serialized values across one or more {@link StorageBackend} instances.
 When reading, computes a canonical value via majority vote:
 - Majority across all tiers wins
 - On tie, majority in highest priority tier wins
 - If still tied, throws an error
 
 After determining the canonical value, heals all backends to match.
 
 @example
 ```ts
 import { createStore } from '\@monochromatic-dev/kv-store-core';
 const store = await createStore({ storeId: 'demo' });
 await store.set('key', { data: 42 });
 const value = await store.get<{ data: number }>('key');
 ```
 */
export type Store = BaseStoreFields<StorageBackend> & {
  /**
   Persist a value by key.
   When key is empty, derives one from the content hash.
   
   @param key - storage key
   @param value - data to persist
   @returns this store for chaining
   */
  readonly set: (
    key: string,
    value: unknown,
  ) => Promise<Store>;

  /**
   Read value by key using consensus and heal backends to canonical result.
   
   @remarks A stored `null` value deserializes back to `null`; a missing key yields {@link ABSENT}, so the two are distinguishable.
   
   @typeParam T - expected deserialized value type
   @param key - lookup key
   @returns deserialized value, or {@link ABSENT} when no backend holds the key
   */
  readonly get: <const T = unknown,>(key: string,) => Promise<T | typeof ABSENT>;

  /**
   Remove entry by key across all backends.
   
   @param key - key to remove
   */
  readonly delete: (key: string,) => Promise<void>;

  /**
   Remove all entries across all backends that support clearing.
   */
  readonly clear: () => Promise<void>;
};

/**
 Synchronous multi-backend key-value store with consensus and self-healing.
 
 All operations are synchronous. Persists serialized values across one or more
 {@link SyncStorageBackend} instances.
 When reading, computes a canonical value via majority vote:
 - Majority across all tiers wins
 - On tie, majority in highest priority tier wins
 - If still tied, throws an error
 
 After determining the canonical value, heals all backends to match.
 
 @example
 ```ts
 import { createSyncStore } from '\@monochromatic-dev/kv-store-core';
 const store = createSyncStore({ storeId: 'demo' });
 store.set('key', { data: 42 });
 const value = store.get<{ data: number }>('key');
 ```
 */
export type SyncStore = BaseStoreFields<SyncStorageBackend> & {
  /**
   Persist a value by key.
   
   @param key - storage key
   @param value - data to persist
   @returns this store for chaining
   */
  readonly set: (
    key: string,
    value: unknown,
  ) => SyncStore;

  /* oxlint-disable typescript/no-unnecessary-type-parameters -- T is the caller-specified return type for typed reads; a single use is the intended call-site-inference shape, not redundancy */
  /**
   Read value by key using consensus and heal backends to canonical result.
   
   @remarks A stored `null` value deserializes back to `null`; a missing key yields {@link ABSENT}, so the two are distinguishable.
   
   @typeParam T - expected deserialized value type
   @param key - lookup key
   @returns deserialized value, or {@link ABSENT} when no backend holds the key
   */
  readonly get: <const T = unknown,>(key: string,) => T | typeof ABSENT;
  /* oxlint-enable typescript/no-unnecessary-type-parameters */

  /**
   Remove entry by key across all backends.
   
   @param key - key to remove
   */
  readonly delete: (key: string,) => void;

  /**
   Remove all entries across all backends that support clearing.
   */
  readonly clear: () => void;

  /**
   Current number of entries (available when all backends support `.size`).
   Returns the size of the first backend that exposes it, or `0`.
   */
  readonly size: number;
};
