/**
 Mapper configuration parsing and validation.
 
 Accepts upstream `p-map`'s option shapes and destructures them in upstream's
 property order and default order, so a mis-typed options object fails at the
 same point with the same diagnostics as upstream `p-map`. Validation of the
 destructured values stays in the mapper call sites' order (input,
 mapper,
 concurrency,
 then backpressure), matching upstream's check sequence.
 
 @module
 */

import {
  InvalidBackpressureError,
  InvalidConcurrencyError,
} from './errors.ts';

//region Types

/**
 Options for the concurrent map, mirroring upstream `p-map`'s `Options`.
 
 @example
 ```ts
 const options: MapOptions = {
   concurrency: 2,
   stopOnError: false,
 };
 ```
 */
export type MapOptions = {
  /**
   Number of concurrently pending mapper promises. Must be a safe integer
   from 1 and up or `Number.POSITIVE_INFINITY`.
   
   @defaultValue Number.POSITIVE_INFINITY
   */
  readonly concurrency?: number;
  /**
   When `true`, the first mapper rejection rejects the run; when `false`, the
   run continues and rejects with an `AggregateError` of every failure.
   
   @defaultValue true
   */
  readonly stopOnError?: boolean;
  /**
   Abort signal rejecting the run with its reason.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream `p-map` 7.0.8 `Options.signal`, typed `AbortSignal | undefined` there because an explicitly `undefined` signal means "no signal"; callers porting option spreads rely on that acceptance
  readonly signal?: AbortSignal | undefined;
};

/**
 Options for the streaming map, mirroring upstream `p-map`'s
 `IterableOptions`.
 
 @example
 ```ts
 const options: IterableMapOptions = {
   concurrency: 2,
   backpressure: 4,
 };
 ```
 */
export type IterableMapOptions = {
  /**
   Number of concurrently pending mapper promises. Must be a safe integer
   from 1 and up or `Number.POSITIVE_INFINITY`.
   
   @defaultValue Number.POSITIVE_INFINITY
   */
  readonly concurrency?: number;
  /**
   Maximum number of mapper promises resolved but not yet collected by the
   consumer. Must be a safe integer from `concurrency` and up or
   `Number.POSITIVE_INFINITY`.
   
   @defaultValue concurrency
   */
  readonly backpressure?: number;
};

/**
 Validated concurrent-map configuration with every non-signal option
 present.
 
 @example
 ```ts
 const resolved: ResolvedMapOptions = {
   concurrency: 1,
   stopOnError: false,
 };
 ```
 */
export type ResolvedMapOptions = {
  /**
   Concurrency bound as the caller passed it; validated by the mapper call
   after the input and mapper checks, in upstream's order.
   */
  readonly concurrency: number;
  /**
   Validated `stopOnError` flag, defaulting to `true` when absent.
   */
  readonly stopOnError: boolean;
  /**
   Abort signal for the run; the property is absent when the caller passed no
   signal.
   */
  readonly signal?: AbortSignal;
};

/**
 Validated streaming-map configuration with every option present.
 
 @example
 ```ts
 const resolved: ResolvedIterableMapOptions = {
   concurrency: 1,
   backpressure: 1,
 };
 ```
 */
export type ResolvedIterableMapOptions = {
  /**
   Concurrency bound as the caller passed it; validated by the mapper call
   after the input and mapper checks, in upstream's order.
   */
  readonly concurrency: number;
  /**
   Backpressure bound as the caller passed or defaulted it; validated by the
   mapper call after the concurrency check, in upstream's order.
   */
  readonly backpressure: number;
};

//endregion Types

//region Validation

/**
 Accepted-bound check input: a candidate value and the floor it must reach.
 
 @example
 ```ts
 const bounds: BoundCheck = {
   value: 4,
   minimum: 1,
 };
 ```
 */
export type BoundCheck = {
  /**
   Candidate bound of unknown runtime type.
   */
  readonly value: unknown;
  /**
   Smallest accepted safe integer, compared only after the safe-integer
   check.
   */
  readonly minimum: number;
};

/**
 Bounds whose candidate survived {@link isBoundedInteger}: a safe integer at
 or above its floor.
 
 @example
 ```ts
 const validated: ValidatedBound = {
   value: 4,
   minimum: 1,
 };
 ```
 */
export type ValidatedBound = {
  /**
   Candidate proven to be a safe integer at or above `minimum`.
   */
  readonly value: number;
  /**
   Floor the candidate was compared against.
   */
  readonly minimum: number;
};

/**
 Checks whether a candidate is an accepted concurrency or backpressure
 bound: a safe integer at or above its floor, or
 `Number.POSITIVE_INFINITY`.
 
 @param bounds - Candidate value and the floor it must reach.
 
 @returns `true` when the candidate is an accepted bound; narrows the
 candidate to `number`.
 
 @example
 ```ts
 isBoundedInteger({
   value: 4,
   minimum: 1,
 },); // => true
 isBoundedInteger({
   value: 0,
   minimum: 1,
 },); // => false
 ```
 */
export function isBoundedInteger(bounds: BoundCheck,): bounds is ValidatedBound {
  return ((typeof bounds.value) === 'number')
    && (((Number.isSafeInteger(bounds.value,)) && (bounds.value >= bounds.minimum))
      || (bounds.value === Number.POSITIVE_INFINITY));
}

/**
 Validates one concurrency value, keeping upstream `p-map`'s acceptance rule
 and error message.
 
 @param concurrency - Candidate concurrency value of unknown runtime type.
 
 @returns The same value, proven to be an accepted concurrency bound.
 
 @throws InvalidConcurrencyError when the value is not a safe integer from 1
 and up and not `Number.POSITIVE_INFINITY`.
 
 @example
 ```ts
 validateConcurrency(2,); // => 2
 ```
 */
export function validateConcurrency(concurrency: unknown,): number {
  /**
   Candidate and its floor, narrowed to `number` by the check below.
   */
  const bounds = {
    value: concurrency,
    minimum: 1,
  };
  if (!isBoundedInteger(bounds,))
    throw new InvalidConcurrencyError(concurrency,);
  return bounds.value;
}

/**
 Validates one backpressure value against its concurrency bound, keeping
 upstream `p-map`'s acceptance rule and error message.
 
 @param backpressure - Candidate backpressure value of unknown runtime
 type.
 
 @param concurrency - Resolved concurrency bound the backpressure must
 reach.
 
 @returns The same backpressure value, proven to be an accepted bound.
 
 @throws InvalidBackpressureError when the value is not a safe integer from
 `concurrency` and up and not `Number.POSITIVE_INFINITY`.
 
 @example
 ```ts
 validateBackpressure({
   backpressure: 4,
   concurrency: 2,
 }); // => 4
 ```
 */
export function validateBackpressure(
  {
    backpressure,
    concurrency,
  }: {
    /**
     Candidate backpressure value of unknown runtime type.
     */
    readonly backpressure: unknown;
    /**
     Resolved concurrency bound the backpressure must reach.
     */
    readonly concurrency: number;
  },
): number {
  /**
   Candidate and its floor, narrowed to `number` by the check below.
   */
  const bounds = {
    value: backpressure,
    minimum: concurrency,
  };
  if (!isBoundedInteger(bounds,))
    throw new InvalidBackpressureError({
      backpressure,
      concurrency,
    },);
  return bounds.value;
}

//endregion Validation

//region Resolution

/**
 Normalizes the concurrent-map options accepted by `pMap` into one record
 with upstream `p-map`'s destructuring defaults.
 
 Validation is deliberately absent: upstream `p-map` checks the input,
 then
 the mapper,
 then the concurrency bound,
 so the mapper call validates the
 destructured values after those checks rather than here. A non-object,
 non-`undefined` options value fails here with a plain `TypeError` at the
 destructuring, matching upstream `p-map`'s parameter destructuring.
 
 @param options - Caller-supplied options or `undefined` for the defaults.
 
 @returns Options with `concurrency` and `stopOnError` filled in.
 
 @throws TypeError when `options` is a non-object, non-`undefined` value
 that cannot be destructured, matching upstream `p-map`.
 
 @example
 ```ts
 resolveMapOptions();
 // => { concurrency: Number.POSITIVE_INFINITY, stopOnError: true }
 ```
 */
export function resolveMapOptions(options: MapOptions = {},): ResolvedMapOptions {
  /**
   Destructured exactly as upstream `p-map`'s parameter destructuring, so
   getter access order and defaulting match.
   */
  const {
    concurrency = Number.POSITIVE_INFINITY,
    stopOnError = true,
    signal,
  } = options;

  return {
    concurrency,
    stopOnError,
    ...(signal === undefined
      ? {}
      : { signal, }),
  };
}

/**
 Normalizes the streaming-map options accepted by `pMapIterable` into one
 record with upstream `p-map`'s destructuring defaults, including
 `backpressure` defaulting to the (still unvalidated) concurrency bound.
 
 Validation is deliberately absent for the same reason as
 {@link resolveMapOptions}: upstream validates input,
 mapper,
 concurrency,
 and
 backpressure in that order, after destructuring.
 
 @param options - Caller-supplied options or `undefined` for the defaults.
 
 @returns Options with `concurrency` and `backpressure` filled in.
 
 @throws TypeError when `options` is a non-object, non-`undefined` value
 that cannot be destructured, matching upstream `p-map`.
 
 @example
 ```ts
 resolveIterableMapOptions({ concurrency: 2, },);
 // => { concurrency: 2, backpressure: 2 }
 ```
 */
export function resolveIterableMapOptions(options: IterableMapOptions = {},): ResolvedIterableMapOptions {
  /**
   Destructured exactly as upstream `p-map`'s parameter destructuring, so the
   `backpressure` default sees the raw concurrency value.
   */
  const {
    concurrency = Number.POSITIVE_INFINITY,
    backpressure = concurrency,
  } = options;

  return {
    concurrency,
    backpressure,
  };
}

//endregion Resolution
