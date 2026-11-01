/**
 `makeAsynchronous` wrapper factory for the make-asynchronous fork.
 
 Runs a serialized function in a worker thread without blocking the main
 thread: every call spawns one worker, posts its argument tuple, awaits the
 single reply, restores the result, and terminates the worker. The wrapped
 function is serialized, so it cannot close over outer variables or
 imports; callers pass everything through arguments instead.
 
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
  makeCallWorkerBody,
  nodeWorkerPreambleSource,
} from './worker-source.ts';

//region Types

/* oxlint-disable typescript/no-explicit-any -- external-boundary mirror of upstream make-asynchronous's `AnyFunction`, whose rest parameter and return are both `any` */
/**
 Call arguments accepted at the external function boundary.
 */
type AnyArguments = any[];
/**
 Return value accepted at the external function boundary.
 */
type AnySettled = any;
/**
 Any function the fork can wrap: sync or async, any arguments, any return.
 
 @example
 ```ts
 const fn: AnyFunction = function double(value: number,): number {
   return value * 2;
 };
 ```
 */
export type AnyFunction = (...callArguments: AnyArguments) => AnySettled;
/* oxlint-enable typescript/no-explicit-any */

/**
 Awaited return of one wrapped function.
 
 @typeParam Wrapped - function whose settled value is derived
 
 @example
 ```ts
 type Settled = AsyncSettled<(value: number,) => number>; // => number
 ```
 */
export type AsyncSettled<Wrapped extends AnyFunction> = Wrapped extends (...callArguments: never[]) => infer Settled
  ? Awaited<Settled>
  : never;



/**
 Async form of one wrapped function: identical arguments, awaited return.
 
 Replaces upstream's `type-fest` `Asyncify` import so the fork keeps zero
 runtime and type dependencies.
 
 @typeParam Wrapped - function whose async form is derived
 
 @example
 ```ts
 type AsyncDouble = AsyncForm<(value: number,) => number>;
 // => (value: number,) => Promise<number>
 ```
 */
export type AsyncForm<Wrapped extends AnyFunction> = Wrapped extends (
  ...callArguments: infer CallArguments
) => infer Settled
  ? (call: AsyncCall<Wrapped & ((...callArguments: CallArguments) => Settled)>,) => Promise<Awaited<Settled>>
  : never;

/**
 Wrapped function plus its abort-signal variant.
 
 The returned function carries `withSignal`, which binds one `AbortSignal`
 and returns the same async form.
 
 @typeParam Wrapped - function whose async form is exposed
 
 @example
 ```ts
 const fn: AsyncWrapped<(value: number,) => number> = makeAsynchronous({
   fn: function double(value: number,): number {
     return value * 2;
   },
 });
 await fn({ args: [2], });
 await fn.withSignal(controller.signal)({ args: [2], });
 ```
 */
export type AsyncWrapped<Wrapped extends AnyFunction> = AsyncForm<Wrapped> & {
  /**
   Binds one abort signal to the same async form. Assigned, not defined,
   exactly like upstream, so the member stays enumerable, writable, and
   configurable.
   
   @param signal - Signal failing the in-flight worker with its reason.
   
   @returns Async form failing with the signal's reason on abort.
   */
  withSignal: (signal: AbortSignal,) => AsyncForm<Wrapped>;
};

/**
 One call to a wrapped function: the argument tuple posted to the worker.
 
 The `args` tuple replaces upstream `make-asynchronous`'s
 `fn(...arguments_)` rest parameter, which repository lint bans; the worker
 still receives the tuple spread as real arguments.
 
 @typeParam Wrapped - wrapped function whose arguments are mirrored
 
 @example
 ```ts
 const call: AsyncCall<(value: number,) => number> = { args: [2], };
 ```
 */
export type AsyncCall<Wrapped extends AnyFunction> = Wrapped extends (
  ...callArguments: infer CallArguments
) => unknown
  ? {
    /**
     Arguments cloned into the worker and spread into the wrapped function.
     */
    readonly args: CallArguments;
  }
  : never;

//endregion Types

//region Factory

