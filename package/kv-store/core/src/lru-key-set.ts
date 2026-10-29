import { ABSENT, } from './constants.ts';

/**
 Ordered key set that tracks LRU access order.
 Uses a `Set<string>` so JS insertion-order iteration gives LRU semantics.
 Does not store values; the store handles that.
 
 Internal helper for store eviction; not part of the package public API.
 
 @example
 ```ts
 const lru = createLruKeySet(1024);
 const evicted = lru.touch('my-key');
 ```
 */
export type LruKeySet = {
  /**
   Mark a key as recently accessed. Returns evicted key, or `ABSENT` when under capacity.
   */
  readonly touch: (key: string,) => string | typeof ABSENT;
  /**
   Remove a key from tracking.
   */
  readonly remove: (key: string,) => void;
  /**
   Clear all tracked keys.
   */
  readonly clear: () => void;
};

/**
 Create an LRU key set that evicts the oldest key when capacity is exceeded.
 
 @param maxSize - maximum tracked keys before eviction
 
 @returns LRU key set where {@link LruKeySet.touch} returns evicted key or {@link ABSENT}
 
 @example
 ```ts
 const lru = createLruKeySet(256);
 const evicted = lru.touch('new-key');
 if (evicted !== ABSENT) {
   store.delete(evicted);
 }
 ```
 */
export function createLruKeySet(
  maxSize: number,
): LruKeySet {
  /**
   Ordered set for insertion-order iteration.
   */
  const keys = new Set<string>();

  return {
    touch(key: string,): string | typeof ABSENT {
      keys.delete(key,);
      keys.add(key,);

      if (keys.size
        > maxSize) {
        /**
         Iterator step naming the oldest key under insertion order.
         */
        const oldest = keys.values()
          .next();
        // oxlint-disable-next-line typescript/strict-boolean-expressions -- IteratorResult.done is boolean|undefined
        if (!oldest.done) {
          keys.delete(oldest.value,);
          return oldest.value;
        }
      }

      return ABSENT;
    },

    remove(key: string,): void {
      keys.delete(key,);
    },

    clear(): void {
      keys.clear();
    },
  };
}

/**
 No-op {@link LruKeySet} used when a store has no eviction policy.
 Tracks nothing and never evicts, so {@link LruKeySet.touch} always returns {@link ABSENT};
 lets call sites treat eviction uniformly without a nullable LRU reference.
 
 @example
 ```ts
 const lru = lruPolicy === undefined ? noopLruKeySet : createLruKeySet(lruPolicy.maxSize);
 ```
 */
export const noopLruKeySet: LruKeySet = {
  touch(_key: string,): typeof ABSENT {
    return ABSENT;
  },

  remove(_key: string,): void {
    // No tracking to update.
  },

  clear(): void {
    // No tracking to clear.
  },
};
