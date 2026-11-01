/**
 `makeAsynchronousIterable` wrapper factory for the make-asynchronous fork.
 
 Runs a serialized iterable-returning function in a worker thread: the first
 message runs the function and normalizes its return into an iterator, later
 messages pull the next item. Each iteration spawns its own worker, so
 iterator state never leaks between iterations. The wrapped function is
 serialized, so it cannot close over outer variables or imports; callers
 pass everything through arguments instead.
 
 Derived from [`make-asynchronous`](https://github.com/sindresorhus/make-asynchronous)
 by Sindre Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README
 for the full attribution. Behavior matches `make-asynchronous` 2.1.0.
 
 @module
 */

import type { MakeAsynchronousOptions, } from './options.ts';
import { getResult, } from './result.ts';
import {
  createWorker,
  type LiveWorker,
} from './worker-lifecycle.ts';
import {
  makeIterableWorkerBody,
  nodeWorkerPreambleSource,
} from './worker-source.ts';

//region Types

/**
 Call arguments accepted at the external iterable boundary.
 
 `never[]` keeps every concrete argument tuple assignable while
 `Parameters<Wrapped>` still resolves to the concrete tuple.
 */
type IterableArguments = never[];
/**
 Any iterable-returning function the fork can wrap: sync or async iterable,
 any arguments.
 
 @example
 ```ts
 const fn: IterableFunction = function * (count: number,): Generator<number> {
   yield count;
 };
 ```
 */
export type IterableFunction = (...callArguments: IterableArguments) => AsyncIterable<unknown> | Iterable<unknown>;

/**
 Value type yielded by one iterable-returning function.
 
 Replaces upstream's `type-fest`-based `IterableFunctionValue` so the fork
 keeps zero runtime and type dependencies.
 
 @typeParam Wrapped - iterable-returning function whose value is derived
 
 @example
 ```ts
 type Value = IterableValue<() => Generator<number>>; // => number
 ```
 */
export type IterableValue<Wrapped extends IterableFunction> = Wrapped extends (...callArguments: never[]) => infer Settled
  ? Settled extends AsyncIterable<infer AsyncValue> | Iterable<infer SyncValue>
    ? AsyncValue | SyncValue
    : unknown
  : unknown;

/**
 Async-iterable form of one wrapped iterable function: identical arguments,
 async-iterable return of the same value type.
 
 Replaces upstream's `type-fest` `SetReturnType` import so the fork keeps
 zero runtime and type dependencies.
 
 @typeParam Wrapped - iterable-returning function whose async form is derived
 
 @example
 ```ts
 type AsyncGen = AsyncIterableForm<() => Generator<number>>;
 // => (args: { args: [] }) => AsyncIterable<number>
 ```
 */
export type AsyncIterableForm<Wrapped extends IterableFunction> = (call: IterableCall<Wrapped>,) => AsyncIterable<
  IterableValue<Wrapped>
  >;

/**
 Wrapped iterable function plus its abort-signal variant.
 
 The returned function carries `withSignal`, which binds one `AbortSignal`
 and returns the same async-iterable form.
 
 @typeParam Wrapped - iterable-returning function being wrapped
 
 @example
 ```ts
 const fn = makeAsynchronousIterable({
   fn: function * (count: number,): Generator<number> {
     yield count;
   },
 });
 for await (const value of fn({ args: [2], })) {
   console.log(value,);
 }
 ```
 */
export type AsyncIterableWrapped<Wrapped extends IterableFunction> = AsyncIterableForm<Wrapped> & {
  /**
   Binds one abort signal to the same async-iterable form.
   
   @param signal - Signal failing the in-flight worker with its reason.
   
   @returns Async-iterable form failing with the signal's reason on abort.
   */
  readonly withSignal: (signal: AbortSignal,) => AsyncIterableForm<Wrapped>;
};

/**
 One iteration call: the argument tuple posted with the first message.
 
 The `args` tuple replaces upstream `make-asynchronous`'s
 `fn(...arguments_)` rest parameter, which repository lint bans; the worker
 still receives the tuple spread as real arguments on its first message.
 
 @typeParam Wrapped - wrapped function whose arguments are mirrored
 
 @example
 ```ts
 const call: IterableCall<(count: number,) => Generator<number>> = {
   args: [2],
 };
 ```
 */
export type IterableCall<Wrapped extends IterableFunction> = {
  /**
   Arguments cloned into the worker with the first pull message.
   */
  readonly args: Parameters<Wrapped>;
};

//endregion Types

//region Factory

