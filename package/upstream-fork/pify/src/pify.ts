/**
 TypeScript fork of `pify`: promisify a callback-style function or module.
 
 Derived from [`pify`](https://github.com/sindresorhus/pify) by Sindre Sorhus
 (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Behavior matches `pify` 6.1.0: a `Proxy` view whose function
 members return promises, member selection through `include` / `exclude` with
 upstream's exact quirks, `this` unwrapping from the proxy to its target, and
 upstream's option merge semantics. The only API-shape deviation is
 lint-mandated: `pify({ input, options })` takes one destructured object and
 promisified calls pass an `args` tuple.
 
 @module
 */

import { InvalidInputError, } from './errors.ts';
import { createKeyFilter, } from './key-filter.ts';
import {
  resolvePifyOptions,
  type PifyOptions,
  type PifyOptionsInput,
  type PromisifyOptions,
} from './pify-options.ts';
import {
  createPromisifiedFunction,
  type Promisify,
  type PromisifiedWrapper,
  type WrappedFunction,
} from './promisify-function.ts';
import type {
  EmptyTuple,
  StringEndsWith,
} from './type-helpers.ts';

//region Types

/**
 Promisified view of a module object: function members become promisified
 wrappers, `include` / `exclude` selections and the default `Sync` / `Stream`
 suffix exclusion pass their members through untouched, and non-function
 members are preserved.
 
 Mirrors upstream `pify`'s `PromisifyModule` modulo this fork's `{ args }`
 call shape. One type-level fix over upstream's `index.d.ts`: `errorFirst` is
 actually forwarded to the result computation (upstream silently drops it).
 
 @typeParam Module - module object whose members are mapped
 
 @typeParam MultiArgs - whether callbacks report their whole result list
 
 @typeParam ErrorFirst - whether callbacks report an error first argument
 
 @typeParam Includes - tuple of member keys selected for promisification
 
 @typeParam Excludes - tuple of member keys left untouched
 */
export type PromisifiedModule<
  Module extends object,
  MultiArgs extends boolean,
  ErrorFirst extends boolean,
  Includes extends readonly (keyof Module)[],
  Excludes extends readonly (keyof Module)[],
  > = {
    [Key in keyof Module]: Module[Key] extends (...callbackResults: infer CallbackArguments) => unknown
      ? Key extends Includes[number]
        ? Promisify<CallbackArguments, PromisifyOptions<Includes, Excludes, MultiArgs, ErrorFirst>>
        : Key extends Excludes[number]
          ? Module[Key]
          : StringEndsWith<Key, 'Sync' | 'Stream'> extends true
            ? Module[Key]
            : Promisify<CallbackArguments, PromisifyOptions<Includes, Excludes, MultiArgs, ErrorFirst>>
      : Module[Key];
  };

//endregion Types

//region Callable narrowing

/**
 Whether an unknown member value is callable: runtime narrowing for the
 `get` trap's function-member branch.
 
 @param value - Member value read from the target.
 
 @returns Whether the value can be wrapped as a callback-style function.
 
 @example
 ```ts
 isCallable(function read(): void {}); // => true
 isCallable(3); // => false
 ```
 */
function isCallable(value: unknown): value is WrappedFunction {
  return (typeof value) === 'function';
}

//endregion Callable narrowing

//region pify

/**
 Promisifies a callback-style function: every call returns a promise settling
 with the callback's outcome.
 
 @param input - Callback-style function to promisify.
 
 @param options - Upstream `pify` options; omitted means the defaults.
 
 @returns Promisified function taking the wrapped function's `args` tuple.
 
 @throws InvalidInputError when `input` is neither a function nor an object.
 
 @example
 ```ts
 import { pify, } from '\@monochromatic-dev/upstream-fork-pify';
 
 const readFile = pify({ input: nodeFs.readFile, });
 const data = await readFile({ args: ['package.json', 'utf8'], });
 ```
 */
export function pify<
  FirstArgument,
  Arguments extends readonly unknown[],
  MultiArgs extends boolean = false,
  ErrorFirst extends boolean = true,
  >(
  {
    input,
    options,
  }: {
    readonly input: (
      argument: FirstArgument,
      ...arguments_: Arguments
    ) => unknown;
    readonly options?: PifyOptions<EmptyTuple, EmptyTuple, MultiArgs, ErrorFirst>;
  },
  ): Promisify<[
    FirstArgument,
    ...Arguments
  ], PromisifyOptions<EmptyTuple, EmptyTuple, MultiArgs, ErrorFirst>>;
/**
 Promisifies a module object (or a function module): selected function members
 become promisified wrappers while the rest of the object is preserved.
 
 @param input - Module object or function module whose members are promisified.
 
 @param options - Upstream `pify` options; omitted means the defaults.
 
 @returns Proxy view of the module with promisified members.
 
 @throws InvalidInputError when `input` is neither a function nor an object.
 
 @example
 ```ts
 import { pify, } from '\@monochromatic-dev/upstream-fork-pify';
 
 const fs = pify({ input: nodeFs, });
 const data = await fs.readFile({ args: ['package.json', 'utf8'], });
 ```
 */
