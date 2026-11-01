/**
 Memoization and decoration errors thrown by the p-memoize fork.
 
 Every class extends `TypeError` to keep the rejection shapes upstream
 `p-memoize` callers already handle (`instanceof TypeError` keeps holding)
 and carries upstream's message text verbatim.
 
 @module
 */

//region Cache clearing errors

/**
 Thrown by `pMemoizeClear` when the given function was never memoized.
 
 @example
 ```ts
 import { NotMemoizedError, pMemoizeClear, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 try {
   pMemoizeClear(async function plain(): Promise<void> {},);
 }
 catch (error) {
   error instanceof NotMemoizedError; // => true
 }
 ```
 */
export class NotMemoizedError extends TypeError {
  /**
   Builds the error with upstream `p-memoize`'s message text so callers
   migrating from `p-memoize` see identical diagnostics.
   */
  constructor() {
    super('Can\'t clear a function that was not memoized!',);
    this.name = 'NotMemoizedError';
  }
}

/**
 Thrown by `pMemoizeClear` when the memoized function was created with
 caching disabled (`cache: false`), so it holds nothing to clear.
 
 @example
 ```ts
 import { CacheDisabledError, pMemoize, pMemoizeClear, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 const memoized = pMemoize({
   fn: async function compute(): Promise<number> {
     return 1;
   },
   options: {
     cache: false,
   },
 },);
 try {
   pMemoizeClear(memoized,);
 }
 catch (error) {
   error instanceof CacheDisabledError; // => true
 }
 ```
 */
export class CacheDisabledError extends TypeError {
  /**
   Builds the error with upstream `p-memoize`'s message text so callers
   migrating from `p-memoize` see identical diagnostics.
   */
  constructor() {
    super('Can\'t clear a function that doesn\'t use a cache!',);
    this.name = 'CacheDisabledError';
  }
}

/**
 Thrown by `pMemoizeClear` when the memoized function's cache storage
 exposes no `clear` method (a `WeakMap`, for example).
 
 @example
 ```ts
 import { UnclearableCacheError, pMemoize, pMemoizeClear, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 const memoized = pMemoize({
   fn: async function compute(key: object,): Promise<number> {
     return 1;
   },
   options: {
     cache: new WeakMap<object, number>(),
   },
 },);
 try {
   pMemoizeClear(memoized,);
 }
 catch (error) {
   error instanceof UnclearableCacheError; // => true
 }
 ```
 */
export class UnclearableCacheError extends TypeError {
  /**
   Builds the error with upstream `p-memoize`'s message text so callers
   migrating from `p-memoize` see identical diagnostics.
   */
  constructor() {
    super('The cache can\'t be cleared!',);
    this.name = 'UnclearableCacheError';
  }
}

//endregion Cache clearing errors

//region Decorator errors

/**
 Thrown by `pMemoizeDecorator` when the decorated declaration is not a
 method (a getter or class field, for example).
 
 @example
 ```ts
 import { NonMethodDecorationError, pMemoizeDecorator, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 try {
   pMemoizeDecorator()(function notAMethod(): void {}, {
     kind: 'getter',
   } as never,);
 }
 catch (error) {
   error instanceof NonMethodDecorationError; // => true
 }
 ```
 */
export class NonMethodDecorationError extends TypeError {
  /**
   Builds the error with upstream `p-memoize`'s message text so callers
   migrating from `p-memoize` see identical diagnostics.
   */
  constructor() {
    super('pMemoizeDecorator can only decorate methods',);
    this.name = 'NonMethodDecorationError';
  }
}

/**
 Thrown by `pMemoizeDecorator` when the decorated method is private, which
 the decorator cannot replace per instance.
 
 @example
 ```ts
 import { pMemoizeDecorator, PrivateMethodDecorationError, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 try {
   pMemoizeDecorator()(function secret(): void {}, {
     kind: 'method',
     private: true,
   } as never,);
 }
 catch (error) {
   error instanceof PrivateMethodDecorationError; // => true
 }
 ```
 */
export class PrivateMethodDecorationError extends TypeError {
  /**
   Builds the error with upstream `p-memoize`'s message text so callers
   migrating from `p-memoize` see identical diagnostics.
   */
  constructor() {
    super('pMemoizeDecorator cannot decorate private methods',);
    this.name = 'PrivateMethodDecorationError';
  }
}

//endregion Decorator errors

//region Function copying errors

/**
 Thrown by the inlined `mimic-function` copy when a property reported by
 `Reflect.ownKeys` has no own descriptor, which the JavaScript object model
 forbids; thrown instead of copying blind.
 
 @example
 ```ts
 import { PropertyDescriptorMissingError, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 new PropertyDescriptorMissingError() instanceof PropertyDescriptorMissingError; // => true
 ```
 */
export class PropertyDescriptorMissingError extends TypeError {
  /**
   Builds the error naming the invariant, since no upstream message text
   exists for this repository-only guard.
   */
  constructor() {
    super('Expected an own property descriptor for a key from Reflect.ownKeys',);
    this.name = 'PropertyDescriptorMissingError';
  }
}

//endregion Function copying errors
