import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pipe,
  piped,
} from '@monochromatic-dev/pipe-core';

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
  name: pipe.name,
  children: [
    it({
      name: 'returns a reusable pipeline applied across multiple inputs',
      fn: async () => {
        const process = pipe({ fn1: increment, fn2: double, },);
        expect(process(2,),).toBe(6,);
        expect(process(10,),).toBe(22,);
        expect(process(0,),).toBe(2,);
      },
    },),

    it({
      name: 'is point-free parity with eager piped: pipe(fns)(value) === piped({ value, fns })',
      fn: async () => {
        const process = pipe({ fn1: increment, fn2: double, fn3: toLabel, },);
        expect(process(5,),).toBe(piped({ value: 5, fn1: increment, fn2: double, fn3: toLabel, },),);
      },
    },),

    it({
      name: 'composes a single step',
      fn: async () => {
        const process = pipe({ fn1: toLabel, },);
        expect(process(7,),).toBe('n=7',);
      },
    },),

    it({
      name: 'threads a supplied logger with composed [pipe] [runPipe] tags at call time',
      fn: async () => {
        const {
          logger,
          messages,
        } = makeCapturingLogger();
        const process = pipe({ fn1: increment, fn2: double, l: logger, },);
        // no work, no log, until the pipeline runs
        expect(messages.length,).toBe(0,);
        process(2,);
        expect(messages.includes('[pipe] [runPipe] 2 steps',),).toBe(true,);
      },
    },),

    it({
      name: 'works without a logger (default singleton)',
      fn: async () => {
        const process = pipe({ fn1: double, },);
        expect(process(10,),).toBe(20,);
      },
    },),
  ],
},);

/**
 Compile-time assertions for {@link pipe}. Exported so it is not an unused local, and never
 invoked at runtime; `lint:types` type-checks the body.
 */
export function typeChecks(): void {
  //region negative: the never-tail keys reject misuse on the deferred (value-less) overloads

  /** Pre-built (non-literal) variable carrying a gap: `fn2` absent, `fn3` present. */
  const gapped = {
    fn1: increment,
    fn3: toLabel,
  };
  // @ts-expect-error - non-contiguous step keys (fn3 present without fn2) must be rejected even for a pre-built variable
  pipe(gapped,);

  // @ts-expect-error - an explicit `undefined` step is rejected
  pipe({ fn1: increment, fn2: undefined, },);

  // @ts-expect-error - zero steps (no fn1) is rejected
  pipe({},);

  /** Pre-built (non-literal) variable carrying a tenth step beyond the nine-overload cap. */
  const overflowing = {
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
  pipe(overflowing,);

  //endregion

  //region positive: inferred pipeline type

  // deferred result is a reusable pipeline typed from fn1's parameter through the last step
  expectTypeOf(pipe({ fn1: increment, fn2: toLabel, },),).toEqualTypeOf<(value: number,) => string>();
  expectTypeOf(pipe({ fn1: increment, fn2: toLabel, },)(2,),).toEqualTypeOf<string>();

  //endregion
}
