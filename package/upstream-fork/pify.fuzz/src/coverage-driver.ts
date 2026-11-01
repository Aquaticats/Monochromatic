/**
 Deterministic coverage driver: exercises every exported code path of the
 `pify-fork` runtime package once, so `coverage-report.ts` can freeze and
 check a per-file covered-function baseline.
 
 No fast-check here: fixed inputs keep the covered-function counts stable
 across runs. Fixtures live in `coverage-fixtures.ts`.
 
 @module
 */

/* oxlint-disable promise/prefer-await-to-callbacks -- External-boundary mirror: this driver wraps node-style callback functions, which is the code surface under coverage; see package/upstream-fork/pify/DECISION.callback-capture.md. */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  InvalidInputError,
  pify,
} from '@monochromatic-dev/upstream-fork-pify/ts';

import {
  type FixtureCallback,
  errorFirstFixture,
  failingFixture,
  frozenModule,
  functionModule,
  multiArgsFixture,
  observedRawOutcomes,
  prototypeClashModule,
  receiverModule,
  swallow,
  swallowCall,
  symbolKey,
  symbolModule,
  throwingFixture,
  TrackingPromise,
  valueOnlyFixture,
} from './coverage-fixtures.ts';

//region Options records

/**
 Options with an explicitly `undefined` `errorFirst`, reaching upstream
 `pify`'s spread-merge quirk where an explicit `undefined` beats the default.
 */
const undefinedErrorFirstOptions = {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mirroring a mis-typed caller: upstream `pify` accepts explicitly-undefined option values at runtime and the quirk is part of the covered surface
  errorFirst: undefined as unknown as boolean,
};

/**
 Options with an explicitly `undefined` `exclude`, reaching the member
 selection crash branch.
 */
const undefinedExcludeOptions = {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mirroring a mis-typed caller: upstream `pify` accepts explicitly-undefined option values at runtime and the crash branch is part of the covered surface
  exclude: undefined as unknown as readonly string[],
};

/**
 Options selecting a `RegExp` pattern, reaching the regex match branch of
 member selection.
 */
const regexIncludeOptions = {
  // oxlint-disable-next-line no-restricted-syntax/no-regex, typescript/no-unsafe-type-assertion -- the regex pattern is needed to reach the regex match branch of member selection (one-pass alternation over member-key strings), and upstream `pify`'s overloads type include entries as `keyof Module` even though its runtime accepts RegExp patterns
  include: [/read/u,] as unknown as readonly ('read')[],
};

/**
 Options with an empty `include`, reaching the selection quirk's
 `Object.prototype` short-circuit branch.
 */
const emptyIncludeOptions = {
  include: [],
} as const;

/**
 Options keeping a function module's own call raw.
 */
const excludeMainOptions = {
  excludeMain: true,
} as const;

/**
 Options enabling `multiArgs` collection.
 */
const multiArgsOptions = {
  multiArgs: true,
} as const;

/**
 Options disabling error-first callback interpretation.
 */
const valueOnlyOptions = {
  errorFirst: false,
} as const;

//endregion Options records

//region Exercise

/**
 Drives every exported code path once.
 */
