/**
 Cache adapters exposing one uniform workload surface over this package's
 fork and upstream `quick-lru`, so the same generated workload can drive
 both.
 
 Read results are projected to strings here (`absent`, `infinite`,
 `remaining:<milliseconds>`) because the caches' value-or-absence returns
 are exactly what the differential compares; doing the projection inside
 the adapters keeps the trace types free of absence encodings.
 
 @module
 */

import QuickLRUUpstream from 'quick-lru';

import { createQuickLru, } from '@monochromatic-dev/upstream-fork-quick-lru/ts';

import {
  type CacheAdapter,
  type CacheSnapshot,
  renderItem,
  renderPair,
} from './cache-workload.ts';

//region Types

/**
 Structural cache surface the snapshot projection reads. Both
 implementations expose these members with identical shapes; only `set`,
 `delete`, and `forEach` call shapes differ, and the snapshot never calls
 those.
 */
type SnapshotSource = {
  /**
   Reported item count.
   */
  readonly size: number;
  /**
   Reported target maximum.
   */
  readonly maxSize: number;
  /**
   Reported global lifetime.
   */
  readonly maxAge: number;
  /**
   The old cache's raw items, upstream's `__oldCache` test hook.
   */
  readonly __oldCache: Map<string, {
    /**
     Stored value.
     */
    readonly value: string;
    /**
     Expiry stamp when the item expires.
     */
    readonly expiry?: number;
  }>;
  /**
   Renders `QuickLRU(<size>/<maxSize>)`.
   */
  readonly toString: () => string;
  /**
   Oldest-first live entries.
   */
  readonly entriesAscending: () => IterableIterator<[
    string,
    string
  ]>;
  /**
   Oldest-first live entries, the `Map` compatibility alias.
   */
  readonly entries: () => IterableIterator<[
    string,
    string
  ]>;
  /**
   Newest-first live entries.
   */
  readonly entriesDescending: () => IterableIterator<[
    string,
    string
  ]>;
  /**
   Default-iterator live entries, recent map before old map.
   */
  readonly [Symbol.iterator]: () => IterableIterator<[
    string,
    string
  ]>;
  /**
   Live keys in default-iterator order.
   */
  readonly keys: () => IterableIterator<string>;
  /**
   Live values in default-iterator order.
   */
  readonly values: () => IterableIterator<string>;
};

//endregion Types

//region Projection

/**
 Materializes one entry iterable into `key=value` projections.
 
 @param entries - Entry iterable to walk.
 
 @returns Projections in iteration order.
 
 @example
 ```ts
 projectEntries(cache.entriesAscending());
 ```
 */
function projectEntries(entries: Iterable<readonly [
  string,
  string
]>,): readonly string[] {
  return [...entries].map(function projectEntry(entry: readonly [
    string,
    string
  ],): string {
    /**
     Key and value of the entry under projection.
     */
    const [key, value,] = entry;
    return renderPair({
      key,
      value,
    },);
  },);
}

/**
 Projects one cache's full observable state.
 
 @param cache - Cache to project.
 
 @returns Snapshot covering every ordering and counter.
 
 @example
 ```ts
 snapshotOf(forkCache);
 ```
 */
function snapshotOf(cache: SnapshotSource): CacheSnapshot {
  return {
    size: String(cache.size,),
    maxSize: String(cache.maxSize,),
    maxAge: String(cache.maxAge,),
    text: cache.toString(),
    ascending: projectEntries(cache.entriesAscending(),),
    entries: projectEntries(cache.entries(),),
    descending: projectEntries(cache.entriesDescending(),),
    iterator: projectEntries(cache,),
    keys: [...cache.keys()],
    values: [...cache.values()],
    oldCache: [...cache.__oldCache].map(function projectItem(entry: readonly [
      string,
      {
        readonly value: string;
        readonly expiry?: number;
      }
    ],): string {
      /**
       Key and raw item of the old-map entry under projection.
       */
      const [key, item,] = entry;
      return renderItem({
        key,
        value: item.value,
        expiry: String(item.expiry,),
      },);
    },),
  };
}

//endregion Projection

//region Adapters

/**
 Adapts this package's fork to the workload surface.
 
 @param options - Constructor bounds and eviction-callback toggle.
 
 @returns Adapter driving the fork's cache and collecting its eviction
 events.
 
 @example
 ```ts
 const adapter = createForkAdapter({
   maxSize: 3,
   maxAge: 500,
   onEviction: true,
 },);
 ```
 */
