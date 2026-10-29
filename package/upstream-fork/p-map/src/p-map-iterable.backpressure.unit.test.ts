/**
 Tests for the streaming map's concurrency and backpressure bounds.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pMapIterable,
  pMapSkip,
} from '../dist/final/neutral/index.mjs';

import {
  createGate,
  type Gate,
  yieldTurn,
} from './test-support.ts';

//region Helpers

/**
 Collects one value from a stream's iterator.
 
 @param call - Iterator under test.
 
 @returns The iterator result handed to the consumer.
 
 @example
 ```ts
 const step = await collectOne(iterator,);
 ```
 */
async function collectOne(
  call: AsyncIterator<unknown>,
): Promise<IteratorResult<unknown>> {
  return await call.next();
}

//endregion Helpers

await describe({
  name: 'pMapIterable bounds',
  children: [
    it({
      name: 'starts at most concurrency mapper calls at once',
      fn: async () => {
        /**
         Overlap counters shared by every mapper call.
         */
        const counts = {
          running: 0,
          maxRunning: 0,
        };
        /**
         Gates for the four mapper calls, released in creation order.
         */
        const gates: Gate[] = Array.from(
          {
            length: 4,
          },
          function makeGate(): Gate {
            return createGate();
          },
        );
        /**
         Stream under test with a bound of two concurrent mapper calls.
         */
        const stream = pMapIterable({
          iterable: [
            1,
            2,
            3,
            4,
          ],
          mapper: async function trackedMapper(value: number,): Promise<number> {
            counts.running += 1;
            counts.maxRunning = Math.max(
              counts.maxRunning,
              counts.running,
            );
            await gates.at(value - 1,)
              ?.open;
            counts.running -= 1;
            return value;
          },
          options: {
            concurrency: 2,
            backpressure: 2,
          },
        });
        /**
         Stream iterator consuming while gates hold.
         */
        const iterator = stream[Symbol.asyncIterator]();
        /**
         First collection, pending until a gate releases.
         */
        const firstCollection = collectOne(iterator,);

        await yieldTurn();
        expect(counts.running,).toBe(2,);

        for (const gate of gates)
          gate.release();
        expect((await firstCollection).value,).toBe(1,);
        /* oxlint-disable no-await-in-loop -- collections happen one at a time, in yield order */
        for (let remaining = 2; remaining <= 4; remaining += 1)
          await collectOne(iterator,);
        /* oxlint-enable no-await-in-loop */
        expect(counts.maxRunning,).toBe(2,);
      },
    },),

    it({
      name: 'never carries more than backpressure resolved results ahead of the consumer',
      fn: async () => {
        /**
         Mapper invocation counter standing in for resolved results.
         */
        const counts = {
          started: 0,
        };
        /**
         Stream under test with a lagging consumer.
         */
        const stream = pMapIterable({
          iterable: [
            1,
            2,
            3,
            4,
            5,
            6,
            7,
            8,
          ],
          mapper: function countingMapper(value: number,): number {
            counts.started += 1;
            return value;
          },
          options: {
            concurrency: 2,
            backpressure: 2,
          },
        });
        /**
         Stream iterator consumed one value per settle.
         */
        const iterator = stream[Symbol.asyncIterator]();

        /* oxlint-disable no-await-in-loop -- the lagging consumer collects one value per step, deliberately */
        for (let collected = 1; collected <= 8; collected += 1) {
          /**
           Collection step for one value.
           */
          const step = await collectOne(iterator,);
          expect(step.value,).toBe(collected,);
          await yieldTurn();
          /**
           Resolved-but-unconsumed backlog at this point; the backpressure
           bound caps it.
           */
          const backlog = counts.started - collected;
          expect(backlog,).toBeLessThanOrEqual(2,);
        }
        /* oxlint-enable no-await-in-loop */
      },
    },),

    it({
      name: 'drops a skipped head result and still yields the rest in order',
      fn: async () => {
        /**
         Streamed values in yield order.
         */
        const streamed: number[] = [];
        for await (const mapped of pMapIterable({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: function dropFirst(value: number,): number | typeof pMapSkip {
            return (value === 1)
              ? pMapSkip
              : value;
          },
          options: {
            concurrency: 1,
            backpressure: 1,
          },
        }))
          streamed.push(mapped,);
        expect(streamed,).toEqual([
          2,
          3,
        ],);
      },
    },),

    it({
      name: 'stops spawning pulls once an earlier mapper failed',
      fn: async () => {
        /**
         Pull counter proving the source stops being walked after the
         failure.
         */
        const pullState = {
          pulls: 0,
        };
        /**
         Endless source of incrementing values.
         */
        const endlessIterable: Iterable<number> = {
          [Symbol.iterator]: function openEndlessIterator(): Iterator<number> {
            return {
              next: function endlessNext(): IteratorResult<number> {
                pullState.pulls += 1;
                return {
                  done: false,
                  value: pullState.pulls,
                };
              },
            };
          },
        };
        /**
         Failure observed from the iteration.
         */
        let caught: unknown;
        try {
          for await (const mapped of pMapIterable({
            iterable: endlessIterable,
            mapper: function failingOnTwo(value: number,): number {
              if (value === 2)
                throw new Error('mapper failed',);
              return value;
            },
            options: {
              concurrency: 2,
              backpressure: 2,
            },
          }))
            void mapped;
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('mapper failed',);
        await yieldTurn();
        /**
         Pulls recorded after the failure settled; spawning stops there.
         */
        const pullsAfterFailure = pullState.pulls;
        await yieldTurn();
        expect(pullState.pulls,).toBe(pullsAfterFailure,);
      },
    },),
  ],
},);
