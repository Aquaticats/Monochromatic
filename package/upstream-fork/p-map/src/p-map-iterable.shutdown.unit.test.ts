/**
 Tests for what the streaming map stops doing once its consumer leaves or a
 stop or failure ends the stream: no more pulls, no more mapper calls for
 elements that nobody can collect, and the elements before the end still
 finish.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pMapIterable,
  pMapStop,
} from '../dist/final/neutral/index.mjs';

import {
  collect,
  countingSyncSource,
  createGate,
  rejectionOf,
  yieldTurn,
} from './test-support.ts';

//region Fixtures

/**
 Pull counter of a finite source.
 */
type PullCount = {
  pulls: number;
};

/**
 Builds a finite synchronous source that counts its pulls, `done` pulls
 included.

 @param length - Number of values the source yields before its `done`.

 @param count - Counter every pull increments.

 @returns Iterable of `0` up to `length` exclusive.

 @example
 ```ts
 const source = finiteCountedSource(5, { pulls: 0, },);
 ```
 */
function finiteCountedSource(
  length: number,
  count: PullCount,
): Iterable<number> {
  return {
    [Symbol.iterator]: function openFiniteSource(): Iterator<number> {
      return {
        next: function finiteNext(): IteratorResult<number> {
          /**
           Position this pull takes.
           */
          const position = count.pulls;
          count.pulls += 1;
          if (position >= length)
            return {
              done: true,
              value: undefined,
            };
          return {
            done: false,
            value: position,
          };
        },
      };
    },
  };
}

//endregion Fixtures

await describe({
  name: `${pMapIterable.name} shutdown`,
  children: [
    describe({
      name: 'at a stop or a failure with preserveOrder true',
      children: [
        it({
          name: 'neither maps nor pulls past an element whose mapper returned pMapStop',
          fn: async () => {
            const source = countingSyncSource();
            /**
             Values the mapper was called with.
             */
            const started: number[] = [];
            const results = await collect(pMapIterable({
              iterable: source,
              mapper: function stopAtOne(value: number,): number | typeof pMapStop {
                started.push(value,);
                return (value === 1)
                  ? pMapStop
                  : value;
              },
              options: {
                concurrency: 1,
                backpressure: Number.POSITIVE_INFINITY,
              },
            },),);
            await yieldTurn();

            expect(results,).toEqual([0],);
            expect(started,).toEqual([
              0,
              1,
            ],);
            // Elements 0 and 1, plus element 2, which was read while the
            // mapper of element 1 ran.
            expect(source.counters.pulls,).toBe(3,);
          },
        },),

        it({
          name: 'neither maps nor pulls past an element whose mapper failed',
          fn: async () => {
            const source = countingSyncSource();
            /**
             Failure the mapper throws for element 1.
             */
            const failure = new Error('mapper failed',);
            /**
             Values the mapper was called with.
             */
            const started: number[] = [];
            const collecting = collect(pMapIterable({
              iterable: source,
              mapper: function failAtOne(value: number,): number {
                started.push(value,);
                if (value === 1)
                  throw failure;
                return value;
              },
              options: {
                concurrency: 1,
                backpressure: Number.POSITIVE_INFINITY,
              },
            },),);

            expect(await rejectionOf(collecting,),).toBe(failure,);
            await yieldTurn();
            expect(started,).toEqual([
              0,
              1,
            ],);
            // Elements 0 and 1, plus element 2, which was read while the
            // mapper of element 1 ran.
            expect(source.counters.pulls,).toBe(3,);
          },
        },),
      ],
    },),

    describe({
      name: 'when the consumer leaves',
      children: [
        it({
          name: 'does not pull the source again for an element that was waiting for a mapper slot',
          fn: async () => {
            /**
             Pull counter of the source.
             */
            const count: PullCount = { pulls: 0, };
            const gate = createGate();
            const iterator = pMapIterable({
              iterable: finiteCountedSource(
                5,
                count,
              ),
              mapper: async function holdFirst(value: number,): Promise<number> {
                if (value === 0)
                  await gate.open;
                return value;
              },
              options: {
                concurrency: 1,
                backpressure: Number.POSITIVE_INFINITY,
              },
            },)[Symbol.asyncIterator]();
            /**
             Step pending on the first mapper.
             */
            const pendingStep = iterator.next();
            await yieldTurn();
            // Element 0 holds the only slot and element 1 was read ahead.
            expect(count.pulls,).toBe(2,);
            /**
             Stop request queued behind the pending step.
             */
            const stopping = iterator.return?.();
            gate.release();
            await pendingStep;
            await stopping;
            await yieldTurn();

            expect(count.pulls,).toBe(2,);
          },
        },),

        it({
          name: 'drops pending elements as soon as throw is called while a next call is pending',
          fn: async () => {
            const gate = createGate();
            /**
             Values the mapper was called with.
             */
            const started: number[] = [];
            /**
             First element, which stays pending until the test releases its
             gate.
             */
            async function heldElement(): Promise<number> {
              await gate.open;
              return 1;
            }
            /**
             Failure thrown into the stream.
             */
            const thrownIn = new Error('thrown in',);
            const iterator = pMapIterable({
              iterable: [heldElement(),],
              mapper: function record(value: number,): number {
                started.push(value,);
                return value;
              },
            },)[Symbol.asyncIterator]();
            /**
             Step pending on the first element.
             */
            const pendingStep = iterator.next();
            await yieldTurn();
            /**
             Throw request queued behind the pending step.
             */
            const throwRequest = nonNullishOrThrow(iterator.throw?.(thrownIn,),);
            const throwing = rejectionOf(throwRequest,);
            gate.release();
            await pendingStep;
            await yieldTurn();

            expect(started,).toEqual([],);
            expect(await throwing,).toBe(thrownIn,);
          },
        },),
      ],
    },),

    describe({
      name: 'after a failure with preserveOrder false',
      children: [
        it({
          name: 'drops an earlier element that is still being read',
          fn: async () => {
            const readGate = createGate();
            const failureGate = createGate();
            /**
             Failure the last element's mapper throws.
             */
            const failure = new Error('mapper failed',);
            /**
             Values the mapper was called with.
             */
            const started: number[] = [];
            /**
             First element, which stays pending until the test releases its
             gate.
             */
            async function heldElement(): Promise<number> {
              await readGate.open;
              return 0;
            }
            const iterator = pMapIterable({
              iterable: [
                heldElement(),
                1,
                2,
              ],
              mapper: async function failLast(value: number,): Promise<number> {
                started.push(value,);
                if (value === 2) {
                  await failureGate.open;
                  throw failure;
                }
                return value;
              },
              options: {
                concurrency: 3,
                preserveOrder: false,
              },
            },)[Symbol.asyncIterator]();
            /**
             Step taken before the failure settles.
             */
            const firstStep = await iterator.next();
            expect(firstStep,).toEqual({
              done: false,
              value: 1,
            },);
            failureGate.release();
            await yieldTurn();
            // The consumer is not waiting, so only the failure itself can have
            // told the stream to stop mapping.
            readGate.release();
            await yieldTurn();
            expect(started,).toEqual([
              1,
              2,
            ],);

            const lastStep = iterator.next();

            expect(await rejectionOf(lastStep,),).toBe(failure,);
          },
        },),
      ],
    },),
  ],
},);
