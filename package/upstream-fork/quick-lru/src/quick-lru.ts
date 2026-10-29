/**
 TypeScript fork of [`quick-lru`](https://github.com/sindresorhus/quick-lru) by
 Sindre Sorhus (MIT): a dual-cache Least Recently Used map with lazy `maxAge`
 expiry, eviction notifications, `resize`, and `evict`.
 
 Derived from `quick-lru` 7.3.0; see `LICENSES/MIT.txt` and this package's
 README for the full attribution. Cache semantics match upstream exactly,
 including its dual-cache size variance and coercion quirks. The only
 API-shape deviations are lint-mandated: the cache is a factory-built object
 instead of a `Map` subclass instance, `set` takes a destructured options
 object instead of three positional parameters, and `forEach` takes one
 destructured options object.
 
 @example
 ```ts
 import { createQuickLru, } from '\@monochromatic-dev/upstream-fork-quick-lru';
 
 const lru = createQuickLru<string, string>({ maxSize: 1000, });
 lru.set({
   key: '🦄',
   value: '🌈',
 },);
 lru.get('🦄',); // => '🌈'
 ```
 
 @packageDocumentation
 */

import {
  hasExpiryStamp,
  type QuickLruItem,
  type QuickLruItemEntry,
} from './quick-lru-item.ts';
import {
  attachMembers,
  NODE_INSPECT_SYMBOL,
} from './quick-lru-members.ts';
import {
  createIteration,
} from './quick-lru-iteration.ts';
import {
  resolveQuickLruOptions,
  type QuickLruOptions,
  validateMaxSize,
} from './quick-lru-options.ts';
import type {
  QuickLru,
  QuickLruForEachOptions,
  QuickLruSetOptions,
} from './quick-lru-types.ts';

//region Cache

/**
 Creates one dual-cache LRU map over upstream `quick-lru`'s constructor
 shape.
 
 The cache keeps between `maxSize` and `2 × maxSize` items because the
 dual-cache design avoids per-write deletes; `size` still reports at most
 `maxSize`, matching upstream.
 
 @param options - `maxSize` plus optional global `maxAge` and `onEviction`.
 
 @returns Cache object with upstream `quick-lru`'s full member surface.
 
 @throws InvalidMaxSizeError when `maxSize` is not truthy and greater than
 `0`.
 
 @throws InvalidMaxAgeError when `maxAge` is exactly the number `0`.
 
 @throws TypeError when `options` is `null`, from the same property read
 upstream performs, matching upstream `quick-lru`.
 
 @example
 ```ts
 import { createQuickLru, } from '\@monochromatic-dev/upstream-fork-quick-lru';
 
 const lru = createQuickLru<string, number>({
   maxSize: 2,
   onEviction: function reportEvicted(key: string, value: number): void {
     console.log('evicted', key, value);
   },
 },);
 ```
 */
