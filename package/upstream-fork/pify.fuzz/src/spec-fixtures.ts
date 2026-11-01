/**
 Materializes generated specs into real callback-style fixtures and option
 records, one fresh instance per implementation under test.
 
 Fresh instances matter: a shared `/g` `RegExp` would leak `lastIndex` state
 between the fork and upstream `pify` runs, and the oracle must compare two
 structurally identical worlds, not one world run twice.
 
 @module
 */

/* oxlint-disable promise/prefer-await-to-callbacks -- External-boundary mirror: this file's fixture builders implement the node-style callback convention the campaign exercises; see package/upstream-fork/pify/DECISION.callback-capture.md. */

import type {
  FunctionBehavior,
  ModuleSpec,
  OptionsSpec,
  SlotSpec,
  WrappedFunctionSpec,
} from './pify-arbitrary.ts';

//region Types

/**
 Node-style callback a fixture invokes with its outcome.
 */
type FixtureCallback = (...callbackResults: readonly unknown[]) => void;

/**
 One materialized wrapped function: the fixture itself plus the caller
 arguments its calls recorded, for forwarding invariants.
 */
export type BuiltFunction = {
  /**
   Callback-style fixture matching the spec's signature and behavior.
   */
  readonly fn: (...args: never[]) => unknown;
  /**
   Caller arguments recorded per call, in call order.
   */
  readonly receivedArguments: readonly (readonly unknown[])[];
};

//endregion Types

//region Behavior

/**
 Executes one spec behavior: deliver the callback payload from the delivery
 mode's turn, or throw synchronously (which no delivery mode defers).
 
 @param behavior - Callback payload or throw message to deliver.
 
 @param delivery - Whether callback delivery waits for a microtask.
 
 @param callback - Node-style callback to invoke.
 
 @param received - Caller arguments to record before the outcome.
 
 @param receivedArguments - Log appending this call's caller arguments.
 */
function runBehavior(
  {
    behavior,
    delivery,
    callback,
    received,
    receivedArguments,
  }: {
    readonly behavior: FunctionBehavior;
    readonly delivery: 'sync' | 'async';
    readonly callback: FixtureCallback;
    readonly received: readonly unknown[];
    readonly receivedArguments: unknown[];
  },
): void {
  receivedArguments.push([...received,],);
  /**
   A raw call (one made without the trailing callback, as probing a
   non-promisified member does) must fail deterministically at call time:
   deferring the failure to a delivery microtask would crash the harness as
   an uncaught asynchronous error instead of surfacing as a call outcome.
   */
  if ((typeof callback) !== 'function')
    throw new TypeError('fixture called without its trailing callback');
  if (behavior.kind === 'throwSync')
    throw new Error(String(behavior.payload[0],),);

  /**
   Delivers the callback payload once the delivery mode's turn arrives.
   */
  function deliver(): void {
    Reflect.apply(
      callback,
      undefined,
      behavior.payload,
    );
  }

  if (delivery === 'sync')
    deliver();
  else
    queueMicrotask(deliver,);
}

/**
 Identity copier for flag slots, whose values need no per-side copies.
 
 @param value - Slot value to pass through.
 
 @returns The same value.
 */
function passThroughValue(value: unknown,): unknown {
  return value;
}

//endregion Behavior

//region Functions

/**
 Fixture builders keyed by declared caller-argument count: each returns a
 fixture whose signature declares that many caller slots before the trailing
 callback, exactly the arity range the arbitraries generate.
 */
const fixtureBuilders: Record<number, (
  behavior: FunctionBehavior,
  delivery: 'sync' | 'async',
  receivedArguments: (readonly unknown[])[],
) => (...args: never[]) => unknown> = {
  0: function buildZero(
    behavior,
    delivery,
    receivedArguments,
  ) {
    return function fixture0(callback: FixtureCallback,): void {
      runBehavior({
        behavior,
        delivery,
        callback,
        received: [],
        receivedArguments,
      },);
    };
  },
  1: function buildOne(
    behavior,
    delivery,
    receivedArguments,
  ) {
    return function fixture1(
      first: unknown,
      callback: FixtureCallback,
    ): void {
      runBehavior({
        behavior,
        delivery,
        callback,
        received: [first,],
        receivedArguments,
      },);
    };
  },
  2: function buildTwo(
    behavior,
    delivery,
    receivedArguments,
  ) {
    return function fixture2(
      first: unknown,
      second: unknown,
      callback: FixtureCallback,
    ): void {
      runBehavior({
        behavior,
        delivery,
        callback,
        received: [
          first,
          second,
        ],
        receivedArguments,
      },);
    };
  },
  3: function buildThree(
    behavior,
    delivery,
    receivedArguments,
  ) {
    return function fixture3(
      first: unknown,
      second: unknown,
      third: unknown,
      callback: FixtureCallback,
    ): void {
      runBehavior({
        behavior,
        delivery,
        callback,
        received: [
          first,
          second,
          third,
        ],
        receivedArguments,
      },);
    };
  },
};

