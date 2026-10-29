/**
 Wrap one callback-style function so every call returns a promise.
 
 Derived from [`pify`](https://github.com/sindresorhus/pify) by Sindre Sorhus
 (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Call semantics match `pify` 6.1.0's `processFunction`: the
 wrapped function runs synchronously inside the promise constructor (so a
 synchronous throw becomes a rejection), receives its callback after the
 caller's arguments, and sees the proxy's target as `this` when the call came
 through the proxy. The only API-shape deviation is lint-mandated: calls pass
 an `args` tuple instead of a rest parameter.
 
 @module
 */

import type {
  PromisifyOptions,
  ResolvedPifyOptions,
} from './pify-options.ts';
import type {
  DropLastArrayElement,
  EmptyTuple,
  LastArrayElement,
} from './type-helpers.ts';

//region Types

/**
 One call to a promisified function: the argument tuple forwarded to the
 wrapped function before its callback.
 
 The `args` tuple replaces upstream `pify`'s `promisified(...args)` rest
 parameter, which repository lint bans.
 
 @typeParam Args - tuple of forwarded argument types
 
 @example
 ```ts
 const call: PromisifiedCall<[string, string]> = {
   args: ['package.json', 'utf8'],
 };
 ```
 */
export type PromisifiedCall<Args extends readonly unknown[]> = {
  /**
   Arguments forwarded to the wrapped function before its callback.
   */
  readonly args: Args;
};

/**
 Promisified form of one wrapped function's signature: takes the wrapped
 function's argument tuple and resolves with the shape its trailing callback
 implies under the active `multiArgs` / `errorFirst` options.
 
 Mirrors upstream `pify`'s `Promisify` modulo this fork's `{ args }` call
 shape, and keeps upstream's result modeling: a callback with a single
 parameter under `errorFirst` always rejects (that parameter is the error), so
 its result stays `unknown`; a function without a trailing callback returns a
 promise that never settles, also `unknown`.
 
 @typeParam Args - full parameter tuple of the wrapped function, trailing
 callback included
 
 @typeParam GenericOptions - option slots deciding the result shape
 
 @example
 ```ts
 type ReadFile = Promisify<
   [string, (error: Error, value: string) => void],
   PromisifyOptions<EmptyTuple, EmptyTuple, false, true>
 >;
 // => (call: { args: [string] }) => Promise<string>
 ```
 */
export type Promisify<
  Args extends readonly unknown[],
  GenericOptions extends PromisifyOptions<readonly unknown[], readonly unknown[], boolean, boolean>,
  > = (
    call: PromisifiedCall<DropLastArrayElement<Args>>,
  ) => LastArrayElement<Args> extends (...callbackResults: infer CallbackResults) => unknown
    ? CallbackResults extends [infer SingleCallbackArg]
      ? GenericOptions extends { errorFirst: true } ? Promise<unknown> : Promise<SingleCallbackArg>
      : Promise<
        GenericOptions extends { multiArgs: false }
          ? LastArrayElement<CallbackResults>
          : CallbackResults
        >
    : Promise<unknown>;

/**
 Runtime shape of one promisified wrapper: the precise result typing lives in
 the `pify` overloads, so the factory returns this wide shape.
 */
export type PromisifiedWrapper = (call: PromisifiedCall<readonly unknown[]>,) => Promise<unknown>;

/**
 Callback-style function shape the wrapper factory accepts: any callable,
 since the trailing callback arrives through this fork's `args` tuple plus the
 appended callback and not through a typed parameter.
 */
export type WrappedFunction = (...args: readonly unknown[]) => unknown;

/**
 Node-style result callback handed to the wrapped function. Its variadic
 signature is dictated by the node callback convention: wrapped functions
 invoke it with a leading error argument plus any number of trailing result
 arguments.
 */
export type NodeStyleCallback = (...callbackResults: readonly unknown[]) => void;

//endregion Types

//region Callbacks

/**
 Builds the `multiArgs` result callback: collects the wrapped function's whole
 result list, matching upstream `pify`'s `(...result)` callback.
 
 @param resolve - Promise resolver to settle on success.
 
 @param reject - Promise rejecter to settle on failure.
 
 @param errorFirst - Whether the first callback argument is the error.
 
 @returns Callback collecting every callback argument.
 
 @example
 ```ts
 const callback = createMultiArgsCallback({ resolve, reject, errorFirst: true, });
 ```
 */
export function createMultiArgsCallback(
  {
    resolve,
    reject,
    errorFirst,
  }: {
    readonly resolve: (value: unknown) => void;
    readonly reject: (reason: unknown) => void;
    readonly errorFirst: boolean;
  },
): NodeStyleCallback {
  return function collectResults(): void {
    /* oxlint-disable eslint/prefer-rest-params -- External-boundary mirror: node-style callbacks deliver their result list as free-form trailing arguments (upstream `pify` collects `(...result)`), and the repository's rest-parameter ban makes the arguments object the only closure capture of that list; both rule sources are cited in DECISION.callback-capture.md. */
    /**
     Every callback argument, captured as one fresh array exactly like
     upstream `pify`'s rest-parameter capture.
     */
    const results: unknown[] = Array.from(arguments,);
    /* oxlint-enable eslint/prefer-rest-params */

    if (errorFirst) {
      /**
     Whether the callback reported a truthy leading error argument; the
   truthiness coercion mirrors upstream `pify`'s `if (result[0])` exactly.
   */
      const errorReported = Boolean(results[0],);
      if (errorReported)
        reject(results,);
      else
        resolve(results.slice(1,),);
    }
    else {
      resolve(results,);
    }
  };
}