export function createQuickLru<Key, Value>(options: QuickLruOptions<Key, Value>,): QuickLru<Key, Value> {
  /**
   Validated bounds; resolving first keeps every later branch free of input
   shape checks.
   */
  const resolved = resolveQuickLruOptions(options,);
  /**
   Mutable cache state, read and written by the closures below. The two
   maps mirror upstream's `#cache` (recent) and `#oldCache` (older); the
   counter mirrors upstream's `#size`, which counts inserts into the
   recent map since the last rollover.
   */
  const state = {
    /**
     Recently used items, mirroring upstream `quick-lru`'s `#cache`.
     */
    cache: new Map<Key, QuickLruItem<Value>>(),
    /**
     Less recently used items, mirroring upstream `quick-lru`'s
     `#oldCache`.
     */
    oldCache: new Map<Key, QuickLruItem<Value>>(),
    /**
     Inserts into `cache` since the last rollover, mirroring upstream
     `quick-lru`'s `#size`.
     */
    cacheSize: 0,
    /**
     Target maximum number of items, replaceable through `resize`.
     */
    maxSize: resolved.maxSize,
    /**
     Global lifetime bound in milliseconds.
     */
    maxAge: resolved.maxAge,
    /**
     Eviction callback as passed in, `undefined` when the input carried
     none, mirroring upstream `quick-lru`'s `#onEviction`.
     */
    onEviction: options.onEviction,
  };
  /**
   The cache object handed to callers. Every member is attached below with
   upstream's prototype descriptor flags, so the surface check in the fuzz
   sidecar can pin it against upstream `quick-lru`.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- `self` gains every QuickLru member through the members table and attachMembers call below before it is returned; the empty shell exists so closures can return the cache itself for chaining
  const self = {} as QuickLru<Key, Value>;

  //region Cache internals

  /**
   Notifies the eviction callback for each item leaving the cache, exactly
   as upstream `#emitEvictions` does: no notification and no iteration at
   all when the input carried no callable callback.
   
   @param items - Leaving items as key and item pairs, map or snapshot.
   */
  function emitEvictions(items: Iterable<readonly [
    Key,
    QuickLruItem<Value>
  ]>,): void {
    /**
     Eviction callback, absent or non-callable exactly when upstream skips
     notification.
     */
    const notify = state.onEviction;
    if ((typeof notify) !== 'function')
      return;

    for (const [key, item] of items)
      notify(
        key,
        item.value,
      );
  }

  /**
   Removes one item from both maps, mirroring upstream's `delete`.
   
   @param key - Key to remove.
   
   @returns Whether an item was removed.
   */
  function deleteEntry(key: Key,): boolean {
    /**
     Whether the recent map held the key.
     */
    const deleted = state.cache
      .delete(key,);
    if (deleted)
      state.cacheSize -= 1;

    return state.oldCache
      .delete(key,)
      || deleted;
  }

  /**
   Lazily expires one item, notifying and removing it when its expiry has
   passed, mirroring upstream's `#deleteIfExpired`.
   
   @param key - Key the item is stored under.
   
   @param item - Stored item to check.
   
   @returns Whether the item expired and was removed now.
   */
  function deleteIfExpired({
    key,
    item,
  }: QuickLruItemEntry<Key, Value>,): boolean {
    if (((typeof item.expiry) === 'number') && (item.expiry <= Date.now())) {
      /**
       Eviction callback, notified exactly as upstream does before the
       lazy removal.
       */
      const notify = state.onEviction;
      if ((typeof notify) === 'function')
        notify(
          key,
          item.value,
        );

      return deleteEntry(key,);
    }

    return false;
  }

  /**
   Inserts one item into the recent map, rolling the maps over when the
   recent one fills, mirroring upstream's `#set`.
   
   @param key - Key to store the item under.
   
   @param item - Stored item to insert.
   */
  function insertRecentItem({
    key,
    item,
  }: QuickLruItemEntry<Key, Value>,): void {
    state.cache
      .set(
      key,
      item,
      );
    state.cacheSize += 1;

    if (state.cacheSize >= state.maxSize) {
      state.cacheSize = 0;
      emitEvictions(state.oldCache,);
      state.oldCache = state.cache;
      state.cache = new Map<Key, QuickLruItem<Value>>();
    }
  }

  /**
   Moves one old item back into the recent map, marking it most recently
   used, mirroring upstream's `#moveToRecent`.
   
   @param key - Key the item is stored under.
   
   @param item - Stored item to promote.
   */
  function promoteItem({
    key,
    item,
  }: QuickLruItemEntry<Key, Value>,): void {
    state.oldCache
      .delete(key,);
    insertRecentItem({
      key,
      item,
    },);
  }

  //endregion Cache internals

  //region Iteration context

  /**
   Reads the current recent map for the iteration orderings.
   
   @returns The recent map as currently installed.
   */
  function readCache(): Map<Key, QuickLruItem<Value>> {
    return state.cache;
  }

  /**
   Reads the current old map for the iteration orderings.
   
   @returns The old map as currently installed.
   */
  function readOldCache(): Map<Key, QuickLruItem<Value>> {
    return state.oldCache;
  }

  /**
   The six dual-cache orderings, bound to this cache's live maps and
   lazy-expiry hook.
   */
  const iteration = createIteration({
    readCache,
    readOldCache,
    deleteIfExpired,
  },);

  //endregion Iteration context

  //region Cache members

  /**
   Stores one value under one key with an optional per-item lifetime,
   refreshing its expiry, mirroring upstream's `set`.
   
   @param key - Key to store the value under.
   
   @param value - Value to store.
   
   @param maxAge - Milliseconds this item may live; falls back to the
   cache's global bound when absent.
   
   @returns The cache itself, so calls chain.
   
   @example
   ```ts
   lru.set({
     key: 'a',
     value: 1,
     maxAge: 500,
   },);
   ```
   */
  function set({
    key,
    value,
    maxAge = state.maxAge,
  }: QuickLruSetOptions<Key, Value>,): QuickLru<Key, Value> {
    /**
     Absolute expiry timestamp, `undefined` for items that never expire,
     computed exactly as upstream computes it.
     */
    const expiry = (((typeof maxAge) === 'number') && (maxAge !== Number.POSITIVE_INFINITY))
      ? (Date.now() + maxAge)
      : undefined;
    /**
     Stored item matching upstream's `{value, expiry}` shape: the stamp is
     present only when the item expires.
     */
    const item = (expiry === undefined)
      ? {
        value,
      }
      : {
        value,
        expiry,
      };

    if (state.cache
      .has(key,))
      state.cache
        .set(
        key,
        item,
        );
    else
      insertRecentItem({
        key,
        item,
      },);

    return self;
  }

  /**
   Reads one value and marks it most recently used when it sat in the old
   map, mirroring upstream's `get`.
   
   @param key - Key to read.
   
   @returns Stored value, or `undefined` when the key is absent or expired
   during this read.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream quick-lru 7.3.0's `get(key): ValueType | undefined` return; absence stays `undefined` so migrating callers and the fuzz differential oracle observe the upstream shape
  function get(key: Key,): Value | undefined {
    /**
     Item stored in the recent map, when present.
     */
    const recentItem = state.cache
      .get(key,);
    if (recentItem !== undefined) {
      if (!hasExpiryStamp(recentItem))
        return recentItem.value;

      return deleteIfExpired({
        key,
        item: recentItem,
      },)
        ? undefined
        : recentItem.value;
    }

    /**
     Item stored in the old map, when present.
     */
    const oldItem = state.oldCache
      .get(key,);
    if ((oldItem !== undefined) && (!deleteIfExpired({
      key,
      item: oldItem,
    },))) {
      promoteItem({
        key,
        item: oldItem,
      },);
      return oldItem.value;
    }

    return undefined;
  }

  /**
   Reads one value without marking it recently used, mirroring upstream's
   `peek`.
   
   @param key - Key to read.
   
   @returns Stored value, or `undefined` when the key is absent or expired
   during this read.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream quick-lru 7.3.0's `peek(key): ValueType | undefined` return; absence stays `undefined` so migrating callers and the fuzz differential oracle observe the upstream shape
  function peek(key: Key,): Value | undefined {
    /**
     Item stored in the recent map, when present.
     */
    const recentItem = state.cache
      .get(key,);
    if (recentItem !== undefined) {
      if (!hasExpiryStamp(recentItem))
        return recentItem.value;

      return deleteIfExpired({
        key,
        item: recentItem,
      },)
        ? undefined
        : recentItem.value;
    }

    /**
     Item stored in the old map, when present.
     */
    const oldItem = state.oldCache
      .get(key,);
    if (oldItem !== undefined) {
      if (!hasExpiryStamp(oldItem))
        return oldItem.value;

      return deleteIfExpired({
        key,
        item: oldItem,
      },)
        ? undefined
        : oldItem.value;
    }

    return undefined;
  }

  /**
   Checks one key without marking it recently used, mirroring upstream's
   `has`.
   
   @param key - Key to check.
   
   @returns Whether a live item exists under the key.
   */
  function has(key: Key,): boolean {
    /**
     Item stored in the recent map, when present.
     */
    const recentItem = state.cache
      .get(key,);
    if (recentItem !== undefined)
      return !deleteIfExpired({
        key,
        item: recentItem,
      },);

    /**
     Item stored in the old map, when present.
     */
    const oldItem = state.oldCache
      .get(key,);
    if (oldItem !== undefined)
      return !deleteIfExpired({
        key,
        item: oldItem,
      },);

    return false;
  }

  /**
   Reads one item's remaining lifetime without lazily expiring it,
   mirroring upstream's `expiresIn`.
   
   @param key - Key to inspect.
   
   @returns Remaining milliseconds, `Number.POSITIVE_INFINITY` for items
   without expiry, or `undefined` when the key is absent.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream quick-lru 7.3.0's `expiresIn(key): number | undefined` return; absence stays `undefined` so migrating callers and the fuzz differential oracle observe the upstream shape
  function expiresIn(key: Key,): number | undefined {
    /**
     Item stored under the key in either map, recent map taking precedence
     exactly as upstream reads it.
     */
    const item = state.cache
      .get(key,)
      ?? state.oldCache
        .get(key,);
    if (item !== undefined)
      return hasExpiryStamp(item)
        ? (item.expiry - Date.now())
        : Number.POSITIVE_INFINITY;

    return undefined;
  }

  /**
   Removes every item without eviction notifications, mirroring upstream's
   `clear`.
   */
  function clear(): void {
    state.cache
      .clear();
    state.oldCache
      .clear();
    state.cacheSize = 0;
  }

  /**
   Updates the target maximum in place, mirroring upstream's `resize`:
   growing keeps every item, shrinking evicts the oldest ones first with
   notifications.
   
   @param maxSize - New target maximum number of items.
   
   @throws InvalidMaxSizeError when the value is not truthy and greater
   than `0`.
   */
  function resize(maxSize: number,): void {
    /**
     Validated new bound, stored exactly as upstream stores it.
     */
    const validatedMaxSize = validateMaxSize(maxSize,);
    /**
     Live items oldest first, collected through the same lazy-expiry walk
     upstream uses.
     */
    const items = [...iteration.iterateItemEntriesAscending()];
    /**
     How many oldest items the new bound no longer fits.
     */
    const removeCount = items.length - validatedMaxSize;

    if (removeCount < 0) {
      state.cache = new Map(items,);
      state.oldCache = new Map<Key, QuickLruItem<Value>>();
      state.cacheSize = items.length;
    }
    else {
      // mutation-test-disable-next-line conditional, equality -- proven equivalent: in this branch removeCount is already non-negative, so the guard differs only at 0 where `items.slice(0, 0)` is empty and the emit cannot notify anything
      if (removeCount > 0)
        emitEvictions(items.slice(
          0,
          removeCount,
        ),);

      state.oldCache = new Map(items.slice(removeCount,),);
      state.cache = new Map<Key, QuickLruItem<Value>>();
      state.cacheSize = 0;
    }

    state.maxSize = validatedMaxSize;
  }

  /**
   Evicts the least recently used items with notifications, keeping at
   least one, mirroring upstream's `evict`.
   
   @param count - Number of items to evict; coerced exactly as upstream
   coerces it, so out-of-type runtime values behave identically.
   
   @defaultValue 1
   */
  function evict(count: unknown = 1,): void {
    /**
     Coerced eviction request, invalid exactly when upstream gives up
     before touching the cache.
     */
    const requested = Number(count,);
    if ((Number.isNaN(requested,)) || (requested <= 0))
      return;

    /**
     Live items oldest first, collected through the same lazy-expiry walk
     upstream uses.
     */
    const items = [...iteration.iterateItemEntriesAscending()];
    /**
     How many oldest items leave, never the final item.
     */
    const evictCount = Math.trunc(Math.min(
      requested,
      Math.max(
        items.length - 1,
        0,
      ),
    ),);
    if (evictCount <= 0)
      return;

    emitEvictions(items.slice(
      0,
      evictCount,
    ),);
    state.oldCache = new Map(items.slice(evictCount,),);
    state.cache = new Map<Key, QuickLruItem<Value>>();
    state.cacheSize = 0;
  }

  /**
   Runs one visitor per live entry oldest first, mirroring upstream's
   `forEach`.
   
   @param callback - Visitor run per live entry, oldest first.
   
   @param thisArgument - `this` value the visitor is called with; defaults
   to the cache itself, matching upstream.
   */
  function forEach({
    callback,
    thisArgument = self,
  }: QuickLruForEachOptions<Key, Value>,): void {
    for (const [key, value] of iteration.iterateEntriesAscending())
      callback.call(
        thisArgument,
        value,
        key,
        self,
      );
  }

  /**
   Reports the stored item count capped at the target maximum, mirroring
   upstream's `size` getter and its duplicate-key accounting.
   
   @returns Stored item count as upstream reports it.
   */
  function readSize(): number {
    if (state.cacheSize === 0)
      return state.oldCache
        .size;

    /**
     Old-map keys absent from the recent map, counted in a container
     because repository lint bans function-root `let` bindings.
     */
    const tally = {
      uniques: 0,
    };
    for (const key of state.oldCache
      .keys()) {
      if (!state.cache
        .has(key,))
        tally.uniques += 1;
    }

    return Math.min(
      state.cacheSize + tally.uniques,
      state.maxSize,
    );
  }

  /**
   Renders the cache like upstream's `toString`.
   
   @returns `QuickLRU(<size>/<maxSize>)`.
   */
  function toString(): string {
    return `QuickLRU(${readSize()}/${state.maxSize})`;
  }

  /**
   Handles `Node.js` custom inspection like upstream: the same text as
   `toString`.
   
   @returns `QuickLRU(<size>/<maxSize>)`.
   */
  function inspectCustom(): string {
    return toString();
  }

  //endregion Cache members

  //region Attachment

  /**
   Every member exactly as upstream's class body carries it: methods are
   writable-or-accessor entries and `attachMembers` pins `enumerable:
   false` to match upstream's class-prototype flags.
   */
  const members = {
    set,
    get,
    has,
    peek,
    delete: deleteEntry,
    clear,
    expiresIn,
    resize,
    evict,
    keys: iteration.iterateKeys,
    values: iteration.iterateValues,
    entries: iteration.iterateEntriesAscending,
    entriesAscending: iteration.iterateEntriesAscending,
    entriesDescending: iteration.iterateEntriesDescending,
    forEach,
    toString,
    [Symbol.iterator]: iteration.iterateEntries,
    [NODE_INSPECT_SYMBOL]: inspectCustom,
    /**
     Stored item count, capped at `maxSize` like upstream's `size` getter.
     */
    get size(): number { return readSize(); },
    /**
     The set maximum number of items.
     */
    get maxSize(): number { return state.maxSize; },
    /**
     The set global lifetime in milliseconds.
     */
    get maxAge(): number { return state.maxAge; },
    /**
     The old cache's raw items, upstream `quick-lru`'s `__oldCache` test
     hook.
     */
    get __oldCache(): Map<Key, QuickLruItem<Value>> { return state.oldCache; },
    /**
     `Object.prototype.toString` tag, kept at upstream's `QuickLRU`.
     */
    get [Symbol.toStringTag](): string { return 'QuickLRU'; },
  };

  attachMembers({
    self,
    members,
  },);

  //endregion Attachment

  return self;
}

//endregion Cache
