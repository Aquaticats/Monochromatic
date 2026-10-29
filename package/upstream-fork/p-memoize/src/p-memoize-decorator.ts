/**
 Class-method decorator that memoizes each instance's method separately.
 
 Derived from upstream `p-memoize` 9.0.0's `pMemoizeDecorator` with the same
 initializer-based installation: each instance gets its own memoized
 function as an own, non-enumerable, writable, configurable property, so
 `pMemoizeClear(instance.method)` clears one instance's cache. An instance
 whose class chain no longer resolves the method to the decorated function
 (a subclass override, a deleted method) keeps what its class defines.
 
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

//region Method lookup

/**
 Finds the method `name` resolves to on `owner`, walking its prototype chain
 by descriptor.

 Descriptors are read rather than the property itself: a plain read would
 run a subclass getter once per instantiation, with the prototype as its
 receiver, which can throw. An accessor in the chain means something replaced
 the method with one, so it is not the function that was decorated.

 @param owner - Object whose chain is searched.

 @param name - Method name or symbol to resolve.

 @returns The data property's value, or `undefined` when the name is missing
 from the whole chain or resolves to an accessor.

 @example
 ```ts
 findMethod({ owner: Example.prototype, name: 'counter', },);
 ```
 */
function findMethod(
  {
    owner,
    name,
  }: {
    readonly owner: object;
    readonly name: string | symbol;
  },
): unknown {
  // The walk ends at the top of the prototype chain, which is why it is a
  // null check: a method missing from the whole chain (deleted by a mixin
  // or a mock, say) is the case the caller stands down for.
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors `Reflect.getPrototypeOf`'s documented `object | null` return at the top of the chain
  for (let link: object | null = owner; link !== null; link = Reflect.getPrototypeOf(link,)) {
    /**
     Own descriptor of the name on this link of the chain.
     */
    const descriptor = Object.getOwnPropertyDescriptor(
      link,
      name,
    );

    if (descriptor !== undefined)
      // mutation-test-disable-next-line conditional -- equivalent: an accessor descriptor has no `value` property, so reading `descriptor.value` on it also yields `undefined`, the same result the `: undefined` branch returns
      return ('value' in descriptor)
        ? descriptor.value as unknown
        : undefined;
  }

  return undefined;
}

/**
 Reports whether a value can hold properties.

 @param value - Value to test.

 @returns `true` for objects and functions.

 @example
 ```ts
 isObjectLike(class Example {},); // => true
 ```
 */
function isObjectLike(value: unknown,): value is object {
  return (((typeof value) === 'object') && (value !== null))
    || ((typeof value) === 'function');
}

//endregion Method lookup

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
      // A subclass can override the method, and defining the memoized copy
      // on the instance would shadow that override. Ask where the method is
      // actually defined (the prototype for an instance method, the class
      // itself for a static one) and stand down if it is not ours.
      /**
       Receiver of the initializer, widened so a type guard can narrow it.
       */
      const receiver: unknown = this;

      if (!isObjectLike(receiver,))
        return;

      /**
       Object that holds the decorated method: the class itself for a static
       method, the prototype otherwise.
       */
      const owner = context.static
        ? receiver
        : Reflect.getPrototypeOf(receiver,);

      if (owner === null)
        return;

      if (findMethod({
        owner,
        name: context.name,
      },) !== value)
        return;

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
