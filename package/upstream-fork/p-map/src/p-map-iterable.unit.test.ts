/**
 Tests for the streaming map's yield order, failure surfacing, and shutdown.
 
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
  name: pMapIterable.name,
  children: [
    it({
      name: 'yields mapper results in input order',
      fn: async () => {
        /**
         Streamed values in yield order.
         */
        const streamed: number[] = [];
        for await (const mapped of pMapIterable({
          iterable: [
            3,
            1,
            2,
          ],
          mapper: function identity(value: number,): number {
            return value;
          },
        }))
          streamed.push(mapped,);
        expect(streamed,).toEqual([
          3,
          1,
          2,
        ],);
      },
    },),

    it({
      name: 'yields nothing for an empty input',
      fn: async () => {
        /**
         Streamed values in yield order.
         */
        const streamed: number[] = [];
        for await (const mapped of pMapIterable({
          iterable: [],
          mapper: function unusedMapper(value: unknown,): unknown {
            return value;
          },
        }))
          streamed.push(mapped as number,);
        expect(streamed,).toEqual([],);
      },
    },),

    it({
      name: 'awaits promise elements before invoking the mapper',
      fn: async () => {
        /**
         Streamed values in yield order.
         */
        const streamed: number[] = [];
        for await (const mapped of pMapIterable({
          iterable: [
            Promise.resolve(1,),
            Promise.resolve(2,),
          ],
          mapper: function double(value: number,): number {
            return value * 2;
          },
        }))
          streamed.push(mapped,);
        expect(streamed,).toEqual([
          2,
          4,
        ],);
      },
    },),

    it({
      name: 'walks an asynchronous source',
      fn: async () => {
        /**
         Asynchronous source yielding two values.
         */
        const asyncIterable: AsyncIterable<number> = {
          [Symbol.asyncIterator]: function openAsyncIterable(): AsyncIterator<number> {
            const cursor = {
              position: 0,
            };
            return {
              next: async function asyncNext(): Promise<IteratorResult<number>> {
                if (cursor.position >= 2)
                  return {
                    done: true,
                    value: undefined,
                  };
                cursor.position += 1;
                return {
                  done: false,
                  value: cursor.position,
                };
              },
            };
          },
        };
        /**
         Streamed values in yield order.
         */
        const streamed: number[] = [];
        for await (const mapped of pMapIterable({
          iterable: asyncIterable,
          mapper: function double(value: number,): number {
            return value * 2;
          },
        }))
          streamed.push(mapped,);
        expect(streamed,).toEqual([
          2,
          4,
        ],);
      },
    },),

    it({
      name: 'drops skipped results instead of yielding them',
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
          mapper: function dropEvens(value: number,): number | typeof pMapSkip {
            return ((value % 2) === 0)
              ? pMapSkip
              : value;
          },
        }))
          streamed.push(mapped,);
        expect(streamed,).toEqual([
          1,
          3,
        ],);
      },
    },),

    it({
      name: 'throws the mapper failure from the async iterator',
      fn: async () => {
        /**
         Failure observed from the iteration.
         */
        let caught: unknown;
        try {
          for await (const mapped of pMapIterable({
            iterable: [
              1,
              2,
            ],
            mapper: function failingOnTwo(value: number,): number {
              if (value === 2)
                throw new Error('mapper failed',);
              return value;
            },
          }))
            void mapped;
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('mapper failed',);
      },
    },),

    it({
      name: 'closes the source and stops spawning once the consumer breaks early',
      fn: async () => {
        /**
         Close telemetry for this source.
         */
        const closeState = {
          returnCalled: false,
        };
        /**
         Pull counter proving the source was not walked past the break.
         */
        const pullState = {
          pulls: 0,
        };
        /**
         Source counting pulls and recording its shutdown.
         */
        const countedIterable: Iterable<number> = {
          [Symbol.iterator]: function openCountedIterator(): Iterator<number> {
            return {
              next: function countedNext(): IteratorResult<number> {
                pullState.pulls += 1;
                return {
                  done: false,
                  value: pullState.pulls,
                };
              },
              return: function countedReturn(): IteratorResult<number> {
                closeState.returnCalled = true;
                return {
                  done: true,
                  value: undefined,
                };
              },
            };
          },
        };

        for await (const mapped of pMapIterable({
          iterable: countedIterable,
          mapper: function identity(value: number,): number {
            return value;
          },
          options: {
            concurrency: 1,
            backpressure: 1,
          },
        })) {
          expect(mapped,).toBe(1,);
          break;
        }
        await yieldTurn();
        expect(closeState.returnCalled,).toBe(true,);
        /**
         Pulls after the consumer's break; upstream stops pulling there, so
         this must stay within the spawn already in flight.
         */
        const pullsAtBreak = pullState.pulls;
        await yieldTurn();
        expect(pullState.pulls,).toBe(pullsAtBreak,);
      },
    },),

    it({
      name: 'bounds mapper concurrency independently of a slack backpressure bound',
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
         Stream under test: concurrency two, backpressure four, so only the
         concurrency bound can hold the mapper calls back.
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
            backpressure: 4,
          },
        });
        /**
         Stream iterator consumed while gates hold.
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
      name: 'leaves an exhausted source open rather than closing it',
      fn: async () => {
        /**
         Close and pull counters for this source.
         */
        const telemetry = {
          pulls: 0,
          closes: 0,
        };
        /**
         Source yielding one value, then done, recording its close.
         */
        const oneValueIterable: Iterable<number> = {
          [Symbol.iterator]: function openOneValueIterator(): Iterator<number> {
            const cursor = {
              position: 0,
            };
            return {
              next: function oneValueNext(): IteratorResult<number> {
                telemetry.pulls += 1;
                if (cursor.position >= 1)
                  return {
                    done: true,
                    value: undefined,
                  };
                cursor.position += 1;
                return {
                  done: false,
                  value: 1,
                };
              },
              return: function oneValueReturn(): IteratorResult<number> {
                telemetry.closes += 1;
                return {
                  done: true,
                  value: undefined,
                };
              },
            };
          },
        };
        /**
         Streamed values in yield order.
         */
        const streamed: number[] = [];
        for await (const mapped of pMapIterable({
          iterable: oneValueIterable,
          mapper: function identity(value: number,): number {
            return value;
          },
          options: {
            concurrency: 2,
            backpressure: 2,
          },
        }))
          streamed.push(mapped,);
        expect(streamed,).toEqual([1],);
        await yieldTurn();
        expect(telemetry.closes,).toBe(0,);
        /**
         Pulls recorded after the source reported `done`; spawning stops
         there.
         */
        const pullsAfterDone = telemetry.pulls;
        await yieldTurn();
        expect(telemetry.pulls,).toBe(pullsAfterDone,);
      },
    },),

    it({
      name: 're-walks the source on a second iteration',
      fn: async () => {
        /**
         Stream under test, iterated twice.
         */
        const stream = pMapIterable({
          iterable: [
            1,
            2,
          ],
          mapper: function double(value: number,): number {
            return value * 2;
          },
        });
        /**
         First walk's streamed values.
         */
        const first: number[] = [];
        for await (const value of stream)
          first.push(value,);
        /**
         Second walk's streamed values.
         */
        const second: number[] = [];
        for await (const value of stream)
          second.push(value,);
        expect(first,).toEqual([
          2,
          4,
        ],);
        expect(second,).toEqual([
          2,
          4,
        ],);
      },
    },),
  ],
},);