/**
 Builds the error-first, single-result callback: matching upstream `pify`'s
 `(error, result)` callback.
 
 @param resolve - Promise resolver to settle on success.
 
 @param reject - Promise rejecter to settle on failure.
 
 @returns Callback forwarding a leading error and one result.
 
 @example
 ```ts
 const callback = createErrorFirstCallback({ resolve, reject, });
 ```
 */
export function createErrorFirstCallback(
  {
    resolve,
    reject,
  }: {
    readonly resolve: (value: unknown) => void;
    readonly reject: (reason: unknown) => void;
  },
): NodeStyleCallback {
  return function collectErrorFirst(
    error: unknown,
    result: unknown,
  ): void {
    /**
     Whether the callback reported a truthy leading error argument; the
     truthiness coercion mirrors upstream `pify`'s `if (error)` exactly.
     */
    const errorReported = Boolean(error,);
    if (errorReported)
      reject(error,);
    else
      resolve(result,);
  };
}

/**
 Chooses the node-style callback handed to the wrapped function: upstream
 `pify`'s three `multiArgs` / `errorFirst` branches, including forwarding
 `resolve` itself for the non-`multiArgs`, non-`errorFirst` case.
 
 @param options - Resolved options selecting the callback shape.
 
 @param resolve - Promise resolver to settle on success.
 
 @param reject - Promise rejecter to settle on failure.
 
 @returns Callback matching the configured shape.
 
 @example
 ```ts
 const callback = selectCallback({ options, resolve, reject, });
 ```
 */
export function selectCallback(
  {
    options,
    resolve,
    reject,
  }: {
    readonly options: ResolvedPifyOptions;
    readonly resolve: (value: unknown) => void;
    readonly reject: (reason: unknown) => void;
  },
): NodeStyleCallback {
  if (options.multiArgs)
    return createMultiArgsCallback({
      resolve,
      reject,
      errorFirst: options.errorFirst,
    },);
  if (options.errorFirst)
    return createErrorFirstCallback({
      resolve,
      reject,
    },);
  return resolve;
}

//endregion Callbacks

//region Factory

/**
 Builds one promisified wrapper around a callback-style function.
 
 @param fn - Callback-style function to wrap.
 
 @param options - Resolved options read at call time exactly like upstream
 `pify`, so mis-typed option values fail where and how upstream fails them.
 
 @param proxy - Proxy through which the call may have arrived.
 
 @param unwrapped - Proxy target handed to `fn` as `this` when the call came
 through the proxy.
 
 @returns Wrapper whose calls return promises settling with `fn`'s callback
 outcome.
 
 @example
 ```ts
 const promisified = createPromisifiedFunction({
   fn: function read(path: string, callback: (error: unknown, value: string) => void): void {
     callback(null, path);
   },
   options: resolvePifyOptions({}),
   proxy,
   unwrapped: target,
 });
 await promisified({ args: ['file.txt'], });
 ```
 */
export function createPromisifiedFunction(
  {
    fn,
    options,
    proxy,
    unwrapped,
  }: {
    readonly fn: WrappedFunction;
    readonly options: ResolvedPifyOptions;
    readonly proxy: object;
    readonly unwrapped: object;
  },
): PromisifiedWrapper {
  /**
   Promisified wrapper replacing `fn` in the proxy's view. Non-arrow and
   non-declaration on purpose: upstream `pify` reads the caller's `this` to
   decide between the proxy's target and an explicit receiver (only a plain
   function receives call-site `this`), while repository lint keeps
   multi-parameter function declarations destructured.
   */
  return function promisified(
    this: unknown,
    call: PromisifiedCall<readonly unknown[]>,
  ): Promise<unknown> {
    /**
     Receiver forwarded to `fn`: the unwrapped proxy target when the call
     arrived through the proxy (so wrapped methods keep their owning object),
     the caller's `this` otherwise. Mirrors upstream `pify`'s
     `this === proxy ? unwrapped : this`.
     */
    const self = this === proxy ? unwrapped : this;
    /**
     Promise constructor from the resolved options, read per call exactly like
     upstream `pify`.
     */
    const P = options.promiseModule;
    return new P(function executor(
      resolve: (value: unknown) => void,
      reject: (reason: unknown) => void,
    ): void {
      /**
       Node-style callback chosen by the resolved options exactly like
       upstream `pify`'s three branches.
       */
      const callback = selectCallback({
        options,
        resolve,
        reject,
      },);
      /**
       Arguments forwarded to `fn`: the caller's tuple plus the callback. A
       fresh array, so `fn` mutating its argument list never reaches the
       caller's tuple (upstream `pify` forwards its own fresh rest array).
       */
      const forwardedArguments = [
        ...call.args,
        callback,
      ];
      Reflect.apply(
        fn,
        self,
        forwardedArguments,
      );
    },);
  };
}

//endregion Factory
