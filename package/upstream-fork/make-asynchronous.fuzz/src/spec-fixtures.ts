/**
 Materializes generated specs into real wrapped functions and option
 records, one fresh instance per implementation under test.
 
 Fresh instances matter: both implementations serialize the wrapped
 function with `Function.prototype.toString`, so each side receives its own
 function object built from the same spec. Sharing one function object
 would still work, but fresh instances keep the worlds structurally
 identical without sharing anything the worker boundary could observe.
 
 @module
 */

import type {
  BaseUrlSpec,
  IterableFunctionSpec,
  WrappedFunctionSpec,
} from './workload-arbitrary.ts';

//region Compilation

/**
 Dispatch sources keyed by outcome kind: each is a self-contained function
 expression resolving, rejecting, or throwing its payload text.
 
 @param kind - Outcome kind selecting the dispatch source.
 
 @returns Function expression source for the outcome.
 
 @example
 ```ts
 dispatchSource('resolve',); // => "function (payloadText) { ... }"
 ```
 */
function dispatchSource(kind: string,): string {
  if (kind === 'rejectError')
    return 'function (payloadText) { throw new Error(JSON.parse(payloadText)); }';
  if (kind === 'throwValue')
    return 'function (payloadText) { throw JSON.parse(payloadText); }';
  return 'function (payloadText) { return JSON.parse(payloadText); }';
}

/**
 Compiles one fixture body into a worker-serializable function.
 
 The body runs inside a fresh function scope with no outer bindings, so
 serializing the result never captures spec state.
 
 @param body - Function body source to compile.
 
 @returns Fixture function serializable into a worker.
 
 @example
 ```ts
 compileFixture('return 1;',); // => function returning 1
 ```
 */
function compileFixture(body: string,): (...callArguments: never[]) => unknown {
  /**
   Fixture factory compiled without closing over the spec object.
   */
  // oxlint-disable-next-line eslint/no-new-func, typescript/no-implied-eval, typescript/no-unsafe-type-assertion -- worker-serialization boundary: the fixture source must be constructed as text to stay closed-over nothing; see DECISION.fixture-compilation.md
  const factory = new Function(`return function fixture() { ${body} };`,) as () => (...callArguments: never[]) => unknown;
  return factory();
}

/**
 Compiles one generator body into a worker-serializable generator.
 
 @param body - Generator body source to compile.
 
 @returns Fixture generator serializable into a worker.
 
 @example
 ```ts
 compileIterableFixture('yield 1;',); // => generator yielding 1
 ```
 */
function compileIterableFixture(body: string,): (...callArguments: never[]) => Generator<unknown, void, unknown> {
  /**
   Fixture factory compiled without closing over the spec object.
   */
  // oxlint-disable-next-line eslint/no-new-func, typescript/no-implied-eval, typescript/no-unsafe-type-assertion -- worker-serialization boundary: the fixture source must be constructed as text to stay closed-over nothing; see DECISION.fixture-compilation.md
  const factory = new Function(`return function * fixture() { ${body} };`,) as () => (...callArguments: never[]) => Generator<unknown, void, unknown>;
  return factory();
}

//endregion Compilation

//region Wrapped functions

/**
 Builds one single-call fixture function from a spec, serializable by
 construction: the behavior switches on cloned argument values, never on
 closed-over scope.
 
 @param spec - Outcome and declared arity to materialize.
 
 @returns Fixture function plus its declared caller-argument count.
 
 @example
 ```ts
 const built = buildWrappedFunction({ spec, });
 await built.fn('a', 'b');
 ```
 */
