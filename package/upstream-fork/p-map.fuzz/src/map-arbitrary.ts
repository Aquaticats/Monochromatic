/**
 fast-check arbitraries generating mapper workloads: bounded concurrency,
 generated calls with mixed outcomes, source shapes, and release-order
 permutations. Used by every workload property in this package.
 
 @module
 */

import {
  type Arbitrary,
  array,
  boolean,
  constant,
  constantFrom,
  integer,
  oneof,
  record,
  shuffledSubarray,
} from 'fast-check';

import type {
  CallSpec,
  MapWorkload,
} from './workload.ts';

//region Atoms

/**
 Generated mapper behavior: resolve after release, reject after release,
 throw synchronously on entry, or resolve with the adapter's skip sentinel.
 */
const behaviorArb: Arbitrary<CallSpec['behavior']> = constantFrom(
  'resolve',
  'reject',
  'throwSync',
  'skip',
);

/**
 Call id derived from a call's scheduling position.
 
 @param index - Scheduling position of the call.
 
 @returns Stable call identifier for that position.
 */
function callId(index: number,): string {
  return `c${String(index,)}`;
}

//endregion Atoms

//region Workload

/**
 Generated workload: one to eight calls with mixed outcomes, a bound of one
 to six (or uncapped), a source shape, and a per-call release permutation.
 */
export const workloadArb: Arbitrary<MapWorkload> = array(
  behaviorArb,
  {
    minLength: 1,
    maxLength: 8,
  },
)
  .chain(function toWorkloadArbitrary(behaviors: readonly CallSpec['behavior'][],): Arbitrary<MapWorkload> {
    /**
     Generated calls, identified by scheduling position.
     */
    const calls: readonly CallSpec[] = behaviors.map(function toSpec(
      behavior: CallSpec['behavior'],
      index: number,
    ): CallSpec {
      return {
        id: callId(index,),
        behavior,
      };
    },);
    /**
     Generated call ids, in scheduling order.
     */
    const ids = calls.map(function toId(spec: CallSpec,): string {
      return spec.id;
    },);

    return record({
      concurrency: oneof(
        integer({
          min: 1,
          max: 6,
        }),
        constant(Number.POSITIVE_INFINITY),
      ),
      stopOnError: boolean(),
      backpressure: integer({
        min: 1,
        max: 6,
      }),
      elementKind: constantFrom(
        'value',
        'promise',
      ),
      sourceKind: constantFrom(
        'sync',
        'async',
      ),
      releaseOrder: shuffledSubarray(
        ids,
        {
          minLength: ids.length,
          maxLength: ids.length,
        },
      ),
    })
      .map(function toWorkload(fields: {
        readonly concurrency: number;
        readonly stopOnError: boolean;
        readonly backpressure: number;
        readonly elementKind: 'value' | 'promise';
        readonly sourceKind: 'sync' | 'async';
        readonly releaseOrder: readonly string[];
      },): MapWorkload {
        return {
          concurrency: fields.concurrency,
          stopOnError: fields.stopOnError,
          /**
           Backpressure at least the concurrency bound, so generated
           streaming runs are always constructible; an uncapped bound forces
           the backpressure uncapped too, matching upstream's acceptance
           rule.
           */
          backpressure: (fields.concurrency === Number.POSITIVE_INFINITY)
            ? Number.POSITIVE_INFINITY
            : Math.max(
              fields.backpressure,
              fields.concurrency,
            ),
          elementKind: fields.elementKind,
          sourceKind: fields.sourceKind,
          calls,
          releaseOrder: fields.releaseOrder,
        };
      },);
  },);

//endregion Workload