/**
 Wraps one iterable-returning function so every iteration drains in a worker
 thread.
 
 Each iteration builds its own worker from the serialized function source,
 so iterator state never leaks between iterations. Later pull messages carry
 no arguments, skipping re-cloning for every item exactly like upstream.
 Cleanup terminates the worker through the `using` disposer, which runs on
 every exit from the iteration below, including early `break` and `return`.
 
 @param fn - Iterable-returning function serialized into the worker.
 
 @param options - Base URL wiring for bare dynamic imports.
 
 @returns Async-iterable form of the function plus `withSignal`.
 
 @example
 ```ts
 import { makeAsynchronousIterable, } from '\@monochromatic-dev/upstream-fork-make-asynchronous';
 
 const fn = makeAsynchronousIterable({
   fn: function * (count: number,): Generator<number> {
     yield count;
   },
 });
 for await (const value of fn({ args: [2], })) {
   console.log(value,);
 }
 ```
 */
export function makeAsynchronousIterable<const Wrapped extends IterableFunction>(
  {
    fn,
    options,
  }: {
    /**
     Iterable-returning function serialized into the worker; must not close
     over outer scope.
     */
    readonly fn: Wrapped;
    /**
     Base URL wiring for bare dynamic imports in Node.js workers.
     */
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream make-asynchronous 2.1.0 `options` parameter, whose explicitly-undefined callers must still typecheck
    readonly options?: MakeAsynchronousOptions | undefined;
  },
): AsyncIterableWrapped<Wrapped> {
  /**
   Worker module source draining one wrapped iterable across messages.
   */
  const content = makeIterableWorkerBody(fn.toString(),);

  /**
   Pulls worker replies until `done`, yielding each value.
   
   @param callArguments - Tuple cloned with the first pull message.
   
   @param signal - Abort signal failing the worker with its reason.
   
   @returns Async generator yielding the worker's values in order.
   */
  async function* iterate(
    {
      callArguments,
      signal,
    }: {
      readonly callArguments: Parameters<Wrapped>;
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream make-asynchronous 2.1.0 iterate `signal` parameter, whose explicitly-undefined callers must still typecheck
      readonly signal?: AbortSignal | undefined;
    },
  ): AsyncGenerator<IterableValue<Wrapped>> {
    /**
     Live worker terminated when this iteration exits.
     */
    using worker: LiveWorker = await createWorker({
      content,
      nodePreamble: nodeWorkerPreambleSource,
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream make-asynchronous 2.1.0 `options` parameter, whose explicitly-undefined callers must still typecheck
      options: options as MakeAsynchronousOptions | undefined,
      signal,
    },);

    // The wrapped function only runs for the first message, so the later
    // ones carry no arguments and skip re-cloning them for every item.
    /**
     Arguments for the next pull: the call tuple once, then absent.
     */
    const pendingArguments: {
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- later pull messages carry no arguments, so absence must travel as a value exactly like upstream passing `undefined`
      next?: Parameters<Wrapped> | undefined;
    } = { next: callArguments, };

    /* oxlint-disable no-await-in-loop -- the consumer pulls worker replies one at a time, in yield order */
    for (;;) {
      /**
       Next iterator result restored from the worker's reply.
       */
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- worker reply carries the wrapped iterator's next result through structured clone
      const outcome = getResult(await worker.request(pendingArguments.next,),) as IteratorResult<
        IterableValue<Wrapped>,
        unknown
        >;
      delete pendingArguments.next;

      if (outcome.done === true)
        break;

      yield outcome.value;
    }
  }

  /**
   Iterates the wrapped function with one argument tuple under a fresh
   worker.
   
   @param call - Argument tuple cloned into the worker.
   
   @returns Async iterable draining the worker's iterator.
   */
  function wrapped(call: IterableCall<Wrapped>,): AsyncIterable<IterableValue<Wrapped>> {
    return {
      [Symbol.asyncIterator]: function startIteration(): AsyncGenerator<IterableValue<Wrapped>> {
        return iterate({ callArguments: call.args, },);
      },
    };
  }

  /**
   Binds one abort signal to the same iteration shape.
   
   @param signal - Signal failing the in-flight worker with its reason.
   
   @returns Async-iterable form failing with the signal's reason on abort.
   */
  function withSignal(signal: AbortSignal,): AsyncIterableForm<Wrapped> {
    /**
     Iterates the wrapped function under the bound signal.
     
     @param call - Argument tuple cloned into the worker.
     
     @returns Async iterable draining the worker's iterator.
     */
    function signaled(call: IterableCall<Wrapped>,): AsyncIterable<IterableValue<Wrapped>> {
      return {
        [Symbol.asyncIterator]: function startSignaledIteration(): AsyncGenerator<
          IterableValue<Wrapped>
          > {
          return iterate({
            callArguments: call.args,
            signal,
          },);
        },
      };
    }

    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- `signaled` takes `{ args }` while the public form takes the same shape through `IterableCall`
    return signaled as unknown as AsyncIterableForm<Wrapped>;
  }

  /**
   `withSignal` attached exactly as upstream `make-asynchronous` attaches
   it: a plain assignment, so the member stays enumerable, writable, and
   configurable and `Object.keys` reports `['withSignal']`.
   */
  wrapped.withSignal = withSignal;

  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- `wrapped` gains its withSignal member through assignment above
  return wrapped as unknown as AsyncIterableWrapped<Wrapped>;
}

//endregion Factory
