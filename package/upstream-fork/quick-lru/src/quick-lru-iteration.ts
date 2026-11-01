/**
 Dual-cache iteration orderings shared by the cache's public iterators.
 
 The orderings pin upstream `quick-lru` 7.3.0's walks exactly:
 `entriesAscending` walks the old map first and the recent map second,
 the default iterator walks the recent map first, and `entriesDescending`
 reverses both. Items that expire during a walk are dropped with the same
 lazy removal upstream performs mid-iteration.
 
 @module
 */

import type {
  QuickLruItem,
  QuickLruItemEntry,
} from './quick-lru-item.ts';

//region Types

/**
 The live cache state the orderings read, plus the lazy-expiry hook they
 call on every item they touch.
 
 The maps are read through the readers at each walk's entry, not captured,
 so a mid-walk rollover is observed exactly as upstream's `this.#cache`
 reads observe it.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 */
export type IterationContext<Key, Value> = {
  /**
   Reads the current recent map, replaced wholesale at every rollover.
   */
  readonly readCache: () => Map<Key, QuickLruItem<Value>>;
  /**
   Reads the current old map, replaced wholesale at every rollover.
   */
  readonly readOldCache: () => Map<Key, QuickLruItem<Value>>;
  /**
   Lazily expires one item when its stamp has passed, notifying and
   removing it exactly as upstream's `#deleteIfExpired` does.
   */
  readonly deleteIfExpired: (entry: QuickLruItemEntry<Key, Value>,) => boolean;
};

/**
 The six orderings over one dual cache.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 */
export type Iteration<Key, Value> = {
  /**
   Raw stored items oldest first, duplicates skipped.
   */
  readonly iterateItemEntriesAscending: () => IterableIterator<readonly [
    Key,
    QuickLruItem<Value>
  ]>;
  /**
   Live entries, recent map before old map.
   */
  readonly iterateEntries: () => IterableIterator<[
    Key,
    Value
  ]>;
  /**
   Live entries, newest first within each map.
   */
  readonly iterateEntriesDescending: () => IterableIterator<[
    Key,
    Value
  ]>;
  /**
   Live entries, oldest first.
   */
  readonly iterateEntriesAscending: () => IterableIterator<[
    Key,
    Value
  ]>;
  /**
   Live keys, recent map before old map.
   */
  readonly iterateKeys: () => IterableIterator<Key>;
  /**
   Live values, recent map before old map.
   */
  readonly iterateValues: () => IterableIterator<Value>;
};

//endregion Types

//region Factory

/**
 Builds the six orderings over one cache state.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 
 @param context - Live maps and the lazy-expiry hook.
 
 @returns Bound zero-argument generators matching upstream's walks.
 
 @example
 ```ts
 const iteration = createIteration(state,);
 [...iteration.iterateKeys()];
 ```
 */
export function createIteration<Key, Value>(context: IterationContext<Key, Value>): Iteration<Key, Value> {
  /**
   Iterates raw stored items oldest first, skipping keys duplicated in the
   recent map and dropping items that expire during iteration, mirroring
   upstream's `#entriesAscending`.
   
   @returns Iterator over old-map entries then recent-map entries.
   */
  function* iterateItemEntriesAscending(): IterableIterator<readonly [
    Key,
    QuickLruItem<Value>
  ]> {
    for (const [key, item] of context.readOldCache()) {
      if ((!context.readCache()
        .has(key,)) && (!context.deleteIfExpired({
          key,
          item,
        },)))
        yield [
          key,
          item,
        ];
    }

    for (const [key, item] of context.readCache()) {
      if (!context.deleteIfExpired({
        key,
        item,
      },))
        yield [
          key,
          item,
        ];
    }
  }

  /**
   Iterates live entries most recently used first: recent map before old
   map, mirroring upstream's `[Symbol.iterator]`.
   
   @returns Iterator over recent-map entries then old-map entries.
   */
  function* iterateEntries(): IterableIterator<[
    Key,
    Value
  ]> {
    for (const [key, item] of context.readCache()) {
      if (!context.deleteIfExpired({
        key,
        item,
      },))
        yield [
          key,
          item.value,
        ];
    }

    for (const [key, item] of context.readOldCache()) {
      if ((!context.readCache()
        .has(key,)) && (!context.deleteIfExpired({
          key,
          item,
        },)))
        yield [
          key,
          item.value,
        ];
    }
  }

  /**
   Iterates live entries newest first within each map, mirroring upstream's
   `entriesDescending`.
   
   @returns Iterator over recent-map entries then old-map entries, each
   map walked in reverse insertion order.
   */
  function* iterateEntriesDescending(): IterableIterator<[
    Key,
    Value
  ]> {
    for (const [key, item] of [...context.readCache()].toReversed()) {
      if (!context.deleteIfExpired({
        key,
        item,
      },))
        yield [
          key,
          item.value,
        ];
    }

    for (const [key, item] of [...context.readOldCache()].toReversed()) {
      if ((!context.readCache()
        .has(key,)) && (!context.deleteIfExpired({
          key,
          item,
        },)))
        yield [
          key,
          item.value,
        ];
    }
  }

  /**
   Iterates live entries oldest first, mirroring upstream's
   `entriesAscending`.
   
   @returns Iterator over old-map entries then recent-map entries.
   */
  function* iterateEntriesAscending(): IterableIterator<[
    Key,
    Value
  ]> {
    for (const [key, item] of iterateItemEntriesAscending())
      yield [
        key,
        item.value,
      ];
  }

  /**
   Iterates live keys most recently used first, mirroring upstream's
   `keys`.
   
   @returns Iterator over live keys in default-iterator order.
   */
  function* iterateKeys(): IterableIterator<Key> {
    for (const [key] of iterateEntries())
      yield key;
  }

  /**
   Iterates live values most recently used first, mirroring upstream's
   `values`.
   
   @returns Iterator over live values in default-iterator order.
   */
  function* iterateValues(): IterableIterator<Value> {
    for (const [, value] of iterateEntries())
      yield value;
  }

  return {
    iterateItemEntriesAscending,
    iterateEntries,
    iterateEntriesDescending,
    iterateEntriesAscending,
    iterateKeys,
    iterateValues,
  };
}

//endregion Factory
