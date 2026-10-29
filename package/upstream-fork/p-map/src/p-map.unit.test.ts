/**
 Tests for the concurrent map's scheduling, result collection, and failure
 handling.
 
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
  yieldTurn,
} from './test-support.ts';

await describe({
  name: pMap.name,
  children: [
    //region Results

    it({
      name: 'collects mapper results in input order',
      fn: async () => {
        const results = await pMap({
          iterable: [
            10,
            20,
            30,
          ],
          mapper: function scaled(value: number, index: number,): number {
            return value + index;
          },
        },);
        expect(results,).toEqual([
          10,
          21,
          32,
        ],);
      },
    },),

    it({
      name: 'collects in input order even when later inputs finish first',
      fn: async () => {
        /**
         Gate held by the first input's mapper call only.
         */
        const gate = createGate();
        /**
         Run promise observed once the slow input's gate releases.
         */
        const running = pMap({
          iterable: [
            'slow',
            'fast',
          ],
          mapper: async function gatedEcho(value: string,): Promise<string> {
            if (value === 'slow')
              await gate.open;
            return value;
          },
        },);
        await yieldTurn();
        gate.release();
        expect(await running,).toEqual([
          'slow',
          'fast',
        ],);
      },
    },),

    it({
      name: 'resolves an empty input to an empty result',
      fn: async () => {
        const results = await pMap({
          iterable: [],
          mapper: function unusedMapper(value: unknown,): unknown {
            return value;
          },
        },);
        expect(results,).toEqual([],);
      },
    },),

    it({
      name: 'awaits promise elements before invoking the mapper',
      fn: async () => {
        /**
         Element order observed by the mapper.
         */
        const seen: number[] = [];
        const results = await pMap({
          iterable: [
            Promise.resolve(1,),
            Promise.resolve(2,),
          ],
          mapper: function recordingMapper(value: number,): number {
            seen.push(value,);
            return value;
          },
        },);
        expect(results,).toEqual([
          1,
          2,
        ],);
        expect(seen,).toEqual([
          1,
          2,
        ],);
      },
    },),

    it({
      name: 'awaits thenable mapper results before collecting them',
      fn: async () => {
        const results = await pMap({
          iterable: ['done',],
          mapper: async function delayedEcho(value: string,): Promise<string> {
            return value;
          },
        },);
        expect(results,).toEqual(['done',],);
      },
    },),

    //endregion Results

    //region Scheduling

    it({
      name: 'starts mapper calls in input order and at most concurrency at once',
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
         Start order recorded as mapper calls run.
         */
        const started: number[] = [];
        /**
         Run promise observed once every gate releases.
         */
        const running = pMap({
          iterable: [
            1,
            2,
            3,
            4,
          ],
          mapper: async function trackedMapper(value: number,): Promise<number> {
            started.push(value,);
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
          options: { concurrency: 2, },
        },);

        await yieldTurn();
        expect(started,).toEqual([
          1,
          2,
        ],);
        expect(counts.running,).toBe(2,);

        for (const gate of gates)
          gate.release();
        await running;
        expect(counts.maxRunning,).toBe(2,);
        expect(started,).toEqual([
          1,
          2,
          3,
          4,
        ],);
      },
    },),

    it({
      name: 'starts the next mapper call after one settles',
      fn: async () => {
        /**
         Gate held by the first mapper call only.
         */
        const gate = createGate();
        /**
         Start order recorded as mapper calls run.
         */
        const started: number[] = [];
        /**
         Run promise observed once the first mapper's gate releases.
         */
        const running = pMap({
          iterable: [
            1,
            2,
          ],
          mapper: async function chainedMapper(value: number,): Promise<number> {
            started.push(value,);
            if (value === 1)
              await gate.open;
            return value;
          },
          options: { concurrency: 1, },
        },);

        await yieldTurn();
        expect(started,).toEqual([1],);

        gate.release();
        await running;
        expect(started,).toEqual([
          1,
          2,
        ],);
      },
    },),

    it({
      name: 'never invokes the mapper during the pMap call itself',
      fn: async () => {
        /**
         Invocation flag set the moment the mapper runs.
         */
        const state = {
          invoked: false,
        };
        const running = pMap({
          iterable: [1],
          mapper: function markInvoked(value: number,): number {
            state.invoked = true;
            return value;
          },
        },);
        expect(state.invoked,).toBe(false,);
        await running;
        expect(state.invoked,).toBe(true,);
      },
    },),

    //endregion Scheduling

    //region Failures

    it({
      name: 'rejects with the synchronous mapper throw, like upstream',
      fn: async () => {
        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await pMap({
            iterable: [1],
            mapper: function throwingMapper(): never {
              throw new Error('sync throw',);
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('sync throw',);
      },
    },),

    it({
      name: 'rejects with the first mapper rejection when stopOnError is true',
      fn: async () => {
        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await pMap({
            iterable: [
              1,
              2,
              3,
            ],
            mapper: function failingOnTwo(value: number,): number {
              if (value === 2)
                throw new Error('first failure',);
              return value;
            },
            options: {
              concurrency: 1,
              stopOnError: true,
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('first failure',);
      },
    },),

    it({
      name: 'rejects with an AggregateError of every failure when stopOnError is false',
      fn: async () => {
        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await pMap({
            iterable: [
              1,
              2,
              3,
            ],
            mapper: function failingOnEvens(value: number,): number {
              if ((value % 2) === 0)
                throw new Error(`failure ${String(value,)}`,);
              return value;
            },
            options: {
              concurrency: 1,
              stopOnError: false,
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(AggregateError,);
        /**
         Aggregated failures as upstream collects them.
         */
        const aggregated = (caught as AggregateError).errors;
        expect(aggregated,).toHaveLength(1,);
        expect((aggregated[0] as Error).message,).toBe('failure 2',);
      },
    },),

    it({
      name: 'resolves normally when stopOnError is false and nothing fails',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
          ],
          mapper: function double(value: number,): number {
            return value * 2;
          },
          options: {
            concurrency: 1,
            stopOnError: false,
          },
        },);
        expect(results,).toEqual([
          2,
          4,
        ],);
      },
    },),

    it({
      name: 'rejects with the iterable failure mid-iteration, even when stopOnError is false',
      fn: async () => {
        /**
         Source yielding one value and then throwing.
         */
        function* oneThenThrow(): Generator<number> {
          yield 1;
          throw new Error('iterable failed',);
        }

        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await pMap({
            iterable: oneThenThrow(),
            mapper: function identity(value: number,): number {
              return value;
            },
            options: {
              concurrency: 1,
              stopOnError: false,
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('iterable failed',);
      },
    },),

    it({
      name: 'rejects with the configuration failure instead of hanging',
      fn: async () => {
        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await pMap({
            iterable: [1],
            mapper: function identity(value: number,): number {
              return value;
            },
            options: { concurrency: 0, },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
      },
    },),

    //endregion Failures

    //region Shutdown

    it({
      name: 'closes the source exactly once when two gated mapper calls fail',
      fn: async () => {
        /**
         Close and pull counters for this source.
         */
        const telemetry = {
          pulls: 0,
          closes: 0,
        };
        /**
         Source counting its pulls and closes.
         */
        const countedIterable: Iterable<number> = {
          [Symbol.iterator]: function openCountedIterator(): Iterator<number> {
            return {
              next: function countedNext(): IteratorResult<number> {
                telemetry.pulls += 1;
                return {
                  done: false,
                  value: telemetry.pulls,
                };
              },
              return: function countedReturn(): IteratorResult<number> {
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
         Gate holding both mapper calls past the settled check, so both
         failures reach the run's rejection handling.
         */
        const gate = createGate();
        /**
         Run promise observed for its rejection.
         */
        const running = pMap({
          iterable: countedIterable,
          mapper: async function gatedFailure(): Promise<never> {
            await gate.open;
            throw new Error('first failure',);
          },
          options: {
            concurrency: 2,
            stopOnError: true,
          },
        },);
        await yieldTurn();
        gate.release();
        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await running;
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('first failure',);
        await yieldTurn();
        expect(telemetry.closes,).toBe(1,);
      },
    },),

    it({
      name: 'leaves an exhausted source open when a mapper fails after the source is done',
      fn: async () => {
        /**
         Close counter for this source.
         */
        const telemetry = {
          closes: 0,
        };
        /**
         Source yielding one value and then reporting `done` on the second
         pull, recording its close.
         */
        const oneValueIterable: Iterable<number> = {
          [Symbol.iterator]: function openOneValueIterator(): Iterator<number> {
            const cursor = {
              position: 0,
            };
            return {
              next: function oneValueNext(): IteratorResult<number> {
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
         Gate holding the single mapper call until the source is done.
         */
        const gate = createGate();
        /**
         Run promise observed for its rejection.
         */
        const running = pMap({
          iterable: oneValueIterable,
          mapper: async function gatedFailure(): Promise<never> {
            await gate.open;
            throw new Error('mapper failed',);
          },
          options: {
            concurrency: 2,
            stopOnError: true,
          },
        },);
        await yieldTurn();
        gate.release();
        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await running;
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('mapper failed',);
        await yieldTurn();
        expect(telemetry.closes,).toBe(0,);
      },
    },),

    //endregion Shutdown
  ],
},);
