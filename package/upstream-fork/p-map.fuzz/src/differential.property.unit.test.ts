/**
 Differential property tests: the fork must produce the same observable
 traces as upstream `p-map` 7.1.1 on the same generated workload.
 
 The oracle is upstream itself: every scheduling, skip, failure, and
 validation behavior this fork claims is checked against the implementation
 it was derived from.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import pMapUpstream, {
  pMapIterable as pMapIterableUpstream,
  pMapKeyed as pMapKeyedUpstream,
  pMapSettled as pMapSettledUpstream,
  pMapSettledKeyed as pMapSettledKeyedUpstream,
} from 'p-map';
import {
  anything,
  assert,
  asyncProperty,
  property,
} from 'fast-check';

import {
  pMap,
  pMapIterable,
  pMapKeyed,
  pMapSettled,
  pMapSettledKeyed,
} from '@monochromatic-dev/upstream-fork-p-map/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import { differentialWorkloadArb, } from './map-arbitrary.ts';
import {
  createForkAdapter,
  createUpstreamAdapter,
} from './map-adapters.ts';
import {
  type MapSurface,
  runMapWorkload,
} from './map-workload.ts';
import { describeRejection, } from './workload.ts';
import { runIterableWorkload, } from './workload-iterable.ts';

//region Helpers

/**
 Renders one configuration attempt's outcome as a comparable message: the
 rejection's rendered text, or the empty string when the call settled.
 
 @param attempt - Call whose configuration outcome is compared.
 
 @returns Rendered rejection text, or `''` when the call settled.
 
 @example
 ```ts
 const message = await attemptMessage(async function bad(): Promise<void> {
   throw new Error('nope',);
 });
 ```
 */
async function attemptMessage(attempt: () => Promise<unknown>,): Promise<string> {
  try {
    await attempt();
  }
  catch (error) {
    return describeRejection(error,)
      .reasonMessage;
  }

  return '';
}

/**
 Renders one synchronous configuration attempt's outcome as a comparable
 message.
 
 @param attempt - Call whose configuration outcome is compared.
 
 @returns Rendered thrown text, or `''` when the call returned.
 
 @example
 ```ts
 const message = attemptMessageSync(function bad(): void {
   throw new Error('nope',);
 });
 ```
 */
function attemptMessageSync(attempt: () => unknown,): string {
  try {
    attempt();
  }
  catch (error) {
    return describeRejection(error,)
      .reasonMessage;
  }

  return '';
}

/**
 Builds the parity test that drives one generated workload through one map
 surface of both implementations and compares the traces.

 @param name - Test name.

 @param surface - Map surface driven: settled or keyed variants.

 @returns The parity test.

 @example
 ```ts
 const test = surfaceTraceTest({ name: 'settled traces match', surface: 'settled', },);
 ```
 */
function surfaceTraceTest(
  {
    name,
    surface,
  }: {
    /**
     Test name.
     */
    readonly name: string;
    /**
     Map surface driven.
     */
    readonly surface: MapSurface;
  },
): ReturnType<typeof it> {
  return it({
    name,
    fn: async () => {
      await assert(
        asyncProperty(differentialWorkloadArb, async (workload,) => {
          const forkTrace = await runMapWorkload({
            adapter: createForkAdapter(),
            workload,
            surface,
          },);
          const upstreamTrace = await runMapWorkload({
            adapter: createUpstreamAdapter(),
            workload,
            surface,
          },);
          expect(forkTrace,).toEqual(upstreamTrace,);
        },),
        {
          numRuns: fuzzRuns,
        },
      );
    },
  },);
}

/**
 Builds the parity test that feeds arbitrary values as input, mapper, and
 options to one asynchronous map surface of both implementations and compares
 the rendered rejection.

 @param name - Test name.

 @param fork - The fork's function under comparison.

 @param upstream - Upstream's function under comparison.

 @returns The parity test.

 @example
 ```ts
 const test = configMessageTest({ name, fork, upstream, },);
 ```
 */