async function exercise(): Promise<void> {
  // Errors: invalid input validation paths.
  swallowCall(function invalidNull(): void {
    pify({
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the invalid-input path under coverage is exactly a value the overloads reject; upstream `pify` throws its TypeError at runtime for it
      input: null as unknown as object,
    },);
  },);
  swallowCall(function invalidString(): void {
    pify({
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the invalid-input path under coverage is exactly a value the overloads reject; upstream `pify` throws its TypeError at runtime for it
      input: 'nope' as unknown as object,
    },);
  },);
  /**
   Error class constructed directly for its own coverage.
   */
  void new InvalidInputError(undefined,);

  // Option resolution: defaults and explicit-undefined override paths.
  await swallow(pify({
    input: errorFirstFixture,
  },)({
    args: [],
  },),);
  await swallow(pify({
    input: failingFixture,
    options: undefinedErrorFirstOptions,
  },)({
    args: [],
  },),);
  swallowCall(function excludeUndefinedCrash(): void {
    /**
     View built with the crashing options; the member read below throws from
     the selection quirk exactly like upstream `pify`.
     */
    const crashedView = pify({
      input: {
        m: errorFirstFixture,
      },
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- mis-typed option records are part of the covered surface and must reach the implementation unvalidated
      options: undefinedExcludeOptions as never,
    },);
    void crashedView.m({
      args: [],
    },);
  },);

  // Callback shapes: error-first, multiArgs, value-only, throw.
  await swallow(pify({
    input: failingFixture,
  },)({
    args: [],
  },),);
  await swallow(pify({
    input: multiArgsFixture,
    options: multiArgsOptions,
  },)({
    args: [],
  },),);
  await swallow(pify({
    input: multiArgsFixture,
    options: {
      ...multiArgsOptions,
      ...valueOnlyOptions,
    },
  },)({
    args: [],
  },),);
  await swallow(pify({
    input: valueOnlyFixture,
    options: valueOnlyOptions,
  },)({
    args: [],
  },),);
  await swallow(pify({
    input: throwingFixture,
  },)({
    args: [],
  },),);

  // Receivers: proxy unwrap and detached calls.
  await swallow(pify({
    input: receiverModule,
  },)
    .read({
      args: [],
    },),);
  /**
   Detached call through `Function.prototype.call`, observing the wrapper's
   receiver handling. The result is awaited through `Promise.resolve` because
   `call` erases the wrapper's precise promise type.
   */
  const detachedResult: unknown = pify({
    input: errorFirstFixture,
  },)
    .call(
    {},
    {
      args: [],
    },
    );
  await swallow(Promise.resolve(detachedResult,),);

  // Promise module.
  await swallow(pify({
    input: errorFirstFixture,
    options: {
      promiseModule: TrackingPromise,
    },
  },)({
    args: [],
  },),);

  // Function modules and excludeMain.
  await swallow(pify({
    input: functionModule,
  },)({
    args: [],
  },),);
  await swallow(pify({
    input: functionModule,
    options: excludeMainOptions,
  },)
    .meow({
      args: [],
    },),);
  swallowCall(function rawMainCall(): void {
    /**
     `excludeMain` keeps the own call raw, so the view is the raw function
     module called without its callback: upstream `pify` behaves the same and
     the fixture throws its deterministic TypeError.
     */
    /* oxlint-disable typescript/no-unsafe-type-assertion -- `excludeMain` leaves the own call raw, which upstream `pify`'s `PromisifyModule` type does not model as callable; the raw call is the branch under coverage */
    Reflect.apply(
      pify({
        input: functionModule,
        options: excludeMainOptions,
      },) as unknown as (...call: readonly unknown[]) => unknown,
      undefined,
      [
        {
          args: [],
        },
      ],
    );
    /* oxlint-enable typescript/no-unsafe-type-assertion */
  },);

  // Member selection: include, exclude, patterns, quirks, keys, descriptors.
  // Each selection probe wraps a fresh module object: upstream `pify`'s
  // shared first-touch decision cache would otherwise answer later wraps
  // from the first wrap's decisions and skip the branch under test (the
  // quirk has its own dedicated probe below).
  await swallow(pify({
    input: {
      read(callback: FixtureCallback,): void {
        callback(
          null,
          'x',
        );
      },
    },
    options: {
      include: ['read',],
    },
  },)
    .read({
      args: [],
    },),);
  /**
   Excluded member left raw: calling it through its own callback signature
   exercises the get trap's pass-through branch.
   */
  pify({
    input: {
      read(callback: FixtureCallback,): void {
        callback(
          null,
          'raw',
        );
      },
    },
    options: {
      exclude: ['read',],
    },
  },)
    .read(function onRawOutcome(
      error: unknown,
      value: unknown,
    ): void {
      observedRawOutcomes.push(`${caughtValueText(error,)}=${caughtValueText(value,)}`,);
    },);
  await swallow(pify({
    input: {
      read(callback: FixtureCallback,): void {
        callback(
          null,
          'x',
        );
      },
    },
    options: regexIncludeOptions,
  },)
    .read({
      args: [],
    },),);
  /**
   Prototype-clashing member read through an empty include, reaching the
   selection quirk's short-circuit branch.
   */
  await swallow(pify({
    input: prototypeClashModule,
    options: emptyIncludeOptions,
  },)
    .toString({
      args: [],
    },),);
  /**
   Second wrap over the same target, reaching the shared decision cache's
   first-touch branch.
   */
  await swallow(pify({
    input: prototypeClashModule,
    options: emptyIncludeOptions,
  },)
    .toString({
      args: [],
    },),);
  /**
   Frozen member read through the proxy-invariant branch.
   */
  void pify({
    input: frozenModule,
  },)
    .prop;
  /**
   Non-function members preserved and enumeration unchanged.
   */
  void Object.keys(pify({
    input: {
      method: errorFirstFixture,
      nonMethod: 3,
    },
  },),);
}

//endregion Exercise

await exercise();
