/**
 Backend querying and default-backend configuration for async stores.
 Internal helpers; not part of the package public API except
 {@link configureDefaultBackendsBuilder} and {@link DefaultBackendsBuilder},
 which are re-exported from the package root.
 */

import { ABSENT, } from './constants.ts';
import type { BackendResult, } from './consensus.ts';
import type { LruKeySet, } from './lru-key-set.ts';
import type { StorageBackend, } from './types.ts';

/**
 Query all backends for a key and return typed results with priority info.
 
 @param backends - storage backends to query
 
 @param key - lookup key
 
 @returns results from all backends
 
 @example
 ```ts
 const results = await queryAllBackends({ backends, key: 'my-key' });
 ```
 */
export async function queryAllBackends({
  backends,
  key,
}: Readonly<{
  backends: readonly [
    StorageBackend,
    ...StorageBackend[],
  ];
  key: string;
}>,): Promise<[
  BackendResult<StorageBackend>,
  ...BackendResult<StorageBackend>[],
]> {
  /**
   Per-backend query results gathered concurrently.
   */
  const results = await Promise.all(
    backends.map(async function queryBackend(backend,) {
      /**
       Raw value returned by this backend before nullish normalisation.
       */
      const raw = await backend.get(key,);
      return {
        value: raw ?? ABSENT,
        priority: backend.priority
          ?? 0,
        backend,
      };
    },),
  );
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- non-empty backends guarantees non-empty results
  return results as [
    BackendResult<StorageBackend>,
    ...BackendResult<StorageBackend>[],
  ];
}

/**
 Builder function that produces platform-specific default backends.
 
 @example
 ```ts
 const builder: DefaultBackendsBuilder = async ({ storeId }) => {
   const fileBackend = await createFileBackend(storeId);
   return [new Map<string, string>(), fileBackend];
 };
 ```
 */
export type DefaultBackendsBuilder = (args: Readonly<{
  storeId: string;
}>,) => Promise<readonly [
  StorageBackend,
  ...StorageBackend[],
]>;

/**
 Module-level builder registry keyed by a sentinel; one entry max, configured once at module load by platform entry.
 */
const builderRegistry = new Map<'default', DefaultBackendsBuilder>();

/**
 Configure a platform-specific default backends builder.
 Call from a platform entry file at module load time.
 
 @param builder - function that creates backends for the current platform
 
 @example
 ```ts
 configureDefaultBackendsBuilder(async ({ storeId }) => {
   return [new Map<string, string>(), await createFileBackend(storeId)];
 });
 ```
 */
export function configureDefaultBackendsBuilder(builder: DefaultBackendsBuilder,): void {
  builderRegistry.set(
    'default',
    builder,
  );
}

/**
 Build the default backends for a store: the registered platform builder's
 output when one was configured, otherwise a single in-memory `Map`.
 Encapsulates the registry lookup so callers never handle an absent builder.
 
 @param storeId - store identifier forwarded to the platform builder
 
 @returns non-empty list of storage backends
 
 @example
 ```ts
 const backends = config.backends ?? await buildDefaultBackends({ storeId });
 ```
 */
export async function buildDefaultBackends({
  storeId,
}: Readonly<{
  storeId: string;
}>,): Promise<readonly [
  StorageBackend,
  ...StorageBackend[],
]> {
  /**
   Platform-specific factory registered via {@link configureDefaultBackendsBuilder}, present only when an entry file set it.
   */
  const builder = builderRegistry.get('default',);
  if (builder !== undefined) {
    /**
     Backends produced by the registered platform builder.
     */
    const built = await builder({ storeId, },);
    return built;
  }
  return [new Map<string, string>(),];
}

/**
 Touch the LRU set for a key and evict the displaced entry from all backends.
 
 Passing a no-op LRU set makes this a no-op (store has no eviction policy).
 
 @param lru - LRU key set tracking access order
 
 @param key - key that was just accessed
 
 @param backends - storage backends to evict from
 
 @param logger - logger for debug output
 
 @example
 ```ts
 await evictLruEntry({
   lru: lruSet,
   key: 'recently-accessed-key',
   backends: [primaryBackend, fallbackBackend],
   logger: { debug(msg) { console.debug(msg); } },
 });
 ```
 */
export async function evictLruEntry(
  {
    lru,
    key,
    backends,
    logger,
  }: Readonly<{
    lru: LruKeySet;
    key: string;
    backends: readonly [
      StorageBackend,
      ...StorageBackend[],
    ];
    logger: { readonly debug: (msg: string,) => void; };
  }>,
): Promise<void> {
  /**
   Key displaced by the LRU touch, or `ABSENT` when nothing was evicted.
   */
  const evicted = lru.touch(key,);
  if (evicted === ABSENT)
    return;
  logger.debug(`store.evict: "${evicted}"`,);
  await Promise.all(
    backends.map(async function evictFromBackend(backend,) {
      await backend.delete(evicted,);
    },),
  );
}
