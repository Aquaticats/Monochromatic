/**
 Adapters exposing one uniform wrap-and-call surface over this package's
 fork and upstream `pify`, so the same generated spec can drive both despite
 their different call shapes.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import pifyUpstream from 'pify';

import { pify, } from '@monochromatic-dev/upstream-fork-pify/ts';

//region Types

/**
 One pified view under test: members are called through the adapter's shape.
 */
export type PifiedView = object;

/**
 Comparable description of a wrap that threw instead of returning a view.
 */
export type WrapFailure = {
  /**
   Text of the thrown value, via `caughtValueText` on both sides.
   */
  readonly thrownMessage: string;
  /**
   Whether the thrown value was a `TypeError`, the only throw class both
   implementations share (the fork throws its `InvalidInputError`
   subclass).
   */
  readonly thrownIsTypeError: boolean;
};

/**
 One implementation's wrap-and-call surface.
 */
export type PifyAdapter = {
  /**
   Implementation name used in oracle reports.
   */
  readonly name: string;
  /**
   Wraps a module or function with the given options record.
   
   @param input - Module object or callback-style function to wrap.
   
   @param options - Runtime options record (possibly mis-typed).
   
   @returns Pified view, or the thrown error's comparable description.
   */
  wrap: (
    input: object | ((...args: readonly unknown[]) => unknown),
    options: Record<string, unknown>,
  ) => PifiedView | WrapFailure;
  /**
   Calls one member of a pified view with a caller-argument tuple.
   
   @param view - Pified view whose member is called.
   
   @param key - Member key to call.
   
   @param args - Caller arguments forwarded before the callback.
   
   @returns The member call's raw return (a promise when promisified).
   */
  callMember: (
    view: PifiedView,
    key: string,
    args: readonly unknown[],
  ) => unknown;
  /**
   Calls a pified function view itself with a caller-argument tuple.
   
   @param view - Pified function view to call.
   
   @param args - Caller arguments forwarded before the callback.
   
   @returns The call's raw return (a promise when promisified).
   */
  callMain: (
    view: (...args: never[]) => unknown,
    args: readonly unknown[],
  ) => unknown;
};

//endregion Types

//region Helpers

/**
 Wraps thrown wrap failures into a comparable description: upstream `pify`
 throws bare `TypeError` where the fork throws its `InvalidInputError`
 subclass, so the oracle compares the text and TypeError-ness (the class
 name difference is this fork's documented deviation).
 
 @param error - Value thrown by a wrap call.
 
 @returns Comparable description of the throw.
 */
function describeThrown(error: unknown,): WrapFailure {
  return {
    thrownMessage: caughtValueText(error,),
    thrownIsTypeError: error instanceof TypeError,
  };
}

/**
 Calls one member through the fork's `{ args }` tuple shape.
 
 @param view - Pified view whose member is called.
 
 @param key - Member key to call.
 
 @param args - Caller arguments forwarded before the callback.
 
 @returns The member call's raw return.
 */
function callMemberFork(
  {
    view,
    key,
    args,
  }: {
    readonly view: PifiedView;
    readonly key: string;
    readonly args: readonly unknown[];
  },
): unknown {
  return Reflect.apply(
    callable(Reflect.get(
      view,
      key,
    ),),
    view,
    [
      {
        args: [...args,],
      },
    ],
  );
}

/**
 Calls one member through upstream `pify`'s rest-argument shape.
 
 @param view - Pified view whose member is called.
 
 @param key - Member key to call.
 
 @param args - Caller arguments forwarded before the callback.
 
 @returns The member call's raw return.
 */
function callMemberUpstream(
  {
    view,
    key,
    args,
  }: {
    readonly view: PifiedView;
    readonly key: string;
    readonly args: readonly unknown[];
  },
): unknown {
  return Reflect.apply(
    callable(Reflect.get(
      view,
      key,
    ),),
    view,
    [...args,],
  );
}

/**
 Narrows an unknown member value to a callable, failing loudly (and
 comparably) when a probed member is not one.
 
 @param value - Member value read from a pified view.
 
 @returns The value, proven callable.
 */
function callable(value: unknown,): (...args: never[]) => unknown {
  if ((typeof value) !== 'function')
    throw new TypeError(`member is not callable: ${caughtValueText(value,)}`,);
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- typeof narrowing proves callability but not a signature; both adapters call probed members with their own shape and compare only the outcomes
  return value as (...args: never[]) => unknown;
}

//endregion Helpers

//region Adapters

/**
 Adapter over this package's fork: wraps through `pify({ input, options })`
 and calls members with the `{ args }` tuple shape.
 */
export const forkAdapter: PifyAdapter = {
  name: 'fork',
  wrap: function wrapWithFork(
    input,
    options,
  ) {
    try {
      return pify({
        input: input as object,
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mis-typed option records are part of the generated input space and must reach the implementation unvalidated, exactly like upstream pify's untyped runtime
        options: options as never,
      },);
    }
    catch (error) {
      return describeThrown(error,);
    }
  },
  callMember: function callForkMember(
    view,
    key,
    args,
  ) {
    return callMemberFork({
      view,
      key,
      args,
    },);
  },
  callMain: function callForkMain(
    view,
    args,
  ) {
    /**
     Raw call result: a promise when the view is promisified.
     */
    const outcome: unknown = Reflect.apply(
      view,
      view,
      [
        {
          args: [...args,],
        },
      ],
    );
    return outcome;
  },
};

/**
 Adapter over upstream `pify`: wraps through `pify(input, options)` and calls
 members with the rest-argument shape.
 */
export const upstreamAdapter: PifyAdapter = {
  name: 'upstream',
  wrap: function wrapWithUpstream(
    input,
    options,
  ) {
    try {
      return pifyUpstream(
        input,
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mis-typed option records are part of the generated input space and must reach upstream `pify` unvalidated, exactly like its untyped runtime
        options as never,
      ) as PifiedView;
    }
    catch (error) {
      return describeThrown(error,);
    }
  },
  callMember: function callUpstreamMember(
    view,
    key,
    args,
  ) {
    return callMemberUpstream({
      view,
      key,
      args,
    },);
  },
  callMain: function callUpstreamMain(
    view,
    args,
  ) {
    /**
     Raw call result: a promise when the view is promisified.
     */
    const outcome: unknown = Reflect.apply(
      view,
      view,
      [...args,],
    );
    return outcome;
  },
};

//endregion Adapters
