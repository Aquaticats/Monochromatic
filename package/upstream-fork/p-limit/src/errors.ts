/**
 Configuration errors thrown by the concurrency limiter.
 
 Both classes extend `TypeError` to keep the rejection shapes upstream
 `p-limit` callers already handle (`instanceof TypeError` keeps holding).
 
 @module
 */

//region Errors

/**
 Thrown when a concurrency value is not a positive integer and not
 `Number.POSITIVE_INFINITY`.
 
 @example
 ```ts
 import { InvalidConcurrencyError, pLimit, } from '\@monochromatic-dev/upstream-fork-p-limit';
 
 try {
   pLimit(0);
 }
 catch (error) {
   error instanceof InvalidConcurrencyError; // => true
 }
 ```
 */
export class InvalidConcurrencyError extends TypeError {
  /**
   Builds the error with upstream `p-limit`'s message text so callers
   migrating from `p-limit` see identical diagnostics.
   */
  constructor() {
    super('Expected `concurrency` to be a number from 1 and up',);
    this.name = 'InvalidConcurrencyError';
  }
}

/**
 Thrown when `rejectOnClear` is present but not a boolean.
 
 @example
 ```ts
 import { InvalidRejectOnClearError, pLimit, } from '\@monochromatic-dev/upstream-fork-p-limit';
 
 try {
   pLimit({
     concurrency: 1,
     rejectOnClear: 'yes',
   });
 }
 catch (error) {
   error instanceof InvalidRejectOnClearError; // => true
 }
 ```
 */
export class InvalidRejectOnClearError extends TypeError {
  /**
   Builds the error with upstream `p-limit`'s message text so callers
   migrating from `p-limit` see identical diagnostics.
   */
  constructor() {
    super('Expected `rejectOnClear` to be a boolean',);
    this.name = 'InvalidRejectOnClearError';
  }
}

//endregion Errors
