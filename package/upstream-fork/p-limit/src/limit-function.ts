/**
 Wrap one function so every call it receives runs under a concurrency bound.
 
 Derived from [`p-limit`](https://github.com/sindresorhus/p-limit) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution.
 
 @module
 */

import type { LimitOptions, } from './limit-options.ts';
import { pLimit, } from './p-limit.ts';

//region Types

/**
 Function returned by {@link limitFunction}: calls the wrapped function with
 a caller-supplied argument tuple under the configured concurrency bound.
 
 @typeParam TArgs - tuple of wrapped function argument types
 
 @typeParam TResult - wrapped function return type, awaited when thenable
 
 @example
 ```ts
 const limitedFetch: LimitedFunction<[string], Response> = limitFunction({
   fn: fetch,
   options: { concurrency: 1, },
 });
 const response = await limitedFetch({ args: ['https://example.com'], });
 ```
 */
export type LimitedFunction<TArgs extends readonly unknown[], TResult> = {
  /**
   Runs the wrapped function with one argument tuple, queued behind earlier
   calls when the bound is reached.
   
   @param call - Argument tuple spread into the wrapped function.
   
   @returns Promise settling with the wrapped function's result or failure.
   */
  (call: { readonly args: TArgs, }): Promise<TResult>;
  /**
   Discards queued calls that never started, forwarding to the underlying
   limiter's `clearQueue`.
   */
  readonly clearQueue: () => void;
};

//endregion Types

//region Factory

/**
 Wraps one function with its own concurrency-limited call queue.
 
 Ideal when a single function's calls must be bounded rather than several
 functions sharing one limiter.
 
 @param fn - Function whose calls are bounded.
 
 @param options - Concurrency bound and optional `rejectOnClear`, forwarded
 to `pLimit`.
 
 @returns Callable wrapper exposing `clearQueue`.
 
 @throws InvalidConcurrencyError when the concurrency value is not a
 positive integer or `Number.POSITIVE_INFINITY`.
 
 @throws InvalidRejectOnClearError when `rejectOnClear` is present but not a
 boolean.
 
 @example
 ```ts
 import { limitFunction, } from '\@monochromatic-dev/upstream-fork-p-limit';
 
 const limitedFetch = limitFunction({
   fn: fetch,
   options: { concurrency: 2, },
 });
 const responses = await Promise.all([
   limitedFetch({ args: ['/a'], }),
   limitedFetch({ args: ['/b'], }),
 ]);
 ```
 */
export function limitFunction<const TArgs extends readonly unknown[], TResult>(
  {
    fn,
    options,
  }: {
    /**
     Function whose calls are bounded; receives the tuple each call passes.
     */
    readonly fn: (...args: TArgs) => TResult | PromiseLike<TResult>;
    /**
     Concurrency bound and optional `rejectOnClear` for the backing limiter.
     */
    readonly options: LimitOptions;
  },
): LimitedFunction<TArgs, TResult> {
  /**
   Backing limiter whose queue holds this wrapper's calls.
   */
  const limit = pLimit(options,);

  /**
   Runs the wrapped function with one argument tuple through the backing
   limiter.
   
   @param args - Arguments spread into the wrapped function.
   
   @returns Promise settling with the wrapped function's result or failure.
   */
  function limited(
    {
      args,
    }: {
      readonly args: TArgs;
    },
  ): Promise<TResult> {
    return limit({
      fn,
      args,
    },);
  }

  /**
   `clearQueue` attached exactly as upstream `p-limit`'s `limitFunction`
   attaches it: a non-enumerable,
   non-writable,
   non-configurable data
   property holding the backing limiter's method,
   so `Object.keys`
   stays empty and the member cannot be replaced.
   */
  Object.defineProperty(
    limited,
    'clearQueue',
    {
      value: limit.clearQueue,
      writable: false,
      enumerable: false,
      configurable: false,
    },
  );

  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- `limited` gains its clearQueue member through defineProperty above
  return limited as LimitedFunction<TArgs, TResult>;
}

//endregion Factory
