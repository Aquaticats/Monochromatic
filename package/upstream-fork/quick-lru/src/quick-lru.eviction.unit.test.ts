/**
 Tests for eviction notification semantics and the `evict` member.
 
 Upstream `quick-lru` 7.3.0 notifies `onEviction` on rollover pressure, TTL
 expiry, and manual `evict` calls, never on `delete` or `clear`. The
 rollover notification even fires for stale duplicate copies that leave the
 old map while their keys survive, which these tests pin.
 
 The root suite runs sequentially (`concurrency: 1`) because the
 mixed-expiry case controls time through the fake clock.
 
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

await describe({
  name: 'cache eviction',
  concurrency: 1,
  children: [
    //region Notification rules

    it({
      name: 'never notifies onEviction for delete',
      fn: async () => {
        /**
         Eviction log that must stay empty across manual deletion.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 2,
          onEviction: function recordEviction(key: string,): void {
            evicted.push(key,);
          },
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        lru.delete('a',);
        expect(evicted,).toEqual([],);
      },
    },),

    it({
      name: 'never notifies onEviction for clear',
      fn: async () => {
        /**
         Eviction log that must stay empty across manual clearing.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 2,
          onEviction: function recordEviction(key: string,): void {
            evicted.push(key,);
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
        lru.clear();
        expect(evicted,).toEqual([],);
      },
    },),

    it({
      name: 'notifies onEviction when a rollover drops the previous old map',
      fn: async () => {
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 2,
          onEviction: function recordEviction(key: string, value: string,): void {
            evicted.push(`${key}=${value}`,);
          },
        },);
        for (const key of [
          'a',
          'b',
          'c',
          'd',
        ]) {
          lru.set({
            key,
            value: key,
          },);
        }
        expect(evicted,).toEqual([
          'a=a',
          'b=b',
        ],);
      },
    },),

    it({
      name: 'notifies stale duplicate copies at rollover while their keys survive, matching upstream',
      fn: async () => {
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 3,
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
            value: key,
          },);
        }
        lru.set({
          key: 'a',
          value: 'a-new',
        },);
        lru.set({
          key: 'b',
          value: 'b-new',
        },);
        lru.set({
          key: 'd',
          value: 'd',
        },);
        expect(evicted,).toEqual([
          'a=a',
          'b=b',
          'c=c',
        ],);
        expect(lru.get('a',),).toBe('a-new',);
        expect(lru.get('b',),).toBe('b-new',);
      },
    },),

    //endregion Notification rules

    //region evict

    it({
      name: 'evicts the least recently used items oldest first with notifications',
      fn: async () => {
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 5,
          onEviction: function recordEviction(key: string,): void {
            evicted.push(key,);
          },
        },);
        for (const key of [
          'a',
          'b',
          'c',
          'd',
          'e',
        ]) {
          lru.set({
            key,
            value: key,
          },);
        }
        lru.evict(2,);
        expect(evicted,).toEqual([
          'a',
          'b',
        ],);
        expect(lru.has('a',),).toBe(false,);
        expect(lru.has('c',),).toBe(true,);
      },
    },),

    it({
      name: 'keeps at least one item when the requested count covers everything',
      fn: async () => {
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 5,
          onEviction: function recordEviction(key: string,): void {
            evicted.push(key,);
          },
        },);
        for (const key of [
          'a',
          'b',
          'c',
          'd',
          'e',
        ]) {
          lru.set({
            key,
            value: key,
          },);
        }
        lru.evict(10,);
        expect(evicted,).toEqual([
          'a',
          'b',
          'c',
          'd',
        ],);
        expect([...lru.keys()],).toEqual(['e'],);
      },
    },),

    it({
      name: 'defaults to evicting one item',
      fn: async () => {
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 5,
          onEviction: function recordEviction(key: string,): void {
            evicted.push(key,);
          },
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
        lru.evict();
        expect(evicted,).toEqual(['a'],);
      },
    },),

    it({
      name: 'does nothing for zero and negative counts',
      fn: async () => {
        /**
         Eviction log that must stay empty across rejected counts.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 5,
          onEviction: function recordEviction(key: string,): void {
            evicted.push(key,);
          },
        },);
        lru.set({
          key: 'a',
          value: 'a',
        },);
        lru.evict(0,);
        lru.evict(-3,);
        lru.evict(Number.NaN,);
        expect(evicted,).toEqual([],);
        expect(lru.has('a',),).toBe(true,);
      },
    },),

    it({
      name: 'does nothing on an empty cache',
      fn: async () => {
        /**
         Eviction log that must stay empty on an empty cache.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 5,
          onEviction: function recordEviction(key: string,): void {
            evicted.push(key,);
          },
        },);
        lru.evict(3,);
        expect(evicted,).toEqual([],);
        expect(lru.size,).toBe(0,);
      },
    },),

    it({
      name: 'keeps a single-item cache intact when asked to evict it',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 5,
        },);
        lru.set({
          key: 'a',
          value: 'a',
        },);
        lru.evict(1,);
        expect(lru.has('a',),).toBe(true,);
        expect(lru.size,).toBe(1,);
        expect(lru.__oldCache.size,).toBe(0,);
      },
    },),

    it({
      name: 'touches nothing, not even lazy expiry, for a rejected evict count',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: 1_700_000_000_000,
        },);
        /**
         Eviction log that must stay empty across rejected counts.
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
          key: 'stale',
          value: 's',
        },);
        lru.set({
          key: 'live',
          value: 'l',
          maxAge: Number.POSITIVE_INFINITY,
        },);
        clock.advance(200,);
        for (const count of [
          0,
          -3,
          Number.NaN,
        ]) {
          lru.evict(count,);
          expect(evicted,).toEqual([],);
          expect(lru.size,).toBe(2,);
          expect(lru.__oldCache.size,).toBe(0,);
        }
      },
    },),

    it({
      name: 'truncates fractional counts toward zero',
      fn: async () => {
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 5,
          onEviction: function recordEviction(key: string,): void {
            evicted.push(key,);
          },
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
        lru.evict(1.9,);
        expect(evicted,).toEqual(['a'],);
      },
    },),

    it({
      name: 'lazily expires TTL items with notifications before evicting live ones',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: 1_700_000_000_000,
        },);
        /**
         Eviction log recorded by the callback below.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 5,
          onEviction: function recordEviction(key: string, value: string,): void {
            evicted.push(`${key}=${value}`,);
          },
        },);
        lru.set({
          key: 'stale',
          value: 's',
          maxAge: 100,
        },);
        lru.set({
          key: 'live-1',
          value: 'l1',
        },);
        lru.set({
          key: 'live-2',
          value: 'l2',
        },);
        clock.advance(200,);
        lru.evict(1,);
        expect(evicted,).toEqual([
          'stale=s',
          'live-1=l1',
        ],);
        expect([...lru.keys()],).toEqual(['live-2'],);
      },
    },),

    it({
      name: 'rebuilds both maps after evict so later writes keep working',
      fn: async () => {
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
        lru.evict(2,);
        lru.set({
          key: 'd',
          value: 'd',
        },);
        expect([...lru.keys()],).toEqual([
          'd',
          'c',
        ],);
        expect(lru.get('d',),).toBe('d',);
      },
    },),

    //endregion evict
  ],
},);