/**
 Wraps one function so every call runs in a worker thread.
 
 Each call builds its own worker from the serialized function source, so
 concurrent calls never share worker state. Cleanup terminates the worker
 through the `using` disposer, which runs on every exit from the call
 below.
 
 @param fn - Function to serialize into the worker.
 
 @param options - Base URL wiring for bare dynamic imports.
 
 @returns Async form of the function plus `withSignal`.
 
 @example
 ```ts
 import { makeAsynchronous, } from '\@monochromatic-dev/upstream-fork-make-asynchronous';
 
 const fn = makeAsynchronous({
   fn: function double(value: number,): number {
     return value * 2;
   },
 });
 await fn({ args: [2], }); // => 4
 ```
 */
export function makeAsynchronous<const Wrapped extends AnyFunction>(
  {
    fn,
    options,
  }: {
    /**
     Function serialized into the worker; must not close over outer scope.
     */
    readonly fn: Wrapped;
    /**
     Base URL wiring for bare dynamic imports in Node.js workers.
     */
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream make-asynchronous 2.1.0 `options` parameter, whose explicitly-undefined callers must still typecheck
    readonly options?: MakeAsynchronousOptions | undefined;
  },
): AsyncWrapped<Wrapped> {
  /**
   Worker module source running one wrapped call per message.
   */
  const content = makeCallWorkerBody(fn.toString(),);

  /**
   Runs one argument tuple through a fresh worker and restores its result.
   
   @param call - Argument tuple cloned into the worker.
   
   @param signal - Abort signal failing the worker with its reason.
   
   @returns Promise settling with the worker's restored result.
   */
  async function run(
    {
      call,
      signal,
    }: {
      readonly call: AsyncCall<Wrapped>;
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream make-asynchronous 2.1.0 run `signal` parameter, whose explicitly-undefined callers must still typecheck
      readonly signal?: AbortSignal | undefined;
    },
  ): Promise<AsyncSettled<Wrapped>> {
    /**
     Live worker terminated when this call exits.
     */
    using worker: LiveWorker = await createWorker({
      content,
      nodePreamble: nodeWorkerPreambleSource,
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream make-asynchronous 2.1.0 `options` parameter, whose explicitly-undefined callers must still typecheck
      options: options as MakeAsynchronousOptions | undefined,
      signal,
    },);

    /**
     Worker reply carrying the wrapped function's awaited return through
     structured clone.
     */
    const settled: unknown = getResult(await worker.request(call.args,),);
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion, typescript/no-unsafe-return -- worker reply carries the wrapped function's awaited return through structured clone; the return narrows `unknown` to that same shape
    return settled as AsyncSettled<Wrapped>;
  }

  /**
   Calls the wrapped function with one argument tuple under a fresh worker.
   
   @param call - Argument tuple cloned into the worker.
   
   @returns Promise settling with the worker's restored result.
   */
  async function wrapped(
    call: AsyncCall<Wrapped>,
  ): Promise<AsyncSettled<Wrapped>> {
    // oxlint-disable-next-line typescript/no-unsafe-return -- `run` settles the wrapped function's awaited return; the generic `any` inside `AsyncSettled` is upstream's `AnyFunction` boundary
    return await run({ call, },);
  }

  /**
   Binds one abort signal to the same call shape.
   
   @param signal - Signal failing the in-flight worker with its reason.
   
   @returns Async form failing with the signal's reason on abort.
   */
  function withSignal(signal: AbortSignal,): AsyncForm<Wrapped> {
    /**
     Calls the wrapped function under the bound signal.
     
     @param call - Argument tuple cloned into the worker.
     
     @returns Promise settling with the worker's restored result.
     */
    async function signaled(
      call: AsyncCall<Wrapped>,
    ): Promise<AsyncSettled<Wrapped>> {
      // oxlint-disable-next-line typescript/no-unsafe-return -- `run` settles the wrapped function's awaited return; the generic `any` inside `AsyncSettled` is upstream's `AnyFunction` boundary
      return await run({
        call,
        signal,
      },);
    }

    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- `signaled` takes `{ args }` while the public form takes the same shape through `AsyncCall`
    return signaled as AsyncForm<Wrapped>;
  }

  /**
   `withSignal` attached exactly as upstream `make-asynchronous` attaches
   it: a plain assignment, so the member stays enumerable, writable, and
   configurable and `Object.keys` reports `['withSignal']`.
   */
  wrapped.withSignal = withSignal;

  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- `wrapped` gains its withSignal member through assignment above
  return wrapped as unknown as AsyncWrapped<Wrapped>;
}

//endregion Factory
