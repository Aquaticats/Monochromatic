/**
 fast-check arbitraries generating cache workloads: constructor bounds,
 mixed operations over a small key universe, per-item lifetime quirks,
 and controlled clock advances. Also generates the junk constructor
 inputs the totality and validation properties feed.
 
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
} from 'fast-check';

import type {
  CacheOp,
  CacheWorkload,
} from './cache-workload.ts';

//region Atoms

/**
 Negative per-item lifetime: stamps expiry in the past so items expire on
 first touch.
 */
const NEGATIVE_LIFETIME_MS = -5;

/**
 Fractional per-item lifetime exercising non-integer expiry stamps.
 */
const FRACTIONAL_LIFETIME_MS = 1.5;

/**
 Short lifetime in milliseconds, smaller than the generated clock
 advances so expiry actually fires.
 */
const SHORT_LIFETIME_MS = 500;

/**
 Long global lifetime in milliseconds, larger than the generated clock
 advances.
 */
const LONG_LIFETIME_MS = 1_000;

/**
 Fractional eviction count exercising upstream's `Math.trunc` rounding.
 */
const FRACTIONAL_EVICT_COUNT = 1.9;

/**
 Negative eviction count exercising the rejected-count gate.
 */
const NEGATIVE_EVICT_COUNT = -1;

/**
 Key universe: four keys keep duplicate-key and both-map states reachable
 while staying enumerable in traces.
 */
const KEY_POOL: readonly string[] = [
  'a',
  'b',
  'c',
  'd',
];

/**
 Value universe for stored items.
 */
const VALUE_POOL: readonly string[] = [
  'v0',
  'v1',
  'v2',
];

/**
 Per-item lifetimes, quirks included: negative and zero stamps, plain
 durations, an infinite one, and `NaN` whose stamp upstream treats as
 never-expires.
 */
const ITEM_MAX_AGE_POOL: readonly number[] = [
  NEGATIVE_LIFETIME_MS,
  0,
  FRACTIONAL_LIFETIME_MS,
  100,
  SHORT_LIFETIME_MS,
  Number.POSITIVE_INFINITY,
  Number.NaN,
];

/**
 Global lifetimes, quirks included; `Number.POSITIVE_INFINITY` stands in
 for upstream's absent `maxAge`.
 */
const GLOBAL_MAX_AGE_POOL: readonly number[] = [
  100,
  LONG_LIFETIME_MS,
  Number.POSITIVE_INFINITY,
  Number.NaN,
];

/**
 Eviction counts, quirks included: zero, negative, fractional, `NaN`, and
 counts larger than the cache.
 */
const EVICT_COUNT_POOL: readonly number[] = [
  0,
  1,
  2,
  10,
  FRACTIONAL_EVICT_COUNT,
  NEGATIVE_EVICT_COUNT,
  Number.NaN,
  100,
];

/**
 Random key from {@link KEY_POOL}.
 */
const keyArb: Arbitrary<string> = constantFrom(...KEY_POOL);

/**
 Random value from {@link VALUE_POOL}.
 */
const valueArb: Arbitrary<string> = constantFrom(...VALUE_POOL);

/**
 Random operation over the key universe.
 */
const opArb: Arbitrary<CacheOp> = oneof(
  record({
    key: keyArb,
    value: valueArb,
  })
    .map(function toSet(fields: {
      readonly key: string;
      readonly value: string;
    },): CacheOp {
      return {
        kind: 'set',
        key: fields.key,
        value: fields.value,
      };
    },),
  record({
    key: keyArb,
    value: valueArb,
    maxAge: constantFrom(...ITEM_MAX_AGE_POOL),
  })
    .map(function toSetWithMaxAge(fields: {
      readonly key: string;
      readonly value: string;
      readonly maxAge: number;
    },): CacheOp {
      return {
        kind: 'setWithMaxAge',
        key: fields.key,
        value: fields.value,
        maxAge: fields.maxAge,
      };
    },),
  keyArb.map(function toGet(key: string,): CacheOp {
    return {
      kind: 'get',
      key,
    };
  },),
  keyArb.map(function toPeek(key: string,): CacheOp {
    return {
      kind: 'peek',
      key,
    };
  },),
  keyArb.map(function toHas(key: string,): CacheOp {
    return {
      kind: 'has',
      key,
    };
  },),
  keyArb.map(function toDelete(key: string,): CacheOp {
    return {
      kind: 'delete',
      key,
    };
  },),
  keyArb.map(function toExpiresIn(key: string,): CacheOp {
    return {
      kind: 'expiresIn',
      key,
    };
  },),
  constant({
    kind: 'clear',
  } satisfies CacheOp),
  oneof(
    integer({
      min: 1,
      max: 6,
    }),
    constantFrom(
      0,
      -1,
      Number.NaN,
    ),
  )
    .map(function toResize(maxSize: number,): CacheOp {
      return {
        kind: 'resize',
        maxSize,
      };
    },),
  constantFrom(...EVICT_COUNT_POOL)
    .map(function toEvict(count: number,): CacheOp {
      return {
        kind: 'evict',
        count,
      };
    },),
  integer({
    min: 0,
    max: 2_000,
  })
    .map(function toAdvanceClock(milliseconds: number,): CacheOp {
      return {
        kind: 'advanceClock',
        milliseconds,
      };
    },),
);

//endregion Atoms

//region Workload

/**
 Generated workload: one to twelve operations over four keys, a target
 maximum of one to four items, a global lifetime from the quirk pool, and
 an eviction-callback toggle.
 */
export const workloadArb: Arbitrary<CacheWorkload> = record({
  maxSize: integer({
    min: 1,
    max: 4,
  }),
  maxAge: constantFrom(...GLOBAL_MAX_AGE_POOL),
  onEviction: boolean(),
  ops: array(
    opArb,
    {
      minLength: 1,
      maxLength: 12,
    },
  ),
})
  .map(function toWorkload(fields: {
    readonly maxSize: number;
    readonly maxAge: number;
    readonly onEviction: boolean;
    readonly ops: readonly CacheOp[];
  },): CacheWorkload {
    return fields;
  },);

//endregion Workload

//region Junk inputs

/**
 Junk constructor values: nullish, numeric, string, boolean, array, and
 object shapes that must each land on exactly one upstream failure mode.
 */
const JUNK_POOL: readonly unknown[] = [
  undefined,
  null,
  0,
  -1,
  Number.NaN,
  Number.POSITIVE_INFINITY,
  1,
  'x',
  '',
  true,
  false,
  [],
  {},
];

/**
 Random junk constructor input, either a bare junk value or an options
 object with junk `maxSize` and `maxAge` fields.
 */
export const junkInputArb: Arbitrary<unknown> = oneof(
  constantFrom(...JUNK_POOL),
  record({
    maxSize: constantFrom(...JUNK_POOL),
    maxAge: constantFrom(...JUNK_POOL),
  })
    .map(function toJunkOptions(fields: {
      readonly maxSize: unknown;
      readonly maxAge: unknown;
    },): unknown {
      return fields;
    },),
);

//endregion Junk inputs