export function buildWrappedFunction(
  {
    spec,
  }: {
    readonly spec: WrappedFunctionSpec;
  },
): {
  /**
   Fixture function serializable into a worker.
   */
  readonly fn: (...callArguments: never[]) => unknown;
  /**
   Caller-argument slots the fixture declares.
   */
  readonly declaredArgumentCount: number;
} {
  /**
   Outcome payload baked into the function source as JSON, so the
   serialized function never closes over the spec object. `undefined`
   payloads are normalized through `null` exactly like the oracle's
   comparison below.
   */
  const payloadJson = JSON.stringify(spec.outcome
    .payload
    ?? null,);

  /**
   Payload JSON encoded once more, so the generated source passes it as a
   string literal instead of inline syntax. Each slot declares its caller
   parameters positionally because the worker spreads the tuple.
   */
  const payloadLiteral = JSON.stringify(payloadJson,);
  /**
   Self-contained fixture sources per declared arity.
   */
  const sources = {
    0: `return (${dispatchSource(spec.outcome
      .kind,)})(${payloadLiteral});`,
    1: `void arguments[0]; return (${dispatchSource(spec.outcome
      .kind,)})(${payloadLiteral});`,
    2: `void arguments[0]; void arguments[1]; return (${dispatchSource(spec.outcome
      .kind,)})(${payloadLiteral});`,
    3: `void arguments[0]; void arguments[1]; void arguments[2]; return (${dispatchSource(spec.outcome
      .kind,)})(${payloadLiteral});`,
  } as const;
  return {
    declaredArgumentCount: spec.declaredArgumentCount,
    fn: compileFixture(sources[spec.declaredArgumentCount],),
  };
}

/**
 Builds one iterable fixture function from a spec: yields every generated
 value, then optionally throws.
 
 @param spec - Values, trailing failure flag, and arity to materialize.
 
 @returns Fixture function plus its declared caller-argument count.
 
 @example
 ```ts
 const built = buildIterableFunction({ spec, });
 for await (const value of built.fn()) {}
 ```
 */
export function buildIterableFunction(
  {
    spec,
  }: {
    readonly spec: IterableFunctionSpec;
  },
): {
  /**
   Fixture generator serializable into a worker.
   */
  readonly fn: (...callArguments: never[]) => Generator<unknown, void, unknown>;
  /**
   Caller-argument slots the fixture declares.
   */
  readonly declaredArgumentCount: number;
} {
  /**
   Values baked into the function source as JSON, so the serialized
   function never closes over the spec object.
   */
  const valuesJson = JSON.stringify(spec.values,);
  /**
   Whether the generator throws after yielding every value.
   */
  const {throwsAfter} = spec;

  /**
   Values JSON encoded once more, so the generated source parses it as a
   string literal instead of inline syntax. Each slot declares its caller
   parameters positionally because the worker spreads the tuple.
   */
  const valuesLiteral = JSON.stringify(valuesJson,);
  /**
   Trailing failure statement baked into every slot source.
   */
  const tail = throwsAfter
    ? "throw new Error('iterable fixture failed');"
    : '';
  /**
   Self-contained generator sources per declared arity.
   */
  const sources = {
    0: `for (const value of JSON.parse(${valuesLiteral})) yield value; ${tail}`,
    1: `void arguments[0]; for (const value of JSON.parse(${valuesLiteral})) yield value; ${tail}`,
    2: `void arguments[0]; void arguments[1]; for (const value of JSON.parse(${valuesLiteral})) yield value; ${tail}`,
  } as const;
  return {
    declaredArgumentCount: spec.declaredArgumentCount,
    fn: compileIterableFixture(sources[spec.declaredArgumentCount],),
  };
}

/**
 Builds one `baseUrl` option record from a spec.
 
 @param spec - Option shape to materialize.
 
 @returns Options record for the wrapper factories, or `undefined` when the
 slot is absent.
 
 @example
 ```ts
 const options = buildBaseUrl({ spec, });
 ```
 */
export function buildBaseUrl(
  {
    spec,
  }: {
    readonly spec: BaseUrlSpec;
  },
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- absent baseUrl slots must travel as a value so both implementations receive identically absent options
): { readonly baseUrl: string | URL; } | undefined {
  if (spec.kind === 'absent')
    return undefined;
  if (spec.kind === 'url')
    return { baseUrl: new URL('file:///fixture.js',), };
  return { baseUrl: spec.value, };
}

//endregion Wrapped functions
