/**
 Tests that a mapper error keeps the caller in its async stack trace, ported
 from `p-map` 7.1.1: the runners are linked to the run's promise, which V8
 follows when it builds the trace.
 
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
  pMapIterable,
  pMapKeyed,
  pMapSettled,
} from '../dist/final/neutral/index.mjs';

import {
  collect,
  rejectionOf,
} from './test-support.ts';

/**
 Mapper that fails after a turn, so its error is created in a later task.

 @returns Never; always rejects.

 @throws Error with the message `foo`.

 @example
 ```ts
 await failLater();
 ```
 */
async function failLater(): Promise<never> {
  await wait(5,);
  throw new Error('foo',);
}

/**
 Calls the concurrent map from a named async function, so the trace can name
 it.

 @returns Nothing; the run rejects.
 */
async function callPMap(): Promise<void> {
  await pMap({
    iterable: [
      1,
      2,
    ],
    mapper: failLater,
    options: { concurrency: 1, },
  },);
}

/**
 Calls the concurrent map for an element mapped by a later runner.

 @returns Nothing; the run rejects.
 */
async function callPMapLater(): Promise<void> {
  await pMap({
    iterable: [
      1,
      2,
      3,
    ],
    mapper: async function failOnThree(value: number,): Promise<void> {
      await wait(5,);
      if (value === 3)
        throw new Error('foo',);
    },
    options: { concurrency: 1, },
  },);
}

/**
 Calls the settled map from a named async function.

 @returns The first settled result.
 */
async function callPMapSettled(): Promise<PromiseSettledResult<never>> {
  const [result] = await pMapSettled({
    iterable: [1],
    mapper: failLater,
  },);
  return result as PromiseSettledResult<never>;
}

/**
 Calls the keyed map from a named async function.

 @returns Nothing; the run rejects.
 */
async function callPMapKeyed(): Promise<void> {
  await pMapKeyed({
    input: { a: 1, },
    mapper: failLater,
  },);
}

/**
 Consumes the stream from a named async function.

 @returns Nothing; the iteration throws.
 */
async function consume(): Promise<void> {
  await collect(pMapIterable({
    iterable: [
      1,
      2,
    ],
    mapper: failLater,
    options: { concurrency: 1, },
  },),);
}

await describe({
  name: 'async stack traces of mapper errors',
  children: [
    it({
      name: 'include the caller of the concurrent map',
      fn: async () => {
        const error = await rejectionOf(callPMap(),);
        expect((error as Error).stack,).toContain('at async callPMap ',);
      },
    },),

    it({
      name: 'include the caller for an element mapped after the first ones',
      fn: async () => {
        const error = await rejectionOf(callPMapLater(),);
        expect((error as Error).stack,).toContain('at async callPMapLater ',);
      },
    },),

    it({
      name: 'include the caller in a settled rejection reason',
      fn: async () => {
        const result = await callPMapSettled();
        expect(result.status,).toBe('rejected',);
        expect(result.status === 'rejected'
          ? (result.reason as Error).stack
          : '',).toContain('at async callPMapSettled ',);
      },
    },),

    it({
      name: 'include the caller of the keyed map',
      fn: async () => {
        const error = await rejectionOf(callPMapKeyed(),);
        expect((error as Error).stack,).toContain('at async callPMapKeyed ',);
      },
    },),

    it({
      name: 'include the consumer of the stream',
      fn: async () => {
        const error = await rejectionOf(consume(),);
        expect((error as Error).stack,).toContain('at async consume ',);
      },
    },),
  ],
},);
