/**
 fast-check arbitraries generating memoization workloads: per-key behaviors,
 cache and predicate modes, and batched synchronous call launches. Used by
 every workload property in this package.
 
 @module
 */

import {
  type Arbitrary,
  array,
  boolean,
  constantFrom,
  record,
} from 'fast-check';

import type {
  KeyBehavior,
  KeyMode,
  ShouldCacheMode,
  Workload,
  WorkloadKey,
} from './workload.ts';

//region Atoms

/**
 Generated per-key wrapped-function behavior.
 */
const behaviorArb: Arbitrary<KeyBehavior> = constantFrom(
  'resolve',
  'reject',
  'throwSync',
);

/**
 Generated cache key derivation mode.
 */
const keyModeArb: Arbitrary<KeyMode> = constantFrom(
  'firstArgument',
  'json',
);

/**
 Generated write predicate mode.
 */
const shouldCacheModeArb: Arbitrary<ShouldCacheMode> = constantFrom(
  'always',
  'never',
  'throw',
);

/**
 Generated cache key domain.
 */
const workloadKeyArb: Arbitrary<WorkloadKey> = constantFrom(
  'a',
  'b',
  'c',
);

//endregion Atoms

//region Workload

/**
 Generated scenario fields without the per-batch clear flags.
 */
const workloadFieldsArb = record({
  behaviorByKey: record({
    a: behaviorArb,
    b: behaviorArb,
    c: behaviorArb,
  }),
  keyMode: keyModeArb,
  shouldCacheMode: shouldCacheModeArb,
  cacheEnabled: boolean(),
  batches: array(
    array(
      workloadKeyArb,
      {
        minLength: 1,
        maxLength: 3,
      },
    ),
    {
      minLength: 1,
      maxLength: 4,
    },
  ),
});

/**
 Generated workload: per-key behaviors, key and predicate modes, a caching
 toggle, one to four synchronous launch batches of one to three calls each,
 and a clear flag per batch.
 */
export const workloadArb: Arbitrary<Workload> = workloadFieldsArb.chain(
  function withClearFlags(fields: {
    readonly behaviorByKey: Workload['behaviorByKey'];
    readonly keyMode: KeyMode;
    readonly shouldCacheMode: ShouldCacheMode;
    readonly cacheEnabled: boolean;
    readonly batches: readonly (readonly WorkloadKey[])[];
  },): Arbitrary<Workload> {
    /**
     Generated launch batches, destructured for short chains.
     */
    const { batches, } = fields;
    /**
     One clear flag per generated batch.
     */
    const clearFlagsArb = array(
      boolean(),
      {
        minLength: batches.length,
        maxLength: batches.length,
      },
    );

    return clearFlagsArb.map(
      function toWorkload(clearAfterBatch: readonly boolean[],): Workload {
        return {
          behaviorByKey: fields.behaviorByKey,
          keyMode: fields.keyMode,
          shouldCacheMode: fields.shouldCacheMode,
          cacheEnabled: fields.cacheEnabled,
          batches,
          clearAfterBatch,
        };
      },
    );
  },
);

//endregion Workload
