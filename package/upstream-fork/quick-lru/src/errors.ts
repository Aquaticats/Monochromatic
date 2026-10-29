/**
 Configuration errors thrown by the LRU cache factory.
 
 Both classes extend `TypeError` to keep the rejection shapes upstream
 `quick-lru` callers already handle (`instanceof TypeError` keeps holding).
 
 @module
 */

//region Errors

/**
 Thrown when a `maxSize` value is not truthy and greater than `0`.
 
 Thrown by `createQuickLru` for the constructor option and by `resize` for
 its new bound, matching upstream `quick-lru`'s single `maxSize` diagnostic.
 
 @example
 ```ts
 import { InvalidMaxSizeError, createQuickLru, } from '\@monochromatic-dev/upstream-fork-quick-lru';
 
 try {
   createQuickLru({ maxSize: 0, });
 }
 catch (error) {
   error instanceof InvalidMaxSizeError; // => true
 }
 ```
 */
export class InvalidMaxSizeError extends TypeError {
  /**
   Builds the error with upstream `quick-lru`'s message text so callers
   migrating from `quick-lru` see identical diagnostics.
   */
  constructor() {
    super('`maxSize` must be a number greater than 0',);
    this.name = 'InvalidMaxSizeError';
  }
}

/**
 Thrown when the global `maxAge` option is the number `0`.
 
 Upstream `quick-lru` rejects exactly `0` here; every other value reaches
 the cache unchanged, so the fork keeps that acceptance rule and message.
 
 @example
 ```ts
 import { InvalidMaxAgeError, createQuickLru, } from '\@monochromatic-dev/upstream-fork-quick-lru';
 
 try {
   createQuickLru({
     maxSize: 1,
     maxAge: 0,
   });
 }
 catch (error) {
   error instanceof InvalidMaxAgeError; // => true
 }
 ```
 */
export class InvalidMaxAgeError extends TypeError {
  /**
   Builds the error with upstream `quick-lru`'s message text so callers
   migrating from `quick-lru` see identical diagnostics.
   */
  constructor() {
    super('`maxAge` must be a number greater than 0',);
    this.name = 'InvalidMaxAgeError';
  }
}

//endregion Errors
