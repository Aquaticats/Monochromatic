import type { Logger, } from '@monochromatic-dev/logger-core/ts';

/**
 Universal pipeline-step supertype for the wide internal (implementation-side) signatures.
 
 `input: never` is contravariant, so every concrete public-overload step
 `(this: void, input: TStepN) => TStepN1` is assignable into `PipeFn` without resorting to the
 banned `any`/`Function` types. `this: void` (matching memoize's step typing) disallows
 method-style steps that rely on a bound `this`.
 
 Not re-exported from the package barrel; this is an implementation type only.
 */
export type PipeFn = (
  this: void,
  input: never,
) => unknown;

/**
 Callable view of {@link PipeFn} after the implementation-only variance widening.
 
 Widens each step's `input: never` to `input: unknown` so the core can apply a step to a value.
 `this: void` is preserved, so applying a step stays safe at the public boundary.
 */
export type CallablePipeFn = (
  this: void,
  input: unknown,
) => unknown;

/**
 Public synchronous step shape used by the overload signatures.
 
 Threads `TInput` to `TOutput` so each overload position carries the previous step's output as
 the next step's input.
 */
export type SyncStep<TInput, TOutput,> = (
  this: void,
  input: TInput,
) => TOutput;

/**
 Public asynchronous step shape used by the overload signatures.
 
 Takes `Awaited<TInput>` because the core awaits every intermediate result before the next step,
 so a step never receives a still-pending promise.
 */
export type AsyncStep<TInput, TOutput,> = (
  this: void,
  input: Awaited<TInput>,
) => TOutput;

/**
 Forbids every step key after `fn1` so one-step overloads resolve exactly.
 
 `fn10?: never` is load-bearing beyond gap rejection: without it a pre-built object carrying
 extra step keys would stay structurally assignable to a smaller-arity overload and resolve to
 it, returning an earlier step's type with no error.
 */
export type NoStepsAfter1 = {
  readonly fn2?: never;
  readonly fn3?: never;
  readonly fn4?: never;
  readonly fn5?: never;
  readonly fn6?: never;
  readonly fn7?: never;
  readonly fn8?: never;
  readonly fn9?: never;
  readonly fn10?: never;
};

/**
 Forbids every step key after `fn2` so two-step overloads resolve exactly.
 
 Keeps `fn10?: never` for the same arity-exact resolution reason as {@link NoStepsAfter1}.
 */
export type NoStepsAfter2 = {
  readonly fn3?: never;
  readonly fn4?: never;
  readonly fn5?: never;
  readonly fn6?: never;
  readonly fn7?: never;
  readonly fn8?: never;
  readonly fn9?: never;
  readonly fn10?: never;
};

/**
 Forbids every step key after `fn3` so three-step overloads resolve exactly.
 
 Keeps `fn10?: never` for the same arity-exact resolution reason as {@link NoStepsAfter1}.
 */
export type NoStepsAfter3 = {
  readonly fn4?: never;
  readonly fn5?: never;
  readonly fn6?: never;
  readonly fn7?: never;
  readonly fn8?: never;
  readonly fn9?: never;
  readonly fn10?: never;
};

/**
 Forbids every step key after `fn4` so four-step overloads resolve exactly.
 
 Keeps `fn10?: never` for the same arity-exact resolution reason as {@link NoStepsAfter1}.
 */
export type NoStepsAfter4 = {
  readonly fn5?: never;
  readonly fn6?: never;
  readonly fn7?: never;
  readonly fn8?: never;
  readonly fn9?: never;
  readonly fn10?: never;
};

/**
 Forbids every step key after `fn5` so five-step overloads resolve exactly.
 
 Keeps `fn10?: never` for the same arity-exact resolution reason as {@link NoStepsAfter1}.
 */
export type NoStepsAfter5 = {
  readonly fn6?: never;
  readonly fn7?: never;
  readonly fn8?: never;
  readonly fn9?: never;
  readonly fn10?: never;
};

