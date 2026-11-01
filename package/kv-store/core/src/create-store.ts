import {
  parse as superjsonParse,
  stringify as superjsonStringify,
} from 'superjson';

import { logger as defaultLogger, } from '@monochromatic-dev/logger-core/ts';

import {
  buildDefaultBackends,
  evictLruEntry,
  queryAllBackends,
} from './backends-async.ts';
import { ABSENT, } from './constants.ts';
import { resolveConsensus, } from './consensus.ts';
import { hashString, } from './hash.ts';
import { healBackends, } from './heal.ts';
import {
  createLruKeySet,
  noopLruKeySet,
} from './lru-key-set.ts';
import { serializeValue, } from './serialize.ts';
import type {
  Deserializer,
  Serializer,
  StorageBackend,
  Store,
  StoreConfig,
} from './types.ts';

/**
 Creates a {@link Store} instance, using platform-specific default backends
 when none are provided in the config.
 
 Falls back to a single in-memory `Map` when no backends are configured
 and no platform builder has been registered.
 
 When {@link StoreConfig.eviction} is set, the store tracks access order
 and evicts entries according to the configured policy.
 
 @param config - store configuration
 
 @returns initialized Store
 
 @example
 ```ts
 const store = await createStore({ storeId: 'my-cache' });
 await store.set('key', [1, 2, 3]);
 const arr = await store.get<number[]>('key');
 ```
 
 @example
 LRU-bounded store:
 ```ts
 const store = await createStore({
   storeId: 'bounded',
   eviction: [{ policy: 'lru', maxSize: 256 }],
 });
 ```
 
 @example
 Custom backends:
 ```ts
 const memoryBackend = new Map<string, string>();
 const store = await createStore({
   backends: [memoryBackend],
   storeId: 'custom',
 });
 ```
 */
export async function createStore(config: StoreConfig = {},): Promise<Store> {
  /**
   Caller-supplied identifier or freshly minted UUID; used in debug logs and passed to the platform backends builder.
   */
  const storeId = config.storeId
    ?? crypto
      .randomUUID();
  /**
   Serializer applied on every `set`; defaults to superjson so structured values round-trip through string backends.
   */
  const serializer: Serializer = config.serializer
    ?? superjsonStringify;
  /**
   Deserializer applied on every `get`; paired with {@link serializer}'s default so superjson output decodes correctly.
   */
  const deserializer: Deserializer = config.deserializer
    ?? superjsonParse;
  /**
   When `true`, circular structures are serialized lossily instead of throwing; opt-in safety net for graph-shaped values.
   */
  const lossyForCircular = config.lossyForCircular
    ?? true;

  /**
   Non-empty list of storage backends; user-supplied, else the platform builder's output, else a single in-memory Map.
   */
  const backends: readonly [
    StorageBackend,
    ...StorageBackend[],
  ] = config.backends
    ?? await buildDefaultBackends({ storeId, },);

  /**
   Configured eviction policies; empty array means unbounded growth.
   */
  const policies = config.eviction
    ?? [];
  /**
   First LRU policy in the list, or undefined when LRU is not configured.
   */
  const lruPolicy = policies.find(function isLru(p,) {
    return p.policy
      === 'lru';
  },);
  /**
   LRU access tracker bounded by {@link lruPolicy}'s `maxSize`, or a no-op tracker when LRU is not configured.
   */
  const lru = lruPolicy !== undefined
    ? createLruKeySet(lruPolicy.maxSize,)
    : noopLruKeySet;

  defaultLogger.debug(
    `store "${storeId}" created with ${String(backends.length,)} backend(s)`,
  );

  /**
   Exposed Store instance; declared as a binding so member methods can self-reference for chaining.
   */
  const store: Store = {
    storeId,
    serializer,
    deserializer,
    lossyForCircular,
    backends,

    async set(
      key: string,
      value: unknown,
    ): Promise<Store> {
      defaultLogger.debug(`store.set: "${key}"`,);
      /**
       Serialized form written to every backend; computed once so all backends agree on byte-identical content.
       */
      const serialized = serializeValue({
        value,
        serializer,
        lossyForCircular,
      },);
      /**
       Effective storage key: the caller's key when non-empty, else a hash of the serialized value so empty keys still address something stable.
       */
      const resolvedKey = key.length
        === 0 ? await hashString(serialized,) : key;

      await Promise.all(
        backends.map(async function persistToBackend(backend,) {
          await backend.set(
            resolvedKey,
            serialized,
          );
        },),
      );

      await evictLruEntry({
        lru,
        key: resolvedKey,
        backends,
        logger: defaultLogger,
      },);

      return store;
    },

    async get<const T = unknown,>(key: string,): Promise<T | typeof ABSENT> {
      defaultLogger.debug(`store.get: "${key}"`,);
      /**
       Per-backend lookup results; feeds both consensus resolution and the healing pass.
       */
      const results = await queryAllBackends({
        backends,
        key,
      },);
      /**
       Consensus value across backends, or `ABSENT` when no backend held the key.
       */
      const canonicalSerialized = resolveConsensus({
        results,
        key,
      },);

      await healBackends({
        results,
        canonicalSerialized,
        key,
      },);

      if (canonicalSerialized !== ABSENT) {
        await evictLruEntry({
          lru,
          key,
          backends,
          logger: defaultLogger,
        },);
      }

      return canonicalSerialized === ABSENT
        ? ABSENT
        : deserializer<T>(canonicalSerialized,);
    },

    async delete(key: string,): Promise<void> {
      defaultLogger.debug(`store.delete: "${key}"`,);
      lru.remove(key,);
      await Promise.all(
        backends.map(async function deleteFromBackend(backend,) {
          await backend.delete(key,);
        },),
      );
    },

    async clear(): Promise<void> {
      defaultLogger.debug(`store.clear`,);
      lru.clear();
      await Promise.all(
        backends.map(async function clearBackend(backend,) {
          // Map and similar backends support clear()
          if (('clear' in backend) && ((typeof backend.clear) === 'function')) {
            // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- runtime check confirms clear is a function
            await (backend.clear as () => unknown)();
          }
        },),
      );
    },
  };

  return store;
}
