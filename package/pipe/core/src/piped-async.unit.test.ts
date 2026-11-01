import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pipedAsync, } from '@monochromatic-dev/pipe-core';

/** Increments by one; a synchronous `number -> number` step fixture. */
function increment(x: number,): number {
  return x + 1;
}

/** Asynchronously doubles; a `number -> Promise<number>` step fixture. */
async function doubleAsync(x: number,): Promise<number> {
  return x * 2;
}

/** Formats a number into a label; a synchronous `number -> string` step fixture. */
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
  name: pipedAsync.name,
  children: [
    it({
      name: 'mixes synchronous and promise-returning steps',
      fn: async () => {
        const result = await pipedAsync({
          value: 2,
          fn1: increment,
          fn2: doubleAsync,
          fn3: toLabel,
        },);
        // 2 -> 3 -> 6 -> 'n=6'
        expect(result,).toBe('n=6',);
      },
    },),

    it({
      name: 'runs a single async step',
      fn: async () => {
        expect(await pipedAsync({ value: 5, fn1: doubleAsync, },),).toBe(10,);
      },
    },),

    it({
      name: 'awaits a value that is itself a promise',
      fn: async () => {
        expect(await pipedAsync({ value: Promise.resolve(4,), fn1: increment, fn2: doubleAsync, },),).toBe(10,);
      },
    },),

    it({
      name: 'preserves left-to-right order across awaited steps',
      fn: async () => {
        const order: number[] = [];
        async function first(x: number,): Promise<number> {
          await Promise.resolve();
          order.push(1,);
          return x + 1;
        }
        function second(x: number,): number {
          order.push(2,);
          return x * 2;
        }
        async function third(x: number,): Promise<number> {
          await Promise.resolve();
          order.push(3,);
          return x - 1;
        }
        const result = await pipedAsync({ value: 1, fn1: first, fn2: second, fn3: third, },);
        expect(order,).toEqual([1, 2, 3,],);
        // 1 -> 2 -> 4 -> 3
        expect(result,).toBe(3,);
      },
    },),

    it({
      name: 'rejects when a step throws, propagating and logging the error (including the last step)',
      fn: async () => {
        const {
          logger,
          messages,
        } = makeCapturingLogger();
        async function boom(): Promise<number> {
          throw new Error('step exploded',);
        }
        // boom is the LAST step: its rejection must still be logged before it propagates.
        await expect(pipedAsync({ value: 1, fn1: increment, fn2: boom, l: logger, },),).rejects.toThrow(
          'step exploded',
        );
        expect(messages.some(function isStepFailed(message,) {
          return message.includes('step failed',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'threads a supplied logger with composed [pipedAsync] [runPipeAsync] tags',
      fn: async () => {
        const {
          logger,
          messages,
        } = makeCapturingLogger();
        await pipedAsync({ value: 2, fn1: increment, fn2: doubleAsync, l: logger, },);
        expect(messages.includes('[pipedAsync] [runPipeAsync] 2 steps',),).toBe(true,);
      },
    },),

    it({
      name: 'works without a logger (default singleton)',
      fn: async () => {
        expect(await pipedAsync({ value: 10, fn1: doubleAsync, },),).toBe(20,);
      },
    },),
  ],
},);

/**
 Compile-time assertions for {@link pipedAsync}. Exported so it is not an unused local, and never
 invoked at runtime; `lint:types` type-checks the body.
 */
export function typeChecks(): void {
  //region negative: the never tails apply to the async overloads too

  /** Pre-built (non-literal) variable carrying a gap. */
  const gapped = {
    value: 1,
    fn1: increment,
    fn3: toLabel,
  };
  // @ts-expect-error - non-contiguous step keys must be rejected even for a pre-built variable
  void pipedAsync(gapped,);

  //endregion

  //region positive: inferred output types

  // eager async resolves to the final step's awaited return
  expectTypeOf(pipedAsync({ value: 2, fn1: doubleAsync, fn2: toLabel, },),).toEqualTypeOf<Promise<string>>();
  // a promised value is accepted and the result is the awaited final type
  expectTypeOf(pipedAsync({ value: Promise.resolve(2,), fn1: increment, },),).toEqualTypeOf<Promise<number>>();

  //endregion
}
