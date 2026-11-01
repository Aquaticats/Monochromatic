/**
 API surface types of the LRU cache factory.
 
 Every member of {@link QuickLru} mirrors one upstream `quick-lru` member,
 and the fuzz sidecar's differential oracle checks that parity against
 upstream `quick-lru` 7.3.0 directly.
 
 @module
 */

import type {
  QuickLruItem,
} from './quick-lru-item.ts';

//region Types

/**
 Destructured call shape of {@link QuickLru}'s `set`.
 
 Upstream's `set(key, value, options)` positional form is banned by
 repository lint, so the key, value, and per-item lifetime arrive as one
 options object.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 
 @example
 ```ts
 const entry: QuickLruSetOptions<string, number> = {
   key: 'a',
   value: 1,
   maxAge: 60_000,
 };
 ```
 */
export type QuickLruSetOptions<Key, Value> = {
  /**
   Key to store the value under.
   */
  readonly key: Key;
  /**
   Value to store.
   */
  readonly value: Value;
  /**
   Milliseconds this item may live, overriding the cache's global `maxAge`.
   Absent means the global bound applies.
   */
  readonly maxAge?: number;
};

/**
 Visitor called per entry by {@link QuickLru}'s `forEach`.
 
 The positional `(value, key, cache)` signature is dictated by `Map`
 compatibility, which upstream `quick-lru` preserves.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 */
export type QuickLruVisitor<Key, Value> = (
  value: Value,
  key: Key,
  cache: QuickLru<Key, Value>,
) => void;

/**
 Destructured call shape of {@link QuickLru}'s `forEach`.
 
 Upstream's `forEach(callbackFunction, thisArgument)` positional form is
 banned by repository lint, so both arrive in one options object.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 
 @example
 ```ts
 lru.forEach({
   callback: function logEntry(value: string, key: string): void {
     console.log(key, value);
   },
 },);
 ```
 */
export type QuickLruForEachOptions<Key, Value> = {
  /**
   Visitor run per live entry, oldest entry first.
   */
  readonly callback: QuickLruVisitor<Key, Value>;
  /**
   `this` value the visitor is called with. Defaults to the cache itself,
   matching upstream `quick-lru`.
   */
  readonly thisArgument?: unknown;
};

/**
 A Least Recently Used cache over upstream `quick-lru`'s dual-cache
 algorithm.
 
 Upstream `quick-lru` returns a `Map` subclass instance; repository lint
 bans classes, so this fork returns a factory-built object with the same
 members instead.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 
 @example
 ```ts
 import { createQuickLru, } from '\@monochromatic-dev/upstream-fork-quick-lru';
 
 const lru: QuickLru<string, number> = createQuickLru({ maxSize: 2, },);
 lru.set({ key: 'a', value: 1, },);
 lru.get('a',); // => 1
 ```
 */
export type QuickLru<Key, Value> = {
  /**
   Stores one value under one key, refreshing its lifetime. Returns the
   cache, so calls chain.
   
   @param options - Key, value, and optional per-item lifetime.
   
   @returns The cache itself.
   */
  readonly set: (options: QuickLruSetOptions<Key, Value>,) => QuickLru<Key, Value>;
  /**
   Reads one value and marks it most recently used.
   
   @param key - Key to read.
   
   @returns Stored value, or `undefined` when the key is absent or its item
   expired during this read.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream quick-lru 7.3.0's `get(key): ValueType | undefined` return; absence stays `undefined` so migrating callers and the fuzz differential oracle observe the upstream shape
  readonly get: (key: Key,) => Value | undefined;
  /**
   Checks one key without marking it recently used; expired items are
   removed during the check.
   
   @param key - Key to check.
   
   @returns Whether a live item exists under the key.
   */
  readonly has: (key: Key,) => boolean;
  /**
   Reads one value without marking it recently used.
   
   @param key - Key to read.
   
   @returns Stored value, or `undefined` when the key is absent or its item
   expired during this read.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream quick-lru 7.3.0's `peek(key): ValueType | undefined` return; absence stays `undefined` so migrating callers and the fuzz differential oracle observe the upstream shape
  readonly peek: (key: Key,) => Value | undefined;
  /**
   Removes one item.
   
   @param key - Key to remove.
   
   @returns Whether an item was removed.
   */
  readonly delete: (key: Key,) => boolean;
  /**
   Removes every item without eviction notifications.
   */
  readonly clear: () => void;
  /**
   Reads one item's remaining lifetime without marking it recently used
   and without removing it when it is already expired.
   
   @param key - Key to inspect.
   
   @returns Remaining milliseconds, `Number.POSITIVE_INFINITY` for items
   without expiry, or `undefined` when the key is absent.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream quick-lru 7.3.0's `expiresIn(key): number | undefined` return; absence stays `undefined` so migrating callers and the fuzz differential oracle observe the upstream shape
  readonly expiresIn: (key: Key,) => number | undefined;
  /**
   Updates `maxSize` in place, discarding items as necessary.
   
   @param maxSize - New target maximum number of items.
   
   @throws InvalidMaxSizeError when the value is not truthy and greater
   than `0`.
   */
  readonly resize: (maxSize: number,) => void;
  /**
   Evicts the least recently used items, keeping at least one.
   
   @param count - Number of items to evict.
   
   @defaultValue 1
   */
  readonly evict: (count?: number,) => void;
  /**
   Iterates every live key, most recently used first.
   */
  readonly keys: () => IterableIterator<Key>;
  /**
   Iterates every live value, most recently used first.
   */
  readonly values: () => IterableIterator<Value>;
  /**
   Iterates every live entry, oldest first. Exists for `Map`
   compatibility; prefer `entriesAscending`.
   */
  readonly entries: () => IterableIterator<[
    Key,
    Value
  ]>;
  /**
   Iterates every live entry, oldest first.
   */
  readonly entriesAscending: () => IterableIterator<[
    Key,
    Value
  ]>;
  /**
   Iterates every live entry, newest first.
   */
  readonly entriesDescending: () => IterableIterator<[
    Key,
    Value
  ]>;
  /**
   Runs one visitor per live entry, oldest first. Exists for `Map`
   compatibility; prefer `entriesAscending`.
   
   @param options - Visitor and optional `this` value.
   */
  readonly forEach: (options: QuickLruForEachOptions<Key, Value>,) => void;
  /**
   Stored item count, capped at `maxSize` even though the dual-cache design
   may hold more.
   */
  readonly size: number;
  /**
   The set maximum number of items.
   */
  readonly maxSize: number;
  /**
   The set global lifetime in milliseconds.
   */
  readonly maxAge: number;
  /**
   The old cache's raw items, upstream `quick-lru`'s `__oldCache` test hook,
   mirrored so tests and the differential oracle can observe promotion.
   */
  readonly __oldCache: Map<Key, QuickLruItem<Value>>;
  /**
   Renders the cache like upstream: `QuickLRU(<size>/<maxSize>)`.
   */
  readonly toString: () => string;
  /**
   Iterates every live entry, most recently used first.
   */
  readonly [Symbol.iterator]: () => IterableIterator<[
    Key,
    Value
  ]>;
  /**
   `Object.prototype.toString` tag, kept at upstream's `QuickLRU`.
   */
  readonly [Symbol.toStringTag]: string;
};

//endregion Types
