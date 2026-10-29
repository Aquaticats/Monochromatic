/**
 Generated specs describing wrapped functions, iterable functions, and
 `baseUrl` options, plus the fast-check arbitraries producing them.
 
 Every payload value is structured-cloneable by construction: primitives,
 plain objects, arrays, and nested combinations of them. Functions never
 appear as payloads because neither implementation can clone them across
 the worker boundary, and comparing two clone failures would prove
 nothing about restoration fidelity.
 
 @module
 */

import {
  type Arbitrary,
  array,
  boolean,
  constant,
  constantFrom,
  integer,
  oneof,
  record,
  string,
} from 'fast-check';

//region Types

/**
 How one wrapped function settles: resolve, reject with an `Error`, or throw
 a non-error value.
 */
export type FunctionOutcome = {
  /**
   Discriminator kept literal so exhaustiveness checks stay cheap.
   */
  readonly kind: 'resolve' | 'rejectError' | 'throwValue';
  /**
   Value resolved, error message, or thrown literal payload.
   */
  readonly payload: unknown;
};

/**
 One wrapped single-call function spec: its outcome and how many caller
 arguments it declares.
 */
export type WrappedFunctionSpec = {
  /**
   Outcome the function produces when called.
   */
  readonly outcome: FunctionOutcome;
  /**
   Caller-argument slots the fixture declares.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-optional-escape -- `0` is the genuine zero-argument arity of a wrapped function (a real member of the arity domain), not an absence sentinel
  readonly declaredArgumentCount: 0 | 1 | 2 | typeof MAX_CALL_ARITY;
};

/**
 One wrapped iterable function spec: values yielded in order, an optional
 trailing failure, and declared caller-argument slots.
 */
export type IterableFunctionSpec = {
  /**
   Values yielded in order.
   */
  readonly values: readonly unknown[];
  /**
   Whether the generator throws after yielding every value.
   */
  readonly throwsAfter: boolean;
  /**
   Caller-argument slots the fixture declares.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-optional-escape -- `0` is the genuine zero-argument arity of a wrapped iterable function (a real member of the arity domain), not an absence sentinel
  readonly declaredArgumentCount: 0 | 1 | typeof MAX_ITERABLE_ARITY;
};

/**
 One `baseUrl` option slot: absent, a string, or a `URL` instance.
 */
export type BaseUrlSpec = {
  /**
   Discriminator selecting the option shape.
   */
  readonly kind: 'absent' | 'string' | 'url';
  /**
   String value carried when the kind is `string`.
   */
  readonly value: string;
};

//endregion Types

//region Constants

/**
 Largest payload array length generated for one function.
 */
const MAX_PAYLOAD_LENGTH = 4;

/**
 Largest value count generated for one iterable function.
 */
const MAX_ITERABLE_VALUES = 5;

/**
 Largest declared caller-argument count generated for one function.
 */
const MAX_CALL_ARITY = 3;

/**
 Largest declared caller-argument count generated for one iterable.
 */
const MAX_ITERABLE_ARITY = 2;

/**
 Fake module URLs carried by the `string` baseUrl shape.
 */
const BASE_URL_STRINGS: readonly string[] = [
  'file:///fixture.js',
  'file:///other/module.mjs',
];

/* oxlint-disable no-restricted-syntax/no-optional-escape -- `0` is the genuine zero-argument arity of a wrapped function (a real member of the arity domain), not an absence sentinel */
/**
 Declared caller-argument counts for single-call fixtures.
 */
const CALL_ARGUMENT_COUNTS: readonly (0 | 1 | 2 | typeof MAX_CALL_ARITY)[] = [
  0,
  1,
  2,
  MAX_CALL_ARITY,
];
/* oxlint-enable no-restricted-syntax/no-optional-escape */

/* oxlint-disable no-restricted-syntax/no-optional-escape -- `0` is the genuine zero-argument arity of a wrapped iterable function (a real member of the arity domain), not an absence sentinel */
/**
 Declared caller-argument counts for iterable fixtures.
 */
const ITERABLE_ARGUMENT_COUNTS: readonly (0 | 1 | typeof MAX_ITERABLE_ARITY)[] = [
  0,
  1,
  MAX_ITERABLE_ARITY,
];
/* oxlint-enable no-restricted-syntax/no-optional-escape */

/**
 Non-error literal payloads a wrapped function can throw: each stays
 distinct from every resolved value so outcome confusion is observable.
 */
const THROWN_LITERALS: readonly unknown[] = [
  'unicorn',
  true,
  0,
  '',
  null,
  false,
];

/**
 Falsy values resolved verbatim to prove the reply protocol keys on
 `error`, not the value.
 */
const FALSY_LITERALS: readonly unknown[] = [
  undefined,
  null,
  false,
  0,
  '',
];

/**
 First nested structured-clone fixture value.
 */
const NESTED_FIRST = 1;

/**
 Second nested structured-clone fixture value.
 */
const NESTED_SECOND = 2;

/**
 Third nested structured-clone fixture value.
 */
const NESTED_THIRD = 3;

/**
 Nested array fixture proving structured-clone fidelity past primitives.
 */
const NESTED_ARRAY_FIXTURE = {
  array: [
    NESTED_FIRST,
    [
      NESTED_SECOND,
      [NESTED_THIRD],
    ],
  ],
};

/**
 Plain-object payloads proving structured-clone fidelity past primitives.
 */
const OBJECT_LITERALS: readonly unknown[] = [
  { message: 'unicorn', },
  { nested: NESTED_ARRAY_FIXTURE, },
];

//endregion Constants

//region Arbitraries

/**
 Arbitrary structured-cloneable payload: primitives, falsy literals, and
 plain objects.
 */
export const payloadArb: Arbitrary<unknown> = oneof(
  string(),
  integer(),
  boolean(),
  constantFrom(...FALSY_LITERALS,),
  constantFrom(...OBJECT_LITERALS,),
  array(
    integer(),
    {
      maxLength: MAX_PAYLOAD_LENGTH,
    },
  ),
);

/**
 Arbitrary single-call function outcome.
 */
export const functionOutcomeArb: Arbitrary<FunctionOutcome> = oneof(
  payloadArb.map(function toResolve(payload: unknown,): FunctionOutcome {
    return {
      kind: 'resolve',
      payload,
    };
  },),
  string()
    .map(function toReject(payload: string,): FunctionOutcome {
      return {
        kind: 'rejectError',
        payload,
      };
    },),
  constantFrom(...THROWN_LITERALS,)
    .map(function toThrow(payload: unknown,): FunctionOutcome {
      return {
        kind: 'throwValue',
        payload,
      };
    },),
);

/**
 Arbitrary single-call wrapped-function spec.
 */
export const wrappedFunctionSpecArb: Arbitrary<WrappedFunctionSpec> = record(
  {
    outcome: functionOutcomeArb,
    declaredArgumentCount: constantFrom(...CALL_ARGUMENT_COUNTS,),
  },
);

/**
 Arbitrary iterable-function values: structured-cloneable items.
 */
export const iterableValuesArb: Arbitrary<readonly unknown[]> = array(
  oneof(
    string(),
    integer(),
    boolean(),
    constantFrom(...FALSY_LITERALS,),
  ),
  {
    maxLength: MAX_ITERABLE_VALUES,
  },
);

/**
 Arbitrary iterable-function spec.
 */
export const iterableFunctionSpecArb: Arbitrary<IterableFunctionSpec> = record(
  {
    values: iterableValuesArb,
    throwsAfter: boolean(),
    declaredArgumentCount: constantFrom(...ITERABLE_ARGUMENT_COUNTS,),
  },
);

/**
 Arbitrary `baseUrl` option slot.
 */
export const baseUrlSpecArb: Arbitrary<BaseUrlSpec> = oneof(
  constant({
    kind: 'absent',
    value: '',
  } satisfies BaseUrlSpec,),
  constantFrom(...BASE_URL_STRINGS,)
    .map(function toStringBase(value: string,): BaseUrlSpec {
      return {
        kind: 'string',
        value,
      };
    },),
  constant({
    kind: 'url',
    value: '',
  } satisfies BaseUrlSpec,),
);

//endregion Arbitraries
