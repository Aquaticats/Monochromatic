import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pipeAsync,
  pipedAsync,
} from '@monochromatic-dev/pipe-core';

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
  name: pipeAsync.name,
  children: [
    it({
      name: 'returns a reusable async pipeline applied across multiple inputs',
      fn: async () => {
        const process = pipeAsync({ fn1: increment, fn2: doubleAsync, },);
        expect(await process(2,),).toBe(6,);
        expect(await process(10,),).toBe(22,);
      },
    },),

    it({
      name: 'accepts a promised input at the call boundary',
      fn: async () => {
        const process = pipeAsync({ fn1: increment, fn2: doubleAsync, },);
        expect(
          await process(Promise.resolve(9,),),
        ).toBe(20,);
      },
    },),

    it({
      name: 'is point-free parity with eager pipedAsync',
      fn: async () => {
        const process = pipeAsync({ fn1: increment, fn2: doubleAsync, fn3: toLabel, },);
        expect(await process(5,),).toBe(
          await pipedAsync({ value: 5, fn1: increment, fn2: doubleAsync, fn3: toLabel, },),
        );
      },
    },),

    it({
      name: 'threads a supplied logger with composed [pipeAsync] [runPipeAsync] tags at call time',
      fn: async () => {
        const {
          logger,
          messages,
        } = makeCapturingLogger();
        const process = pipeAsync({ fn1: increment, fn2: doubleAsync, l: logger, },);
        expect(messages.length,).toBe(0,);
        await process(2,);
        expect(messages.includes('[pipeAsync] [runPipeAsync] 2 steps',),).toBe(true,);
      },
    },),

    it({
      name: 'works without a logger (default singleton)',
      fn: async () => {
        const process = pipeAsync({ fn1: doubleAsync, },);
        expect(await process(10,),).toBe(20,);
      },
    },),
  ],
},);

/**
 Compile-time assertions for {@link pipeAsync}. Exported so it is not an unused local, and never
 invoked at runtime; `lint:types` type-checks the body.
 */
export function typeChecks(): void {
  //region negative: the never tails reject misuse on the deferred async overloads

  /** Pre-built (non-literal) variable carrying a gap. */
  const gapped = {
    fn1: doubleAsync,
    fn3: toLabel,
  };
  // @ts-expect-error - non-contiguous step keys must be rejected even for a pre-built variable
  pipeAsync(gapped,);

  //endregion

  //region positive: inferred pipeline type

  // deferred async result is a reusable pipeline accepting T | Promise<T>, resolving to the last awaited type
  expectTypeOf(pipeAsync({ fn1: increment, fn2: doubleAsync, },),).toEqualTypeOf<
    (value: number | Promise<number>,) => Promise<number>
    >();
  expectTypeOf(pipeAsync({ fn1: doubleAsync, fn2: toLabel, },)(2,),).toEqualTypeOf<Promise<string>>();

  //endregion
}
