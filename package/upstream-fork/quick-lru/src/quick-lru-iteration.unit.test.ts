/**
 Tests for iteration order, duplicate handling, and `forEach` semantics.
 
 The orderings pin upstream `quick-lru` 7.3.0's dual-cache walk exactly:
 `entriesAscending` walks the old map first and the recent map second,
 `[Symbol.iterator]` walks the recent map first, and `entriesDescending`
 reverses both.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createQuickLru,
} from '../dist/final/neutral/index.mjs';

import {
  installFakeClock,
} from './test-support.ts';

/**
 Builds a cache holding `a`, `b`, and `c` across the dual-cache boundary:
 `c` sits alone in the recent map while `a` and `b` wait in the old map.
 
 @returns Cache primed for ordering assertions.
 
 @example
 ```ts
 const lru = createPrimedCache();
 ```
 */
function createPrimedCache() {
  const lru = createQuickLru<string, string>({
    maxSize: 2,
  },);
  for (const key of [
    'a',
    'b',
    'c',
  ]) {
    lru.set({
      key,
      value: `value-${key}`,
    },);
  }
  return lru;
}

/**
 Builds a cache whose three items are all past their TTL once the fake
 clock advances: `a` and `b` wait in the old map while `c` sits in the
 recent map.
 @param evicted - Log the cache's eviction callback appends to.
 
 @returns Cache primed for lazy-expiry walk assertions.
 
 @example
 ```ts
 const lru = createExpiredCache(evicted,);
 ```
 */
function createExpiredCache(evicted: string[],) {
  const lru = createQuickLru<string, string>({
    maxSize: 2,
    maxAge: 100,
    onEviction: function recordEviction(key: string, value: string,): void {
      evicted.push(`${key}=${value}`,);
    },
  },);
  for (const key of [
    'a',
    'b',
    'c',
  ]) {
    lru.set({
      key,
      value: `value-${key}`,
    },);
  }
  return lru;
}

