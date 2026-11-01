/**
 Tests for `pMapSettled`, ported from `p-map` 7.1.1's `pMapSettled` suite: a
 settled result per element in input order, no rejection for a mapper or
 element failure, skips and stops, and the run-level failures that still
 reject.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidConcurrencyError,
  InvalidInputError,
  MapperRequiredError,
  pMapSettled,
  pMapSkip,
  pMapStop,
} from '../dist/final/neutral/index.mjs';

import { rejectionOf, } from './test-support.ts';

await describe({
  name: pMapSettled.name,
  children: [
    it({
      name: 'settles every element in input order, like Promise.allSettled',
      fn: async () => {
        /**
         Failure the third element's mapper call produces.
         */
        const failure = new Error('three',);
        const settled = await pMapSettled({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: async function failOnThree(value: number,): Promise<number> {
            await wait((value === 1)
              ? 10
              : 0,);
            if (value === 3)
              throw failure;
            return value * 2;
          },
        },);
        expect(settled,).toEqual([
          {
            status: 'fulfilled',
            value: 2,
          },
          {
            status: 'fulfilled',
            value: 4,
          },
          {
            status: 'rejected',
            reason: failure,
          },
        ],);
        expect(settled,).toEqual(await Promise.allSettled([
          Promise.resolve(2,),
          Promise.resolve(4,),
          Promise.reject(failure,),
        ],),);
      },
    },),

    it({
      name: 'settles an element that rejects without calling the mapper for it',
      fn: async () => {
        /**
         Values the mapper was called with.
         */
        const mapped: number[] = [];
        /**
         Reason the second element rejects with.
         */
        const reason = new Error('element failed',);
        const settled = await pMapSettled({
          iterable: [
            1,
            Promise.reject<number>(reason,),
            3,
          ],
          mapper: function record(value: number,): number {
            mapped.push(value,);
            return value;
          },
        },);
        expect(settled,).toEqual([
          {
            status: 'fulfilled',
            value: 1,
          },
          {
            status: 'rejected',
            reason,
          },
          {
            status: 'fulfilled',
            value: 3,
          },
        ],);
        expect(mapped,).toEqual([
          1,
          3,
        ],);
      },
    },),

    it({
      name: 'respects the concurrency bound',
      fn: async () => {
        /**
         Overlap counters shared by every mapper call.
         */
        const counts = {
          running: 0,
          maxRunning: 0,
        };
        await pMapSettled({
          iterable: [
            1,
            2,
            3,
            4,
            5,
            6,
          ],
          mapper: async function track(value: number,): Promise<number> {
            counts.running += 1;
            counts.maxRunning = Math.max(
              counts.maxRunning,
              counts.running,
            );
            await wait(5,);
            counts.running -= 1;
            return value;
          },
          options: { concurrency: 2, },
        },);
        expect(counts.maxRunning,).toBe(2,);
      },
    },),

    it({
      name: 'keeps an undefined mapper error and a falsy mapper result',
      fn: async () => {
        const settled = await pMapSettled({
          iterable: [
            1,
            2,
          ],
          mapper: function throwUndefined(value: number,): number {
            if (value === 1)
              // oxlint-disable-next-line no-throw-literal, typescript/only-throw-error -- the case under test is a mapper that throws `undefined`
              throw undefined;
            return 0;
          },
        },);
        expect(settled,).toEqual([
          {
            status: 'rejected',
            reason: undefined,
          },
          {
            status: 'fulfilled',
            value: 0,
          },
        ],);
      },
    },),

    it({
      name: 'leaves out pMapSkip results and wraps a result that looks settled',
      fn: async () => {
        const settled = await pMapSettled({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: function skipTwo(value: number,): unknown {
            if (value === 2)
              return pMapSkip;
            return {
              status: 'rejected',
              reason: value,
            };
          },
        },);
        expect(settled,).toEqual([
          {
            status: 'fulfilled',
            value: {
              status: 'rejected',
              reason: 1,
            },
          },
          {
            status: 'fulfilled',
            value: {
              status: 'rejected',
              reason: 3,
            },
          },
        ],);
      },
    },),

    it({
      name: 'stops at pMapStop, keeping earlier failures as results and ignoring later ones',
      fn: async () => {
        const settled = await pMapSettled({
          iterable: [
            1,
            2,
            3,
            4,
          ],
          mapper: async function mixed(value: number,): Promise<number | typeof pMapStop> {
            if (value === 1)
              throw new Error('before',);
            if (value === 3) {
              await wait(10,);
              return pMapStop;
            }

            if (value === 4)
              throw new Error('after',);
            return value;
          },
        },);
        expect(settled.map(function statusOf(result: PromiseSettledResult<unknown>,): string {
          return result.status;
        },),).toEqual([
          'rejected',
          'fulfilled',
        ],);
      },
    },),

    it({
      name: 'turns a thrown pMapSkip or pMapStop into a rejected result, not a skip or a stop',
      fn: async () => {
        const settled = await pMapSettled({
          iterable: [
            1,
            2,
          ],
          mapper: function throwSentinel(value: number,): number {
            // oxlint-disable-next-line typescript/only-throw-error -- the case under test is a mapper that throws a sentinel
            throw (value === 1)
              ? pMapSkip
              : pMapStop;
          },
        },);
        expect(settled,).toEqual([
          {
            status: 'rejected',
            reason: pMapSkip,
          },
          {
            status: 'rejected',
            reason: pMapStop,
          },
        ],);
      },
    },),

    it({
      name: 'maps an asynchronous iterable and an empty input',
      fn: async () => {
        /**
         Asynchronous source of two values.
         */
        async function* twoValues(): AsyncGenerator<number> {
          yield 1;
          yield 2;
          await Promise.resolve();
        }
        expect(await pMapSettled({
          iterable: twoValues(),
          mapper: function identity(value: number,): number {
            return value;
          },
        },),).toEqual([
          {
            status: 'fulfilled',
            value: 1,
          },
          {
            status: 'fulfilled',
            value: 2,
          },
        ],);
        expect(await pMapSettled({
          iterable: [],
          mapper: function identity(value: number,): number {
            return value;
          },
        },),).toEqual([],);
      },
    },),

    it({
      name: 'still rejects when the source throws or the signal aborts',
      fn: async () => {
        /**
         Failure the source throws.
         */
        const failure = new Error('source failed',);
        /**
         Source that throws on its second pull.
         */
        function* failingSource(): Generator<number> {
          yield 1;
          throw failure;
        }
        const sourceError = await rejectionOf(pMapSettled({
          iterable: failingSource(),
          mapper: function identity(value: number,): number {
            return value;
          },
        },),);
        expect(sourceError,).toBe(failure,);

        const controller = new AbortController();
        const aborted = pMapSettled({
          iterable: [
            1,
            2,
          ],
          mapper: async function slow(value: number,): Promise<number> {
            await wait(30,);
            return value;
          },
          options: { signal: controller.signal, },
        },);
        controller.abort(new Error('aborted',),);
        expect(((await rejectionOf(aborted,)) as Error).message,).toBe('aborted',);
      },
    },),

    it({
      name: 'validates the arguments and ignores stopOnError false',
      fn: async () => {
        /**
         Rejections of the three invalid runs.
         */
        const invalidInput = await rejectionOf(pMapSettled({
          iterable: 5 as never,
          mapper: function identity(value: number,): number {
            return value;
          },
        },),);
        const missingMapper = await rejectionOf(pMapSettled({
          iterable: [1],
          mapper: undefined as never,
        },),);
        const invalidConcurrency = await rejectionOf(pMapSettled({
          iterable: [1],
          mapper: function identity(value: number,): number {
            return value;
          },
          options: { concurrency: 0, },
        },),);
        expect(invalidInput,).toBeInstanceOf(InvalidInputError,);
        expect(missingMapper,).toBeInstanceOf(MapperRequiredError,);
        expect(invalidConcurrency,).toBeInstanceOf(InvalidConcurrencyError,);

        const settled = await pMapSettled({
          iterable: [1],
          mapper: function failing(): number {
            throw new Error('mapper failed',);
          },
          // The type has no `stopOnError`, but a JavaScript caller can pass it.
          options: { stopOnError: false, } as never,
        },);
        expect(settled.map(function statusOf(result: PromiseSettledResult<unknown>,): string {
          return result.status;
        },),).toEqual(['rejected',],);
      },
    },),
  ],
},);
