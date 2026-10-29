/**
 Tests for `mapIterableAsync`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { mapIterableAsync, } from './map-iterable-async.ts';

await describe({
  name: mapIterableAsync.name,
  children: [
    it({
      name: 'maps a sync iterable preserving order',
      fn: async () => {
        const result = await mapIterableAsync({
          fn: async (n: number) => n * 2,
          iterable: [1, 2, 3],
        },);

        expect(result,).toEqual([2, 4, 6],);
      },
    },),

    it({
      name: 'maps an async iterable preserving order',
      fn: async () => {
        async function* source() {
          yield 'a';
          yield 'b';
          yield 'c';
        }

        const result = await mapIterableAsync({
          fn: async (s: string) => s.toUpperCase(),
          iterable: source(),
        },);

        expect(result,).toEqual(['A', 'B', 'C'],);
      },
    },),

    it({
      name: 'returns an empty array for empty input',
      fn: async () => {
        const result = await mapIterableAsync({
          fn: async (n: number) => n,
          iterable: [],
        },);

        expect(result,).toEqual([],);
      },
    },),

    it({
      name: 'starts every mapper eagerly and preserves order when later mappers resolve first',
      fn: async () => {
        // Each item carries its own resolve delay; earlier items wait longer, so
        // the last item resolves first and output order cannot be a side effect
        // of resolution order.
        const plan = [
          { value: 'a', delayMs: 60 },
          { value: 'b', delayMs: 40 },
          { value: 'c', delayMs: 20 },
        ];

        // `starts` records the synchronous call order of each mapper, proving
        // every call begins during iteration rather than one-at-a-time.
        const starts: string[] = [];

        const pending = mapIterableAsync({
          fn: async (item: { value: string; delayMs: number }) => {
            starts.push(item.value,);
            await new Promise(function resolveAfterDelay(resolve,) {
              setTimeout(resolve, item.delayMs,);
            },);
            return item.value;
          },
          iterable: plan,
        },);

        // A 0ms macrotask runs after every microtask the for-await loop queues,
        // yet before the mapper delays (>=20ms) elapse: the snapshot point where
        // all mappers have started but none has resolved.
        await new Promise(function flushMicrotasks(resolve,) {
          setTimeout(resolve, 0,);
        },);

        expect(starts,).toEqual(['a', 'b', 'c'],);

        // Output follows input order despite 'c' resolving before 'a'.
        expect(await pending,).toEqual(['a', 'b', 'c'],);
      },
    },),

    it({
      name: 'propagates mapper rejection',
      fn: async () => {
        const rejection = mapIterableAsync({
          fn: async (n: number) => {
            throw new Error(`boom ${String(n,)}`,);
          },
          iterable: [1, 2, 3],
        },);

        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        await expect(rejection,).rejects.toThrow('boom',);
      },
    },),
  ],
},);
