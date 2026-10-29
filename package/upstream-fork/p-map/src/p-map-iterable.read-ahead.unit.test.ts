/**
 Tests for the streaming map's read-ahead, ordering of failures, and prompt
 stop: the `p-map` 7.1.x `pMapIterable` semantics.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMapIterable, } from '../dist/final/neutral/index.mjs';

import {
  createGate,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: `${pMapIterable.name} read-ahead and stop`,
  children: [
    it({
      name: 'reads the next element while the only mapper slot is busy when backpressure exceeds concurrency',
      fn: async () => {
        /**
         Gate holding the first mapper call.
         */
        const gate = createGate();
        /**
         Source pulls and mapper starts observed so far.
         */
        const seen = {
          pulls: 0,
          started: 0,
        };
        /**
         Source over three values that counts its pulls.
         */
        const source: Iterable<number> = {
          [Symbol.iterator]: function openCounted(): Iterator<number> {
            /**
             Cursor shared by this iterator's pulls.
             */
            const cursor = { position: 0, };
            return {
              next: function countedNext(): IteratorResult<number> {
                seen.pulls += 1;
                cursor.position += 1;
                if (cursor.position > 3)
                  return {
                    done: true,
                    value: undefined,
                  };
                return {
                  done: false,
                  value: cursor.position,
                };
              },
            };
          },
        };
        /**
         Iterator driving the stream.
         */
        const iterator = pMapIterable({
          iterable: source,
          mapper: async function hold(value: number,): Promise<number> {
            seen.started += 1;
            await gate.open;
            return value;
          },
          options: {
            concurrency: 1,
            backpressure: 3,
          },
        },)[Symbol.asyncIterator]();
        /**
         First step, pending until the gate opens.
         */
        const first = iterator.next();
        await yieldTurn();
        // One mapper runs, and the second element is already read and waits
        // for the slot.
        expect(seen.started,).toBe(1,);
        expect(seen.pulls,).toBe(2,);
        gate.release();
        expect(await first,).toEqual({
          done: false,
          value: 1,
        },);
      },
    },),

    it({
      name: 'yields the results before a failing element, then throws its error',
      fn: async () => {
        /**
         Gate holding the first element's promise.
         */
        const gate = createGate();
        /**
         Failure the second element's mapper throws.
         */
        const failure = new Error('second failed',);
        /**
         Iterator driving the stream.
         */
        const iterator = pMapIterable({
          iterable: [
            gate.open.then(function firstValue(): number {
              return 1;
            },),
            2,
          ],
          mapper: function failOnSecond(value: number,): number {
            if (value === 2)
              throw failure;
            return value * 10;
          },
          options: { concurrency: 2, },
        },)[Symbol.asyncIterator]();
        /**
         First step, pending on the first element.
         */
        const first = iterator.next();
        await yieldTurn();
        // The second element failed while the first was still being read.
        gate.release();
        expect(await first,).toEqual({
          done: false,
          value: 10,
        },);
        await expect(iterator.next(),).rejects.toBe(failure,);
      },
    },),

    it({
      name: 'drops pending elements as soon as return is called while a next call is pending',
      fn: async () => {
        /**
         Gate holding the first element's promise.
         */
        const gate = createGate();
        /**
         Mapper starts observed so far.
         */
        const started: number[] = [];
        /**
         Close telemetry for the source.
         */
        const closes = { count: 0, };
        /**
         Source whose first element is a pending promise.
         */
        const source: Iterable<number | Promise<number>> = {
          [Symbol.iterator]: function openSource(): Iterator<number | Promise<number>> {
            /**
             Cursor shared by this iterator's pulls.
             */
            const cursor = { position: 0, };
            return {
              next: function sourceNext(): IteratorResult<number | Promise<number>> {
                cursor.position += 1;
                if (cursor.position === 1)
                  return {
                    done: false,
                    value: gate.open.then(function firstValue(): number {
                      return 1;
                    },),
                  };
                return {
                  done: true,
                  value: undefined,
                };
              },
              return: function sourceReturn(): IteratorResult<number | Promise<number>> {
                closes.count += 1;
                return {
                  done: true,
                  value: undefined,
                };
              },
            };
          },
        };
        /**
         Iterator driving the stream.
         */
        const iterator = pMapIterable({
          iterable: source,
          mapper: function record(value: number,): number {
            started.push(value,);
            return value;
          },
        },)[Symbol.asyncIterator]();
        /**
         Step pending on the first element.
         */
        const pending = iterator.next();
        await yieldTurn();
        /**
         Stop request queued behind the pending step.
         */
        const stopped = iterator.return?.();
        gate.release();
        await pending;
        await stopped;
        await yieldTurn();
        expect(started,).toEqual([],);
      },
    },),
  ],
},);
