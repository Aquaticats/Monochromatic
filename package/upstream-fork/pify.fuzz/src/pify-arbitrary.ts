/**
 Generated specs describing wrapped functions, modules, and `pify` options,
 plus the fast-check arbitraries producing them.
 
 A spec is plain data: the fixture builders in `spec-fixtures.ts` materialize
 one spec per implementation under test, so the fork and upstream `pify`
 always see structurally identical inputs (including a fresh `RegExp` per side,
 since a shared `/g` pattern's `lastIndex` would leak state between runs).
 
 @module
 */

import {
  type Arbitrary,
  array,
  constant,
  constantFrom,
  integer,
  oneof,
  record,
  string,
} from 'fast-check';

//region Types

/**
 How one wrapped function delivers its callback outcome.
 */
export type Delivery = 'sync' | 'async';

/**
 How one wrapped function behaves when called: report through its callback,
 or throw synchronously.
 */
export type FunctionBehavior = {
  /**
   Discriminator kept literal so exhaustiveness checks stay cheap.
   */
  readonly kind: 'callback' | 'throwSync';
  /**
   Arguments handed to the callback for `callback`, failure message for
   `throwSync`.
   */
  readonly payload: readonly unknown[];
};

/**
 One wrapped function's full behavior: what it does, when, and how many
 caller arguments its signature declares before the trailing callback.
 */
