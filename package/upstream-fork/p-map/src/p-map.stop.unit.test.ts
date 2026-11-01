/**
 Tests for `pMapStop` in the concurrent map, ported from `p-map` 7.1.1's
 `pMapStop` suite: the result ends before the stopping element, the lowest
 stop wins, errors before the stop still reject, and the source is closed.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pMap,
  pMapSkip,
  pMapStop,
} from '../dist/final/neutral/index.mjs';

import {
  countingAsyncSource,
  countingSyncSource,
  rejectionOf,
} from './test-support.ts';

await describe({
  name: 'pMap with pMapStop',
  children: [
    it({
      name: 'resolves with the results before the stop',
      fn: async () => {
        const result = await pMap({
          iterable: [
            1,
            2,
            3,
            4,
            5,
            6,
          ],
          mapper: async function stopAtFour(value: number,): Promise<number | typeof pMapStop> {
            await wait(5,);
            return (value === 4)
              ? pMapStop
              : value;
          },
          options: { concurrency: 2, },
        },);
        expect(result,).toEqual([
          1,
          2,
          3,
        ],);
        expect(result.length,).toBe(Object.keys(result,).length,);
      },
    },),

    it({
      name: 'drops the later results that finished first, and waits for the earlier ones',
      fn: async () => {
        const result = await pMap({
          iterable: [
            1,
            2,
            3,
            4,
            5,
          ],
          mapper: async function stopAtThree(value: number,): Promise<number | typeof pMapStop> {
            await wait((value === 1)
              ? 30
              : 5,);
            return (value === 3)
              ? pMapStop
              : value;
          },
        },);
        expect(result,).toEqual([
          1,
          2,
        ],);
        expect(result.length,).toBe(Object.keys(result,).length,);
      },
    },),

    it({
      name: 'stops at the first element and at the last',
      fn: async () => {
        expect(await pMap({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: function stopAtOne(value: number,): number | typeof pMapStop {
            return (value === 1)
              ? pMapStop
              : value;
          },
        },),).toEqual([],);
        expect(await pMap({
          iterable: [
            1,
            2,
            3,
            pMapStop,
          ],
          mapper: function passThrough(value: number | typeof pMapStop,): number | typeof pMapStop {
            return value;
          },
        },),).toEqual([
          1,
          2,
          3,
        ],);
      },
    },),

    it({
      name: 'does not wait for the mappers of later elements',
      fn: async () => {
        /**
         Whether the slow mapper of the third element finished.
         */
        const slow = { isDone: false, };
        const result = await pMap({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: async function slowThird(value: number,): Promise<number | typeof pMapStop> {
            if (value === 3) {
              await wait(60,);
              slow.isDone = true;
              return value;
            }

            return (value === 2)
              ? pMapStop
              : value;
          },
        },);
        expect(result,).toEqual([1],);
        expect(slow.isDone,).toBe(false,);
      },
    },),

    it({
      name: 'keeps the lowest stop, whatever order the stops settle in',
      fn: async () => {
        const result = await pMap({
          iterable: [
            1,
            2,
            3,
            4,
            5,
          ],
          mapper: async function lateFirstStop(value: number,): Promise<number | typeof pMapStop> {
            if (value === 2) {
              await wait(30,);
              return pMapStop;
            }

            return (value === 4)
              ? pMapStop
              : value;
          },
        },);
        expect(result,).toEqual([1],);
      },
    },),

    it({
      name: 'rejects for an error before the stop and ignores one after it',
      fn: async () => {
        const before = await rejectionOf(pMap({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: async function failBeforeStop(value: number,): Promise<number | typeof pMapStop> {
            if (value === 1) {
              await wait(10,);
              throw new Error('before',);
            }

            return (value === 2)
              ? pMapStop
              : value;
          },
        },),);
        expect((before as Error).message,).toBe('before',);

        /**
         Gate holding the first element until the error has landed.
         */
        const firstGate = Promise.withResolvers<void>();
        /**
         Gate holding the error until the stop has landed.
         */
        const errorGate = Promise.withResolvers<void>();
        const after = await pMap({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: async function failAfterStop(value: number,): Promise<number | typeof pMapStop> {
            if (value === 1) {
              await firstGate.promise;
              return value;
            }

            if (value === 2) {
              // Let every element be pulled first.
              await wait(5,);
              errorGate.resolve();
              return pMapStop;
            }

            await errorGate.promise;
            firstGate.resolve();
            throw new Error('after',);
          },
        },);
        expect(after,).toEqual([1],);
      },
    },),

    it({
      name: 'collects only the errors before the stop with stopOnError false',
      fn: async () => {
        const error = await rejectionOf(pMap({
          iterable: [
            1,
            2,
            3,
            4,
          ],
          mapper: async function mixed(value: number,): Promise<number | typeof pMapStop> {
            if (value === 1)
              throw new Error('before',);
            if (value === 4)
              throw new Error('after',);
            if (value === 3) {
              // The later error settles first, before the stop is known.
              await wait(10,);
              return pMapStop;
            }

            return value;
          },
          options: { stopOnError: false, },
        },),);
        expect(error,).toBeInstanceOf(AggregateError,);
        expect((error as AggregateError).errors.map(function messageOf(inner: Error,): string {
          return inner.message;
        },),).toEqual(['before',],);
      },
    },),

    it({
      name: 'ends an endless synchronous source with the default concurrency and closes it once',
      fn: async () => {
        const source = countingSyncSource();
        const result = await pMap({
          iterable: source,
          mapper: function stopAtThree(value: number,): number | typeof pMapStop {
            return (value === 3)
              ? pMapStop
              : value;
          },
        },);
        expect(result,).toEqual([
          0,
          1,
          2,
        ],);
        /**
         Pulls at the moment the run settled.
         */
        const { pulls, } = source.counters;
        expect(pulls < 20,).toBe(true,);
        expect(source.counters.returnCalls,).toBe(1,);
        await wait(20,);
        expect(source.counters.pulls,).toBe(pulls,);
      },
    },),

    it({
      name: 'stops pulling from an asynchronous source and closes it',
      fn: async () => {
        const source = countingAsyncSource();
        const result = await pMap({
          iterable: source,
          mapper: async function stopAtTwo(value: number,): Promise<number | typeof pMapStop> {
            if (value !== 2)
              await wait((value === 0)
                ? 40
                : 15,);
            return (value === 2)
              ? pMapStop
              : value;
          },
          options: { concurrency: 3, },
        },);
        expect(result,).toEqual([
          0,
          1,
        ],);
        expect(source.counters.pulls,).toBe(3,);
        expect(source.counters.returnCalls,).toBe(1,);
        await wait(30,);
        expect(source.counters.pulls,).toBe(3,);
      },
    },),

    it({
      name: 'does not map a later element that resolves after the stop',
      fn: async () => {
        /**
         Values the mapper was called with, in order.
         */
        const mapped: number[] = [];
        /**
         Gates for the elements that resolve after the stop.
         */
        const later = Promise.withResolvers<number>();
        const first = Promise.withResolvers<number>();
        const result = await pMap({
          iterable: [
            first.promise,
            2,
            later.promise,
          ],
          mapper: async function stopAtTwo(value: number,): Promise<number | typeof pMapStop> {
            mapped.push(value,);
            if (value === 2) {
              // Let every element be pulled first.
              await wait(5,);
              // Resolve the later element only after the stop, and the first
              // one only after that, so the result cannot settle before the
              // later element is seen.
              setTimeout(
                function releaseLater(): void {
                  later.resolve(3,);
                  setTimeout(
                    function releaseFirst(): void {
                      first.resolve(1,);
                    },
                    0,
                  );
                },
                0,
              );
              return pMapStop;
            }

            return value;
          },
        },);
        expect(result,).toEqual([1],);
        expect(mapped,).toEqual([
          2,
          1,
        ],);
      },
    },),

    it({
      name: 'handles skips before the stop, a promised stop, an index-based stop, and a stop element passed to the mapper',
      fn: async () => {
        expect(await pMap({
          iterable: [
            1,
            pMapSkip,
            3,
            pMapStop,
            5,
          ],
          mapper: function passThrough(value: unknown,): unknown {
            return value;
          },
        },),).toEqual([
          1,
          3,
        ],);
        expect(await pMap({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: function promisedStop(value: number,): number | Promise<typeof pMapStop> {
            return (value === 2)
              ? Promise.resolve(pMapStop,)
              : value;
          },
        },),).toEqual([1],);
        expect(await pMap({
          iterable: [
            'a',
            'b',
            'c',
            'd',
          ],
          mapper: function stopAtIndex(
            value: string,
            index: number,
          ): string | typeof pMapStop {
            return (index === 2)
              ? pMapStop
              : `${value}${String(index,)}`;
          },
        },),).toEqual([
          'a0',
          'b1',
        ],);
        expect(await pMap({
          iterable: [
            Promise.resolve(pMapStop,),
            2,
          ],
          mapper: function markStopElement(value: unknown,): unknown {
            return (value === pMapStop)
              ? 'kept'
              : value;
          },
        },),).toEqual([
          'kept',
          2,
        ],);
      },
    },),

    it({
      name: 'does not close a source that is already done',
      fn: async () => {
        /**
         Number of `return()` calls the source received.
         */
        const closes = { count: 0, };
        /**
         Source over three values that counts its closes.
         */
        const source: Iterable<number> = {
          [Symbol.iterator]: function openSource(): Iterator<number> {
            const inner = [
              1,
              2,
              3,
            ][Symbol.iterator]();
            return {
              next: function nextInner(): IteratorResult<number> {
                return inner.next();
              },
              return: function returnInner(): IteratorResult<number> {
                closes.count += 1;
                return {
                  done: true,
                  value: undefined,
                };
              },
            };
          },
        };
        const result = await pMap({
          iterable: source,
          mapper: async function stopAtThree(value: number,): Promise<number | typeof pMapStop> {
            if (value === 3) {
              await wait(10,);
              return pMapStop;
            }

            return value;
          },
        },);
        expect(result,).toEqual([
          1,
          2,
        ],);
        expect(closes.count,).toBe(0,);
      },
    },),

    it({
      name: 'closes the source once when several elements stop, and not again after a rejection',
      fn: async () => {
        const several = countingSyncSource();
        const result = await pMap({
          iterable: several,
          mapper: async function stopTwice(value: number,): Promise<number | typeof pMapStop> {
            if (value === 1) {
              await wait(15,);
              return pMapStop;
            }

            return (value === 3)
              ? pMapStop
              : value;
          },
          options: { concurrency: 4, },
        },);
        expect(result,).toEqual([0],);
        expect(several.counters.returnCalls,).toBe(1,);

        const rejected = countingSyncSource();
        const error = await rejectionOf(pMap({
          iterable: rejected,
          mapper: async function failThenStop(value: number,): Promise<number | typeof pMapStop> {
            if (value === 0) {
              await wait(10,);
              throw new Error('before',);
            }

            await wait(25,);
            return pMapStop;
          },
          options: { concurrency: 2, },
        },),);
        expect((error as Error).message,).toBe('before',);
        await wait(40,);
        expect(rejected.counters.returnCalls,).toBe(1,);
      },
    },),
  ],
},);