export function createForkAdapter(options: {
  /**
   Target maximum number of items.
   */
  readonly maxSize: number;
  /**
   Global lifetime in milliseconds.
   */
  readonly maxAge: number;
  /**
   Whether the cache carries an `onEviction` callback.
   */
  readonly onEviction: boolean;
},): CacheAdapter {
  /**
   Eviction events fired by the fork cache, in order.
   */
  const evictions: string[] = [];
  /**
   Fork cache under test.
   */
  const cache = createQuickLru<string, string>({
    maxSize: options.maxSize,
    maxAge: options.maxAge,
    ...(options.onEviction
      ? {
        onEviction: function recordEviction(
          key: string,
          value: string,
        ): void {
          evictions.push(renderPair({
            key,
            value,
          },),);
        },
      }
      : {}),
  },);

  return {
    set: function setFork(item: {
      readonly key: string;
      readonly value: string;
      readonly maxAge?: number;
    },): void {
      cache.set(item,);
    },
    get: function getFork(key: string,): string {
      /**
       Value read from the cache, `undefined` when absent or expired.
       */
      const value = cache.get(key,);
      return (value === undefined) ? 'absent' : `value:${value}`;
    },
    peek: function peekFork(key: string,): string {
      /**
       Value read from the cache, `undefined` when absent or expired.
       */
      const value = cache.peek(key,);
      return (value === undefined) ? 'absent' : `value:${value}`;
    },
    has: function hasFork(key: string,): boolean {
      return cache.has(key,);
    },
    remove: function removeFork(key: string,): boolean {
      return cache.delete(key,);
    },
    expiresIn: function expiresInFork(key: string,): string {
      /**
       Remaining lifetime read from the cache, `undefined` when absent.
       */
      const remaining = cache.expiresIn(key,);
      if (remaining === undefined)
        return 'absent';
      if (remaining === Number.POSITIVE_INFINITY)
        return 'infinite';
      return `remaining:${String(remaining,)}`;
    },
    clear: function clearFork(): void {
      cache.clear();
    },
    resize: function resizeFork(maxSize: number,): void {
      cache.resize(maxSize,);
    },
    evict: function evictFork(count: number,): void {
      cache.evict(count,);
    },
    snapshot: function snapshotFork(): CacheSnapshot {
      return snapshotOf(cache);
    },
    evictions,
  };
}

/**
 Adapts upstream `quick-lru` to the workload surface.
 
 @param options - Constructor bounds and eviction-callback toggle.
 
 @returns Adapter driving an upstream cache and collecting its eviction
 events.
 
 @example
 ```ts
 const adapter = createUpstreamAdapter({
   maxSize: 3,
   maxAge: 500,
   onEviction: true,
 },);
 ```
 */
export function createUpstreamAdapter(options: {
  /**
   Target maximum number of items.
   */
  readonly maxSize: number;
  /**
   Global lifetime in milliseconds.
   */
  readonly maxAge: number;
  /**
   Whether the cache carries an `onEviction` callback.
   */
  readonly onEviction: boolean;
},): CacheAdapter {
  /**
   Eviction events fired by the upstream cache, in order.
   */
  const evictions: string[] = [];
  /**
   Upstream cache under comparison.
   */
  const cache = new QuickLRUUpstream<string, string>({
    maxSize: options.maxSize,
    maxAge: options.maxAge,
    ...(options.onEviction
      ? {
        onEviction: function recordEviction(
          key: string,
          value: string,
        ): void {
          evictions.push(renderPair({
            key,
            value,
          },),);
        },
      }
      : {}),
  },);

  return {
    set: function setUpstream(item: {
      readonly key: string;
      readonly value: string;
      readonly maxAge?: number;
    },): void {
      if (item.maxAge === undefined)
        cache.set(
          item.key,
          item.value,
        );
      else
        cache.set(
          item.key,
          item.value,
          {
            maxAge: item.maxAge,
          },
        );
    },
    get: function getUpstream(key: string,): string {
      /**
       Value read from the cache, `undefined` when absent or expired.
       */
      const value = cache.get(key,);
      return (value === undefined) ? 'absent' : `value:${value}`;
    },
    peek: function peekUpstream(key: string,): string {
      /**
       Value read from the cache, `undefined` when absent or expired.
       */
      const value = cache.peek(key,);
      return (value === undefined) ? 'absent' : `value:${value}`;
    },
    has: function hasUpstream(key: string,): boolean {
      return cache.has(key,);
    },
    remove: function removeUpstream(key: string,): boolean {
      return cache.delete(key,);
    },
    expiresIn: function expiresInUpstream(key: string,): string {
      /**
       Remaining lifetime read from the cache, `undefined` when absent.
       */
      const remaining = cache.expiresIn(key,);
      if (remaining === undefined)
        return 'absent';
      if (remaining === Number.POSITIVE_INFINITY)
        return 'infinite';
      return `remaining:${String(remaining,)}`;
    },
    clear: function clearUpstream(): void {
      cache.clear();
    },
    resize: function resizeUpstream(maxSize: number,): void {
      cache.resize(maxSize,);
    },
    evict: function evictUpstream(count: number,): void {
      cache.evict(count,);
    },
    snapshot: function snapshotUpstream(): CacheSnapshot {
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- upstream quick-lru exposes `__oldCache` at runtime as its documented test hook but keeps it out of index.d.ts; the snapshot reads the same hook on both caches for the differential
      return snapshotOf(cache as unknown as SnapshotSource);
    },
    evictions,
  };
}

//endregion Adapters