function configMessageTest(
  {
    name,
    fork,
    upstream,
  }: {
    /**
     Test name.
     */
    readonly name: string;
    /**
     The fork's function under comparison, called with one run object.
     */
    readonly fork: (run: {
      readonly input: never;
      readonly mapper: never;
      readonly options: never;
    },) => Promise<unknown>;
    /**
     Upstream's function under comparison, called positionally.
     */
    readonly upstream: (
      input: never,
      mapper: never,
      options: never,
    ) => Promise<unknown>;
  },
): ReturnType<typeof it> {
  return it({
    name,
    fn: async () => {
      await assert(
        asyncProperty(
          anything(),
          anything(),
          anything(),
          async (input: unknown, mapper: unknown, options: unknown,) => {
            const forkMessage = await attemptMessage(async function forkAttempt(): Promise<unknown> {
              return await fork({
                input: input as never,
                mapper: mapper as never,
                options: options as never,
              },);
            },);
            const upstreamMessage = await attemptMessage(async function upstreamAttempt(): Promise<unknown> {
              return await upstream(
                input as never,
                mapper as never,
                options as never,
              );
            },);
            expect(forkMessage,).toBe(upstreamMessage,);
          },
        ),
        {
          numRuns: fuzzRuns,
        },
      );
    },
  },);
}

//endregion Helpers

await describe({
  name: 'upstream p-map parity',
  children: [
    it({
      name: 'concurrent-map workload traces match upstream p-map on every generated workload',
      fn: async () => {
        await assert(
          asyncProperty(differentialWorkloadArb, async (workload,) => {
            const forkTrace = await runMapWorkload({
              adapter: createForkAdapter(),
              workload,
            },);
            const upstreamTrace = await runMapWorkload({
              adapter: createUpstreamAdapter(),
              workload,
            },);
            expect(forkTrace,).toEqual(upstreamTrace,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'streaming-map workload traces match upstream p-map on every generated workload',
      fn: async () => {
        await assert(
          asyncProperty(differentialWorkloadArb, async (workload,) => {
            const forkTrace = await runIterableWorkload({
              adapter: createForkAdapter(),
              workload,
            },);
            const upstreamTrace = await runIterableWorkload({
              adapter: createUpstreamAdapter(),
              workload,
            },);
            expect(forkTrace,).toEqual(upstreamTrace,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'concurrent-map configuration messages match upstream p-map',
      fn: async () => {
        await assert(
          asyncProperty(
            anything(),
            anything(),
            anything(),
            async (input: unknown, mapper: unknown, options: unknown,) => {
              /**
               Rendered rejection text of the fork's run.
               */
              const forkMessage = await attemptMessage(async function forkAttempt(): Promise<unknown> {
                return await pMap({
                  iterable: input as never,
                  mapper: mapper as never,
                  options: options as never,
                },);
              },);
              /**
               Rendered rejection text of upstream's run.
               */
              const upstreamMessage = await attemptMessage(async function upstreamAttempt(): Promise<unknown> {
                return await pMapUpstream(
                  input as never,
                  mapper as never,
                  options as never,
                );
              },);
              expect(forkMessage,).toBe(upstreamMessage,);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'streaming-map configuration messages match upstream p-map',
      fn: async () => {
        assert(
          property(
            anything(),
            anything(),
            anything(),
            (input: unknown, mapper: unknown, options: unknown,) => {
              /**
               Rendered thrown text of the fork's call.
               */
              const forkMessage = attemptMessageSync(function forkAttempt(): unknown {
                return pMapIterable({
                  iterable: input as never,
                  mapper: mapper as never,
                  options: options as never,
                },);
              },);
              /**
               Rendered thrown text of upstream's call.
               */
              const upstreamMessage = attemptMessageSync(function upstreamAttempt(): unknown {
                return pMapIterableUpstream(
                  input as never,
                  mapper as never,
                  options as never,
                );
              },);
              expect(forkMessage,).toBe(upstreamMessage,);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    surfaceTraceTest({
      name: 'settled-map workload traces match upstream p-map on every generated workload',
      surface: 'settled',
    },),

    surfaceTraceTest({
      name: 'keyed-map workload traces match upstream p-map on every generated workload',
      surface: 'keyed',
    },),

    surfaceTraceTest({
      name: 'settled-keyed-map workload traces match upstream p-map on every generated workload',
      surface: 'settledKeyed',
    },),

    configMessageTest({
      name: 'settled-map configuration messages match upstream p-map',
      fork: function forkSettled(run: {
        readonly input: never;
        readonly mapper: never;
        readonly options: never;
      },): Promise<unknown> {
        return pMapSettled({
          iterable: run.input,
          mapper: run.mapper,
          options: run.options,
        },);
      },
      upstream: pMapSettledUpstream,
    },),

    configMessageTest({
      name: 'keyed-map configuration messages match upstream p-map',
      fork: pMapKeyed,
      upstream: pMapKeyedUpstream,
    },),

    configMessageTest({
      name: 'settled-keyed-map configuration messages match upstream p-map',
      fork: pMapSettledKeyed,
      upstream: pMapSettledKeyedUpstream,
    },),
  ],
},);