export type WrappedFunctionSpec = {
  /**
   Behavior executed when the function is called.
   */
  readonly behavior: FunctionBehavior;
  /**
   Whether the outcome is delivered immediately or from a microtask.
   */
  readonly delivery: Delivery;
  /**
   Caller-argument slots the fixture's signature declares before its callback.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-optional-escape -- `0` is the genuine zero-argument arity of a wrapped function (a real member of the arity domain), not an absence sentinel
  readonly declaredArgumentCount: 0 | 1 | 2 | typeof MAX_DECLARED_ARITY;
};

/**
 One member of a generated module: its key and the wrapped function behind it.
 */
export type ModuleMemberSpec = {
  /**
   Member key, drawn from a pool that includes `Object.prototype` names the
   upstream selection quirk keys on.
   */
  readonly key: string;
  /**
   Behavior of the member function.
   */
  readonly spec: WrappedFunctionSpec;
};

/**
 A generated module: named function members plus plain value members.
 */
export type ModuleSpec = {
  /**
   Function members to build.
   */
  readonly members: readonly ModuleMemberSpec[];
  /**
   Non-function member values to preserve through the proxy view.
   */
  readonly values: Readonly<Record<string, unknown>>;
};

/**
 One option slot: whether the caller supplies the key, and what value.
 
 `provided` with `value` at `undefined` models upstream `pify`'s explicit
 `undefined` quirk, which the spread merge treats differently from an absent
 key.
 */
export type SlotSpec = {
  /**
   Whether the caller supplies the key at all.
   */
  readonly provided: boolean;
  /**
   Value supplied when `provided`; `undefined` models an explicit undefined.
   */
  readonly value: unknown;
};

/**
 Generated `pify` options: one slot per upstream option key.
 */
export type OptionsSpec = {
  /**
   `multiArgs` slot.
   */
  readonly multiArgs: SlotSpec;
  /**
   `errorFirst` slot.
   */
  readonly errorFirst: SlotSpec;
  /**
   `excludeMain` slot.
   */
  readonly excludeMain: SlotSpec;
  /**
   `include` slot; patterns are strings or `RegExp` instances.
   */
  readonly include: SlotSpec;
  /**
   `exclude` slot; patterns are strings or `RegExp` instances.
   */
  readonly exclude: SlotSpec;
};

//endregion Types

//region Constants

/**
 Largest pattern count generated per `include` / `exclude` slot.
 */
const MAX_PATTERNS_PER_SLOT = 3;

/**
 Largest caller-argument count fixtures declare before their trailing
 callback.
 */
export const MAX_DECLARED_ARITY = 3;

/* oxlint-disable no-restricted-syntax/no-optional-escape -- `0` is the genuine zero-argument arity of a wrapped function (a real member of the arity domain), not an absence sentinel */
/**
 Caller-argument counts fixtures declare before their trailing callback.
 */
export const DECLARED_ARGUMENT_COUNTS: readonly (0 | 1 | 2 | typeof MAX_DECLARED_ARITY)[] = [
  0,
  1,
  2,
  MAX_DECLARED_ARITY,
];
/* oxlint-enable no-restricted-syntax/no-optional-escape */

/**
 Member keys for generated modules. `Object.prototype` names are included on
 purpose: upstream `pify`'s selection cache leaks them past include / exclude
 evaluation, and the differential oracle pins that behavior.
 */
const MEMBER_KEYS: readonly string[] = [
  'a',
  'b',
  'method1',
  'meow',
  'read',
  'readSync',
  'createStream',
  'toString',
  'valueOf',
  'constructor',
  'hasOwnProperty',
];

/**
 Pattern pool for `include` / `exclude` entries: member-key strings and two
 `RegExp` shapes (`u`-flagged and unflagged, since upstream's default pattern
 is unflagged and `lastIndex` behavior differs for `/g`).
 */
const PATTERN_POOL: readonly (string | RegExp)[] = [
  'a',
  'method1',
  'readSync',
  'toString',
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- bounded member-name suffix patterns under test; differential member selection must see both flagged and unflagged regex shapes exactly as upstream pify's `pattern.test(key)` receives them
  /Read|Alpha/u,
  // oxlint-disable-next-line no-restricted-syntax/no-regex, eslint/require-unicode-regexp -- unflagged variant mirrors upstream pify 6.1.0's own default `exclude` pattern style; the missing `u` flag is the parity requirement, and the pattern is a single-pass suffix alternation over bounded member-key input
  /.+Sync|Stream$/,
];

//endregion Constants

//region Arbitraries

/**
 Arbitrary callback arguments: primitives both implementations must compare
 structurally.
 */
export const callbackArgumentArb: Arbitrary<unknown> = oneof(
  string(),
  integer(),
  constant(null,),
  constant(true,),
  constant(false,),
);

/**
 Arbitrary flag slot (`multiArgs` / `errorFirst` / `excludeMain`): absent,
 present with a boolean, or present as explicit `undefined`.
 */
export const flagSlotArb: Arbitrary<SlotSpec> = record({
  provided: constantFrom(
    true,
    false,
  ),
  value: oneof(
    constant(true,),
    constant(false,),
    constant(undefined,),
  ),
},);

/**
 Arbitrary pattern-list slot (`include` / `exclude`): absent, present with a
 pattern array, or present as explicit `undefined`.
 */
export const patternSlotArb: Arbitrary<SlotSpec> = record({
  provided: constantFrom(
    true,
    false,
  ),
  value: oneof(
    array(
      constantFrom(...PATTERN_POOL,),
      {
        maxLength: 3,
      },
    ),
    constant(undefined,),
  ),
},);

/**
 Arbitrary wrapped-function behavior: a callback report or a synchronous
 throw.
 */
export const functionBehaviorArb: Arbitrary<FunctionBehavior> = record(
  {
    kind: constantFrom(
      'callback',
      'throwSync',
    ),
    payload: array(
      callbackArgumentArb,
      {
        minLength: 0,
        maxLength: 4,
      },
    ),
  },
);

/**
 Arbitrary wrapped-function spec.
 */
export const wrappedFunctionSpecArb: Arbitrary<WrappedFunctionSpec> = record(
  {
    behavior: functionBehaviorArb,
    delivery: constantFrom(
      'sync',
      'async',
    ),
    declaredArgumentCount: constantFrom(...DECLARED_ARGUMENT_COUNTS,),
  },
);

/**
 Arbitrary module member.
 */
export const moduleMemberSpecArb: Arbitrary<ModuleMemberSpec> = record(
  {
    key: constantFrom(...MEMBER_KEYS,),
    spec: wrappedFunctionSpecArb,
  },
);

/**
 Arbitrary module spec: function members plus preserved value members.
 */
export const moduleSpecArb: Arbitrary<ModuleSpec> = record(
  {
    members: array(
      moduleMemberSpecArb,
      {
        minLength: 1,
        maxLength: 5,
      },
    ),
    values: integer({
      min: 0,
      max: 3,
    },)
      .map(function toValues(count: number,): Record<string, unknown> {
      /**
       Value members keyed positionally, built from the generated count.
       */
        const values: Record<string, unknown> = {};
        for (let index = 0; index < count; index += 1)
          values[`value${index}`] = index;
        return values;
      },),
  },
);

/**
 Arbitrary options spec: one slot per upstream option key.
 */
export const optionsSpecArb: Arbitrary<OptionsSpec> = record(
  {
    multiArgs: flagSlotArb,
    errorFirst: flagSlotArb,
    excludeMain: flagSlotArb,
    include: patternSlotArb,
    exclude: patternSlotArb,
  },
);

//endregion Arbitraries
