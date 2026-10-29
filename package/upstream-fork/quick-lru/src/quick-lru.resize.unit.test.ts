/**
 Tests for `resize`: bound changes, item retention, and eviction
 notification on shrink.
 
 The TTL case controls time through the fake clock, so the root suite runs
 sequentially (`concurrency: 1`).
 
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
  name: 'cache resize',
  concurrency: 1,
  children: [
    it({
      name: 'growing keeps every item and moves them into the recent map',
      fn: async () => {
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
            value: key,
          },);
        }
        lru.resize(5,);
        expect(lru.maxSize,).toBe(5,);
        expect(lru.size,).toBe(3,);
        expect(lru.__oldCache.size,).toBe(0,);
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
      name: 'shrinking evicts the oldest items with notifications and keeps the rest',
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
        lru.resize(3,);
        expect(evicted,).toEqual([
          'a',
          'b',
        ],);
        expect(lru.maxSize,).toBe(3,);
        expect(lru.size,).toBe(3,);
        expect(lru.has('a',),).toBe(false,);
        expect(lru.has('c',),).toBe(true,);
        expect(lru.get('e',),).toBe('e',);
      },
    },),

    it({
      name: 'resizing to the current bound keeps items and notifies nothing',
      fn: async () => {
        /**
         Eviction log that must stay empty across an equal-bound resize.
         */
        const evicted: string[] = [];
        const lru = createQuickLru<string, string>({
          maxSize: 3,
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
        lru.resize(3,);
        expect(evicted,).toEqual([],);
        expect(lru.__oldCache.size,).toBe(3,);
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
      name: 'lazily expires TTL items with notifications while collecting for resize',
      fn: async () => {
        using clock = installFakeClock({
          startMilliseconds: 1_700_000_000_000,
        },);
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
        lru.set({
          key: 'stale',
          value: 's',
          maxAge: 100,
        },);
        lru.set({
          key: 'b',
          value: 'b',
        },);
        lru.set({
          key: 'c',
          value: 'c',
        },);
        clock.advance(200,);
        lru.resize(5,);
        expect(evicted,).toEqual(['stale=s'],);
        expect(lru.size,).toBe(2,);
        expect([...lru.keys()],).toEqual([
          'b',
          'c',
        ],);
      },
    },),

    it({
      name: 'keeps dual-cache rollover working at the new bound',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 2,
        },);
        lru.resize(3,);
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
        expect(lru.__oldCache.size,).toBe(3,);
        lru.set({
          key: 'd',
          value: 'd',
        },);
        expect(lru.size,).toBe(3,);
        expect(lru.get('a',),).toBe('a',);
      },
    },),
  ],
},);
