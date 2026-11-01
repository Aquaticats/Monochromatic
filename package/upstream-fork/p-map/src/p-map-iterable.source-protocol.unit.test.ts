/**
 Tests for how the streaming map treats a source that settles its `next()`
 calls out of order or answers some of them after it reported `done`: the
 `done` is final, so what the source produces after it is dropped, and a pull
 that never answers must not hold the stream open.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMapIterable, } from '../dist/final/neutral/index.mjs';

import {
  collect,
  createGate,
  pullAt,
  scriptedAsyncSource,
  yieldDone,
  yieldElement,
  yieldTurn,
} from './test-support.ts';

//region Fixtures

/**
 Maps an element to itself.

 @param value - Element to hand back.

 @returns The element.

 @example
 ```ts
 identity('a',); // => 'a'
 ```
 */
function identity(value: string,): string {
  return value;
}

/**
 Options of the unordered runs: three at a time with no read-ahead limit.
 */
const unorderedOptions = {
  concurrency: 3,
  backpressure: Number.POSITIVE_INFINITY,
  preserveOrder: false,
} as const;

//endregion Fixtures

await describe({
  name: `${pMapIterable.name} source protocol`,
  children: [
    describe({
      name: 'with preserveOrder false',
      children: [
        it({
          name: 'drops an element a pull produces after the source reported done',
          fn: async () => {
            const source = scriptedAsyncSource<string>();
            const heldElement = createGate();
            /**
             Element that stays pending until the test releases its gate.
             */
            async function heldValue(): Promise<string> {
              await heldElement.open;
              return 'zero';
            }
            const collecting = collect(pMapIterable({
              iterable: source.iterable,
              mapper: identity,
              options: unorderedOptions,
            },),);
            await yieldTurn();
            yieldElement({
              pull: pullAt({
                source,
                index: 0,
              },),
              value: heldValue(),
            },);
            await yieldTurn();
            yieldElement({
              pull: pullAt({
                source,
                index: 1,
              },),
              value: 'one',
            },);
            await yieldTurn();
            // Pulls 2 and 3 are in flight while the source is still open.
            expect(source.pulls,).toHaveLength(4,);
            yieldDone(pullAt({
              source,
              index: 2,
            },),);
            await yieldTurn();
            yieldElement({
              pull: pullAt({
                source,
                index: 3,
              },),
              value: 'late',
            },);
            await yieldTurn();
            heldElement.release();

            expect(await collecting,).toEqual([
              'one',
              'zero',
            ],);
          },
        },),

        it({
          name: 'ignores a failure of a pull made after the source reported done',
          fn: async () => {
            const source = scriptedAsyncSource<string>();
            const heldElement = createGate();
            /**
             Element that stays pending until the test releases its gate.
             */
            async function heldValue(): Promise<string> {
              await heldElement.open;
              return 'zero';
            }
            const collecting = collect(pMapIterable({
              iterable: source.iterable,
              mapper: identity,
              options: unorderedOptions,
            },),);
            await yieldTurn();
            yieldElement({
              pull: pullAt({
                source,
                index: 0,
              },),
              value: heldValue(),
            },);
            await yieldTurn();
            yieldElement({
              pull: pullAt({
                source,
                index: 1,
              },),
              value: 'one',
            },);
            await yieldTurn();
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
            heldElement.release();

            expect(await collecting,).toEqual([
              'one',
              'zero',
            ],);
          },
        },),

        it({
          name: 'ends the stream without waiting for a pull the source never answers after done',
          fn: async () => {
            const source = scriptedAsyncSource<string>();
            const heldElement = createGate();
            /**
             Element that stays pending until the test releases its gate.
             */
            async function heldValue(): Promise<string> {
              await heldElement.open;
              return 'zero';
            }
            /**
             Whether the stream ended.
             */
            const ended = { isEnded: false, };
            /**
             Drains the stream and records that it ended.
             */
            async function drain(): Promise<string[]> {
              const values = await collect(pMapIterable({
                iterable: source.iterable,
                mapper: identity,
                options: unorderedOptions,
              },),);
              ended.isEnded = true;
              return values;
            }
            const draining = drain();
            await yieldTurn();
            yieldElement({
              pull: pullAt({
                source,
                index: 0,
              },),
              value: heldValue(),
            },);
            await yieldTurn();
            yieldElement({
              pull: pullAt({
                source,
                index: 1,
              },),
              value: 'one',
            },);
            await yieldTurn();
            yieldDone(pullAt({
              source,
              index: 2,
            },),);
            await yieldTurn();
            heldElement.release();
            await yieldTurn();

            // Pull 3 never answers, like an exhausted queue.
            expect(ended.isEnded,).toBe(true,);
            expect(await draining,).toEqual([
              'one',
              'zero',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'with preserveOrder true',
      children: [
        it({
          name: 'ends the stream at the done and drops an element a later pull produced',
          fn: async () => {
            const source = scriptedAsyncSource<string>();
            const collecting = collect(pMapIterable({
              iterable: source.iterable,
              mapper: identity,
              options: {
                concurrency: 2,
                backpressure: Number.POSITIVE_INFINITY,
              },
            },),);
            await yieldTurn();
            yieldElement({
              pull: pullAt({
                source,
                index: 0,
              },),
              value: 'a',
            },);
            await yieldTurn();
            // Pulls 1 and 2 are in flight while the source is still open.
            expect(source.pulls,).toHaveLength(3,);
            yieldDone(pullAt({
              source,
              index: 1,
            },),);
            yieldElement({
              pull: pullAt({
                source,
                index: 2,
              },),
              value: 'late',
            },);

            expect(await collecting,).toEqual(['a'],);
          },
        },),
      ],
    },),
  ],
},);
