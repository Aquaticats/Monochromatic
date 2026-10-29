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
} from '@monochromatic-dev/upstream-fork-p-map/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import { workloadArb, } from './map-arbitrary.ts';
import {
  createForkAdapter,
  createUpstreamAdapter,
} from './map-adapters.ts';
import { runMapWorkload, } from './map-workload.ts';
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

//endregion Helpers

await describe({
  name: 'upstream p-map parity',
  children: [
    it({
      name: 'concurrent-map workload traces match upstream p-map on every generated workload',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
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
          asyncProperty(workloadArb, async (workload,) => {
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
  ],
},);
