/**
 Property tests proving the fork's scheduling invariants: FIFO starts, the
 concurrency bound at every start, outcome fidelity, counter consistency,
 `map` order, and constructor-input totality.
 
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
  array,
  asyncProperty,
  integer,
  property,
} from 'fast-check';

import {
  InvalidConcurrencyError,
  InvalidRejectOnClearError,
  limitFunction,
  pLimit,
} from '@monochromatic-dev/upstream-fork-p-limit/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import { workloadArb, } from './limit-arbitrary.ts';
import { createForkAdapter, } from './limiter-adapters.ts';
import {
  type CallSettlement,
  type WorkloadTrace,
  runWorkload,
} from './workload.ts';

//region Helpers

/**
 Finds one call's recorded settlement.
 
 @param trace - Trace to search.
 
 @param id - Call id to look up.
 
 @returns The call's settlement, or a synthetic unsettled record when the
 trace lost it.
 */
function settlementFor(trace: WorkloadTrace, id: string,): CallSettlement {
  return trace.settlements.find(function byId(entry: CallSettlement,): boolean {
    return entry.id === id;
  },)
    ?? {
      id,
      status: 'unsettled',
      reasonName: 'missing',
      reasonMessage: 'missing',
    };
}

//endregion Helpers

await describe({
  name: 'fork invariants',
  children: [
    it({
      name: 'starts calls in FIFO order within the bound and settles each as specified',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            const trace = await runWorkload({
              adapter: createForkAdapter({
                concurrency: workload.concurrency,
                rejectOnClear: workload.rejectOnClear,
              },),
              workload,
            },);

            /**
             Call ids in scheduling order, truncated to the started prefix.
             */
            const scheduledIds = workload.calls
              .map(function toId(spec,) {
                return spec.id;
              },)
              .slice(
              0,
              trace.startOrder.length,
              );
            expect(trace.startOrder,).toEqual(scheduledIds,);

            for (const [index, active,] of trace.activeAtStart.entries())
              expect(active,)
                .toBeLessThanOrEqual(trace.boundAtStart[index]
                  ?? 0,);

            for (const spec of workload.calls) {
              /**
               Recorded settlement for this generated call.
               */
              const settlement = settlementFor(
                trace,
                spec.id,
              );
              /**
               Whether this call's function ever ran.
               */
              const started = trace.startOrder.includes(spec.id,);

              if (!started) {
                if (workload.rejectOnClear) {
                  expect(settlement.status,).toBe('rejected',);
                  expect(settlement.reasonName,).toBe('AbortError',);
                }
                else
                  expect(settlement.status,).toBe('unsettled',);
                continue;
              }

              if (spec.behavior === 'resolve') {
                expect(settlement.status,).toBe('resolved',);
                continue;
              }

              expect(settlement.status,).toBe('rejected',);
              expect(settlement.reasonMessage,).toBe(`failure ${spec.id}`,);
            }

            expect(trace.finalActiveCount,).toBe(0,);
            expect(trace.finalPendingCount,).toBe(0,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'map collects mapper results in input order under any bound',
      fn: async () => {
        await assert(
          asyncProperty(
            array(
              integer(),
              {
                maxLength: 20,
              },
            ),
            integer({
              min: 1,
              max: 6,
            }),
            async (values: readonly number[], concurrency: number,) => {
              const limit = pLimit({ concurrency, },);
              const results = await limit.map({
                iterable: values,
                mapper: async function scaledMapper(
                  value: number,
                  index: number,
                ): Promise<number> {
                  return (value * 10) + index;
                },
              },);
              expect(results,).toEqual(values.map(function expectedValue(
                value: number,
                index: number,
              ): number {
                return (value * 10) + index;
              },),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'limitFunction forwards args and resolves every call under any bound',
      fn: async () => {
        await assert(
          asyncProperty(
            array(
              integer(),
              {
                maxLength: 12,
              },
            ),
            integer({
              min: 1,
              max: 4,
            }),
            async (values: readonly number[], concurrency: number,) => {
              const limited = limitFunction({
                fn: function double(value: number,): number {
                  return value * 2;
                },
                options: { concurrency, },
              },);
              const results = await Promise.all(values.map(function callLimited(value: number,): Promise<number> {
                return limited({ args: [value], },);
              },),);
              expect(results,).toEqual(values.map(function doubled(value: number,): number {
                return value * 2;
              },),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'constructor input totality: any value yields a limiter or one configuration error',
      fn: async () => {
        assert(
          property(anything(), (value,) => {
            /**
             Outcome classification for this constructor input.
             */
            let outcome = 'limiter';
            try {
              pLimit(value as never,);
            }
            catch (error) {
              if (error instanceof InvalidConcurrencyError)
                outcome = 'invalidConcurrency';
              else if (error instanceof InvalidRejectOnClearError)
                outcome = 'invalidRejectOnClear';
              else if (error instanceof TypeError)
                outcome = 'typeError';
              else
                outcome = 'unexpected';
            }
            expect([
              'limiter',
              'invalidConcurrency',
              'invalidRejectOnClear',
              'typeError',
            ],).toContain(outcome,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),
  ],
},);
