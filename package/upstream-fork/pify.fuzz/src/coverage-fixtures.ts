/**
 Fixtures and swallow helpers for the coverage driver: deterministic
 callback-style functions, modules, and a promise-module stand-in covering
 every behavior branch the `pify-fork` runtime exposes.
 
 @module
 */

/* oxlint-disable promise/prefer-await-to-callbacks -- External-boundary mirror: this file's fixtures implement the node-style callback convention the driver exercises; see package/upstream-fork/pify/DECISION.callback-capture.md. */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Swallow helpers

/**
 Texts of failures swallowed by the driver's helpers: kept so caught values
 stay visible (repository logging rules forbid silently discarded catch
 bindings) and so tests can assert the driver saw real failures.
 */
export const swallowedFailures: string[] = [];

/**
 Runs one promise and swallows its rejection, keeping the driver total.
 
 @param promise - Promise to observe without failing the driver.

 @example
 ```ts
 await swallow(somePromise,);
 ```
  */
export async function swallow(promise: Promise<unknown>,): Promise<void> {
  try {
    await promise;
  }
  catch (error) {
    swallowedFailures.push(caughtValueText(error,),);
  }
}

/**
 Runs one call and swallows its throw, keeping the driver total.
 
 @param call - Call to run without failing the driver.

 @example
 ```ts
 swallowCall(function riskyCall(): void {
   pify({ input: null as unknown as object, },);
 },);
 ```
  */
export function swallowCall(call: () => unknown,): void {
  try {
    call();
  }
  catch (error) {
    swallowedFailures.push(caughtValueText(error,),);
  }
}

/**
 Outcomes observed by raw (non-promisified) member probes, kept so caught
 values stay visible and the driver can prove its raw path ran.
 */
export const observedRawOutcomes: string[] = [];

//endregion Swallow helpers

//region Function fixtures

/**
 Callback-style fixture with a leading error argument.
 
 @param callback - Node-style callback receiving the outcome.

 @example
 ```ts
 errorFirstFixture(function onOutcome(error: unknown, value: unknown): void {
   void error;
   void value;
 },);
 ```
  */
export function errorFirstFixture(callback: (
  error: unknown,
  value: unknown
) => void): void {
  callback(
    null,
    'ok',
  );
}

/**
 Callback-style fixture reporting multiple result arguments.
 
 @param callback - Node-style callback receiving the outcome list.

 @example
 ```ts
 multiArgsFixture(function onOutcome(error: unknown, ...results: unknown[]): void {
   void error;
   void results;
 },);
 ```
  */
export function multiArgsFixture(callback: (
  error: unknown,
  ...results: unknown[]
) => void): void {
  callback(
    null,
    'unicorn',
    'rainbow',
  );
}

/**
 Callback-style fixture failing with a truthy error argument.
 
 @param callback - Node-style callback receiving the failure.

 @example
 ```ts
 failingFixture(function onFailure(error: unknown, value: unknown): void {
   void error;
   void value;
 },);
 ```
  */
export function failingFixture(callback: (
  error: unknown,
  value: unknown
) => void): void {
  callback(
    'boom',
    undefined,
  );
}

/**
 Callback-style fixture throwing synchronously.

 @example
 ```ts
 throwingFixture(); // throws 'sync-boom'
 ```
  */
export function throwingFixture(): never {
  throw new Error('sync-boom',);
}

/**
 Callback-style fixture whose callback reports only a value.
 
 @param callback - Value-only callback receiving the outcome.

 @example
 ```ts
 valueOnlyFixture(function onValue(value: unknown): void {
   void value;
 },);
 ```
  */
export function valueOnlyFixture(callback: (value: unknown) => void): void {
  callback('direct',);
}

//endregion Function fixtures

//region Module fixtures

/**
 Error-first node-style callback shape shared by the module fixtures.
 */
export type FixtureCallback = (
  error: unknown,
  value: unknown
) => void;

/**
 Module fixture whose method reads its receiver through `this`.
 */
export const receiverModule = {
  x: 'foo',
  read(callback: (
    error: unknown,
    value: unknown
  ) => void): void {
    callback(
      null,
      this.x,
    );
  },
};

/**
 Function module fixture with one member: a callable carrying an expando
 member, built via `Object.assign` so isolated declarations see the whole
 shape.
 */
export const functionModule: ((callback: FixtureCallback,) => void) & {
  readonly meow: (callback: FixtureCallback,) => void;
} = Object.assign(
  function functionModule(callback: FixtureCallback,): void {
    callback(
      null,
      'main',
    );
  },
  {
    meow: function meow(callback: FixtureCallback,): void {
      callback(
        null,
        'meow',
      );
    },
  },
);

/**
 Fixture module with a prototype-clashing member name, for the selection
 quirk's reachable branch.
 */
export const prototypeClashModule = {
  toString(callback: (
    error: unknown,
    value: unknown
  ) => void): void {
    callback(
      null,
      't',
    );
  },
};

/**
 Description carrying enough words for the symbol lint rule.
 */
export const symbolKey: symbol = Symbol('coverage driver member key for symbol path',);

/**
 Fixture with a symbol-keyed member.
 */
export const symbolModule: Readonly<Record<symbol, (callback: FixtureCallback,) => void>> = {
  [symbolKey](callback: FixtureCallback,): void {
    callback(
      null,
      'symbol',
    );
  },
};

/**
 Fixture with a frozen own member, for the proxy-invariant branch.
 */
export const frozenModule: Record<string, unknown> = {};
Object.defineProperty(
  frozenModule,
  'prop',
  {
    value(callback: (
      error: unknown,
      value: unknown
    ) => void): void {
      callback(
        null,
        'x',
      );
    },
    writable: false,
    configurable: false,
  },
);

//endregion Module fixtures

//region Promise module fixture

/* oxlint-disable typescript/no-unsafe-type-assertion -- the stand-in's call signature is dictated by the `Promise` constructor contract `pify` invokes it through, and the assertion only states that contract */
/**
 Minimal promise constructor stand-in for the `promiseModule` branch. Its
 signature is dictated by the `Promise` constructor contract `pify` calls it
 with, so it stays a positional two-slot constructor.
 */
export const TrackingPromise = function trackingPromise<T>(
  this: unknown,
  executor: (
    resolve: (value: T | PromiseLike<T>) => void,
    reject: (reason: unknown) => void
  ) => void,
): Promise<T> {
  // oxlint-disable-next-line promise/avoid-new -- the stand-in must delegate to the native promise constructor semantics under test; `pify` invokes it as `new P(executor)`
  return new Promise(executor,);
} as unknown as PromiseConstructor;
/* oxlint-enable typescript/no-unsafe-type-assertion */

//endregion Promise module fixture
