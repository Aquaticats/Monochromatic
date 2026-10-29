/**
 Class-method decorator that memoizes each instance's method separately.
 
 Derived from upstream `p-memoize`'s `pMemoizeDecorator` with the same
 initializer-based installation: each instance gets its own memoized
 function as an own, non-enumerable, writable, configurable property, so
 `pMemoizeClear(instance.method)` clears one instance's cache.
 
 @module
 */

import {
  NonMethodDecorationError,
  PrivateMethodDecorationError,
} from './errors.ts';
import {
  type MemoizeOptions,
  pMemoize,
} from './p-memoize.ts';

//region Decorator

/**
 Builds a decorator that memoizes class methods (instance and static).
 
 Only the new ECMAScript decorators (TypeScript 5.0+) are supported: legacy
 `experimentalDecorators` and Babel's legacy variant implement different
 proposal semantics. Private methods are not supported because their slot
 cannot be replaced per instance.
 
 @typeParam TArgs - tuple of decorated method argument types, defaulting to
 an untyped tuple for bare `@pMemoizeDecorator()` calls
 
 @typeParam TResult - decorated method return type, awaited when thenable
 
 @typeParam CacheKeyType - key type produced by the `cacheKey` function
 
 @param options - Cache configuration, identical to `pMemoize`'s.
 
 @returns Decorator applying per-instance memoization to a method.
 
 @throws NonMethodDecorationError when the decorated declaration is not a
 method.
 
 @throws PrivateMethodDecorationError when the decorated method is private.
 
 @example
 ```ts
 import { pMemoizeDecorator, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 class Example {
   index = 0;
 
   @pMemoizeDecorator()
   async counter(): Promise<number> {
     return ++this.index;
   }
 }
 ```
 */
export function pMemoizeDecorator<
  TArgs extends unknown[] = unknown[],
  TResult = unknown,
  CacheKeyType = TArgs[0],
  >(
  options: MemoizeOptions<TArgs, TResult, CacheKeyType> = {},
  ): <This>(
    value: (
      this: This,
      ...callArgs: TArgs
    ) => TResult | PromiseLike<TResult>,
    context: ClassMethodDecoratorContext<This>,
  ) => void {
  /**
   Decorator body; its two parameters come from the decorator protocol. The
   context keeps its default method signature because the decorated method
   type arrives on `value` itself.
   */
  return function decorate<This>(
    value: (
      this: This,
      ...callArgs: TArgs
    ) => TResult | PromiseLike<TResult>,
    context: ClassMethodDecoratorContext<This>,
  ): void {
    if (context.kind !== 'method')
      throw new NonMethodDecorationError();

    if (context.private)
      throw new PrivateMethodDecorationError();

    context.addInitializer(function initializeMemoizedMethod(this: This): void {
      /**
       Per-instance memoized replacement for the decorated method.
       */
      const memoizedMethod = pMemoize({
        fn: value,
        options,
      },);
      Object.defineProperty(
        this,
        context.name,
        {
          configurable: true,
          writable: true,
          enumerable: false,
          value: memoizedMethod,
        },
      );
    });
  };
}

//endregion Decorator