/**
 Builds one wrapped-function fixture from a spec.
 
 @param spec - Behavior, delivery mode, and declared caller-argument count.
 
 @returns Fixture and its caller-argument recording log.
 
 @example
 ```ts
 const built = buildWrappedFunction({ spec: generatedSpec, });
 ```
 */
export function buildWrappedFunction(
  {
    spec,
  }: {
    readonly spec: WrappedFunctionSpec;
  },
): BuiltFunction {
  /**
   Caller-argument log shared by the fixture's calls.
   */
  const receivedArguments: (readonly unknown[])[] = [];
  /**
   Builder for the spec's declared caller-argument count.
   */
  const builder = fixtureBuilders[spec.declaredArgumentCount];
  if (builder === undefined)
    throw new Error(`no fixture builder for declared argument count ${spec.declaredArgumentCount}`,);
  return {
    fn: builder(
      spec.behavior,
      spec.delivery,
      receivedArguments,
    ),
    receivedArguments,
  };
}

//endregion Functions

//region Options

/**
 Copies one pattern value for a single side: `RegExp` instances are rebuilt
 so `lastIndex` state cannot leak between implementations, arrays are copied
 element-wise, and everything else passes through unchanged (mis-typed
 values must reach both implementations untouched).
 
 @param value - Pattern slot value to copy.
 
 @returns Copy safe to hand to exactly one implementation.
 
 @example
 ```ts
 copyPatternValue([/Alpha/u]); // => fresh RegExp in a fresh array
 ```
 */
export function copyPatternValue(value: unknown): unknown {
  if (value instanceof RegExp)
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- copying a generated member-selection pattern needs a fresh RegExp per side so `/g` lastIndex state cannot leak between the fork and upstream runs; source and flags are the copy's only inputs
    return new RegExp(
      value.source,
      value.flags,
    );
  if (Array.isArray(value,))
    return value.map(function copyEntry(entry: unknown,): unknown {
      return copyPatternValue(entry,);
    },);
  return value;
}

/**
 Builds the runtime options record one spec describes: only the provided
 keys are present, so an absent key and an explicit `undefined` stay distinct
 exactly as upstream `pify`'s spread merge distinguishes them.
 
 @param spec - Per-key slots describing the caller's options record.
 
 @returns Options record for exactly one implementation.
 
 @example
 ```ts
 const options = buildOptions({ spec: generatedOptions, });
 ```
 */
export function buildOptions(
  {
    spec,
  }: {
    readonly spec: OptionsSpec;
  },
): Record<string, unknown> {
  /**
   Runtime options record with only provided keys present.
   */
  const options: Record<string, unknown> = {};

  /**
   Places one slot's key when the caller provides it.
   
   @param key - Options record key the slot governs.
   
   @param slot - Slot describing presence and value.
   
   @param copy - Value copier applied before placement.
   */
  function place(
    {
      key,
      slot,
      copy,
    }: {
      readonly key: string;
      readonly slot: SlotSpec;
      readonly copy: (value: unknown,) => unknown;
    },
  ): void {
    if (slot.provided)
      options[key] = copy(slot.value,);
  }

  place({
    key: 'multiArgs',
    slot: spec.multiArgs,
    copy: passThroughValue,
  },);
  place({
    key: 'errorFirst',
    slot: spec.errorFirst,
    copy: passThroughValue,
  },);
  place({
    key: 'excludeMain',
    slot: spec.excludeMain,
    copy: passThroughValue,
  },);
  place({
    key: 'include',
    slot: spec.include,
    copy: copyPatternValue,
  },);
  place({
    key: 'exclude',
    slot: spec.exclude,
    copy: copyPatternValue,
  },);
  return options;
}

//endregion Options

//region Modules

/**
 Builds one module object from a spec: value members preserved, function
 members built from their behaviors.
 
 @param spec - Members and values describing the module.
 
 @returns Module object for exactly one implementation.
 
 @example
 ```ts
 const module = buildModule({ spec: generatedModule, });
 ```
 */
export function buildModule(
  {
    spec,
  }: {
    readonly spec: ModuleSpec;
  },
): Record<string, unknown> {
  /**
   Module object carrying the spec's values and function members.
   */
  const module: Record<string, unknown> = {
    ...spec.values,
  };
  for (const member of spec.members)
    module[member.key] = buildWrappedFunction({
      spec: member.spec,
    },)
      .fn;
  return module;
}

//endregion Modules
