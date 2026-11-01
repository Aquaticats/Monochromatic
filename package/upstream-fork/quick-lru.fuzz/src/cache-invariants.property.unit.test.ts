/**
 Invariant and totality property tests over the fork alone: state
 relations that must hold after every generated operation, and a
 constructor that always lands on exactly one upstream failure mode.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assert,
  property,
} from 'fast-check';

import {
  createQuickLru,
  InvalidMaxAgeError,
  InvalidMaxSizeError,
} from '@monochromatic-dev/upstream-fork-quick-lru/ts';

import { installFakeClock, } from '@monochromatic-dev/upstream-fork-quick-lru/ts/test-support.ts';

import {
  junkInputArb,
  workloadArb,
} from './cache-arbitrary.ts';
import { fuzzRuns, } from './fuzz-budget.ts';
import {
  createForkAdapter,
} from './cache-adapters.ts';
import {
  runWorkload,
} from './cache-workload.ts';

//region Fixtures

/**
 Instant every workload run starts from; realistic enough that expiry
 timestamps stay away from upstream's falsy-stamp quirks.
 */
const CLOCK_START = 1_700_000_000_000;

//endregion Fixtures

await describe({
  name: 'cache workload invariants',
  concurrency: 1,
  children: [
    it({
      name: 'holds every state relation after each generated operation',
      fn: async () => {
        assert(
          property(
            workloadArb,
            function checkInvariants(workload,): void {
              using clock = installFakeClock({
                startMilliseconds: CLOCK_START,
              },);
              /**
               Fork cache trace over the generated workload.
               */
              const trace = runWorkload({
                adapter: createForkAdapter({
                  maxSize: workload.maxSize,
                  maxAge: workload.maxAge,
                  onEviction: workload.onEviction,
                },),
                workload,
                clock,
                startMilliseconds: CLOCK_START,
              },);

              for (const step of trace.steps) {
                /**
                 Snapshot taken after this step's operation.
                 */
                const { snapshot, } = step;
                expect(snapshot.descending.toReversed(),).toEqual(snapshot.ascending,);
                expect(snapshot.entries,).toEqual(snapshot.ascending,);
                expect(snapshot.keys.length,).toBe(snapshot.values.length,);
                expect(snapshot.iterator.length,).toBe(snapshot.ascending.length,);
                expect(snapshot.keys.length,).toBe(snapshot.ascending.length,);
                expect(Number(snapshot.size,),)
                  .toBeLessThanOrEqual(Number(snapshot.maxSize,),);
                expect(snapshot.text,)
                  .toBe(`QuickLRU(${snapshot.size}/${snapshot.maxSize})`,);
              }
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'lands every constructor input on a cache or exactly one configuration failure',
      fn: async () => {
        assert(
          property(
            junkInputArb,
            function checkTotality(input,): void {
              try {
                /**
                 Cache built from the junk input, when it builds.
                 */
                const lru = createQuickLru<string, string>(input as never,);
                expect(typeof lru.get,).toBe('function',);
              }
              catch (error) {
                expect(
                  (error instanceof InvalidMaxSizeError)
                    || (error instanceof InvalidMaxAgeError)
                    || ((Error.isError(error,)) && (error.name === 'TypeError')),
                ).toBe(true,);
              }
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
