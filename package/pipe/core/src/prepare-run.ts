/**
 The prologue the synchronous and asynchronous dispatch ladders share.

 `runPipe` and `runPipeAsync` apply the same steps in the same order, so the logger tagging, the
 variance widening, and the gap and overflow validation cannot meaningfully differ between them.
 Owning that here means a new validation rule or a log change lands once; only the arity ladder
 stays written twice, and each ladder's own documentation states that arrangement.

 @module
 */

import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import {
  assertContiguousSteps,
  assertNoOverflowStep,
} from './run-steps.ts';

import type {
  RunArgs,
  RunCallableArgs,
} from './types.ts';

/**
 Everything one dispatch ladder needs after preparation: the widened callable arguments plus the
 logger tagged with the calling ladder's name.
 */
export type PreparedRun = RunCallableArgs & {
  readonly l: Logger;
};

/**
 Inputs of {@link prepareRun}: the caller's arguments and the ladder preparing them.
 */
export type PrepareRunArgs = {
  readonly args: RunArgs;
  readonly tag: string;
};

/**
 Tags the logger, logs entry, widens the step inputs, and validates the step slots.

 @param args - caller's value, step functions, and optional logger

 @param tag - name of the calling ladder, so log lines name the entry point that ran

 @returns widened callable arguments and the tagged logger

 @throws {@link PipeStepGapError} when step keys are not contiguous

 @throws {@link PipeStepOverflowError} when `fn10` is present

 @example
 ```ts
 const { l, value, fn1 } = prepareRun({ args, tag: runPipe.name });
 ```
 */
export function prepareRun({
  args,
  tag,
}: PrepareRunArgs,): PreparedRun {
  /**
   Logger tagged at the calling ladder's boundary.
   */
  const l = tagged(args.l === undefined
    ? { tag, }
    : {
      tag,
      l: args.l,
    },);
  l.debug('entry',);

  /* oxlint-disable typescript/no-unsafe-type-assertion -- widen each step's `never` input to `unknown` for application; RunCallableArgs preserves `this: void`, so this is parameter-variance widening only and the typed overloads remain the type-safe public surface */
  /**
   Callable arguments with step inputs widened for internal invocation.
   */
  const callableArgs = args as RunCallableArgs;
  /* oxlint-enable typescript/no-unsafe-type-assertion */

  /**
   Value and step slots read exactly once, so a step supplied through a getter runs once.
   */
  const {
    value,
    fn1,
    fn2,
    fn3,
    fn4,
    fn5,
    fn6,
    fn7,
    fn8,
    fn9,
    fn10,
  } = callableArgs;

  /**
   Ordered step slots used only for runtime contiguity validation.
   */
  const steps = [
    fn1,
    fn2,
    fn3,
    fn4,
    fn5,
    fn6,
    fn7,
    fn8,
    fn9,
  ] as const;

  try {
    assertNoOverflowStep(fn10,);
    assertContiguousSteps(steps,);
  }
  catch (error) {
    l.error(`invalid pipe arguments: ${String(error,)}`,);
    throw error;
  }

  return {
    value,
    fn1,
    fn2,
    fn3,
    fn4,
    fn5,
    fn6,
    fn7,
    fn8,
    fn9,
    l,
  };
}
