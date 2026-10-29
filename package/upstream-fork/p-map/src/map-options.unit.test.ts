/**
 Tests for mapper option destructuring and validation, exercised through the
 public `pMap` and `pMapIterable` entry points (options handling is
 package-internal).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidBackpressureError,
  InvalidConcurrencyError,
  MapperRequiredError,
  pMap,
  pMapIterable,
} from '../dist/final/neutral/index.mjs';

import {
  collect,
  createGate,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: 'map option handling',
  children: [
    describe({
      name: 'defaults',
      children: [
        it({
          name: 'maps every input uncapped when options are omitted',
          fn: async () => {
            const results = await pMap({
              iterable: [
                1,
                2,
                3,
                4,
              ],
              mapper: function double(value: number,): number {
                return value * 2;
              },
            },);
            expect(results,).toEqual([
              2,
              4,
              6,
              8,
            ],);
          },
        },),

        it({
          name: 'stops on the first mapper failure when stopOnError is omitted',
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
                mapper: function failingMapper(): never {
                  throw new Error('first failure',);
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
          name: 'defaults backpressure to concurrency for the streaming map',
          fn: async () => {
            /**
             Streamed values proving the run completes under the defaulted
             backpressure bound.
             */
            const streamed: number[] = [];
            for await (const mapped of pMapIterable({
              iterable: [1],
              mapper: function identity(value: number,): number {
                return value;
              },
              options: { concurrency: 1, },
            }))
              streamed.push(mapped,);
            expect(streamed,).toEqual([1],);
          },
        },),

        it({
          name: 'yields in input order for the streaming map when preserveOrder is omitted',
          fn: async () => {
            /**
             Gates letting the second element finish before the first.
             */
            const gates = [
              createGate(),
              createGate(),
            ];
            const collecting = collect(pMapIterable({
              iterable: [
                0,
                1,
              ],
              mapper: async function gated(value: number,): Promise<number> {
                await gates[value]?.open;
                return value;
              },
              options: { concurrency: 2, },
            },),);
            await yieldTurn();
            gates[1]?.release();
            await yieldTurn();
            gates[0]?.release();

            expect(await collecting,).toEqual([
              0,
              1,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'destructuring failures',
      children: [
        it({
          name: 'rejects like upstream when options is null instead of an object',
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
                options: null as never,
              },);
            }
            catch (error) {
              caught = error;
            }
            // The text comes from the engine's destructuring failure, so the
            // live parity check is the fuzz sidecar's differential oracle;
            // here the destructured field name must surface.
            expect(caught,).toBeInstanceOf(TypeError,);
            expect((caught as Error).message,).toContain('concurrency',);
          },
        },),

        it({
          name: 'throws synchronously like upstream when streaming options is null',
          fn: async () => {
            /**
             Failure caught at the call site, matching upstream
             `pMapIterable`'s synchronous destructuring.
             */
            let caught: unknown;
            try {
              pMapIterable({
                iterable: [1],
                mapper: function identity(value: number,): number {
                  return value;
                },
                options: null as never,
              },);
            }
            catch (error) {
              caught = error;
            }
            // Engine destructuring text again; the fuzz oracle compares the
            // exact message against upstream.
            expect(caught,).toBeInstanceOf(TypeError,);
            expect((caught as Error).message,).toContain('concurrency',);
          },
        },),
      ],
    },),

    describe({
      name: 'concurrency validation',
      children: [
        it({
          name: 'rejects a zero bound with upstream p-map message text',
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
            expect(caught,).toBeInstanceOf(InvalidConcurrencyError,);
            expect((caught as Error).message,).toBe('Expected `concurrency` to be an integer from 1 and up or `Infinity`, got `0` (number)',);
          },
        },),

        it({
          name: 'rejects a fractional bound',
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
                options: { concurrency: 1.5, },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(InvalidConcurrencyError,);
          },
        },),

        it({
          name: 'rejects an unsafe integer bound, keeping upstream isSafeInteger acceptance',
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
                options: {
                  concurrency: Number.MAX_SAFE_INTEGER + 1,
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(InvalidConcurrencyError,);
          },
        },),

        it({
          name: 'accepts Number.POSITIVE_INFINITY as the uncapped bound',
          fn: async () => {
            const results = await pMap({
              iterable: [7],
              mapper: function identity(value: number,): number {
                return value;
              },
              options: { concurrency: Number.POSITIVE_INFINITY, },
            },);
            expect(results,).toEqual([7],);
          },
        },),

        it({
          name: 'throws synchronously for the streaming map, like upstream',
          fn: async () => {
            /**
             Failure caught at the call site, matching upstream
             `pMapIterable`'s synchronous validation.
             */
            let caught: unknown;
            try {
              pMapIterable({
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
            expect(caught,).toBeInstanceOf(InvalidConcurrencyError,);
          },
        },),
      ],
    },),

    describe({
      name: 'backpressure validation',
      children: [
        it({
          name: 'rejects a backpressure bound below concurrency with upstream message text',
          fn: async () => {
            /**
             Failure caught at the call site, matching upstream
             `pMapIterable`'s synchronous validation.
             */
            let caught: unknown;
            try {
              pMapIterable({
                iterable: [1],
                mapper: function identity(value: number,): number {
                  return value;
                },
                options: {
                  concurrency: 4,
                  backpressure: 2,
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(InvalidBackpressureError,);
            expect((caught as Error).message,).toBe('Expected `backpressure` to be an integer from `concurrency` (4) and up or `Infinity`, got `2` (number)',);
          },
        },),

        it({
          name: 'accepts Number.POSITIVE_INFINITY as the uncapped backpressure bound',
          fn: async () => {
            /**
             Streamed values proving the run completes under the uncapped
             backpressure bound.
             */
            const streamed: number[] = [];
            for await (const mapped of pMapIterable({
              iterable: [3],
              mapper: function identity(value: number,): number {
                return value;
              },
              options: {
                concurrency: 1,
                backpressure: Number.POSITIVE_INFINITY,
              },
            }))
              streamed.push(mapped,);
            expect(streamed,).toEqual([3],);
          },
        },),
      ],
    },),

    describe({
      name: 'signal resolution',
      children: [
        it({
          name: 'forwards a present signal to the run so an abort rejects it',
          fn: async () => {
            /**
             AbortController whose signal aborts before the run starts.
             */
            const abortController = new AbortController();
            abortController.abort('stopped early',);
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
                options: {
                  signal: abortController.signal,
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBe('stopped early',);
          },
        },),
      ],
    },),

    describe({
      name: 'check ordering',
      children: [
        it({
          name: 'reports the input failure before the mapper failure, like upstream',
          fn: async () => {
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await pMap({
                iterable: 5 as never,
                mapper: undefined as never,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('Expected `input` to be either an `Iterable` or `AsyncIterable`, got (number)',);
          },
        },),

        it({
          name: 'reports the mapper failure before the concurrency failure, like upstream',
          fn: async () => {
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await pMap({
                iterable: [1],
                mapper: undefined as never,
                options: { concurrency: 0, },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(MapperRequiredError,);
          },
        },),
      ],
    },),
  ],
},);
