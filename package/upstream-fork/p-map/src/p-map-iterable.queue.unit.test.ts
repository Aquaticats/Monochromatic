/**
 Tests for the bookkeeping of the streaming map's queue of pending pulls: the
 backpressure bound counts only the entries the consumer has not collected,
 and the collected prefix is dropped in halves.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMapIterable, } from '../dist/final/neutral/index.mjs';

import {
  countingSyncSource,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: `${pMapIterable.name} queue`,
  children: [
    it({
      name: 'reads one more element for each result the consumer collects, whatever it collected before',
      fn: async () => {
        const source = countingSyncSource();
        const iterator = pMapIterable({
          iterable: source,
          mapper: function identity(value: number,): number {
            return value;
          },
          options: {
            concurrency: 4,
            backpressure: 4,
          },
        },)[Symbol.asyncIterator]();
        await iterator.next();
        await yieldTurn();
        // Four results were ahead of the consumer, one was collected, so one
        // more can be read.
        expect(source.counters.pulls,).toBe(5,);
        await iterator.next();
        await yieldTurn();
        expect(source.counters.pulls,).toBe(6,);

        await iterator.return?.();
      },
    },),

    it({
      name: 'drops the collected prefix of the queue once it is half of the queue',
      fn: async ctx => {
        /**
         Spy on the array method the queue uses to drop its collected prefix.
         */
        const splice = ctx.sinon.spy(
          Array.prototype,
          'splice',
        );
        /**
         Number of results the consumer collects before it leaves.
         */
        const collectedCount = 10;
        /**
         Results collected so far.
         */
        const collected: number[] = [];
        for await (const result of pMapIterable({
          iterable: countingSyncSource(),
          mapper: function identity(value: number,): number {
            return value;
          },
          options: {
            concurrency: 4,
            backpressure: 4,
          },
        },)) {
          collected.push(result,);
          if (collected.length === collectedCount)
            break;
        }
        /**
         Drops from the front of an array, as the queue makes them.
         */
        const frontDrops = splice.getCalls()
          .map(function argumentsOf(call,): readonly unknown[] {
            return call.args;
          },)
          .filter(function dropsFromFront(args,): boolean {
            return (args.length === 2) && (args[0] === 0);
          },);

        expect(collected,).toHaveLength(collectedCount,);
        expect(frontDrops,).toEqual([
          [
            0,
            3,
          ],
          [
            0,
            3,
          ],
          [
            0,
            3,
          ],
        ],);
      },
    },),
  ],
},);
