/**
 One test-body attempt owns its context, timeout, and sandbox disposal. @module
 */
import { withTimeout, } from '@monochromatic-dev/async-time-core/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ScopedExpect, } from '@monochromatic-dev/test-harness-core-expect/ts';
import {
  createOwnedSandbox,
  SandboxCleanupError,
  type SandboxOwner,
} from '@monochromatic-dev/test-harness-core-sandbox/ts';
import type { TestContext, } from './it.ts';
import { sandboxRuntime, } from './sandbox-runtime.ts';

/**
 Executes one attempt and closes its context before restoring any target state.
 Timeout ends ownership even when the original asynchronous body keeps running.

 @param fn - test callback, unchanged across repeated attempts
 
 @param expect - assertion counter shared with the runner's verdict checks
 
 @param timeout - optional deadline for the body
 
 @param name - test name for timeout and ownership diagnostics
 
 @param l - test hierarchy logger
 
 @throws original body or timeout error, with disposal errors retained by `using`
 
 @example
 ```ts
 await runItAttempt({ fn, expect, name: 'saves', l });
 ```
 */
export async function runItAttempt({
  fn,
  expect,
  timeout,
  name,
  l,
}: {
  readonly fn: (ctx: TestContext,) => Promise<void>;
  readonly expect: ScopedExpect;
  readonly timeout?: number;
  readonly name: string;
  readonly l: Logger;
},): Promise<void> {
  /**
   Resolve lazily after descriptor observation has established the runtime context.
   */
  const runtime = await sandboxRuntime();
  /**
   Fresh identity prevents a previous attempt's retained context from reopening.
   */
  const owner: SandboxOwner = {
    name,
    l,
    phase: 'running',
  };
  /**
   Runner-owned lifetime closes before raw Sinon restoration.
   */
  using sandbox = createOwnedSandbox({
    owner,
    runtime,
  },);
  /**
   The callback receives a new context on every invocation.
   */
  const ctx: TestContext = {
    expect,
    sinon: sandbox.sinon,
  };
  try {
    await runtime.run({
      owner,
      async body(): Promise<void> {
        /**
         The timeout boundary must remain inside the attempt's async context.
         */
        const promise = fn(ctx,);
        await (timeout === undefined ? promise : withTimeout({
          promise,
          ms: timeout,
          label: name,
        },));
      },
    },);
  }
  catch (error) {
    try {
      // Explicitly close here to retain both failures; automatic disposal is then an idempotent no-op.
      sandbox[Symbol.dispose]();
    }
    catch (cleanupError) {
      throw new SandboxCleanupError(
        [
          error,
          cleanupError,
        ],
        'Test body and sandbox cleanup failed',
      );
    }
    throw error;
  }
}
