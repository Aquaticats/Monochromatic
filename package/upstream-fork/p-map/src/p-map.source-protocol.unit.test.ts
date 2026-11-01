/**
 Tests for how the concurrent map treats its source: a source that reported
 `done` is never pulled again, positions follow the order of the `next()`
 calls, failures are reported in input order, and a thenable answer is
 subscribed to once. These are the `p-map` 7.1.x source semantics.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMap, } from '../dist/final/neutral/index.mjs';

import {
  createGate,
  type Gate,
  pullAt,
  scriptedAsyncSource,
  yieldDone,
  yieldElement,
  yieldTurn,
} from './test-support.ts';

//region Fixtures

/**
 Pull counter written by the instrumented sources.
 */
type PullCount = {
  pulls: number;
};

/**
 Builds a synchronous source over fixed values that counts its pulls.

 @param values - Values yielded in order.

 @param count - Counter every pull increments.

 @returns Iterable whose iterator counts every `next()` call.

 @example
 ```ts
 const source = countedSyncSource([1,], { pulls: 0, },);
 ```
 */
function countedSyncSource(
  values: readonly number[],
  count: PullCount,
): Iterable<number> {
  return {
    [Symbol.iterator]: function openCountedIterator(): Iterator<number> {
      /**
       Cursor shared by this iterator's pulls.
       */
      const cursor = { position: 0, };
      return {
        next: function countedNext(): IteratorResult<number> {
          count.pulls += 1;
          if (cursor.position >= values.length)
            return {
              done: true,
              value: undefined,
            };
          /**
           Value at the cursor, proven present by the bound check above.
           */
          const value = values[cursor.position] ?? 0;
          cursor.position += 1;
          return {
            done: false,
            value,
          };
        },
      };
    },
  };
}

//endregion Fixtures

