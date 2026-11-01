/**
 Tests for abort-signal handling in the concurrent map.
 
 @module
 */

import { getEventListeners, } from 'node:events';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMap, } from '../dist/final/neutral/index.mjs';

import {
  createGate,
  rejectionOf,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: 'pMap abort signal',
  children: [
    it({
      name: 'rejects with the signal reason before any pull when the signal is already aborted',
      fn: async () => {
        /**
         AbortController whose signal aborts before the run starts.
         */
        const abortController = new AbortController();
        abortController.abort('stopped early',);
        /**
         Pull counter proving the source was never walked.
         */
        const state = {
          pulls: 0,
        };
        /**
         Source counting its `next` calls.
         */
        const countingIterable: Iterable<number> = {
          [Symbol.iterator]: function openCountingIterator(): Iterator<number> {
            return {
              next: function countingNext(): IteratorResult<number> {
                state.pulls += 1;
                return {
                  done: true,
                  value: undefined,
                };
              },
            };
          },
        };
        /**
         Mapper invocation counter proving the mapper never ran.
         */
        const mapperState = {
          invoked: false,
        };

        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await pMap({
            iterable: countingIterable,
            mapper: function markInvoked(value: number,): number {
              mapperState.invoked = true;
              return value;
            },
            options: {
              signal: abortController.signal,
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe('stopped early',);
        expect(state.pulls,).toBe(0,);
        expect(mapperState.invoked,).toBe(false,);
      },
    },),

    it({
      name: 'rejects with the signal reason and closes the source when aborted mid-run',
      fn: async () => {
        /**
         AbortController whose signal aborts while the mapper is gated.
         */
        const abortController = new AbortController();
        /**
         Gate holding the single mapper call until the abort fires.
         */
        const gate = createGate();
        /**
         Close telemetry for this source.
         */
        const closeState = {
          returnCalled: false,
        };
        /**
         Source whose shutdown is observable.
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
                closeState.returnCalled = true;
                return {
                  done: true,
                  value: undefined,
                };
              },
            };
          },
        };

        /**
         Run promise observed for its rejection.
         */
        const running = pMap({
          iterable: oneValueIterable,
          mapper: async function gatedMapper(value: number,): Promise<number> {
            await gate.open;
            return value;
          },
          options: {
            concurrency: 1,
            signal: abortController.signal,
          },
        },);
        await yieldTurn();
        abortController.abort();
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
        expect(caught,).toBeInstanceOf(DOMException,);
        expect((caught as DOMException).name,).toBe('AbortError',);
        await yieldTurn();
        expect(closeState.returnCalled,).toBe(true,);
        gate.release();
      },
    },),

    it({
      name: 'ignores an abort that arrives after the run resolved',
      fn: async () => {
        /**
         AbortController whose signal aborts only after completion.
         */
        const abortController = new AbortController();
        const results = await pMap({
          iterable: [5],
          mapper: function identity(value: number,): number {
            return value;
          },
          options: {
            signal: abortController.signal,
          },
        },);
        expect(results,).toEqual([5],);
        abortController.abort();
        await yieldTurn();
      },
    },),

    it({
      name: 'listens for abort while the run is going and stops listening once it resolved',
      fn: async () => {
        const abortController = new AbortController();
        const gate = createGate();
        const running = pMap({
          iterable: [5],
          mapper: async function held(value: number,): Promise<number> {
            await gate.open;
            return value;
          },
          options: {
            signal: abortController.signal,
          },
        },);
        await yieldTurn();
        /**
         Abort listeners the signal holds while the mapper is still going.
         */
        const listenersWhileRunning = getEventListeners(
          abortController.signal,
          'abort',
        ).length;
        gate.release();
        await running;
        /**
         Abort listeners the signal holds after the run resolved.
         */
        const listenersAfterResolve = getEventListeners(
          abortController.signal,
          'abort',
        ).length;

        expect(listenersWhileRunning,).toBe(1,);
        expect(listenersAfterResolve,).toBe(0,);
      },
    },),

    it({
      name: 'stops listening for abort once the run rejected',
      fn: async () => {
        const abortController = new AbortController();
        /**
         Failure the mapper throws.
         */
        const failure = new Error('mapper failed',);
        const caught = await rejectionOf(pMap({
          iterable: [5],
          mapper: function failing(): number {
            throw failure;
          },
          options: {
            signal: abortController.signal,
          },
        },),);

        expect(caught,).toBe(failure,);
        expect(getEventListeners(
          abortController.signal,
          'abort',
        ),).toHaveLength(0,);
      },
    },),
  ],
},);
