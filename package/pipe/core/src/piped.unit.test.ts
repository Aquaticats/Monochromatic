import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { piped, } from '@monochromatic-dev/pipe-core';

/** Increments by one; a `number -> number` step fixture. */
function increment(x: number,): number {
  return x + 1;
}

/** Doubles; a `number -> number` step fixture. */
function double(x: number,): number {
  return x * 2;
}

/** Formats a number into a label; a `number -> string` step fixture. */
function toLabel(x: number,): string {
  return `n=${String(x,)}`;
}

/** Length of a string; a `string -> number` step fixture, for type-changing chains. */
function lengthOf(text: string,): number {
  return text.length;
}

/**
 Runs `fn`, returns the `Error` it throws.
 
 @param fn - thunk expected to throw, so the named error can be asserted on
 
 @returns thrown error
 
 @throws Error when `fn` does not throw, or throws a non-Error
 */
function runAndCatch(fn: () => void,): Error {
  try {
    fn();
  } catch (error) {
    if (Error.isError(error,))
      return error;
    throw new Error(`expected an Error, got ${String(error,)}`, { cause: error, },);
  }
  throw new Error('expected a throw, but none occurred',);
}

/** Capturing logger plus the buffer it appends every message to, for tag assertions. */
function makeCapturingLogger(): {
  readonly logger: Logger;
  readonly messages: readonly string[];
} {
  const messages: string[] = [];
  function record(message: string,): void {
    messages.push(message,);
  }
  const logger: Logger = {
    debug: record,
    error: record,
    fatal: record,
    info: record,
    trace: record,
    warn: record,
    flush: async function flush(): Promise<void> {},
  };
  return {
    logger,
    messages,
  };
}

await describe({
  name: piped.name,
  children: [
    it({
      name: 'runs a single step',
      fn: async () => {
        expect(piped({ value: 5, fn1: increment, },),).toBe(6,);
      },
    },),

    it({
      name: 'runs several steps left-to-right',
      fn: async () => {
        expect(piped({ value: 2, fn1: increment, fn2: double, fn3: increment, },),).toBe(7,);
      },
    },),

    it({
      name: 'runs the full nine-step chain with types changing across steps',
      fn: async () => {
        const result = piped({
          value: 1,
          fn1: increment,
          fn2: double,
          fn3: increment,
          fn4: toLabel,
          fn5: lengthOf,
          fn6: increment,
          fn7: double,
          fn8: increment,
          fn9: toLabel,
        },);
        // 1 -> 2 -> 4 -> 5 -> 'n=5' -> 3 -> 4 -> 8 -> 9 -> 'n=9'
        expect(result,).toBe('n=9',);
      },
    },),

    it({
      name: 'changes the value type across a two-step chain',
      fn: async () => {
        expect(piped({ value: 41, fn1: toLabel, fn2: lengthOf, },),).toBe(4,);
      },
    },),

    it({
      name: 'threads a supplied logger with composed [piped] [runPipe] tags',
      fn: async () => {
        const {
          logger,
          messages,
        } = makeCapturingLogger();
        piped({ value: 2, fn1: increment, fn2: double, l: logger, },);
        expect(messages.some(function hasComposedTag(message,) {
          return message.includes('[piped] [runPipe]',);
        },),).toBe(true,);
        expect(messages.includes('[piped] [runPipe] 2 steps',),).toBe(true,);
      },
    },),

    it({
      name: 'works without a logger (default singleton)',
      fn: async () => {
        expect(piped({ value: 10, fn1: double, },),).toBe(20,);
      },
    },),

    it({
      name: 'throws PipeStepGapError when forced a gapped object that bypasses the types',
      fn: async () => {
        const error = runAndCatch(function gappedCall() {
          piped({ value: 1, fn1: increment, fn3: increment, } as never,);
        },);
        expect(error.name,).toBe('PipeStepGapError',);
        expect(error.message,).toBe(
          'Pipeline step gap before fn2; provide contiguous function keys from fn1.',
        );
      },
    },),

    it({
      name: 'throws PipeStepOverflowError, logged as invalid pipe arguments, when forced a tenth step',
      fn: async () => {
        const {
          logger,
          messages,
        } = makeCapturingLogger();
        const error = runAndCatch(function overflowCall() {
          piped(
            {
              value: 1,
              fn1: increment,
              fn2: increment,
              fn3: increment,
              fn4: increment,
              fn5: increment,
              fn6: increment,
              fn7: increment,
              fn8: increment,
              fn9: increment,
              fn10: increment,
              l: logger,
            } as never,
          );
        },);
        expect(error.name,).toBe('PipeStepOverflowError',);
        expect(error.message,).toBe('Pipeline supports fn1 through fn9; fn10 was provided.',);
        // footgun 3: a validation failure logs under its own label, not step failed.
        expect(messages.some(function isInvalidArgs(message,) {
          return message.includes('invalid pipe arguments',);
        },),).toBe(true,);
        expect(messages.some(function isStepFailed(message,) {
          return message.includes('step failed',);
        },),).toBe(false,);
      },
    },),
  ],
},);

/**
 Compile-time assertions for {@link piped}. Exported so it is not an unused local, and never
 invoked at runtime; `lint:types` type-checks the body, where `@ts-expect-error` lines must each
 surface an error and `expectTypeOf` lines lock the inferred output types against the emitted
 dist overloads.
 */
export function typeChecks(): void {
  //region negative: the never-tail keys reject misuse (checked against the emitted dist overloads)

  /** Pre-built (non-literal) variable carrying a gap: `fn2` absent, `fn3` present. */
  const gapped = {
    value: 1,
    fn1: increment,
    fn3: toLabel,
  };
  // @ts-expect-error - non-contiguous step keys (fn3 present without fn2) must be rejected even for a pre-built variable
  piped(gapped,);

  // @ts-expect-error - an explicit `undefined` step is rejected (exactOptionalPropertyTypes plus the never tail)
  piped({ value: 1, fn1: increment, fn2: undefined, },);

  // @ts-expect-error - zero steps (no fn1) is rejected; fn1 is required by every overload
  piped({ value: 1, },);

  /** Pre-built (non-literal) variable carrying a tenth step beyond the nine-overload cap. */
  const overflowing = {
    value: 1,
    fn1: increment,
    fn2: increment,
    fn3: increment,
    fn4: increment,
    fn5: increment,
    fn6: increment,
    fn7: increment,
    fn8: increment,
    fn9: increment,
    fn10: increment,
  };
  // @ts-expect-error - a tenth step (fn10) exceeds the nine-overload cap, including on a pre-built variable where excess-property checks do not fire
  piped(overflowing,);

  //endregion

  //region positive: inferred output types

  // eager result type is the final step's return
  expectTypeOf(piped({ value: 2, fn1: increment, fn2: toLabel, },),).toEqualTypeOf<string>();
  // step parameters infer without annotation from the value
  expectTypeOf(piped({ value: 2, fn1: (x,) => x + 1, },),).toEqualTypeOf<number>();

  //endregion
}