await describe({
  name: 'pMap source protocol',
  children: [
    it({
      name: 'does not pull a synchronous source again after it reported done',
      fn: async () => {
        /**
         Pull counter for the source.
         */
        const count: PullCount = { pulls: 0, };
        try {
          await pMap({
            iterable: countedSyncSource(
              [1],
              count,
            ),
            mapper: function failing(): number {
              throw new Error('mapper failed',);
            },
            options: {
              concurrency: 2,
              stopOnError: false,
            },
          },);
        }
        catch (error) {
          // The run rejects with an `AggregateError`; only the pulls matter here.
          expect(error,).toBeInstanceOf(AggregateError,);
        }
        // One pull for the element, one for its `done`, and none after.
        expect(count.pulls,).toBe(2,);
      },
    },),

    it({
      name: 'settles without waiting on a pull that started after the source reported done',
      fn: async () => {
        /**
         Pull counter for the source.
         */
        const count: PullCount = { pulls: 0, };
        /**
         Source whose pulls after its `done` never settle, like an exhausted
         queue; with several runners, a pull can already be in flight when the
         `done` arrives, and the run must not wait on it.
         */
        const queue: AsyncIterable<number> = {
          [Symbol.asyncIterator]: function openQueue(): AsyncIterator<number> {
            return {
              next: function queueNext(): Promise<IteratorResult<number>> {
                count.pulls += 1;
                if (count.pulls === 1)
                  return Promise.resolve({
                    done: false,
                    value: 1,
                  },);
                if (count.pulls === 2)
                  return Promise.resolve({
                    done: true,
                    value: undefined,
                  },);
                return new Promise<IteratorResult<number>>(function neverSettles(): void {});
              },
            };
          },
        };
        const results = await pMap({
          iterable: queue,
          mapper: function identity(value: number,): number {
            return value;
          },
          options: { concurrency: 3, },
        },);
        expect(results,).toEqual([1],);
      },
    },),

    it({
      name: 'ignores the failure of a pull that started after the source reported done',
      fn: async () => {
        const source = scriptedAsyncSource<string>();
        const gate = createGate();
        const running = pMap({
          iterable: source.iterable,
          mapper: async function holdFirst(value: string,): Promise<string> {
            if (value === 'a')
              await gate.open;
            return value;
          },
          options: { concurrency: 3, },
        },);
        await yieldTurn();
        yieldElement({
          pull: pullAt({
            source,
            index: 0,
          },),
          value: 'a',
        },);
        await yieldTurn();
        yieldElement({
          pull: pullAt({
            source,
            index: 1,
          },),
          value: 'b',
        },);
        await yieldTurn();
        // Pull 2 comes from the spawn loop and pull 3 from the runner of 'b',
        // which finished at once.
        expect(source.pulls,).toHaveLength(4,);
        yieldDone(pullAt({
          source,
          index: 2,
        },),);
        await yieldTurn();
        pullAt({
          source,
          index: 3,
        },).reject(new Error('late failure',),);
        await yieldTurn();
        gate.release();

        // The run was still waiting for 'a' when the late pull failed.
        expect(await running,).toEqual([
          'a',
          'b',
        ],);
      },
    },),

    it({
      name: 'numbers positions by the order of the next calls when an asynchronous source settles them out of order',
      fn: async () => {
        /**
         Gates deciding when each of the four pulls settles, by call order.
         */
        const pullGates: Gate[] = [
          createGate(),
          createGate(),
          createGate(),
          createGate(),
        ];
        /**
         Gates holding the first two mapper calls until the test releases
         them, so the second pair of pulls overlaps.
         */
        const mapperGates: Gate[] = [
          createGate(),
          createGate(),
        ];
        /**
         Values by pull call order, `undefined` marking a `done`.
         */
        const pulled = ['a', 'b', 'c', 'd'] as const;
        /**
         Next pull call number.
         */
        const cursor = { call: 0, };
        /**
         Mapper calls as `value@position`, in start order.
         */
        const mapperCalls: string[] = [];
        /**
         Source whose `next()` calls settle when their gate opens.
         */
        const source: AsyncIterable<string> = {
          [Symbol.asyncIterator]: function openSource(): AsyncIterator<string> {
            return {
              next: async function gatedNext(): Promise<IteratorResult<string>> {
                const { call, } = cursor;
                cursor.call += 1;
                await pullGates[call]?.open;
                const value = pulled[call];
                if (value === undefined)
                  return {
                    done: true,
                    value: undefined,
                  };
                return {
                  done: false,
                  value,
                };
              },
            };
          },
        };
        const run = pMap({
          iterable: source,
          mapper: async function record(value: string, position: number,): Promise<string> {
            mapperCalls.push(`${value}@${position}`,);
            if (position < 2)
              await mapperGates[position]?.open;
            return `${value}${position}`;
          },
          options: { concurrency: 2, },
        },);
        pullGates[0]?.release();
        await yieldTurn();
        pullGates[1]?.release();
        await yieldTurn();
        mapperGates[0]?.release();
        mapperGates[1]?.release();
        await yieldTurn();
        // Both runners now have a pull in flight; the later call settles first.
        pullGates[3]?.release();
        await yieldTurn();
        pullGates[2]?.release();
        await yieldTurn();
        const results = await run;
        expect(mapperCalls,).toEqual([
          'a@0',
          'b@1',
          'd@3',
          'c@2',
        ],);
        expect(results,).toEqual([
          'a0',
          'b1',
          'c2',
          'd3',
        ],);
      },
    },),

    it({
      name: 'reports stopOnError false failures in input order, not in the order they happened',
      fn: async () => {
        /**
         Gates deciding when each mapper call fails.
         */
        const gates: Gate[] = [
          createGate(),
          createGate(),
        ];
        const run = pMap({
          iterable: [
            0,
            1,
          ],
          mapper: async function failLater(position: number,): Promise<number> {
            await gates[position]?.open;
            throw new Error(position === 0 ? 'first' : 'second',);
          },
          options: {
            concurrency: 2,
            stopOnError: false,
          },
        },);
        // The later input fails first.
        gates[1]?.release();
        await yieldTurn();
        gates[0]?.release();
        /**
         Failure the run rejected with.
         */
        let caught: unknown;
        try {
          await run;
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(AggregateError,);
        expect((caught as AggregateError).errors.map(function messageOf(error: Error,): string {
          return error.message;
        },),).toEqual([
          'first',
          'second',
        ],);
      },
    },),

    it({
      name: 'subscribes once to a thenable answer from an asynchronous source',
      fn: async () => {
        /**
         Counter of `then` calls across every answer.
         */
        const subscriptions = { count: 0, };
        /**
         Pull counter for the source.
         */
        const pulls = { count: 0, };
        /**
         Source answering every pull with a foreign thenable that is not a
         native promise.
         */
        const source: AsyncIterable<number> = {
          [Symbol.asyncIterator]: function openSource(): AsyncIterator<number> {
            return {
              next: function thenableNext(): Promise<IteratorResult<number>> {
                pulls.count += 1;
                /**
                 Result this answer settles with.
                 */
                const settled: IteratorResult<number> = pulls.count === 1
                  ? {
                    done: false,
                    value: 7,
                  }
                  : {
                    done: true,
                    value: undefined,
                  };
                // A proxy is not a native promise, so the engine treats it as a
                // foreign thenable; only calls to its `then` count.
                return new Proxy(
                  Promise.resolve(settled,),
                  {
                    get: function getCounted(target: Promise<IteratorResult<number>>, key: string | symbol,): unknown {
                      if (key !== 'then')
                        return Reflect.get(
                          target,
                          key,
                        );
                      return function countedThen(
                        onFulfilled: Parameters<typeof target.then>[0],
                        onRejected: Parameters<typeof target.then>[1],
                      ): unknown {
                        subscriptions.count += 1;
                        return target.then(
                          onFulfilled,
                          onRejected,
                        );
                      };
                    },
                  },
                );
              },
            };
          },
        };
        const results = await pMap({
          iterable: source,
          mapper: function identity(value: number,): number {
            return value;
          },
        },);
        expect(results,).toEqual([7],);
        expect(subscriptions.count,).toBe(pulls.count,);
      },
    },),
  ],
},);