await describe({
  name: 'cache iteration',
  concurrency: 1,
  children: [
    //region Orderings

    it({
      name: 'iterates entriesAscending oldest map first',
      fn: async () => {
        const lru = createPrimedCache();
        expect([...lru.entriesAscending()].map(function readKey(entry: [string, string],): string {
          return entry[0];
        },),).toEqual([
          'a',
          'b',
          'c',
        ],);
      },
    },),

    it({
      name: 'iterates entriesDescending newest first within each map',
      fn: async () => {
        const lru = createPrimedCache();
        expect([...lru.entriesDescending()].map(function readKey(entry: [string, string],): string {
          return entry[0];
        },),).toEqual([
          'c',
          'b',
          'a',
        ],);
      },
    },),

    it({
      name: 'iterates the default iterator recent map first',
      fn: async () => {
        const lru = createPrimedCache();
        expect([...lru].map(function readKey(entry: [string, string],): string {
          return entry[0];
        },),).toEqual([
          'c',
          'a',
          'b',
        ],);
      },
    },),

    it({
      name: 'returns entriesAscending from entries for Map compatibility',
      fn: async () => {
        const lru = createPrimedCache();
        expect([...lru.entries()],).toEqual([...lru.entriesAscending()],);
      },
    },),

    it({
      name: 'iterates keys and values in the default iterator order',
      fn: async () => {
        const lru = createPrimedCache();
        expect([...lru.keys()],).toEqual([
          'c',
          'a',
          'b',
        ],);
        expect([...lru.values()],).toEqual([
          'value-c',
          'value-a',
          'value-b',
        ],);
      },
    },),

    it({
      name: 'yields key and value pairs from entriesAscending',
      fn: async () => {
        const lru = createPrimedCache();
        expect([...lru.entriesAscending()],).toEqual([
          [
            'a',
            'value-a',
          ],
          [
            'b',
            'value-b',
          ],
          [
            'c',
            'value-c',
          ],
        ],);
      },
    },),

    //endregion Orderings

    //region Recency changes

    it({
      name: 'promotes an old-map item on get and reorders entriesAscending',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        for (const key of [
          'a',
          'b',
          'c',
          'd',
        ]) {
          lru.set({
            key,
            value: `value-${key}`,
          },);
        }
        expect(lru.get('a',),).toBe('value-a',);
        expect([...lru.entriesAscending()].map(function readKey(entry: [string, string],): string {
          return entry[0];
        },),).toEqual([
          'b',
          'c',
          'd',
          'a',
        ],);
      },
    },),

    it({
      name: 'leaves recency untouched on peek',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        for (const key of [
          'a',
          'b',
          'c',
          'd',
        ]) {
          lru.set({
            key,
            value: `value-${key}`,
          },);
        }
        expect(lru.peek('a',),).toBe('value-a',);
        expect([...lru.entriesAscending()].map(function readKey(entry: [string, string],): string {
          return entry[0];
        },),).toEqual([
          'a',
          'b',
          'c',
          'd',
        ],);
      },
    },),

    it({
      name: 'keeps a recent-map key in place when set again, matching upstream',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 5,
        },);
        for (const key of [
          'a',
          'b',
          'c',
        ]) {
          lru.set({
            key,
            value: 1,
          },);
        }
        lru.set({
          key: 'a',
          value: 2,
        },);
        expect([...lru.keys()],).toEqual([
          'a',
          'b',
          'c',
        ],);
        expect(lru.get('a',),).toBe(2,);
      },
    },),

    //endregion Recency changes

    //region Duplicates and special keys

    it({
      name: 'skips old-map duplicates in every ordering',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        for (const key of [
          'a',
          'b',
          'c',
          'd',
        ]) {
          lru.set({
            key,
            value: `value-${key}`,
          },);
        }
        lru.set({
          key: 'a',
          value: 'value-a-new',
        },);
        expect([...lru.entriesAscending()].map(function readKey(entry: [string, string],): string {
          return entry[0];
        },),).toEqual([
          'b',
          'c',
          'd',
          'a',
        ],);
        expect([...lru.entriesDescending()].map(function readKey(entry: [string, string],): string {
          return entry[0];
        },),).toEqual([
          'a',
          'd',
          'c',
          'b',
        ],);
        expect([...lru.keys()],).toEqual([
          'd',
          'a',
          'b',
          'c',
        ],);
        expect(lru.size,).toBe(3,);
      },
    },),

    it({
      name: 'iterates symbol keys like any other key',
      fn: async () => {
        /**
         Symbol key used through the cache.
         */
        const symbolKey = Symbol('symbol cache key for iteration',);
        const lru = createQuickLru<symbol, number>({
          maxSize: 2,
        },);
        lru.set({
          key: symbolKey,
          value: 7,
        },);
        expect([...lru.keys()],).toEqual([symbolKey],);
        expect([...lru.values()],).toEqual([7],);
      },
    },),

    it({
      name: 'creates one fresh generator per iteration',
      fn: async () => {
        const lru = createPrimedCache();
        /**
         First walk over the cache.
         */
        const first = [...lru.entriesAscending()];
        /**
         Second walk over the same cache.
         */
        const second = [...lru.entriesAscending()];
        expect(second,).toEqual(first,);
      },
    },),

    //endregion Duplicates and special keys

    //region forEach

    it({
      name: 'visits entries oldest first with value, key, and cache arguments',
      fn: async () => {
        const lru = createPrimedCache();
        /**
         Visit log recorded by the callback below.
         */
        const visits: string[] = [];
        lru.forEach({
          callback: function recordVisit(value: string, key: string, cache: typeof lru,): void {
            visits.push(`${key}=${value}:${String(cache === lru,)}`,);
          },
        },);
        expect(visits,).toEqual([
          'a=value-a:true',
          'b=value-b:true',
          'c=value-c:true',
        ],);
      },
    },),

    it({
      name: 'calls the visitor with the cache as this by default',
      fn: async () => {
        const lru = createPrimedCache();
        /**
         `this` value observed inside the visitor.
         */
        let observed: unknown;
        lru.forEach({
          callback: function observeThis(this: unknown,): void {
            observed = this;
          },
        },);
        expect(observed,).toBe(lru,);
      },
    },),

    it({
      name: 'calls the visitor with the supplied thisArgument instead',
      fn: async () => {
        const lru = createPrimedCache();
        /**
         Stand-in `this` value for the visitor.
         */
        const thisArgument = {
          tag: 'custom',
        };
        /**
         `this` value observed inside the visitor.
         */
        let observed: unknown;
        lru.forEach({
          callback: function observeThis(this: unknown,): void {
            observed = this;
          },
          thisArgument,
        },);
        expect(observed,).toBe(thisArgument,);
      },
    },),

    //endregion forEach

    //region Lazy expiry during walks

    it({
      name: 'drops expired items from the default iterator with notifications',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: 1_700_000_000_000,
        },);
        /**
         Eviction log recorded by the cache's callback.
         */
        const evicted: string[] = [];
        const lru = createExpiredCache(evicted,);
        clock.advance(200,);
        expect([...lru.keys()],).toEqual([],);
        expect(evicted,).toEqual([
          'c=value-c',
          'a=value-a',
          'b=value-b',
        ],);
      },
    },),

    it({
      name: 'drops expired items from entriesAscending with notifications',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: 1_700_000_000_000,
        },);
        /**
         Eviction log recorded by the cache's callback.
         */
        const evicted: string[] = [];
        const lru = createExpiredCache(evicted,);
        clock.advance(200,);
        expect([...lru.entriesAscending()],).toEqual([],);
        expect(evicted,).toEqual([
          'a=value-a',
          'b=value-b',
          'c=value-c',
        ],);
      },
    },),

    it({
      name: 'drops expired items from entriesDescending with notifications',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: 1_700_000_000_000,
        },);
        /**
         Eviction log recorded by the cache's callback.
         */
        const evicted: string[] = [];
        const lru = createExpiredCache(evicted,);
        clock.advance(200,);
        expect([...lru.entriesDescending()],).toEqual([],);
        expect(evicted,).toEqual([
          'c=value-c',
          'b=value-b',
          'a=value-a',
        ],);
      },
    },),

    it({
      name: 'drops expired old-map duplicates through the raw walk without double notification',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: 1_700_000_000_000,
        },);
        /**
         Eviction log recorded by the cache's callback.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 100,
          onEviction: function recordEviction(key: string, value: string,): void {
            evicted.push(`${key}=${value}`,);
          },
        },);
        for (const key of [
          'a',
          'b',
          'c',
        ]) {
          lru.set({
            key,
            value: `value-${key}`,
          },);
        }
        lru.set({
          key: 'a',
          value: 'value-a-new',
        },);
        clock.advance(200,);
        expect([...lru.entriesAscending()],).toEqual([],);
        expect(evicted,).toEqual([
          'b=value-b',
          'c=value-c',
          'a=value-a-new',
        ],);
      },
    },),

    //endregion Lazy expiry during walks
  ],
},);
