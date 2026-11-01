/**
 fast-check arbitraries generating limiter workloads: bounded concurrency,
 generated calls with mixed outcomes, release-order permutations, and
 interleaved in-flight actions. Used by every workload property in this
 package.
 
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
  Workload,
  WorkloadAction,
} from './workload.ts';

//region Atoms

/**
 Generated call behavior: resolve after release, reject after release, or
 throw synchronously on entry.
 */
const behaviorArb: Arbitrary<CallSpec['behavior']> = constantFrom(
  'resolve',
  'reject',
  'throwSync',
);

/**
 Generated in-flight action: nothing, a queue clear, or a bound change to a
 small value or `Number.POSITIVE_INFINITY`.
 */
const actionArb: Arbitrary<WorkloadAction> = oneof(
  constant({
    kind: 'none',
  } satisfies WorkloadAction),
  constant({
    kind: 'clearQueue',
  } satisfies WorkloadAction),
  record({
    concurrency: oneof(
      integer({
        min: 1,
        max: 6,
      }),
      constant(Number.POSITIVE_INFINITY),
    ),
  })
    .map(function toSetConcurrency(fields: {
      readonly concurrency: number;
    },): WorkloadAction {
      return {
        kind: 'setConcurrency',
        concurrency: fields.concurrency,
      };
    },),
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
 Generated workload: one to eight calls with mixed outcomes, an initial
 bound of one to six (or uncapped), a per-call release permutation, and one
 action slot per release step.
 */
export const workloadArb: Arbitrary<Workload> = array(
  behaviorArb,
  {
    minLength: 1,
    maxLength: 8,
  },
)
  .chain(function toWorkloadArbitrary(behaviors: readonly CallSpec['behavior'][],): Arbitrary<Workload> {
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
      rejectOnClear: boolean(),
      releaseOrder: shuffledSubarray(
        ids,
        {
          minLength: ids.length,
          maxLength: ids.length,
        },
      ),
      actions: array(
        actionArb,
        {
          minLength: ids.length,
          maxLength: ids.length,
        },
      ),
    })
      .map(function toWorkload(fields: {
        readonly concurrency: number;
        readonly rejectOnClear: boolean;
        readonly releaseOrder: readonly string[];
        readonly actions: readonly WorkloadAction[];
      },): Workload {
        return {
          concurrency: fields.concurrency,
          rejectOnClear: fields.rejectOnClear,
          calls,
          releaseOrder: fields.releaseOrder,
          actions: fields.actions,
        };
      },);
  },);

//endregion Workload
