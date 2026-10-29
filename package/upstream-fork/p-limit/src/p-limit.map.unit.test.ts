/**
 Tests for `map`: iterating inputs under the concurrency bound with results
 collected in input order.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pLimit, } from '../dist/final/neutral/index.mjs';

import {
  createGate,
  type Gate,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: 'map',
  children: [
    it({
      name: 'collects mapper results in input order',
      fn: async () => {
        const limit = pLimit(2,);
        const results = await limit.map({
          iterable: [
            1,
            2,
            3,
            4,
          ],
          mapper: async function double(value: number,): Promise<number> {
            return value * 2;
          },
        },);
        expect(results,).toEqual([
          2,
          4,
          6,
          8,
        ],);
      },
    },),

    it({
      name: 'passes each input and its index to the mapper',
      fn: async () => {
        const limit = pLimit(1,);
        const results = await limit.map({
          iterable: [
            'a',
            'b',
          ],
          mapper: function label(value: string, index: number,): string {
            return `${value}${String(index,)}`;
          },
        },);
        expect(results,).toEqual([
          'a0',
          'b1',
        ],);
      },
    },),

    it({
      name: 'resolves for an empty iterable',
      fn: async () => {
        const limit = pLimit(1,);
        const results = await limit.map({
          iterable: [],
          mapper: function unreachableMapper(value: unknown,): unknown {
            return value;
          },
        },);
        expect(results,).toEqual([],);
      },
    },),

    it({
      name: 'respects the concurrency bound while mapping',
      fn: async () => {
        const limit = pLimit(2,);
        /**
         Overlap counters shared by every mapped call.
         */
        const counts = {
          running: 0,
          maxRunning: 0,
        };
        /**
         Gates for the six mapped inputs, released in creation order.
         */
        const gates: Gate[] = Array.from(
          { length: 6, },
          function makeGate(): Gate {
            return createGate();
          },
        );
        /**
         Mapped results awaited after the gates below are released.
         */
        const resultsPromise = limit.map({
          iterable: gates,
          mapper: async function trackedMapper(gate: Gate, index: number,): Promise<number> {
            counts.running += 1;
            counts.maxRunning = Math.max(
              counts.maxRunning,
              counts.running,
            );
            await gate.open;
            counts.running -= 1;
            return index;
          },
        },);

        await yieldTurn();
        expect(counts.running,).toBe(2,);
        for (const gate of gates)
          gate.release();
        expect(await resultsPromise,).toEqual([
          0,
          1,
          2,
          3,
          4,
          5,
        ],);
        expect(counts.maxRunning,).toBe(2,);
      },
    },),

    it({
      name: 'rejects when the mapper rejects',
      fn: async () => {
        const limit = pLimit(2,);
        /**
         Failure the mapper rejects with on its second input.
         */
        const failure = new Error('mapper failed',);
        let caught: unknown;
        try {
          await limit.map({
            iterable: [
              1,
              2,
              3,
            ],
            mapper: async function failingMapper(value: number,): Promise<number> {
              if (value === 2)
                throw failure;
              return value;
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);
      },
    },),

    it({
      name: 'rejects with the iterable failure and still runs already scheduled calls',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Failure the input iterable throws after yielding two items.
         */
        const failure = new Error('iterable failed',);
        /**
         Call outcomes recorded by the mapper, proving scheduled calls still
         ran after the iterable threw.
         */
        const mapped: number[] = [];
        /**
         Input iterable that yields two values then throws.
         */
        function* failingIterable(): Generator<number> {
          yield 1;
          yield 2;
          throw failure;
        }

        let caught: unknown;
        try {
          await limit.map({
            iterable: failingIterable(),
            mapper: async function recordMapper(value: number,): Promise<number> {
              mapped.push(value,);
              return value;
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);

        await yieldTurn();
        expect(mapped,).toEqual([
          1,
          2,
        ],);
        expect(limit.activeCount,).toBe(0,);
        expect(limit.pendingCount,).toBe(0,);
      },
    },),
  ],
},);