/**
 Forbids every step key after `fn6` so six-step overloads resolve exactly.
 
 Keeps `fn10?: never` for the same arity-exact resolution reason as {@link NoStepsAfter1}.
 */
export type NoStepsAfter6 = {
  readonly fn7?: never;
  readonly fn8?: never;
  readonly fn9?: never;
  readonly fn10?: never;
};

/**
 Forbids every step key after `fn7` so seven-step overloads resolve exactly.
 
 Keeps `fn10?: never` for the same arity-exact resolution reason as {@link NoStepsAfter1}.
 */
export type NoStepsAfter7 = {
  readonly fn8?: never;
  readonly fn9?: never;
  readonly fn10?: never;
};

/**
 Forbids every step key after `fn8` so eight-step overloads resolve exactly.
 
 Keeps `fn10?: never` for the same arity-exact resolution reason as {@link NoStepsAfter1}.
 */
export type NoStepsAfter8 = {
  readonly fn9?: never;
  readonly fn10?: never;
};

/**
 Forbids the unsupported `fn10` key so nine-step overloads reject a tenth step.
 
 `fn10?: never` stays even on the largest arity: TypeScript runs excess-property checks only on
 fresh object literals, so a pre-built `{ fn1..fn10 }` would otherwise compile and silently drop
 `fn10`. The runtime overflow guard is the matching defense for callers that bypass the types.
 */
export type NoStepsAfter9 = {
  readonly fn10?: never;
};

/**
 Arguments consumed by the eager cores {@link runPipe}/{@link runPipeAsync}: initial `value`, a
 required `fn1`, optional `fn2..fn9`, and an optional logger.
 
 Width-typed (every step is {@link PipeFn}); the public overloads are the type-safe surface that
 constrains arity and threads the `TStepN` types. Not re-exported.
 */
export type RunArgs = {
  readonly value: unknown;
  readonly fn1: PipeFn;
  readonly fn2?: PipeFn;
  readonly fn3?: PipeFn;
  readonly fn4?: PipeFn;
  readonly fn5?: PipeFn;
  readonly fn6?: PipeFn;
  readonly fn7?: PipeFn;
  readonly fn8?: PipeFn;
  readonly fn9?: PipeFn;
  readonly l?: Logger;
};

/**
 Arguments for the deferred public functions {@link pipe}/{@link pipeAsync}: same shape as
 {@link RunArgs} minus `value`, which each invocation of the returned pipeline supplies.
 
 Not re-exported.
 */
export type DeferredArgs = {
  readonly fn1: PipeFn;
  readonly fn2?: PipeFn;
  readonly fn3?: PipeFn;
  readonly fn4?: PipeFn;
  readonly fn5?: PipeFn;
  readonly fn6?: PipeFn;
  readonly fn7?: PipeFn;
  readonly fn8?: PipeFn;
  readonly fn9?: PipeFn;
  readonly l?: Logger;
};

/**
 Callable view of {@link RunArgs} that widens each step's `never` input to `unknown` for
 application.
 
 The single `as`-cast from {@link RunArgs} in the core targets this type; the cast only widens
 parameter variance (`this: void` is preserved), so applying a step stays safe and the typed
 overloads remain the public surface. `fn10?: never` keeps the runtime overflow destructure
 typed. Not re-exported.
 */
export type RunCallableArgs = {
  readonly value: unknown;
  readonly fn1: CallablePipeFn;
  readonly fn2?: CallablePipeFn;
  readonly fn3?: CallablePipeFn;
  readonly fn4?: CallablePipeFn;
  readonly fn5?: CallablePipeFn;
  readonly fn6?: CallablePipeFn;
  readonly fn7?: CallablePipeFn;
  readonly fn8?: CallablePipeFn;
  readonly fn9?: CallablePipeFn;
  readonly fn10?: never;
  readonly l?: Logger;
};
