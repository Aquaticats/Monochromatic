/**
 Tests for `maxAge` lifetime handling: lazy expiry, `expiresIn`, and the
 expiry quirks upstream `quick-lru` 7.3.0 carries.
 
 Every test here controls time through the fake clock, so the root suite
 runs sequentially (`concurrency: 1`).
 
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
 Realistic fake time start, far from zero so expiry timestamps never trip
 upstream's falsy-expiry quirks unless a test aims at them deliberately.
 */
const CLOCK_START = 1_700_000_000_000;

await describe({
  name: 'cache expiry',
  concurrency: 1,
  children: [
    //region Lazy expiry

    it({
      name: 'returns a live item and drops it once the global maxAge passes',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 500,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        clock.advance(499,);
        expect(lru.get('a',),).toBe('alpha',);
        clock.advance(2,);
        expect(lru.get('a',),).toBe(undefined,);
        expect(lru.size,).toBe(0,);
      },
    },),

    it({
      name: 'reports an expired item as absent through has and removes it',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 500,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        clock.advance(600,);
        expect(lru.has('a',),).toBe(false,);
        expect(lru.size,).toBe(0,);
      },
    },),

    it({
      name: 'removes an expired item lazily on peek',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 500,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        clock.advance(600,);
        expect(lru.peek('a',),).toBe(undefined,);
        expect(lru.size,).toBe(0,);
      },
    },),

    it({
      name: 'removes an expired old-map item lazily on get',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 2,
          maxAge: 500,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        lru.set({
          key: 'b',
          value: 'beta',
        },);
        lru.set({
          key: 'c',
          value: 'gamma',
        },);
        clock.advance(600,);
        expect(lru.get('a',),).toBe(undefined,);
        expect(lru.has('b',),).toBe(false,);
      },
    },),

    it({
      name: 'removes an expired old-map item lazily on peek',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 2,
          maxAge: 100,
          onEviction: function recordEviction(key: string, value: string,): void {
            evicted.push(`${key}=${value}`,);
          },
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        lru.set({
          key: 'b',
          value: 'beta',
        },);
        lru.set({
          key: 'c',
          value: 'gamma',
        },);
        clock.advance(200,);
        expect(lru.peek('a',),).toBe(undefined,);
        expect(lru.__oldCache.has('a',),).toBe(false,);
        expect(evicted,).toEqual(['a=alpha'],);
      },
    },),

    it({
      name: 'skips expired items in every ordering while lazily removing them',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
          maxAge: 100,
        },);
        lru.set({
          key: 'b',
          value: 'beta',
          maxAge: 100,
        },);
        lru.set({
          key: 'c',
          value: 'gamma',
        },);
        clock.advance(200,);
        expect([...lru.keys()],).toEqual(['c'],);
        expect([...lru.values()],).toEqual(['gamma'],);
        expect([...lru.entriesAscending()],).toEqual([['c', 'gamma']],);
        expect([...lru.entriesDescending()],).toEqual([['c', 'gamma']],);
        expect(lru.size,).toBe(1,);
      },
    },),

    it({
      name: 'excludes expired items from forEach',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 100,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        lru.set({
          key: 'b',
          value: 'beta',
        },);
        clock.advance(150,);
        lru.set({
          key: 'c',
          value: 'gamma',
        },);
        /**
         Visit log recorded by the callback below.
         */
        const visits: string[] = [];
        lru.forEach({
          callback: function recordVisit(value: string, key: string,): void {
            visits.push(`${key}=${value}`,);
          },
        },);
        expect(visits,).toEqual(['c=gamma'],);
      },
    },),

    //endregion Lazy expiry

    //region Per-item lifetimes

    it({
      name: 'lets a per-item maxAge override the global bound',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 10_000,
        },);
        lru.set({
          key: 'short',
          value: 's',
          maxAge: 100,
        },);
        lru.set({
          key: 'long',
          value: 'l',
        },);
        clock.advance(200,);
        expect(lru.get('short',),).toBe(undefined,);
        expect(lru.get('long',),).toBe('l',);
      },
    },),

    it({
      name: 'falls back to the global bound when the per-item maxAge is explicitly undefined',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 100,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
          maxAge: undefined,
        } as never,);
        clock.advance(150,);
        expect(lru.get('a',),).toBe(undefined,);
      },
    },),

    it({
      name: 'keeps a non-numeric per-item maxAge from expiring at all, matching upstream',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 100,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
          maxAge: 'x' as never,
        },);
        clock.advance(1_000_000,);
        expect(lru.get('a',),).toBe('alpha',);
        expect(lru.expiresIn('a',),).toBe(Number.POSITIVE_INFINITY,);
      },
    },),

    it({
      name: 'keeps an infinite per-item maxAge from expiring',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 100,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
          maxAge: Number.POSITIVE_INFINITY,
        },);
        clock.advance(1_000_000,);
        expect(lru.get('a',),).toBe('alpha',);
      },
    },),

    it({
      name: 'expires a zero per-item maxAge at the current instant',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
          maxAge: 0,
        },);
        expect(lru.get('a',),).toBe(undefined,);
      },
    },),

    it({
      name: 'keeps a zero-timestamp expiry readable but deletes it through has, matching upstream quirk',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: 0,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
          maxAge: 0,
        },);
        expect(lru.get('a',),).toBe('alpha',);
        expect(lru.has('a',),).toBe(false,);
        expect(lru.size,).toBe(0,);
      },
    },),

    it({
      name: 'stores no expiry stamp for an infinite per-item maxAge, matching upstream',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 2,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
          maxAge: Number.POSITIVE_INFINITY,
        },);
        lru.set({
          key: 'b',
          value: 'beta',
          maxAge: 100,
        },);
        lru.set({
          key: 'c',
          value: 'gamma',
        },);
        expect(lru.__oldCache.get('a',)?.expiry,).toBe(undefined,);
        expect(lru.__oldCache.get('b',)?.expiry,).toBe(CLOCK_START + 100,);
      },
    },),

    it({
      name: 'expires a negative per-item maxAge immediately',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
          maxAge: -5,
        },);
        expect(lru.get('a',),).toBe(undefined,);
      },
    },),

    it({
      name: 'refreshes expiry when the same key is set again',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 500,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        clock.advance(400,);
        lru.set({
          key: 'a',
          value: 'alpha-2',
        },);
        clock.advance(300,);
        expect(lru.get('a',),).toBe('alpha-2',);
        clock.advance(300,);
        expect(lru.get('a',),).toBe(undefined,);
      },
    },),

    it({
      name: 'keeps an undefined value live through has while it is unexpired',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, undefined>({
          maxSize: 3,
          maxAge: 100,
        },);
        lru.set({
          key: 'a',
          value: undefined,
        },);
        expect(lru.has('a',),).toBe(true,);
        clock.advance(200,);
        expect(lru.has('a',),).toBe(false,);
      },
    },),

    //endregion Per-item lifetimes

    //region expiresIn

    it({
      name: 'reports remaining milliseconds for a live expiring item',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 500,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        expect(lru.expiresIn('a',),).toBe(500,);
        clock.advance(200,);
        expect(lru.expiresIn('a',),).toBe(300,);
      },
    },),

    it({
      name: 'reports infinity for an item without expiry and undefined for a missing key',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        expect(lru.expiresIn('a',),).toBe(Number.POSITIVE_INFINITY,);
        expect(lru.expiresIn('missing',),).toBe(undefined,);
      },
    },),

    it({
      name: 'reports non-positive remaining time without removing the item',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 100,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        clock.advance(150,);
        expect(lru.expiresIn('a',),).toBe(-50,);
        expect(lru.size,).toBe(1,);
        expect(lru.get('a',),).toBe(undefined,);
      },
    },),

    it({
      name: 'reads the recent-map expiry when a key is duplicated, matching upstream precedence',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        const lru = createQuickLru<string, string>({
          maxSize: 3,
        },);
        for (const key of [
          'a',
          'b',
          'c',
        ]) {
          lru.set({
            key,
            value: key,
          },);
        }
        lru.set({
          key: 'd',
          value: 'd',
        },);
        lru.set({
          key: 'a',
          value: 'a-short',
          maxAge: 100,
        },);
        expect(lru.expiresIn('a',),).toBe(100,);
        clock.advance(200,);
        expect(lru.get('a',),).toBe(undefined,);
      },
    },),

    //endregion expiresIn

    //region Eviction notification

    it({
      name: 'notifies onEviction exactly once for a lazily expired item',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 3,
          maxAge: 100,
          onEviction: function recordEviction(key: string, value: string,): void {
            evicted.push(`${key}=${value}`,);
          },
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        clock.advance(200,);
        expect(lru.get('a',),).toBe(undefined,);
        expect(lru.has('a',),).toBe(false,);
        expect(evicted,).toEqual(['a=alpha'],);
      },
    },),

    it({
      name: 'notifies onEviction for a lazily expired old-map item during iteration',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: CLOCK_START,
        },);
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 2,
          maxAge: 100,
          onEviction: function recordEviction(key: string, value: string,): void {
            evicted.push(`${key}=${value}`,);
          },
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        lru.set({
          key: 'b',
          value: 'beta',
        },);
        lru.set({
          key: 'c',
          value: 'gamma',
        },);
        clock.advance(200,);
        expect([...lru.keys()],).toEqual([],);
        expect(evicted.toSorted(),).toEqual([
          'a=alpha',
          'b=beta',
          'c=gamma',
        ],);
      },
    },),

    //endregion Eviction notification
  ],
},);
