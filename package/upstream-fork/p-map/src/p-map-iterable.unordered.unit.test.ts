/**
 Tests for the streaming map's `preserveOrder` option and its `pMapStop`
 handling, ported from `p-map` 7.1.1's `pMapIterable` suites: settle-order
 yields, prompt errors, input-position indexes, and stops with both orders.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pMapIterable,
  pMapSkip,
  pMapStop,
} from '../dist/final/neutral/index.mjs';

import {
  collect,
  countingAsyncSource,
  countingSyncSource,
  createGate,
  type Gate,
  rejectionOf,
  yieldTurn,
} from './test-support.ts';

/**
 Builds one gate per value, so a test decides when each mapper call ends.

 @param count - Number of gates.

 @returns Fresh gates, one per index.

 @example
 ```ts
 const gates = createGates(3,);
 ```
 */
function createGates(count: number,): Gate[] {
  return Array.from(
    { length: count, },
    function makeGate(): Gate {
      return createGate();
    },
  );
}

await describe({
  name: `${pMapIterable.name} with preserveOrder and pMapStop`,
  children: [
    it({
      name: 'yields the results in the order they settle with preserveOrder false',
      fn: async () => {
        const gates = createGates(3,);
        const stream = pMapIterable({
          iterable: [
            0,
            1,
            2,
          ],
          mapper: async function gated(value: number,): Promise<number> {
            await gates[value]?.open;
            return value;
          },
          options: {
            concurrency: 3,
            preserveOrder: false,
          },
        },);
        const collecting = collect(stream,);
        await yieldTurn();
        for (const value of [
          2,
          0,
          1,
        ]) {
          gates[value]?.release();
          // eslint-disable-next-line no-await-in-loop -- one release per turn keeps the settle order fixed
          await yieldTurn();
        }

        expect(await collecting,).toEqual([
          2,
          0,
          1,
        ],);
      },
    },),

    it({
      name: 'defaults to input order',
      fn: async () => {
        const gates = createGates(2,);
        const collecting = collect(pMapIterable({
          iterable: [
            0,
            1,
          ],
          mapper: async function gated(value: number,): Promise<number> {
            await gates[value]?.open;
            return value;
          },
          options: { concurrency: 2, },
        },),);
        await yieldTurn();
        gates[1]?.release();
        await yieldTurn();
        gates[0]?.release();
        expect(await collecting,).toEqual([
          0,
          1,
        ],);
      },
    },),

    it({
      name: 'keeps mapping while an earlier element is pending with preserveOrder false',
      fn: async () => {
        const gate = createGate();
        /**
         Values the mapper was called with.
         */
        const started: number[] = [];
        const iterator = pMapIterable({
          iterable: [
            0,
            1,
            2,
            3,
          ],
          mapper: async function holdFirst(value: number,): Promise<number> {
            started.push(value,);
            if (value === 0)
              await gate.open;
            return value;
          },
          options: {
            concurrency: 2,
            preserveOrder: false,
          },
        },)[Symbol.asyncIterator]();
        /**
         The first three yields, none of them the held element.
         */
        const firstThree = [
          await iterator.next(),
          await iterator.next(),
          await iterator.next(),
        ];
        /**
         Values of the first three yields.
         */
        const firstValues = firstThree.map(function valueOf(step: IteratorResult<number>,): unknown {
          return step.value as unknown;
        },);
        expect(firstValues,).toEqual([
          1,
          2,
          3,
        ],);
        expect(started,).toEqual([
          0,
          1,
          2,
          3,
        ],);
        gate.release();
        /**
         The held element, once released.
         */
        const last = await iterator.next();
        expect(last,).toEqual({
          done: false,
          value: 0,
        },);
      },
    },),

    it({
      name: 'throws a mapper error at once with preserveOrder false while an earlier element is pending',
      fn: async () => {
        const gate = createGate();
        /**
         Failure the second element's mapper throws.
         */
        const failure = new Error('second failed',);
        const iterator = pMapIterable({
          iterable: [
            0,
            1,
          ],
          mapper: async function failSecond(value: number,): Promise<number> {
            if (value === 0)
              await gate.open;
            else
              throw failure;
            return value;
          },
          options: {
            concurrency: 2,
            preserveOrder: false,
          },
        },)[Symbol.asyncIterator]();
        /**
         Error the first step throws.
         */
        const thrown = await rejectionOf(iterator.next(),);
        expect(thrown,).toBe(failure,);
      },
    },),

    it({
      name: 'passes the input position as the index with preserveOrder false',
      fn: async () => {
        /**
         Index paired with each value, in mapper-start order.
         */
        const pairs: string[] = [];
        await collect(pMapIterable({
          iterable: [
            'a',
            'b',
            'c',
          ],
          mapper: async function recordIndex(
            value: string,
            index: number,
          ): Promise<string> {
            await wait((value === 'a')
              ? 10
              : 0,);
            pairs.push(`${value}${String(index,)}`,);
            return value;
          },
          options: {
            concurrency: 3,
            preserveOrder: false,
          },
        },),);
        expect(pairs.toSorted(),).toEqual([
          'a0',
          'b1',
          'c2',
        ],);
      },
    },),

    it({
      name: 'handles skips, falsy results, and an empty input with preserveOrder false',
      fn: async () => {
        const dropped = await collect(pMapIterable({
          iterable: [
            0,
            1,
            2,
            3,
          ],
          mapper: function dropOdd(value: number,): number | typeof pMapSkip {
            return (value % 2) === 1
              ? pMapSkip
              : value;
          },
          options: { preserveOrder: false, },
        },),);
        const empty = await collect(pMapIterable({
          iterable: [],
          mapper: function identity(value: number,): number {
            return value;
          },
          options: { preserveOrder: false, },
        },),);
        expect(dropped,).toEqual([
          0,
          2,
        ],);
        expect(empty,).toEqual([],);
      },
    },),

    it({
      name: 'closes the source and stops pulling when the consumer breaks with preserveOrder false',
      fn: async () => {
        const source = countingAsyncSource();
        for await (const yielded of pMapIterable({
          iterable: source,
          mapper: function identity(value: number,): number {
            return value;
          },
          options: {
            concurrency: 1,
            backpressure: 2,
            preserveOrder: false,
          },
        },)) {
          void yielded;
          break;
        }

        await yieldTurn();
        /**
         Pulls once the break has been seen.
         */
        const { pulls, } = source.counters;
        await wait(20,);
        expect(source.counters.returnCalls,).toBe(1,);
        expect(source.counters.pulls,).toBe(pulls,);
      },
    },),

    it({
      name: 'ends the stream after the earlier results at a stop, and closes the source',
      fn: async () => {
        const source = countingSyncSource();
        const result = await collect(pMapIterable({
          iterable: source,
          mapper: function stopAtThree(value: number,): number | typeof pMapStop {
            return (value === 3)
              ? pMapStop
              : value;
          },
          options: { concurrency: 2, },
        },),);
        expect(result,).toEqual([
          0,
          1,
          2,
        ],);
        await yieldTurn();
        expect(source.counters.returnCalls,).toBe(1,);
        /**
         Pulls once the stream ended.
         */
        const { pulls, } = source.counters;
        await wait(20,);
        expect(source.counters.pulls,).toBe(pulls,);
      },
    },),

    it({
      name: 'stops at the first element without a result',
      fn: async () => {
        const result = await collect(pMapIterable({
          iterable: [
            0,
            1,
          ],
          mapper: function stopAtOnce(): typeof pMapStop {
            return pMapStop;
          },
        },),);
        expect(result,).toEqual([],);
      },
    },),

    it({
      name: 'ends the unordered stream after the results that were ready before the stop',
      fn: async () => {
        const gates = createGates(3,);
        const collecting = collect(pMapIterable({
          iterable: [
            0,
            1,
            2,
          ],
          mapper: async function gated(value: number,): Promise<number | typeof pMapStop> {
            await gates[value]?.open;
            return (value === 1)
              ? pMapStop
              : value;
          },
          options: {
            concurrency: 3,
            preserveOrder: false,
          },
        },),);
        await yieldTurn();
        gates[2]?.release();
        await yieldTurn();
        gates[1]?.release();
        await yieldTurn();
        gates[0]?.release();
        expect(await collecting,).toEqual([2],);
      },
    },),

    it({
      name: 'ignores an error after the stop, and keeps an earlier element that is still pending',
      fn: async () => {
        const gate = createGate();
        const collecting = collect(pMapIterable({
          iterable: [
            0,
            1,
            2,
          ],
          mapper: async function control(value: number,): Promise<number | typeof pMapStop> {
            if (value === 0) {
              await gate.open;
              return value;
            }

            if (value === 1) {
              await yieldTurn();
              gate.release();
              return pMapStop;
            }

            await wait(30,);
            throw new Error('after the stop',);
          },
          options: { concurrency: 3, },
        },),);
        expect(await collecting,).toEqual([0],);
      },
    },),
  ],
},);
