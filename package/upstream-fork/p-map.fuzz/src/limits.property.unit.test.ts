/**
 Property tests proving the fork's iteration invariants: FIFO mapper starts,
 the concurrency bound at every start, outcome fidelity, result order,
 streaming order, the backpressure bound, and configuration-input totality.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  anything,
  assert,
  asyncProperty,
  boolean,
  property,
  record,
} from 'fast-check';

import {
  pMap,
  pMapIterable,
} from '@monochromatic-dev/upstream-fork-p-map/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import { workloadArb, } from './map-arbitrary.ts';
import { createForkAdapter, } from './map-adapters.ts';
import { runMapWorkload, } from './map-workload.ts';
import type {
  CallSpec,
  MapWorkload,
} from './workload.ts';
import { runIterableWorkload, } from './workload-iterable.ts';

//region Helpers

/**
 Calls whose mapper must resolve (no failures, no skips), used by the
 ordering properties where the expected result is trivially computable.
 
 @param workload - Generated workload to sanitize.
 
 @returns Workload whose calls all resolve.
 */
function resolveOnly(workload: MapWorkload,): MapWorkload {
  return {
    ...workload,
    calls: workload.calls.map(function toResolving(spec: CallSpec,): CallSpec {
      return {
        id: spec.id,
        behavior: 'resolve',
      };
    },),
  };
}

/**
 Expected result ids for a run whose calls all resolve, in input order.
 
 @param workload - Generated workload to project.
 
 @returns Input-order call ids.
 */
function expectedIds(workload: MapWorkload,): readonly string[] {
  return workload.calls
    .filter(function isResolving(spec: CallSpec,): boolean {
      return spec.behavior === 'resolve';
    },)
    .map(function toId(spec: CallSpec,): string {
      return spec.id;
    },);
}

/**
 Failure message one generated failing call produces.
 
 @param id - Id of the failing call.
 
 @returns Deterministic failure message for that call.
 */
function failureMessage(id: string,): string {
  return `failure ${id}`;
}

/**
 Ids of the generated calls whose mapper must fail.
 
 @param workload - Generated workload to project.
 
 @returns Failing call ids in input order.
 */
function failingIds(workload: MapWorkload,): readonly string[] {
  return workload.calls
    .filter(function isFailure(spec: CallSpec,): boolean {
      return (spec.behavior === 'reject')
        || (spec.behavior === 'throwSync');
    },)
    .map(function toId(spec: CallSpec,): string {
      return spec.id;
    },);
}

//endregion Helpers

await describe({
  name: 'fork invariants',
  children: [
    it({
      name: 'starts mapper calls in FIFO order within the bound and settles each as specified',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            const trace = await runMapWorkload({
              adapter: createForkAdapter(),
              workload,
            },);

            /**
             Call ids in scheduling order, truncated to the started prefix.
             */
            const scheduledIds = workload.calls
              .map(function toId(spec: CallSpec,): string {
                return spec.id;
              },)
              .slice(
              0,
              trace.startOrder.length,
              );
            expect(trace.startOrder,).toEqual(scheduledIds,);

            if (Number.isFinite(workload.concurrency))
              for (const active of trace.activeAtStart)
                expect(active,)
                  .toBeLessThanOrEqual(workload.concurrency,);

            /**
             Failure messages of the generated failing calls.
             */
            const failures = failingIds(workload,)
              .map(failureMessage,);

            if (trace.outcome.status === 'resolved') {
              expect(trace.outcome.values,).toEqual(expectedIds(workload,),);
              return;
            }

            /**
             Failing calls that actually started before the run settled.
             */
            const startedFailures = failures.filter(function started(message: string,): boolean {
              return trace.startOrder.some(function byId(id: string,): boolean {
                return failureMessage(id,) === message;
              },);
            },);
            expect(startedFailures.length,)
              .toBeGreaterThan(0,);

            if (workload.stopOnError)
              expect(startedFailures,)
                .toContain(trace.outcome.reasonMessage,);
            else
              expect([...trace.outcome.aggregated,]
                .toSorted(),).toEqual([...startedFailures,]
                  .toSorted(),);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'collects every mapper result in input order under any bound',
      fn: async () => {
        await assert(
          asyncProperty(
            workloadArb.map(resolveOnly,),
            async (workload,) => {
              const trace = await runMapWorkload({
                adapter: createForkAdapter(),
                workload,
              },);
              expect(trace.outcome.status,).toBe('resolved',);
              expect(trace.outcome.values,).toEqual(expectedIds(workload,),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'streams mapper results in input order and keeps the backlog within backpressure',
      fn: async () => {
        await assert(
          asyncProperty(
            workloadArb.map(resolveOnly,),
            async (workload,) => {
              const trace = await runIterableWorkload({
                adapter: createForkAdapter(),
                workload,
              },);
              expect(trace.yielded,).toEqual(expectedIds(workload,),);
              expect(trace.thrownMessage,).toBe('',);
              expect(trace.maxBacklog,)
                .toBeLessThanOrEqual(workload.backpressure,);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'surfaces one mapper failure from the async iterator',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            const trace = await runIterableWorkload({
              adapter: createForkAdapter(),
              workload,
            },);

            if (trace.thrownMessage !== '')
              expect(failingIds(workload,)
                .map(failureMessage,),)
                .toContain(trace.thrownMessage,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'configuration input totality: any options value schedules the run or one configuration error',
      fn: async () => {
        await assert(
          asyncProperty(anything(), async (options: unknown,) => {
            /**
             Outcome classification for this configuration input.
             */
            let outcome = 'resolved';
            try {
              await pMap({
                iterable: [1],
                mapper: function identity(value: number,): number {
                  return value;
                },
                options: options as never,
              },);
            }
            catch (error) {
              outcome = Error.isError(error,)
                ? error.name
                : 'ThrownValue';
            }
            expect([
              'resolved',
              'InvalidConcurrencyError',
              'TypeError',
            ],).toContain(outcome,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'streaming configuration input totality: any input, mapper, and options yield a stream or one configuration error',
      fn: async () => {
        assert(
          property(
            anything(),
            anything(),
            anything(),
            (input: unknown, mapper: unknown, options: unknown,) => {
              /**
               Outcome classification for this configuration input.
               */
              let outcome = 'stream';
              try {
                void pMapIterable({
                  iterable: input as never,
                  mapper: mapper as never,
                  options: options as never,
                },);
              }
              catch (error) {
                outcome = Error.isError(error,)
                  ? error.name
                  : 'ThrownValue';
              }
              expect([
                'stream',
                'InvalidInputError',
                'MapperRequiredError',
                'InvalidBackpressureError',
                'InvalidConcurrencyError',
                'TypeError',
              ],).toContain(outcome,);
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
