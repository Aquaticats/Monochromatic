/**
 Adapters exposing one uniform wrap-and-call surface over this package's
 fork and upstream `make-asynchronous`, so the same generated spec can drive
 both despite their different call shapes.
 
 @module
 */

import makeAsynchronousUpstream, {
  makeAsynchronousIterable as makeAsynchronousIterableUpstream,
} from 'make-asynchronous';

import {
  makeAsynchronous,
  makeAsynchronousIterable,
} from '@monochromatic-dev/upstream-fork-make-asynchronous/ts';

//region Types

/**
 One implementation's wrap-and-call surface.
 */
export type AsyncAdapter = {
  /**
   Implementation name used in oracle reports.
   */
  readonly name: string;
  /**
   Wraps a single-call function and resolves one argument tuple.
   
   @param fn - Fixture function to wrap.
   
   @param options - Runtime options record.
   
   @param args - Caller arguments forwarded to the worker.
   
   @returns Promise settling with the worker's restored result.
   */
  call: (
    fn: (...callArguments: never[]) => unknown,
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- options records are possibly absent by construction; the adapters forward them unvalidated exactly like both untyped runtimes
    options: Record<string, unknown> | undefined,
    args: readonly unknown[],
  ) => Promise<unknown>;
  /**
   Wraps an iterable function and drains one iteration.
   
   @param fn - Fixture generator to wrap.
   
   @param options - Runtime options record.
   
   @param args - Caller arguments forwarded with the first message.
   
   @returns Values in yield order.
   */
  iterate: (
    fn: (...callArguments: never[]) => Generator<unknown, void, unknown>,
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- options records are possibly absent by construction; the adapters forward them unvalidated exactly like both untyped runtimes
    options: Record<string, unknown> | undefined,
    args: readonly unknown[],
  ) => Promise<readonly unknown[]>;
};

//endregion Types

//region Adapters

/**
 Tests whether a drained value is an async iterable.
 
 @param value - Value returned from the wrapped call.
 
 @returns Whether the value exposes `Symbol.asyncIterator`.
 
 @example
 ```ts
 isAsyncIterable(iterable); // => true
 ```
 */
function isAsyncIterable(value: unknown,): value is AsyncIterable<unknown> {
  return ((typeof value) === 'object')
    && ((value !== null))
    && (Symbol.asyncIterator in value);
}

/**
 Adapter over this package's fork: wraps through
 `makeAsynchronous({ fn, options })` and calls with the `{ args }` tuple
 shape.
 */
export const forkAdapter: AsyncAdapter = {
  name: 'fork',
  call: async function callFork(
    fn: (...callArguments: never[]) => unknown,
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- options records are possibly absent by construction; the adapters forward them unvalidated exactly like both untyped runtimes
    options: Record<string, unknown> | undefined,
    args: readonly unknown[],
  ): Promise<unknown> {
    /**
     Wrapped function under comparison.
     */
    const wrapped = makeAsynchronous({
      fn,
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mis-typed option records are part of the generated input space and must reach the implementation unvalidated, exactly like upstream's untyped runtime
      options: options as never,
    },);
    return await Reflect.apply(
      wrapped as (...callArguments: never[]) => unknown,
      undefined,
      [
        { args: [...args,], },
      ],
    );
  },
  iterate: async function iterateFork(
    fn: (...callArguments: never[]) => Generator<unknown, void, unknown>,
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- options records are possibly absent by construction; the adapters forward them unvalidated exactly like both untyped runtimes
    options: Record<string, unknown> | undefined,
    args: readonly unknown[],
  ): Promise<readonly unknown[]> {
    /**
     Wrapped iterable under comparison.
     */
    const wrapped = makeAsynchronousIterable({
      fn,
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mis-typed option records are part of the generated input space and must reach the implementation unvalidated, exactly like upstream's untyped runtime
      options: options as never,
    },);
    /**
     Values drained in yield order.
     */
    const values: unknown[] = [];
    /**
     Iterable drained through the `{ args }` tuple shape.
     */
    const iterable: unknown = Reflect.apply(
      wrapped as (...callArguments: never[]) => unknown,
      undefined,
      [
        { args: [...args,], },
      ],
    );
    if (!isAsyncIterable(iterable,))
      throw new TypeError('Fork adapter did not return an async iterable',);
    for await (const value of iterable)
      values.push(value,);
    return values;
  },
};

/**
 Adapter over upstream `make-asynchronous`: wraps through
 `makeAsynchronous(fn, options)` and calls with the rest-argument shape.
 */
export const upstreamAdapter: AsyncAdapter = {
  name: 'upstream',
  call: async function callUpstream(
    fn: (...callArguments: never[]) => unknown,
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- options records are possibly absent by construction; the adapters forward them unvalidated exactly like both untyped runtimes
    options: Record<string, unknown> | undefined,
    args: readonly unknown[],
  ): Promise<unknown> {
    /**
     Wrapped function under comparison.
     */
    const wrapped = makeAsynchronousUpstream(
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- fixture functions use the fork's narrow `never[]` signature; upstream accepts any function shape and serializes it the same way
      fn as (...callArguments: never[]) => unknown as never,
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mis-typed option records are part of the generated input space and must reach upstream unvalidated, exactly like its untyped runtime
      options as never,
    );
    return await Reflect.apply(
      wrapped as (...callArguments: never[]) => unknown,
      undefined,
      [...args,],
    );
  },
  iterate: async function iterateUpstream(
    fn: (...callArguments: never[]) => unknown,
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- options records are possibly absent by construction; the adapters forward them unvalidated exactly like both untyped runtimes
    options: Record<string, unknown> | undefined,
    args: readonly unknown[],
  ): Promise<readonly unknown[]> {
    /**
     Wrapped iterable under comparison.
     */
    const wrapped = makeAsynchronousIterableUpstream(
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- fixture generators use the fork's narrow `never[]` signature; upstream accepts any iterable-function shape and serializes it the same way
      fn as (...callArguments: never[]) => unknown as never,
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mis-typed option records are part of the generated input space and must reach upstream unvalidated, exactly like its untyped runtime
      options as never,
    );
    /**
     Values drained in yield order.
     */
    const values: unknown[] = [];
    /**
     Iterable drained through the rest-argument shape.
     */
    const iterable: unknown = Reflect.apply(
      wrapped as (...callArguments: never[]) => unknown,
      undefined,
      [...args,],
    );
    if (!isAsyncIterable(iterable,))
      throw new TypeError('Upstream adapter did not return an async iterable',);
    for await (const value of iterable)
      values.push(value,);
    return values;
  },
};

//endregion Adapters
