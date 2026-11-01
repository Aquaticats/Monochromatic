/**
 Limiter configuration parsing and validation.
 
 Accepts upstream `p-limit`'s constructor shapes (bare concurrency number or
 options object) and normalizes them into one resolved record, so the limiter
 itself never branches on its input shape.
 
 @module
 */

import {
  InvalidConcurrencyError,
  InvalidRejectOnClearError,
} from './errors.ts';

//region Types

/**
 Limiter configuration accepted by `pLimit` and `limitFunction`.
 
 Mirrors upstream `p-limit`'s `Options` with the same defaults and the same
 validation error messages.
 
 @example
 ```ts
 const options: LimitOptions = {
   concurrency: 4,
   rejectOnClear: true,
 };
 ```
 */
export type LimitOptions = {
  /**
   Maximum number of calls running at once. Must be a positive integer or
   `Number.POSITIVE_INFINITY` (no cap).
   */
  readonly concurrency: number;
  /**
   Reject queued (not yet started) calls with `AbortSignal.abort().reason`
   when `clearQueue()` runs, instead of leaving their promises pending.
   
   @defaultValue false
   */
  readonly rejectOnClear?: boolean;
};

/**
 Validated limiter configuration with every option present.
 
 @example
 ```ts
 const resolved: ResolvedLimitOptions = {
   concurrency: 1,
   rejectOnClear: false,
 };
 ```
 */
export type ResolvedLimitOptions = {
  /**
   Validated concurrency bound.
   */
  readonly concurrency: number;
  /**
   Validated `rejectOnClear` flag, defaulting to `false` when absent.
   */
  readonly rejectOnClear: boolean;
};

//endregion Types

//region Validation

/**
 Checks whether a value is an accepted concurrency bound: a positive integer
 or `Number.POSITIVE_INFINITY`.
 
 @param value - Candidate concurrency value of unknown runtime type.
 
 @returns `true` when the value is an accepted concurrency bound.
 
 @example
 ```ts
 isConcurrency(4); // => true
 isConcurrency(0); // => false
 ```
 */
export function isConcurrency(value: unknown,): value is number {
  return ((typeof value) === 'number')
    && ((value > 0)
      && ((Number.isInteger(value,)) || (value === Number.POSITIVE_INFINITY)));
}

/**
 Validates one concurrency value, keeping upstream `p-limit`'s acceptance
 rules and error message.
 
 @param concurrency - Candidate concurrency value of unknown runtime type.
 
 @returns The same value, proven to be an accepted concurrency bound.
 
 @throws InvalidConcurrencyError when the value is not a positive integer or
 `Number.POSITIVE_INFINITY`.
 
 @example
 ```ts
 validateConcurrency(2); // => 2
 ```
 */
export function validateConcurrency(concurrency: unknown,): number {
  if (!isConcurrency(concurrency,))
    throw new InvalidConcurrencyError();
  return concurrency;
}

//endregion Validation

//region Resolution

/**
 Normalizes the constructor input accepted by `pLimit` into one validated
 record.
 
 A bare number means `{ concurrency, rejectOnClear: false }`. The object form
 validates `concurrency` first and `rejectOnClear` second, matching upstream
 `p-limit`'s error precedence when both are invalid.
 
 @param concurrencyOrOptions - Bare concurrency number or limiter options.
 
 @returns Validated options with every field present.
 
 @throws InvalidConcurrencyError when the concurrency value is invalid.
 
 @throws InvalidRejectOnClearError when `rejectOnClear` is present but not a
 boolean.
 
 @throws TypeError when the input is a non-object, non-number value that
 cannot be destructured, matching upstream `p-limit`.
 
 @example
 ```ts
 resolveLimitOptions(2,);
 // => { concurrency: 2, rejectOnClear: false }
 ```
 */
export function resolveLimitOptions(concurrencyOrOptions: number | LimitOptions,): ResolvedLimitOptions {
  if ((typeof concurrencyOrOptions) === 'number')
    return {
      concurrency: validateConcurrency(concurrencyOrOptions,),
      rejectOnClear: false,
    };

  /**
   Destructuring matches upstream `p-limit`: a non-object, non-number input
   fails here with a plain `TypeError`, before any option validation runs.
   */
  const {
    concurrency,
    rejectOnClear = false,
  } = concurrencyOrOptions;

  /**
   Validated concurrency bound, checked before `rejectOnClear` to preserve
   upstream `p-limit`'s error precedence.
   */
  const validatedConcurrency = validateConcurrency(concurrency,);

  if ((typeof rejectOnClear) !== 'boolean')
    throw new InvalidRejectOnClearError();

  return {
    concurrency: validatedConcurrency,
    rejectOnClear,
  };
}

//endregion Resolution
