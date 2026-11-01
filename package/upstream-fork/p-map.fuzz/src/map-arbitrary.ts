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
 Generated mapper behavior including the stop sentinel, for the workloads
 that only the differential oracle compares.
 */
const behaviorWithStopArb: Arbitrary<CallSpec['behavior']> = constantFrom(
  'resolve',
  'reject',
  'throwSync',
  'skip',
  'stop',
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
 Builds a workload arbitrary over the given behaviors and ordering policy.

 @param behaviors - Arbitrary of one call's behavior.

 @param preserveOrder - Arbitrary of the streaming order flag.

 @returns Arbitrary of one to eight calls with mixed outcomes, a bound of one
 to six (or uncapped), a source shape, and a per-call release permutation.

 @example
 ```ts
 const arb = createWorkloadArb({ behaviors: behaviorArb, preserveOrder: constant(true,), },);
 ```
 */
function createWorkloadArb(
  {
    behaviors: behaviorChoice,
    preserveOrder: preserveOrderChoice,
  }: {
    /**
     Arbitrary of one call's behavior.
     */
    readonly behaviors: Arbitrary<CallSpec['behavior']>;
    /**
     Arbitrary of the streaming order flag.
     */
    readonly preserveOrder: Arbitrary<boolean>;
  },
): Arbitrary<MapWorkload> {
  return array(
    behaviorChoice,
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
        preserveOrder: preserveOrderChoice,
        inputShape: constantFrom(
          'object',
          'map',
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
          readonly preserveOrder: boolean;
          readonly inputShape: 'object' | 'map';
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
            preserveOrder: fields.preserveOrder,
            inputShape: fields.inputShape,
            calls,
            releaseOrder: fields.releaseOrder,
          };
        },);
    },);
}

/**
 Generated workload for the fork's own invariants: one to eight calls with
 mixed outcomes, a bound of one to six (or uncapped), a source shape, and a
 per-call release permutation, always in input order.
 */
export const workloadArb: Arbitrary<MapWorkload> = createWorkloadArb({
  behaviors: behaviorArb,
  preserveOrder: constant(true,),
});

/**
 Generated workload for the differential oracle: the same shapes plus stop
 calls and both streaming orders.
 */
export const differentialWorkloadArb: Arbitrary<MapWorkload> = createWorkloadArb({
  behaviors: behaviorWithStopArb,
  preserveOrder: boolean(),
});

//endregion Workload