export function pify<
  Module extends object,
  Includes extends readonly (keyof Module)[] = EmptyTuple,
  Excludes extends readonly (keyof Module)[] = EmptyTuple,
  MultiArgs extends boolean = false,
  ErrorFirst extends boolean = true,
  >(
  {
    input,
    options,
  }: {
    readonly input: Module;
    readonly options?: PifyOptions<Includes, Excludes, MultiArgs, ErrorFirst, true>;
  },
  ): PromisifiedModule<Module, MultiArgs, ErrorFirst, Includes, Excludes>;
/**
 Implementation behind both `pify` overloads.
 
 The options merge runs before input validation exactly like upstream `pify`,
 so a mis-typed options record fails where upstream fails it.
 
 @param input - Callback-style function, module object, or function module.
 
 @param options - Upstream `pify` options; omitted means the defaults.
 
 @returns Proxy view whose function members return promises.
 
 @throws InvalidInputError when `input` is neither a function nor an object.
 
 @example
 ```ts
 pify({ input: nodeFs.readFile, },);
 ```
 */
export function pify(
  {
    input,
    options,
  }: {
    readonly input: object | WrappedFunction;
    readonly options?: PifyOptionsInput;
  },
): unknown {
  /**
   Options with upstream `pify`'s defaults merged in, resolved first exactly
   like upstream `pify` resolves its options before validating the input.
   */
  const resolvedOptions = resolvePifyOptions({ options, },);

  if (!((input !== null) && (((typeof input) === 'object') || ((typeof input) === 'function'))))
    throw new InvalidInputError(input,);

  /**
   Member-selection filter shared by the `get` trap, backed by upstream
   `pify`'s module-level decision cache.
   */
  const keyFilter = createKeyFilter({ options: resolvedOptions, },);
  /**
   Promisified wrappers memoized per wrapped function, exactly like upstream
   `pify`'s per-call cache: members sharing one function share one wrapper,
   and the proxy's own `apply` shares the wrapper with a member holding the
   target function itself.
   */
  const cache = new WeakMap<object, PromisifiedWrapper | WrappedFunction>();

  /**
   Proxy view returned to the caller: the input object (or function) with the
   `apply` / `get` traps installed, exactly like upstream `pify`'s proxy.
   */
  const proxy = new Proxy(
    input,
    {
    /**
     Handles proxy calls: promisifies the target itself unless `excludeMain`
     keeps it raw.
     
     @param target - Proxy target the call arrived on.
     
     @param thisArgument - Receiver of the proxy call.
     
     @param argumentsList - Arguments of the proxy call.
     
     @returns Promisified call result, or the raw call under `excludeMain`.
     */
      apply: function applyTrap(
        target: WrappedFunction,
        thisArgument: unknown,
        argumentsList: readonly unknown[],
      ): unknown {
      /**
       Cached promisified wrapper for this target, built on first call.
       */
        const cached = cache.get(target,);
        if (cached)
          return Reflect.apply(
            cached,
            thisArgument,
            argumentsList,
          );

      /**
       Promisified wrapper for the target, or the raw target under
       `excludeMain` (upstream `pify` returns the target through the trap the
       same way).
       */
        const pified = resolvedOptions.excludeMain
          ? target
          : createPromisifiedFunction({
            fn: target,
            options: resolvedOptions,
            proxy,
            unwrapped: target,
          },);
        cache.set(
          target,
          pified,
        );
        return Reflect.apply(
          pified,
          thisArgument,
          argumentsList,
        );
      },

    /**
     Handles proxy member reads: promisifies selected function members and
     passes everything else through untouched.
     
     @param target - Proxy target whose member is read.
     
     @param key - Member key being read.
     
     @returns Promisified wrapper for selected function members, the member
     itself otherwise.
     */
      get: function getTrap(
        target: object,
        key: PropertyKey
      ): unknown {
      /**
       Current member value, read before selection exactly like upstream
       `pify` (so accessor side effects fire in the same order).
       */
        const property: unknown = Reflect.get(
          target,
          key,
        );
        if ((!keyFilter({
          target,
          key,
        },)) || (property === Reflect.get(
          Function.prototype,
          key,
        )))
          return property;

        if (isCallable(property,)) {
        /**
         Cached promisified wrapper for this function, built on first read.
         */
          const cached = cache.get(property,);
          if (cached)
            return cached;

        /**
         Promisified wrapper replacing this function member.
         */
          const pified = createPromisifiedFunction({
            fn: property,
            options: resolvedOptions,
            proxy,
            unwrapped: target,
          },);
          cache.set(
            property,
            pified,
          );
          return pified;
        }

        return property;
      },
    },
  );

  return proxy;
}

//endregion pify
